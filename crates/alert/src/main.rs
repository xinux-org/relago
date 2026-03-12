// mod notification;
mod window;

use serde_json::{Result, Value};
use std::env;
use std::fs;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let error = load_error(&args);
    // let v: Value = from_str(&content)?;

    // println!("{} {} {}", v["unit"], v["exe"], v["message"]);

    window::open(error);

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
