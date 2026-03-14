#![allow(unused_must_use)]

use std::{process::exit, fs, env};

use clap::Parser;
use notify::modal;
use notify::window::Modal;

fn main() -> anyhow::Result<()> {
    let error = Modal {
        unit: "xyz.service".to_string(),
        exe: "firefox".to_string(),
        message: "yebat".to_string(),
    };

    modal(error);

    Ok(())
}
