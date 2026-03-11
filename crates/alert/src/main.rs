// mod notification;
mod window;

use serde_json::{Result, Value, from_str};
use std::env;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let content = load_error(&args);
    let v: Value = from_str(&content)?;

    println!("{} {} {}", v["unit"], v["exe"], v["message"]);

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
