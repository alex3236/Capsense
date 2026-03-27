use crate::window::create_alert_window;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::RwLock;

pub const CONFIG_PATH: &str = "config.toml";

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

pub fn load_config() {
    let config = if let Ok(content) = fs::read_to_string(CONFIG_PATH) {
        let config: Config = toml::from_str(&content).unwrap_or_else(|_| Config::default());

        // Save back to file to fill in missing default values
        let new_content = toml::to_string(&config).unwrap();
        if content != new_content {
            let _ = fs::write(CONFIG_PATH, new_content);
        }
        config
    } else {
        let default = Config::default();
        let _ = fs::write(CONFIG_PATH, toml::to_string(&default).unwrap());
        create_alert_window(
            "Capsense is in no-English mode by default, \
            which prevent your Chinese IME from entering English mode after layout or focus changes.\n\
            You can change this setting in the configuration file.",
        );
        default
    };
    let mut global_conf = CONFIG.write().unwrap();
    *global_conf = Some(config);
    println!("Config loaded/reloaded.");
}
