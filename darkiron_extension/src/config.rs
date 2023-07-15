use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use toml;

use crate::console::console_write;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub server_alert_url: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: None,
            icon: None,
            server_alert_url: None,
        }
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let game_path = std::env::current_dir().unwrap();
    let config_path = game_path.join("darkiron.toml");

    if config_path.exists() {
        let config_str = std::fs::read_to_string(config_path).unwrap();
        
        return match toml::from_str(config_str.as_str()) {
            Ok(config) => config,
            Err(e) => {
                let err_string = format!("[config] failed to load darkiron.toml: {}", e);
                console_write(&err_string, crate::console::ConsoleColor::Error);

                Config::default()
            }
        };
    }

    console_write("[config] generating darkiron.toml with defaults", crate::console::ConsoleColor::Warning);

    let config = Config::default();
    let config_str = toml::to_string_pretty(&config).unwrap();

    // TODO: log error
    _ = std::fs::write(config_path, config_str);

    config
});
