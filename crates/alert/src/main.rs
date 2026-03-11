// mod notification;
mod window;

use serde_json::Value;
use std::env;
use std::fs;
use std::sync::mpsc;

fn main() {;
    let errors = load_errors(&args);

    if errors.is_empty() {
        eprintln!("No errors to display.");
        return;
    }

    window::open(errors);
}

fn load_errors(args: &[String]) -> Vec<Value> {
    let path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .map(|s| s.as_str())
        .unwrap_or("errors.json");

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read '{}': {}", path, e);
            return vec![];
        }
    };

    match serde_json::from_str::<Value>(&content) {
        Ok(Value::Array(arr)) => arr,
        Ok(Value::Object(obj)) => {
            for (_, v) in &obj {
                if let Value::Array(arr) = v {
                    return arr.clone();
                }
            }
            vec![Value::Object(obj)]
        }
        Ok(v) => vec![v],
        Err(e) => {
            eprintln!("Failed to parse JSON: {}", e);
            vec![]
        }
    }
}
