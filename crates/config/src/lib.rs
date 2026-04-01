pub mod error;

use error::{Error, Result};
use get_fields::GetFields;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, GetFields)]
#[get_fields(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Config {
    pub thread_count: u32,
    pub tmp_dir: PathBuf,
    pub xinux_config: String,
    pub problems_interface: String,
}

pub struct Builder {
    instance: Config,
}

pub enum Field {
    ThreadCount,
    TmpDir,
    XinuxConfig,
    Unknown,
    ProblemsInterface,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            thread_count: 4,
            xinux_config: "xinux-config".to_string(),
            tmp_dir: PathBuf::from("/tmp/relago"),
            problems_interface: "org.freedesktop.problems.daemon".to_string(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a file at given path and return it as String
    fn read_file(path: PathBuf) -> Result<String> {
        if !(path.is_absolute()) {
            return Err(Error::NonExistent("Given path is not absolute".to_string()));
        }

        if !(path.is_file()) {
            return Err(Error::NonExistent(
                "This file probably doesn't exist".to_string(),
            ));
        }

        let result = match std::fs::read_to_string(path) {
            Ok(d) => d,
            Err(e) => return Err(Error::Read(e)),
        };

        Ok(result)
    }

    /// Save current instance of configuration to a file
    pub fn export(&self, mut path: PathBuf) -> Result<()> {
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            path = path.join("config.toml");
        }

        let output = toml::to_string_pretty(self).map_err(Error::Serialization)?;
        std::fs::write(&path, output).map_err(Error::Write)?;

        Ok(())
    }

    /// Read a file at given path, parse and set values to current instance
    pub fn import(&mut self, path: PathBuf) -> Result<()> {
        let file = std::fs::read_to_string(&path).map_err(Error::Read)?;
        let new: Config = toml::from_str(&file).map_err(Error::Deserialization)?;

        *self = new;

        Ok(())
    }

    /// Attempt to deserialize a file at given path
    pub fn validate(path: PathBuf) -> Result<()> {
        let file = Config::read_file(path)?;
        toml::from_str::<Config>(&file).map_err(Error::Deserialization)?;

        Ok(())
    }
}
