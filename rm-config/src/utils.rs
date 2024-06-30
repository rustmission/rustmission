use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
    sync::OnceLock,
};

use anyhow::Result;
use serde::de::DeserializeOwned;
use thiserror::Error;
use xdg::BaseDirectories;

#[derive(Error, Debug)]
pub enum ConfigFetchingError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
}

pub fn xdg_dirs() -> &'static BaseDirectories {
    static XDG_DIRS: OnceLock<BaseDirectories> = OnceLock::new();
    XDG_DIRS.get_or_init(|| xdg::BaseDirectories::with_prefix("rustmission").unwrap())
}

pub fn get_config_path(filename: &str) -> PathBuf {
    xdg_dirs().place_config_file(filename).unwrap()
}

pub fn fetch_config<T: DeserializeOwned>(config_name: &str) -> Result<T, ConfigFetchingError> {
    let config_path = xdg_dirs().find_config_file(config_name).ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, format!("{config_name} not found"))
    })?;

    let mut config_buf = String::new();
    let mut config_file = File::open(config_path)?;
    config_file.read_to_string(&mut config_buf)?;

    Ok(toml::from_str(&config_buf)?)
}

pub fn put_config<T: DeserializeOwned>(
    content: &'static str,
    filename: &str,
) -> Result<T, io::Error> {
    let config_path = get_config_path(filename);
    let mut config_file = File::create(config_path)?;
    config_file.write_all(content.as_bytes())?;
    Ok(toml::from_str(content).expect("default configs are correct"))
}
