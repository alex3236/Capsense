pub struct I18n {
    pub title: &'static str,
    pub running_pid: &'static str,
    pub stop_instance: &'static str,
    pub reload_config: &'static str,
    pub edit_config: &'static str,
    pub stop_signal_sent: &'static str,
    pub reload_signal_sent: &'static str,
    pub background_started: &'static str,
    pub already_running: &'static str,
    pub started_monitoring: &'static str,
    pub config_loaded: &'static str,
    pub start_on_login: &'static str,
    pub no_longer_start_on_login: &'static str,
    pub no_running_instance: &'static str,
    pub background_started_cli: &'static str,
}

pub const EN_US: I18n = I18n {
    title: "Capsense",
    running_pid: "Capsense is running with PID: {}",
    stop_instance: "Stop Instance",
    reload_config: "Reload Config",
    edit_config: "Edit Config",
    stop_signal_sent: "Sent stop signal to instance.",
    reload_signal_sent: "Sent reload signal to instance.",
    background_started: "Capsense started in background.\nRun again to show control panel or use CLI commands to control it.",
    already_running: "Another instance is already running. Use --stop or --reload.",
    started_monitoring: "Started. Monitoring CapsLock...",
    config_loaded: "Config loaded or reloaded.",
    start_on_login: "Capsense Will now start on user login.",
    no_longer_start_on_login: "Capsense Will no longer start on user login.",
    no_running_instance: "No running instance found.",
    background_started_cli: "Capsense started in background.",
};

pub const ZH_CN: I18n = I18n {
    title: "Capsense",
    running_pid: "Capsense 正以 PID {} 运行",
    stop_instance: "停止实例",
    reload_config: "重载配置",
    edit_config: "编辑配置",
    stop_signal_sent: "已向实例发送停止信号",
    reload_signal_sent: "已向实例发送重载信号",
    background_started: "Capsense 已在后台启动\n再次运行可打开控制面板，或使用命令行进行控制",
    already_running: "另一个实例正在运行。请使用 --stop 或 --reload",
    started_monitoring: "已启动。正在监控 CapsLock...",
    config_loaded: "配置已加载",
    start_on_login: "Capsense 现在将随用户登录启动",
    no_longer_start_on_login: "Capsense 将不再随用户登录启动",
    no_running_instance: "未发现正在运行的实例",
    background_started_cli: "Capsense 已在后台启动",
};

pub fn get_i18n() -> &'static I18n {
    let lang = std::env::var("LANG").unwrap_or_default();
    if lang.starts_with("zh") {
        &ZH_CN
    } else {
        unsafe {
            let lid = windows_sys::Win32::Globalization::GetUserDefaultUILanguage();
            if (lid & 0xFF) == 0x04 {
                return &ZH_CN;
            }
        }
        &EN_US
    }
}
