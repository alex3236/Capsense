pub struct I18n {
    pub title: &'static str,
    pub no_english_tip: &'static str,
    pub config_loaded: &'static str,

    pub started_monitoring: &'static str,
    pub running_pid: &'static str,
    pub already_running: &'static str,
    pub no_running_instance: &'static str,

    pub edit_config: &'static str,
    pub reload_config: &'static str,
    pub stop_instance: &'static str,

    pub background_started: &'static str,
    pub background_started_cli: &'static str,
    pub reload_signal_sent: &'static str,
    pub stop_signal_sent: &'static str,

    pub elevated: &'static str,
    pub admin_privilege_warning: &'static str,
    pub permission_denied: &'static str,
    pub registry_limit_tip: &'static str,
    pub use_user_flag: &'static str,

    pub startup_status: &'static str,
    pub registry: &'static str,
    pub task_scheduler: &'static str,
    pub enabled: &'static str,
    pub disabled: &'static str,
    pub start_on_login: &'static str,
    pub no_longer_start_on_login: &'static str,
    pub enable_registry: &'static str,
    pub enable_task: &'static str,
    pub disable_all: &'static str,
    pub already_set_to_start_on_login: &'static str,
}

pub const EN_US: I18n = I18n {
    title: "Capsense",
    no_english_tip: "Capsense is in no-English mode by default, \
    which prevent your Chinese IME from entering English mode after layout or focus changes.\n\
    You can change this setting in the configuration file.",
    config_loaded: "Config loaded or reloaded.",

    started_monitoring: "Started. Monitoring CapsLock...",
    running_pid: "Capsense is running with PID: {}",
    already_running: "Another instance is already running. Use --stop or --reload.",
    no_running_instance: "No running instance found.",

    edit_config: "Edit Config",
    reload_config: "Reload Config",
    stop_instance: "Stop Instance",

    background_started: "Capsense started in background.\nRun again to show control panel or use CLI commands to control it.",
    background_started_cli: "Capsense started in background.",
    reload_signal_sent: "Sent reload signal to instance.",
    stop_signal_sent: "Sent stop signal to instance.",

    elevated: "Elevated",
    admin_privilege_warning: "Capsense is not running with administrator privileges and may not be able to control applications that are.",
    permission_denied: "Permission denied. Please run as administrator.",
    registry_limit_tip: "Registry startup cannot control elevated applications. Run as administrator to enable task scheduler startup.",
    use_user_flag: "Use --user flag to control user-level startup.",

    startup_status: "Current Startup Status:",
    registry: "Registry (User)",
    task_scheduler: "Task Scheduler (Machine)",
    enabled: "Enabled",
    disabled: "Disabled",
    start_on_login: "Capsense Will now start on user login.",
    no_longer_start_on_login: "Capsense Will no longer start on user login.",
    enable_registry: "Enable Registry",
    enable_task: "Enable Task",
    disable_all: "Disable All",
    already_set_to_start_on_login: "Capsense is already set to start on login. Please disable it first and try again.",
};

pub const ZH_CN: I18n = I18n {
    title: "Capsense",
    no_english_tip: "Capsense 默认处于 No-English 模式，\n\
    防止中文输入法在布局切换或焦点更改后切换为英文。\n\
    你可以在配置文件中更改此设置。",
    config_loaded: "配置已加载",

    started_monitoring: "已启动。正在监控 CapsLock...",
    running_pid: "Capsense 正以 PID {} 运行",
    already_running: "另一个实例正在运行。请使用 --stop or --reload",
    no_running_instance: "未发现正在运行的实例",

    edit_config: "编辑配置",
    reload_config: "重载配置",
    stop_instance: "停止实例",

    background_started: "Capsense 已在后台启动\n再次运行可打开控制面板，或使用命令行进行控制",
    background_started_cli: "Capsense 已在后台启动",
    reload_signal_sent: "已向实例发送重载信号",
    stop_signal_sent: "已向实例发送停止信号",

    elevated: "管理员权限",
    admin_privilege_warning: "Capsense 未以管理员权限运行，可能无法控制以管理员权限运行的应用。",
    permission_denied: "权限不足。请以管理员身份运行。",
    registry_limit_tip: "从注册表自启动无法控制以管理员权限运行的应用。以管理员身份运行以启用任务计划程序自启动。",
    use_user_flag: "使用 --user 标志以回退至用户级自启。",

    startup_status: "当前自启状态:",
    registry: "注册表 (用户级)",
    task_scheduler: "任务计划程序 (系统级)",
    enabled: "已启用",
    disabled: "已禁用",
    start_on_login: "Capsense 现在将随用户登录启动",
    no_longer_start_on_login: "Capsense 将不再随用户登录启动",
    enable_registry: "注册表启用",
    enable_task: "任务计划启用",
    disable_all: "禁用全部",
    already_set_to_start_on_login: "Capsense 已被设置为随登录启动，请禁用后重试。",
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
