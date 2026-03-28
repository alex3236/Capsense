#[cfg(feature = "gui")]
pub fn create_alert_window(message: &str) {
    crate::window::create_alert_window(message);
}

#[cfg(not(feature = "gui"))]
pub fn create_alert_window(message: &str) {
    println!("{}", message);
}

#[cfg(feature = "gui")]
pub fn show_instance_manager_window(pid: u32) {
    crate::window::show_instance_manager_window(pid);
}

#[cfg(not(feature = "gui"))]
pub fn show_instance_manager_window(pid: u32) {
    println!("Another instance is already running with PID: {}", pid);
    println!("Use --status, --stop, or --reload to manage it.");
}

