use anyhow::Context;
use anyhow::Result;
use ignore::WalkBuilder;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use sysinfo::{Disks, Networks, System};
use systemd::journal::{self, JournalSeek};

#[derive(Serialize, Debug, Clone, Hash, PartialEq, Eq)]
struct JournalLog {
    timestamp: String,
    entry: BTreeMap<String, String>,
}

#[derive(Serialize)]
pub struct SystemInfo {
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub system_name: Option<String>,
    pub kernel_version: Option<String>,
    pub os_version: Option<String>,
    pub host_name: Option<String>,
    pub cpu_vendor: String,
    pub cpu_brand: String,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<String>,
}

#[derive(Serialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
}

pub fn collect_system_info() -> Result<SystemInfo> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let disks = Disks::new_with_refreshed_list();
    let disk_info: Vec<DiskInfo> = disks
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().to_string(),
            mount_point: disk.mount_point().to_string_lossy().to_string(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            file_system: disk.file_system().to_string_lossy().to_string(),
        })
        .collect();

    let networks = Networks::new_with_refreshed_list();
    let network_info: Vec<String> = networks.iter().map(|(name, _data)| name.clone()).collect();

    sys.refresh_cpu_usage();
    let cpu_vendor = sys
        .cpus()
        .first()
        .map(|cpu| cpu.vendor_id().to_string())
        .unwrap_or_default();
    let cpu_brand = sys
        .cpus()
        .first()
        .map(|cpu| cpu.brand().to_string())
        .unwrap_or_default();

    Ok(SystemInfo {
        total_memory: sys.total_memory(),
        used_memory: sys.used_memory(),
        total_swap: sys.total_swap(),
        used_swap: sys.used_swap(),
        system_name: System::name(),
        kernel_version: System::kernel_version(),
        os_version: System::os_version(),
        host_name: System::host_name(),
        cpu_vendor,
        cpu_brand,
        disks: disk_info,
        networks: network_info,
    })
}

pub fn collect_journal_all(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let mut reader = journal::OpenOptions::default()
        .open()
        .context("Could not open journal")?;

    reader
        .seek(JournalSeek::Head)
        .context("Could not seek to head of journal")?;

    let mut count: usize = 0;

    while let Some(entry) = reader.next_entry()? {
        let writable: JournalLog = JournalLog {
            // NOTE:
            // We're using timestamp as u64.
            // Because default Journal.timestamp() uses EPOCH standard in SystemTime struct.
            // Though we're sending it via API, we decided to use u64 version to not to load client application
            timestamp: reader.timestamp_usec()?.to_string(),
            entry: entry,
        };

        serde_json::to_writer(&mut writer, &writable)?;
        writeln!(writer)?;

        count += 1;

        if count.is_multiple_of(1000) {
            eprint!("\rProcessed {} entries...", count);
        }
    }

    writer.flush()?;
    println!("\nCollected {} journal entries", count);

    Ok(())
}

pub fn collect_journal_recent(path: &Path, num_entries: usize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to JSON
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    let mut reader = journal::OpenOptions::default()
        .open()
        .context("Could not open journal")?;

    // Seek to end
    reader
        .seek(JournalSeek::Tail)
        .context("Could not seek to tail")?;

    let mut entries: HashSet<JournalLog> = HashSet::new();

    for _count in 0..num_entries {
        if reader.previous()? == 0 {
            break;
        }

        if let Some(entry) = reader.previous_entry()? {
            let writable: JournalLog = JournalLog {
                // NOTE:
                // We're using timestamp as u64.
                // Because default Journal.timestamp() uses EPOCH standard in SystemTime struct.
                // Though we're sending it via API, we decided to use u64 version to not to load client application
                timestamp: reader.timestamp_usec()?.to_string(),
                entry: entry,
            };

            entries.insert(writable.clone());
            println!("{:?}", &writable);
        };
    }

    serde_json::to_writer(&mut writer, &entries)?;
    writeln!(writer)?;

    writer.flush()?;

    println!("Collected {} journal entries", entries.len());

    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    let walker = WalkBuilder::new(src).hidden(false).build();

    for entry in walker {
        let entry = entry?;
        let src_path = entry.path();

        let relative = src_path.strip_prefix(src).unwrap_or(src_path);
        let dest_path = dest.join(relative);

        if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
            fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(src_path, &dest_path)?;
        }
    }

    Ok(())
}
