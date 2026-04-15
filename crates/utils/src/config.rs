use confique::Layer;
use serde::Serialize;
use state::LocalInitCell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};

pub static CONFIG: LocalInitCell<Config> = LocalInitCell::new();

#[derive(confique::Config, Clone, Debug)]
#[config(layer_attr(derive(clap::Args, Serialize)))]
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
    #[config(default = "/var/lib/relago/keys")]
    #[config(layer_attr(arg(long)))]
    pub keys: PathBuf,
}

pub type ConfigLayer = <Config as confique::Config>::Layer;

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
        <Config as confique::Config>::from_file(path).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn save_config(path: impl AsRef<Path>, config: ConfigLayer) -> anyhow::Result<()> {
        let path = &PathBuf::from(path.as_ref());
        let contents: io::Result<String> = match fs::read_to_string(path) {
            io::Result::Ok(content) => io::Result::Ok(content),
            io::Result::Err(err) => match err.kind() {
                io::ErrorKind::NotFound => {
                    let contents = toml_edit::ser::to_string(&ConfigLayer::default_values())?;

                    if let Some(parent) = path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::File::create_new(path)?.write_all(contents.as_bytes())?;

                    io::Result::Ok(contents)
                }
                _ => io::Result::Err(err),
            },
        };

        let mut document = str::parse::<toml_edit::DocumentMut>(&contents?)?;

        set_document_field!(document, config, parallel_compression);
        set_document_field!(document, config, tmp_dir);
        set_document_field!(document, config, data_dir);
        set_document_field!(document, config, nix_config);
        set_document_field!(document, config, server);
        set_document_field!(document, config, keys);

        fs::write(path, document.to_string())?;

        Ok(())
    }
}
