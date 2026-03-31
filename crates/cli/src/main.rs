#![allow(unused_must_use)]

use cli::run;
use config::get_config;

fn main() -> anyhow::Result<()> {
    get_config();
    run()
}
