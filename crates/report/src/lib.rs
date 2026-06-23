pub mod compress;
pub mod encrypt;
pub mod info;

use anyhow::Context;
use compress as cmp;
use encrypt as enc;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("Compression failed: {0}")]
    Compression(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("System error: {0}")]
    System(String),

    #[error("Encryption failed: {0}")]
    Encryption(String),
}

pub struct Report {
    pub file: PathBuf,
}

pub enum JournalMode {
    All,
    Recent(usize),
}

pub struct ReportBuilder {
    output_dir: String,
    system_info: bool,
    journal: Option<JournalMode>,
    nixos_config: Option<String>,
    encrypt_key: Option<String>,
    custom_data: Vec<(String, String)>,
    log_file: Option<PathBuf>
}

impl ReportBuilder {
    pub fn new(output_dir: &str) -> Self {
        Self {
            output_dir: output_dir.to_string(),
            system_info: false,
            journal: None,
            nixos_config: None,
            encrypt_key: None,
            custom_data: Vec::new(),
            log_file: None
        }
    }

    pub fn system_info(mut self) -> Self {
        self.system_info = true;
        self
    }

    pub fn journal(mut self, mode: JournalMode) -> Self {
        self.journal = Some(mode);
        self
    }

    pub fn nixos_config(mut self, path: &str) -> Self {
        self.nixos_config = Some(path.to_string());
        self
    }

    pub fn encrypt(mut self, key_path: &str) -> Self {
        self.encrypt_key = Some(key_path.to_string());
        self
    }

    pub fn custom(mut self, key: &str, value: &str) -> Self {
        self.custom_data.push((key.to_string(), value.to_string()));
        self
    }
    pub fn log_file(mut self, file: impl AsRef<Path>) -> Self {
        self.log_file = Some(PathBuf::from(file.as_ref()));
        self
    }
    pub fn build(self) -> Result<Report, ReportError> {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
        let report_dir = PathBuf::from(&self.output_dir).join(format!("report_{}", timestamp));

        fs::create_dir_all(&report_dir)?;

        if self.system_info {
            println!("Collecting system information...");
            let system_info =
                info::collect_system_info().map_err(|e| ReportError::System(e.to_string()))?;
            let file = File::create(report_dir.join("system_info.json"))?;
            serde_json::to_writer_pretty(file, &system_info)?;
        }

        if let Some(mode) = &self.journal {
            let journal_path = report_dir.join("journal_report.json");
            match mode {
                JournalMode::All => {
                    info::collect_journal_all(&journal_path)
                        .map_err(|e| ReportError::System(e.to_string()))?;
                }
                JournalMode::Recent(n) => {
                    info::collect_journal_recent(&journal_path, *n)
                        .map_err(|e| ReportError::System(e.to_string()))?;
                }
            }
            println!("Compressing journal file...");
            cmp::compress(&journal_path, &report_dir)
                .map_err(|e| ReportError::Compression(e.to_string()))?;
            fs::remove_file(&journal_path)?;
        }

        if let Some(config_path) = &self.nixos_config {
            let config_path = shellexpand::tilde(config_path).to_string();
            let src = PathBuf::from(&config_path);
            if src.exists() {
                println!("Copying NixOS configuration from: {}", src.display());
                let dest = report_dir.join("nixos-config");
                info::copy_dir_recursive(&src, &dest)
                    .map_err(|e| ReportError::System(e.to_string()))?;
            }
        }

        if !self.custom_data.is_empty() {
            let custom: serde_json::Map<String, serde_json::Value> = self
                .custom_data
                .into_iter()
                .map(|(k, v)| (k, serde_json::Value::String(v)))
                .collect();
            let file = File::create(report_dir.join("meta.json"))?;
            serde_json::to_writer_pretty(file, &custom)?;
        }

        if self.log_file.is_some() {
            let log = self.log_file
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "missing path"))
            .and_then(fs::read_to_string)?;
            report_dir.join(log);
        }

        cmp::compress_zip(&report_dir, &self.output_dir)
            .map_err(|e| ReportError::Compression(e.to_string()))?;
        fs::remove_dir_all(&report_dir).ok();
        let zip_path = report_dir.with_extension("zip");

        match self.encrypt_key {
            Some(key_path) => {
                let key_path = shellexpand::tilde(&key_path).to_string();
                match enc::encrypt_file(&zip_path, &key_path) {
                    Ok(encrypted_path) => {
                        fs::remove_file(&zip_path).ok();
                        Ok(Report {
                            file: encrypted_path,
                        })
                    }
                    Err(e) => Err(ReportError::Encryption(e.to_string())),
                }
            }
            None => Ok(Report { file: zip_path }),
        }
    }
}
