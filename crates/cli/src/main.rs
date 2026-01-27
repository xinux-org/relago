#![allow(unused_must_use)]

use std::process::exit;

use clap::Parser;
use cli::run;

fn main() -> anyhow::Result<()> {
    println!("Relago daemon application is started without fuckery!!!");

    let _ = daemon::core::run();
    // run();

    Ok(())
}
