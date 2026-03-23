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

use crate::hook::{WM_RELOAD_CONFIG, run_hook_loop};
use crate::utils::{attach_console, encode_wide, send_msg_to_instance, set_startup};

lazy_static::lazy_static! {
    static ref MUTEX_NAME: Vec<u16> = encode_wide("CapsCustomHookMutex");
}

const CONFIG_PATH: &str = "config.toml";

// Configuration

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Config {
    pub tap_threshold_ms: u64,
    pub tap_action: String,
    pub tap_shortcut: Vec<String>, // ["LWIN", "SPACE"]
    pub layouts: Vec<i32>,
    pub no_en: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tap_threshold_ms: 300,
            tap_action: "switch_layout".to_string(),
            tap_shortcut: vec!["LWIN".to_string(), "SPACE".to_string()],
            layouts: vec![0x0804, 0x0409],
            no_en: true,
        }
    }
}

// Global static config
pub static CONFIG: RwLock<Option<Config>> = RwLock::new(None);

// CLI Arguments

#[derive(clap::ValueEnum, Clone, Debug)]
enum ToggleAction {
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

    #[arg(long, short = 'd')]
    daemon: bool,

    #[arg(long, value_enum)]
    startup: Option<ToggleAction>,

    #[arg(long, value_enum)]
    no_en: Option<ToggleAction>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        attach_console();
    }

    let args = Args::parse();

    // Handle daemonization
    if args.daemon {
        let exe_path = std::env::current_exe()?;
        let mut command = std::process::Command::new(exe_path);

        // Strip the daemon flag to avoid infinite recursion
        for arg in std::env::args().skip(1) {
            if arg != "--daemon" && arg != "-d" {
                command.arg(arg);
            }
        }

        command.spawn()?;
        println!("Capsense started in background.");
        return Ok(());
    }

    // Handle startup command
    if let Some(action) = args.startup {
        match action {
            ToggleAction::Enable => {
                set_startup(true)?;
                println!("Capsense Will now start on system startup.");
            }
            ToggleAction::Disable => {
                set_startup(false)?;
                println!("Capsense Will no longer start on system startup.");
            }
        }
        return Ok(());
    }

    if let Some(action) = args.no_en {
        let enabled = matches!(action, ToggleAction::Enable);
        update_no_en_setting(enabled)?;
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
    let config = if let Ok(content) = fs::read_to_string(CONFIG_PATH) {
        toml::from_str(&content).unwrap_or_else(|_| Config::default())
    } else {
        let default = Config::default();
        let _ = fs::write(CONFIG_PATH, toml::to_string(&default).unwrap());
        default
    };
    let mut global_conf = CONFIG.write().unwrap();
    *global_conf = Some(config);
    println!("Config loaded/reloaded.");
}

fn read_config_from_disk() -> Config {
    fs::read_to_string(CONFIG_PATH)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

fn write_config_to_disk(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    fs::write(CONFIG_PATH, toml::to_string(config)?)?;
    Ok(())
}

fn update_no_en_setting(enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut config = read_config_from_disk();
    config.no_en = enabled;
    write_config_to_disk(&config)?;

    if send_msg_to_instance(WM_RELOAD_CONFIG) {
        println!("No-English mode updated and reloaded.");
    } else {
        println!("No-English mode updated.");
    }

    Ok(())
}
