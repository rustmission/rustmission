pub mod keymap;
mod main_config;
mod utils;

use std::path::PathBuf;

use anyhow::Result;
use keymap::KeymapConfig;
use main_config::MainConfig;

pub struct Config {
    pub general: main_config::General,
    pub connection: main_config::Connection,
    pub keybindings: KeymapConfig,
    pub directories: Directories,
}

pub struct Directories {
    pub main_path: &'static PathBuf,
    pub keymap_path: &'static PathBuf,
}

impl Config {
    pub fn init() -> Result<Self> {
        let main_config = MainConfig::init()?;
        let keybindings = KeymapConfig::init()?;

        let directories = Directories {
            main_path: MainConfig::path(),
            keymap_path: KeymapConfig::path(),
        };

        Ok(Self {
            general: main_config.general,
            connection: main_config.connection,
            keybindings: keybindings.clone(),
            directories,
        })
    }
}
