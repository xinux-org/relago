use serde::Deserialize;
use std::{env, fs};

const file_path: &str = "./config.toml";

#[derive(Debug, Deserialize)]
pub struct Config {
    thread_count: u8,           // report/src/compress.rs
    tmp_dir: String,            // cli/src/lib.rs
    xinux_config: String,       // report/src/lib.rs
    problems_interface: String, // daemon/src/core.rs, utils/src/notify.rs
}

pub fn get_config() -> Config {
    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
    let config: Config = toml::from_str(contents.as_str()).unwrap_or(default_config());
    config
}

fn default_config() -> Config {
    Config {
        thread_count: 4,
        tmp_dir: "/tmp/relago".to_owned(),
        xinux_config: "xinux-config".to_owned(),
        problems_interface: "org.freedesktop.problems.daemon".to_owned(),
    }
}
