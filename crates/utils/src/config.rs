use std::path::PathBuf;

use anyhow::anyhow;
use config_macro::Persist;
use serde::{Deserialize, Serialize};
use state::LocalInitCell;

pub static CONFIG: LocalInitCell<Config> = LocalInitCell::new();

#[derive(Debug, Serialize, Deserialize, Clone, Persist)]
#[persist(path = "./config.toml")]
pub struct Config {
    pub parallel_compression: u32,
    pub tmp_dir: PathBuf,
    pub data_dir: PathBuf,
    pub nix_config: PathBuf,
    pub server: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            parallel_compression: 4,
            tmp_dir: PathBuf::from("tmp"),
            data_dir: PathBuf::from("data"),
            nix_config: PathBuf::from("/etc/nixos/xinux-config"),
            server: "https://cocomelon.uz".to_string(),
        }
    }
}

impl Config {
    pub fn get_config() -> Self {
        Config::parse(&Config::load()).unwrap_or_else(|_| Config::default())
    }
}
