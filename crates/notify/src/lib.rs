mod window;

use std::{env, fs};

pub fn modal(error: String) {
    println!("{}", error);
    window::open(error);
}
