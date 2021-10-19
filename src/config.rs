use crate::control::Player;
use crate::error::Error;
use serde::Deserialize;
use std::default::Default;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub mpd_port: usize,
    pub mpd_host: String,
    pub priority: Player,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mpd_port: 6600,
            mpd_host: String::from("localhost"),
            priority: Player::Mpd,
        }
    }
}
impl Config {
    pub fn try_load() -> Result<Self, Error> {
        let xdg_dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"))?;
        let config_path = xdg_dirs.place_config_file(format!("{}.yml", env!("CARGO_PKG_NAME")))?;
        let config: Self = serde_yaml::from_str(&std::fs::read_to_string(config_path)?)?;
        Ok(config)
    }
}
