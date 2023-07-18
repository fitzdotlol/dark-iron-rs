use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use toml;

use crate::console::console_write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DebugConfig {
    verify_framexml: bool,
    verify_gluexml: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            verify_framexml: true,
            verify_gluexml: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub server_alert_url: Option<String>,
    pub archives: Vec<String>,
    pub debug: DebugConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: None,
            icon: None,
            server_alert_url: None,
            archives: vec![
                String::from("vanilla.MPQ"),
                String::from("vanilla-2.MPQ"),
                String::from("vanilla-3.MPQ"),
            ],

            debug: DebugConfig::default(),
        }
    }
}

const CONFIG_FILENAME: &str = "darkiron.toml";

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let game_path = std::env::current_dir().unwrap();
    let config_path = game_path.join(CONFIG_FILENAME);

    if config_path.exists() {
        let config_str = std::fs::read_to_string(config_path).unwrap();

        return match toml::from_str(config_str.as_str()) {
            Ok(config) => config,
            Err(e) => {
                let err_string = format!("[config] failed to read {CONFIG_FILENAME}: {e}");
                console_write(&err_string, crate::console::ConsoleColor::Error);

                Config::default()
            }
        };
    }

    let text = format!("[config] generating {CONFIG_FILENAME} with defaults");
    console_write(&text, crate::console::ConsoleColor::Warning);

    let config = Config::default();
    let config_str = toml::to_string_pretty(&config).unwrap();

    match std::fs::write(config_path, config_str) {
        Ok(_) => (),
        Err(e) => {
            let err_string = format!("[config] failed to write {CONFIG_FILENAME}: {e}");
            console_write(&err_string, crate::console::ConsoleColor::Error);
        }
    }

    config
});
