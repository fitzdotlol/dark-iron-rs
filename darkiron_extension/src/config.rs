use std::path::PathBuf;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use toml;

use crate::{console::console_write, data};


fn default_base_archives() -> Vec<String> {
    Vec::from_iter(
        data::BASE_ARCHIVE_NAMES.iter().map(|s| {
            String::from(*s)
        })
    )
}

//

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct DebugConfig {
//     pub validate_interface: bool,
// }

// impl Default for DebugConfig {
//     fn default() -> Self {
//         Self {
//             validate_interface: true,
//         }
//     }
// }

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct WindowConfig {
    pub title: Option<String>,
    pub icon: Option<String>,
    pub server_alert_url: Option<String>,
}

//

fn default_path() -> String {
    String::from("Data\\")
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct DataConfig {
    #[serde(default = "default_path")]
    pub path: String,
    #[serde(default = "default_base_archives")]
    pub base_archives: Vec<String>,
    pub patches: Option<Vec<String>>,
    #[serde(default)]
    pub validate_interface: bool,
}

// impl Default for DataConfig {
//     fn default() -> Self {
//         Self {
//             path: None,
//             base_archives: None,//Some(get_default_base_archives()),
//             patches: None,
//         }
//     }
// }

//

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub data: DataConfig,

    // #[serde(default)]
    // pub debug: DebugConfig,
}

impl Config {
    fn from_path(path: &PathBuf) -> Self {
        let config_str = std::fs::read_to_string(path).unwrap();

        match toml::from_str(config_str.as_str()) {
            Ok(config) => config,
            Err(e) => {
                let err_string = format!("[config] failed to read {CONFIG_FILENAME}: {e}");
                console_write(&err_string, crate::console::ConsoleColor::Error);

                Config::default()
            }
        }
    }
}

const CONFIG_FILENAME: &str = "darkiron.toml";

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    let game_path = std::env::current_dir().unwrap();
    let config_path = game_path.join(CONFIG_FILENAME);

    if config_path.exists() {
        return Config::from_path(&config_path);
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
