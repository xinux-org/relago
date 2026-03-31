#![allow(unused_must_use)]

use cli::run;
use config::{get_config, Config};

fn main() -> anyhow::Result<()> {
    println!("{:#?}", get_config());
    run()
}
