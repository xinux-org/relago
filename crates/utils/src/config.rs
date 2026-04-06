use serde::Deserialize;
use state::LocalInitCell;
use std::{fs, path::PathBuf};

pub static CONFIG: LocalInitCell<Config> = LocalInitCell::new();

#[derive(Debug, Deserialize, Clone)]
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
    pub fn get_config(config: PathBuf) -> Self {
        let contents = fs::read_to_string(config).expect("Should have been able to read the file");
        let res = toml::from_str(contents.as_str()).unwrap_or(Config::default());
        println!("{:?}", res);
        res
    }
}
