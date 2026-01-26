#![allow(unused_must_use)]

use std::process::exit;

use clap::Parser;
use cli::run;

fn main() -> anyhow::Result<()> {
    print!("hellooo");

    run();

    Ok(())
}
