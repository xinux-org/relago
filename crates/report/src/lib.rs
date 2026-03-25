pub mod compress;
pub mod encrypt;
pub mod info;

use compress as cmp;
use encrypt as enc;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("File not found")]
    Compression,

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Something wrong: {0}")]
    System(String),
}

struct Report {
    file: PathBuf,
}

pub fn run(
    output_dir: &str,
    nixos_config_path: Option<&str>,
    recent_entries: Option<usize>,
    public_key_path: Option<&str>,
) -> anyhow::Result<()> {
    let _ = create_report(output_dir, nixos_config_path, recent_entries, public_key_path);
    Ok(())
}

pub fn create_report(
    output_dir: &str,
    nixos_config_path: Option<&str>,
    recent_entries: Option<usize>,
    public_key_path: Option<&str>,
) -> Result<Report, ReportError> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let report_dir = PathBuf::from(&output_dir).join(format!("report_{}", timestamp));

    println!("Creating report directory: {}", report_dir.display());
    fs::create_dir_all(&report_dir).map_err(|x| ReportError::System(x.to_string()));

    // 1. Collect and save system information
    println!("Collecting system information...");
    let system_info = info::collect_system_info().expect("System info failed");
    let system_info_path = report_dir.join("system_info.json");

    let file = File::create(&system_info_path).expect("File create failed");

    serde_json::to_writer_pretty(file, &system_info).expect("serialization failed");
    println!("System info saved: {}", system_info_path.display());

    // 2. Collect journal entries
    let journal_path = report_dir.join("journal_report.json");
    if let Some(num) = recent_entries {
        info::collect_journal_recent(&journal_path, num);
    } else {
        info::collect_journal_all(&journal_path);
    }

    // Compress .json then remove it
    println!("Compressing journal file...");
    cmp::compress(&journal_path, &report_dir).expect("Compression failed");
    fs::remove_file(&journal_path).expect("Remove failed");

    // 3. Copy NixOS configuration if provided
    if let Some(config_path) = nixos_config_path {
        let config_path = shellexpand::tilde(config_path).to_string();
        let src = PathBuf::from(&config_path);

        if !src.exists() {
            eprintln!("Warning: NixOS config path does not exist: {}", config_path);
        } else {
            println!("Copying NixOS configuration from: {}", src.display());
            let dest = report_dir.join("nixos-config");
            info::copy_dir_recursive(&src, &dest);
            println!("NixOS config copied: {}", dest.display());
        }
    }
    let mut key_path = public_key_path.map(|p| shellexpand::tilde(p).to_string());

    if system_info.system_name == Some("XinuxOS".to_string()) {
        let src = Path::new("/etc/nixos");
        let dest = report_dir.join("xinux-config");
        info::copy_dir_recursive(&src, &dest);

        if key_path.is_none() {
            key_path = Some("/etc/xinux/keys/public.asc".to_string());
        }
    }

    cmp::compress_zip(&report_dir, &output_dir).expect("Zip compression failed");
    let zip_path = report_dir.with_extension("zip");

    let final_path = if let Some(key_path) = key_path {
        println!("Encrypting report with PGP...");
        match enc::encrypt_file(&zip_path, &key_path) {
            Ok(encrypted_path) => {
                // Remove unencrypted zip after successful encryption
                fs::remove_file(&zip_path).ok();
                encrypted_path
            }
            Err(e) => {
                eprintln!("Warning: Encryption failed: {}. Keeping unencrypted zip.", e);
                zip_path
            }
        }
    } else {
        zip_path
    };

    println!("Report created successfully!");
    println!("Location: {}", final_path.display());

    Ok(Report {
        file: final_path,
    })
}
