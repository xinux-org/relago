pub mod compress;
pub mod info;

use compress as cmp;
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
) -> anyhow::Result<()> {
    let _ = create_report(output_dir, nixos_config_path, recent_entries);
    Ok(())
}

pub fn create_report(
    output_dir: &str,
    nixos_config_path: Option<&str>,
    recent_entries: Option<usize>,
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
    if system_info.system_name.to_owned() == Some("XinuxOS".to_string()) {
        let src = Path::new("/etc/nixos");
        let dest = report_dir.join("xinux-config");
        info::copy_dir_recursive(&src, &dest);
    }

    // TODO: delete original file after compressed
    let _ = cmp::compress_zip(&report_dir, &output_dir);

    println!("Report created successfully!");
    println!("Location: {}", report_dir.display());

    Ok(Report {
        file: report_dir.to_owned(),
    })
}
