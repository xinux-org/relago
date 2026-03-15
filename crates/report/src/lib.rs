pub mod compress;
pub mod info;

use std::fs::{self, File};
use std::path::PathBuf;
use compress as cmp;

pub fn run() -> anyhow::Result<()> {
    create_report("/tmp/relago", None, None)
}

pub fn create_report(
    output_dir: &str,
    nixos_config_path: Option<&str>,
    recent_entries: Option<usize>,
) -> anyhow::Result<()> {

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let report_dir = PathBuf::from(output_dir).join(format!("report_{}", timestamp));

    println!("Creating report directory: {}", report_dir.display());
    fs::create_dir_all(&report_dir)?;

    // 1. Collect and save system information
    println!("Collecting system information...");
    let system_info = info::collect_system_info()?;
    let system_info_path = report_dir.join("system_info.json");
    let file = File::create(&system_info_path)?;
    serde_json::to_writer_pretty(file, &system_info)?;
    println!("System info saved: {}", system_info_path.display());

    // 2. Collect journal entries
    let journal_path = report_dir.join("journal_report.json");
    if let Some(num) = recent_entries {
        info::collect_journal_recent(&journal_path, num)?;
    } else {
        info::collect_journal_all(&journal_path)?;
    }

    // Compress .json then remove it
    println!("Compressing journal file...");
    cmp::compress(&journal_path, &report_dir)?;
    fs::remove_file(&journal_path)?;

    // 3. Copy NixOS configuration if provided
    if let Some(config_path) = nixos_config_path {
        let config_path = shellexpand::tilde(config_path).to_string();
        let src = PathBuf::from(&config_path);

        if !src.exists() {
            eprintln!("Warning: NixOS config path does not exist: {}", config_path);
        } else {
            println!("Copying NixOS configuration from: {}", src.display());
            let dest = report_dir.join("nixos-config");
            info::copy_dir_recursive(&src, &dest)?;
            println!("NixOS config copied: {}", dest.display());
        }
    }

    println!("Report created successfully!");
    println!("Location: {}", report_dir.display());

    Ok(())
}
