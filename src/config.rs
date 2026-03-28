use crate::i18n::get_i18n;
use crate::ui::create_alert_window;
use crate::DISPLAY_GUI;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::RwLock;

pub const CONFIG_FILENAME: &str = "config.toml";

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

lazy_static::lazy_static! {
    pub static ref CONFIG: RwLock<Option<Config>> = RwLock::new(None);
}

pub fn get_config_path() -> std::path::PathBuf {
    std::env::current_exe()
        .map(|path| {
            path.parent()
                .unwrap_or(std::path::Path::new("."))
                .join(CONFIG_FILENAME)
        })
        .unwrap_or_else(|_| std::path::PathBuf::from(CONFIG_FILENAME))
}

pub fn load_config() {
    let config_path = get_config_path();
    let config = if let Ok(content) = fs::read_to_string(&config_path) {
        let config: Config = toml::from_str(&content).unwrap_or_else(|_| Config::default());

        // Save back to file to fill in missing default values
        let new_content = toml::to_string(&config).unwrap();
        if content != new_content {
            let _ = fs::write(&config_path, new_content);
        }
        config
    } else {
        let default = Config::default();
        let _ = fs::write(&config_path, toml::to_string(&default).unwrap());
        if *DISPLAY_GUI {
            create_alert_window(get_i18n().no_english_tip);
        }
        default
    };
    let mut global_conf = CONFIG.write().unwrap();
    *global_conf = Some(config);
    println!("{}", get_i18n().config_loaded);
}
