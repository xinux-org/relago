#![allow(unused_must_use)]

use cli::run;
use utils::config::{Config, CONFIG};

fn main() -> anyhow::Result<()> {
    CONFIG.set(|| Config::get_config());

    run()
}
