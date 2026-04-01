#![allow(unused_must_use)]

use std::{path::PathBuf, process::exit};

use cli::run;
use config::Config;

fn main() -> anyhow::Result<()> {
    let CONFIG_FILE: PathBuf = PathBuf::from("config.toml");
    let mut conf: Config = Config::new();

    match conf.import(CONFIG_FILE) {
        Ok(_) => println!("Configuration has been loaded successfully!"),
        Err(e) => {
            println!("Oops, failed loading configurations: {}", e);
            exit(1);
        }
    };
    run(conf)
}
