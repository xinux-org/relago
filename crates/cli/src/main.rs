#![allow(unused_must_use)]

use cli::run;
use config::Config;

fn main() -> anyhow::Result<()> {
    let conf: Config = Config::get_config();
    run(conf)
}
