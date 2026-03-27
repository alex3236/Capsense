#![windows_subsystem = "windows"]

use clap::Parser;
use std::ptr::null_mut;

use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE;

pub mod config;
pub mod hook;
pub mod i18n;
pub mod utils;
pub mod window;

use crate::i18n::get_i18n;
use crate::window::create_alert_window;
use config::load_config;
use hook::{run_hook_loop, WM_RELOAD_CONFIG};
use utils::{
    attach_console, encode_wide, get_parent_process_name, get_startup_command,
    send_msg_to_instance, set_startup,
};

lazy_static::lazy_static! {
    static ref MUTEX_NAME: Vec<u16> = encode_wide("CapsCustomHookMutex");
}

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

    #[arg(long)]
    gui: bool,

    #[arg(long)]
    headless: bool,

    #[arg(long, value_enum)]
    startup: Option<ToggleAction>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        attach_console();
    }

    let mut args = Args::parse();

    // Ensure startup command has --headless if it exists
    if let Some(cmd) = get_startup_command() {
        if !cmd.contains("--headless") {
            args.headless = true; // Run headless the first time we know startup argument wrong
            let _ = set_startup(true);
        }
    }

    // Determine whether to display GUI based on arguments and parent process
    let display_gui = if args.headless {
        false
    } else if args.gui {
        true
    } else if let Some(parent) = get_parent_process_name() {
        parent.to_lowercase().contains("explorer.exe")
    } else {
        false
    };

    // Handle startup command
    if let Some(action) = args.startup {
        match action {
            ToggleAction::Enable => {
                set_startup(true)?;
                println!("{}", get_i18n().start_on_login);
            }
            ToggleAction::Disable => {
                set_startup(false)?;
                println!("{}", get_i18n().no_longer_start_on_login);
            }
        }
        return Ok(());
    }

    // Handle status command
    if args.status {
        if let Some(pid) = utils::get_instance_pid() {
            println!("{}", get_i18n().running_pid.replace("{}", &pid.to_string()));
        } else {
            println!("{}", get_i18n().no_running_instance);
        }
        return Ok(());
    }

    // Handle stop command
    if args.stop {
        if send_msg_to_instance(WM_CLOSE) {
            println!("{}", get_i18n().stop_signal_sent);
        } else {
            println!("{}", get_i18n().no_running_instance);
        }
        return Ok(());
    }

    // Handle reload command
    if args.reload {
        if send_msg_to_instance(WM_RELOAD_CONFIG) {
            println!("{}", get_i18n().reload_signal_sent);
        } else {
            println!("{}", get_i18n().no_running_instance);
        }
        return Ok(());
    }

    // Single instance detection
    unsafe {
        let handle = CreateMutexW(null_mut(), 1, MUTEX_NAME.as_ptr());
        if handle == 0 || windows_sys::Win32::Foundation::GetLastError() == 183 {
            // 183 = ERROR_ALREADY_EXISTS

            // Summon instance manager window if started from explorer
            if display_gui && let Some(pid) = utils::get_instance_pid() {
                window::show_instance_manager_window(pid);
                return Ok(());
            }

            eprintln!("{}", get_i18n().already_running);
            return Ok(());
        }
    }

    // Handle demonization after single instance check
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
        println!("{}", get_i18n().background_started_cli);
        return Ok(());
    }

    // Load configuration
    load_config();

    // Run hook loop
    if display_gui && !args.headless {
        create_alert_window(get_i18n().background_started);
    }
    println!("{}", get_i18n().started_monitoring);

    run_hook_loop()?;

    Ok(())
}
