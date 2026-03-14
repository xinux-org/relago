#![allow(unused_must_use)]

use std::{process::exit, fs, env};

use clap::Parser;
use notify::modal;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let error = load_error(&args);

    modal(error);

    Ok(())
}

fn load_error(args: &[String]) -> String {
    let path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .unwrap_or("error.json");

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", path, e);
            return "".to_string();
        }
    };

    content
}
