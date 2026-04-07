use clap::{ArgMatches, Args, CommandFactory, FromArgMatches, Parser};
use confique::{Config as Conf, Layer};
use serde::{Deserialize, Serialize};
use state::LocalInitCell;
use std::fs;
use std::path::PathBuf;

pub static CONFIG: LocalInitCell<Config> = LocalInitCell::new();

#[derive(Conf, Debug)]
#[config(layer_attr(derive(clap::Args)))]
pub struct Config {
    #[config(default = 4)]
    #[config(layer_attr(arg(long)))]
    pub parallel_compression: u32,
    #[config(default = "tmp")]
    #[config(layer_attr(arg(long)))]
    pub tmp_dir: PathBuf,
    #[config(default = "data")]
    #[config(layer_attr(arg(long)))]
    pub data_dir: PathBuf,
    #[config(default = "/etc/nixos/xinux-config")]
    #[config(layer_attr(arg(long)))]
    pub nix_config: PathBuf,
    #[config(default = "https://cocomelon.uz")]
    #[config(layer_attr(arg(long)))]
    pub server: String,
}

type ConfigLayer = <Config as Conf>::Layer;

macro_rules! set_document_field {
    ($document:expr, $config:expr, $field:ident) => {
        if let Some(value) = $config.$field {
            $document[stringify!($field)] = toml_edit::value(serde::Serialize::serialize(
                &value,
                toml_edit::ser::ValueSerializer::new(),
            )?);
        }
    };
}

impl Config {
    pub fn get_config(path: impl Into<PathBuf>) -> anyhow::Result<Config> {
        Config::from_file(path).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn get_command() -> clap::Command {
        ConfigLayer::augment_args(clap::Command::new("configure"))
    }

    pub fn get_from_cli(args: &ArgMatches) -> anyhow::Result<ConfigLayer> {
        ConfigLayer::from_arg_matches(args).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn save_config(path: &impl Into<PathBuf>, config: ConfigLayer) -> anyhow::Result<()> {
        let mut document = str::parse::<toml_edit::DocumentMut>(&fs::read_to_string(path)?)?;

        set_document_field!(document, config, parallel_compression);
        set_document_field!(document, config, tmp_dir);
        set_document_field!(document, config, data_dir);
        set_document_field!(document, config, nix_config);
        set_document_field!(document, config, server);

        fs::write(path, document.to_string())?;

        Ok(())
    }
}
