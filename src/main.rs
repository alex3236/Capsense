#![windows_subsystem = "windows"]

use clap::Parser;
use std::ptr::null_mut;

use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::WM_CLOSE;

pub mod config;
pub mod hook;
pub mod i18n;
pub mod ui;
pub mod utils;
#[cfg(feature = "gui")]
pub mod window;

use crate::i18n::get_i18n;
use crate::ui::create_alert_window;
use crate::utils::is_task_enabled;
use config::load_config;
use hook::{run_hook_loop, WM_RELOAD_CONFIG};
#[cfg(feature = "gui")]
use utils::get_parent_process_name;
use utils::{
    attach_console, encode_wide, get_startup_command, is_elevated, send_msg_to_instance,
    set_startup,
};

lazy_static::lazy_static! {
    static ref MUTEX_NAME: Vec<u16> = encode_wide("CapsCustomHookMutex");
    pub(crate) static ref DISPLAY_GUI: bool = {
        #[cfg(not(feature = "gui"))]
        {
            false
        }

        #[cfg(feature = "gui")]
        {
        let args = Args::parse();
        if args.headless {
            false
        } else if args.gui {
            true
        } else if let Some(parent) = get_parent_process_name() {
            parent.to_lowercase().contains("explorer.exe")
        } else {
            false
        }
        }
    };
}

// CLI Arguments

#[derive(clap::ValueEnum, Clone, Debug)]
enum ToggleAction {
    Enable,
    Disable,
    Show,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Capsense",
    long_about = "CapsLock that makes sense."
)]
struct Args {
    #[arg(long, short = 's', help = "Tell the running instance to stop.")]
    stop: bool,

    #[arg(
        long,
        short = 'r',
        help = "Tell the running instance to reload configuration."
    )]
    reload: bool,

    #[arg(
        long,
        value_enum,
        help = "Manage startup: enable, disable, or show current status (default if no value)",
        num_args = 0..=1,
        default_missing_value = "show"
    )]
    startup: Option<ToggleAction>,

    #[arg(long, short = 'S', help = "Show status of the running instance.")]
    status: bool,

    #[arg(long, short = 'd', help = "Run as a background process.")]
    daemon: bool,

    #[arg(long, help = "Force show GUI.", conflicts_with = "headless")]
    gui: bool,

    #[arg(long, help = "Run without GUI.", conflicts_with = "gui")]
    headless: bool,

    #[arg(
        long,
        requires = "startup",
        help_heading = "Options of --startup",
        help = "User level startup (registry) instead of machine level (task scheduler)."
    )]
    user: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        attach_console();
    }

    let mut args = Args::parse();

    #[cfg(not(feature = "gui"))]
    if args.gui {
        println!("This build does not include GUI support; continuing in headless mode.");
    }

    // Ensure startup command has --headless if it exists
    if let Some(cmd) = get_startup_command() {
        if !cmd.contains("--headless") {
            args.headless = true; // Run headless the first time we know startup argument wrong
            let _ = set_startup(true, false);
        }
    }

    // Handle startup command
    if let Some(action) = args.startup {
        match action {
            ToggleAction::Show => {
                println!("{}", get_i18n().startup_status);
                let reg_status = if get_startup_command().is_some() {
                    get_i18n().enabled
                } else {
                    get_i18n().disabled
                };
                let task_status = if is_task_enabled() {
                    get_i18n().enabled
                } else {
                    get_i18n().disabled
                };

                println!("{}: {}", get_i18n().registry, reg_status);
                println!("{}: {}", get_i18n().task_scheduler, task_status);
                return Ok(());
            }
            ToggleAction::Enable | ToggleAction::Disable => {}
        }

        if !args.user && !is_elevated() {
            eprintln!("{}", get_i18n().permission_denied);
            eprintln!("{}", get_i18n().use_user_flag);
            return Ok(());
        }

        // Clear both startup settings to avoid confusion, then set the correct one

        match action {
            ToggleAction::Enable => {
                if get_startup_command().is_some() || is_task_enabled() {
                    println!("{}", get_i18n().already_set_to_start_on_login);
                    return Ok(());
                }
                set_startup(true, !args.user)?;
                println!("{}", get_i18n().start_on_login);
            }
            ToggleAction::Disable => {
                let _ = set_startup(false, false);
                if is_elevated() {
                    let _ = set_startup(false, true);
                }
                println!("{}", get_i18n().no_longer_start_on_login);
            }
            ToggleAction::Show => {}
        }
        return Ok(());
    }

    // Handle status command
    if args.status {
        if let Some(pid) = utils::get_instance_pid() {
            let message = {
                if utils::is_process_elevated(pid) {
                    &*format!("({}) ", get_i18n().elevated)
                } else {
                    ""
                }
            };
            println!(
                "{}{}",
                message,
                get_i18n().running_pid.replace("{}", &pid.to_string())
            );
        } else {
            println!("{}", get_i18n().no_running_instance);
        }
        return Ok(());
    }

    // Handle stop command
    if args.stop {
        send_msg_to_instance(WM_CLOSE).map(|success| {
            if success {
                println!("{}", get_i18n().stop_signal_sent);
            } else {
                println!("{}", get_i18n().no_running_instance);
            }
        });
        return Ok(());
    }

    // Handle reload command
    if args.reload {
        send_msg_to_instance(WM_RELOAD_CONFIG).map(|success| {
            if success {
                println!("{}", get_i18n().reload_signal_sent);
            } else {
                println!("{}", get_i18n().no_running_instance);
            }
        });
        return Ok(());
    }

    // Single instance detection
    unsafe {
        let handle = CreateMutexW(null_mut(), 1, MUTEX_NAME.as_ptr());
        if handle == 0 || windows_sys::Win32::Foundation::GetLastError() == 183 {
            // 183 = ERROR_ALREADY_EXISTS

            // Summon instance manager window if started from explorer
            if *DISPLAY_GUI && let Some(pid) = utils::get_instance_pid() {
                ui::show_instance_manager_window(pid);
                return Ok(());
            }

            eprintln!("{}", get_i18n().already_running);
            return Ok(());
        }
    }

    // Warn if not running with admin privileges
    if !is_elevated() {
        println!("{}", get_i18n().admin_privilege_warning);
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
    if *DISPLAY_GUI {
        let mut message = get_i18n().background_started.to_string();
        if !is_elevated() {
            message = format!("{}\n{}", message, get_i18n().admin_privilege_warning);
        }
        create_alert_window(&*message);
    }
    println!("{}", get_i18n().started_monitoring);

    run_hook_loop()?;

    Ok(())
}
