#![windows_subsystem = "windows"]

use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ptr::null_mut;
use std::sync::RwLock;

use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE;

mod hook;
pub mod utils;

use crate::hook::{run_hook_loop, WM_RELOAD_CONFIG};
use crate::utils::{attach_console, encode_wide, send_msg_to_instance, set_startup};

lazy_static::lazy_static! {
    static ref MUTEX_NAME: Vec<u16> = encode_wide("CapsCustomHookMutex");
}

// Configuration

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub tap_threshold_ms: u64,
    pub tap_shortcut: Vec<String>, // ["LWIN", "SPACE"]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tap_threshold_ms: 300,
            tap_shortcut: vec!["LWIN".to_string(), "SPACE".to_string()],
        }
    }
}

// Global static config
pub static CONFIG: RwLock<Option<Config>> = RwLock::new(None);

// CLI Arguments

#[derive(clap::ValueEnum, Clone, Debug)]
enum StartupAction {
    Enable,
    Disable,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "CapsLock Hook Utility")]
struct Args {
    #[arg(long, short = 's')]
    stop: bool,

    #[arg(long, short = 'r')]
    reload: bool,

    #[arg(long, short = 'S')]
    status: bool,

    #[arg(long, value_enum)]
    startup: Option<StartupAction>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        attach_console();
    }

    let args = Args::parse();

    // Handle startup command
    if let Some(action) = args.startup {
        match action {
            StartupAction::Enable => {
                set_startup(true)?;
                println!("Capsense Will now start on system startup.");
            }
            StartupAction::Disable => {
                set_startup(false)?;
                println!("Capsense Will no longer start on system startup.");
            }
        }
        return Ok(());
    }

    // Handle status command
    if args.status {
        if let Some(pid) = crate::utils::get_instance_pid() {
            println!("Capsense running with PID: {}", pid);
        } else {
            println!("No running instance found.");
        }
        return Ok(());
    }

    // Handle stop command
    if args.stop {
        if send_msg_to_instance(WM_CLOSE) {
            println!("Sent stop signal.");
        } else {
            println!("No running instance found.");
        }
        return Ok(());
    }

    // Handle reload command
    if args.reload {
        if send_msg_to_instance(WM_RELOAD_CONFIG) {
            println!("Sent reload signal.");
        } else {
            println!("No running instance found.");
        }
        return Ok(());
    }

    // Single instance detection
    unsafe {
        let handle = CreateMutexW(null_mut(), 1, MUTEX_NAME.as_ptr());
        if handle == 0 || windows_sys::Win32::Foundation::GetLastError() == 183 {
            // 183 = ERROR_ALREADY_EXISTS
            eprintln!("Another instance is already running. Use --stop or --reload.");
            return Ok(());
        }
    }

    // Load configuration
    load_config();

    // Run hook loop
    println!("Started. Monitoring CapsLock...");
    run_hook_loop()?;

    Ok(())
}

pub fn load_config() {
    let path = "config.toml";
    let config = if let Ok(content) = fs::read_to_string(path) {
        toml::from_str(&content).unwrap_or_else(|_| Config::default())
    } else {
        let default = Config::default();
        let _ = fs::write(path, toml::to_string(&default).unwrap());
        default
    };
    let mut global_conf = CONFIG.write().unwrap();
    *global_conf = Some(config);
    println!("Config loaded/reloaded.");
}
