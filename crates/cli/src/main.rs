#![allow(unused_must_use)]

use std::exit;

use clap::Parser;
use cli::run;

fn main() -> anyhow::Result<()> {
    run();

    Ok(())
}
