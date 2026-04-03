use serde::Deserialize;
use state::LocalInitCell;
use std::io::Write;
use std::{fs, path::PathBuf};
use anyhow::anyhow;
use toml_edit::DocumentMut;

const FILE_PATH: &str = "./config.toml";

pub static CONFIG: LocalInitCell<Config> = LocalInitCell::new();

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub parallel_compression: u32,
    pub tmp_dir: PathBuf,
    pub nix_config: PathBuf,
    pub problems_interface: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            parallel_compression: 4,
            tmp_dir: PathBuf::from("/tmp/relago"),
            nix_config: PathBuf::from("/etc/nixos/xinux-config"),
            problems_interface: "org.freedesktop.problems.daemon".to_string(),
        }
    }
}

impl Config {
    pub fn get_config() -> Self {
        toml::from_str(Self::read_contents().as_str()).unwrap_or(Config::default())
    }

    pub fn set_nix_config(value: &PathBuf) -> anyhow::Result<()> {
        if !fs::exists(value)? {
            return Err(anyhow!("Invalid path, or path doesnt exist"));
        }

        let mut doc = str::parse::<DocumentMut>(&Self::read_contents())?;

        doc["nix_config"] = toml_edit::value(value.to_string_lossy().into_owned());

        fs::write(FILE_PATH, doc.to_string())?;

        Ok(())
    }

    fn read_contents() -> String {
        fs::read_to_string(FILE_PATH).expect("Should have been able to read the file")
    }
}
