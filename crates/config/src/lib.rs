use clap::Parser;
use serde::Deserialize;
use std::{fs, path::PathBuf, sync::Mutex};

const FILE_PATH: &str = "./config.toml";

#[derive(Debug, Deserialize, Clone, Parser)]
pub struct Config {
    pub thread_count: u32,          // report/src/compress.rs
    pub tmp_dir: PathBuf,           // cli/src/lib.rs
    pub xinux_config: String,       // report/src/lib.rs
    pub problems_interface: String, // daemon/src/core.rs, utils/src/notify.rs
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
