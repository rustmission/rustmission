use std::{path::PathBuf, sync::OnceLock};

use anyhow::Result;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::utils::{self, put_config};

#[derive(Serialize, Deserialize)]
pub struct MainConfig {
    pub general: General,
    pub connection: Connection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct General {
    #[serde(default)]
    pub auto_hide: bool,
    #[serde(default = "default_accent_color")]
    pub accent_color: Color,
    #[serde(default = "default_beginner_mode")]
    pub beginner_mode: bool,
    #[serde(default)]
    pub headers_hide: bool,
}

fn default_accent_color() -> Color {
    Color::LightMagenta
}

fn default_beginner_mode() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Connection {
    pub username: Option<String>,
    pub password: Option<String>,
    pub url: Url,
    #[serde(default)]
    pub torrents_refresh: u64,
    #[serde(default)]
    pub stats_refresh: u64,
    #[serde(default)]
    pub free_space_refresh: u64,
}

impl MainConfig {
    pub(crate) const FILENAME: &'static str = "config.toml";
    const DEFAULT_CONFIG: &'static str = include_str!("../defaults/config.toml");

    pub(crate) fn init() -> Result<Self> {
        let Ok(config) = utils::fetch_config(Self::FILENAME) else {
            put_config(Self::DEFAULT_CONFIG, Self::FILENAME)?;
            // TODO: check if the user really changed the config.
            println!("Update {:?} and start rustmission again", Self::path());
            std::process::exit(0);
        };

        Ok(config)
    }

    pub(crate) fn path() -> &'static PathBuf {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| utils::get_config_path(Self::FILENAME))
    }
}
