use serde::Deserialize;
use state::LocalInitCell;
use std::{fs, path::PathBuf};

const FILE_PATH: &str = "./config.toml";

pub static CONFIG: LocalInitCell<Config> = LocalInitCell::new();

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub thread_count: u32,
    pub tmp_dir: PathBuf,
    pub xinux_config: String,
    pub problems_interface: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thread_count: 4,
            tmp_dir: PathBuf::from("/tmp/relago"),
            xinux_config: "xinux-config".to_string(),
            problems_interface: "org.freedesktop.problems.daemon".to_string(),
        }
    }
}

impl Config {
    pub fn get_config() -> Self {
        let contents =
            fs::read_to_string(FILE_PATH).expect("Should have been able to read the file");
        toml::from_str(contents.as_str()).unwrap_or(Config::default())
    }
}
