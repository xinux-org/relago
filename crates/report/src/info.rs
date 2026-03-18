use anyhow::{anyhow, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use sysinfo::{Disks, Networks, System};
use systemd::journal::{self, JournalSeek};
use serde_json::Serializer;
use serde::ser::SerializeSeq;
use serde::Serializer as _;
use ignore::WalkBuilder;

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
    let network_info: Vec<String> = networks
        .iter()
        .map(|(name, _data)| name.clone())
        .collect();

    sys.refresh_cpu_usage();
    let cpu_vendor = sys.cpus().first()
        .map(|cpu| cpu.vendor_id().to_string())
        .unwrap_or_default();
    let cpu_brand = sys.cpus().first()
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
    let writer = BufWriter::new(file);
    let mut ser = Serializer::pretty(writer);
    let mut seq = ser.serialize_seq(None)?;

    let mut reader = journal::OpenOptions::default()
        .open()
        .map_err(|e| anyhow!("Could not open journal: {e}"))?;

    reader
        .seek(JournalSeek::Head)
        .map_err(|e| anyhow!("Could not seek to head of journal: {e}"))?;

    let mut count: usize = 0;

    while let Some(entry) = reader.next_entry()? {
        seq.serialize_element(&entry)?;
        count += 1;

        if count % 1000 == 0 {
            eprint!("\rProcessed {} entries...", count);
        }
    }

    seq.end()?;
    println!("\nCollected {} journal entries", count);

    Ok(())
}

pub fn collect_journal_recent(path: &Path, num_entries: usize) -> Result<()> {
    let mut reader = journal::OpenOptions::default()
        .open()
        .map_err(|e| anyhow!("Could not open journal: {e}"))?;

    // Seek to end
    reader
        .seek(JournalSeek::Tail)
        .map_err(|e| anyhow!("Could not seek to tail: {e}"))?;

    let mut entries: Vec<BTreeMap<String, String>> = Vec::with_capacity(num_entries);

    for _ in 0..num_entries {
        if reader.previous()? == 0 {
            break;
        }

        let mut entry_map: BTreeMap<String, String> = BTreeMap::new();

        reader.restart_data();
        while let Some(field) = reader.enumerate_data()? {
            let name = String::from_utf8_lossy(field.name()).into_owned();
            if let Some(value) = field.value() {
                let value_str = String::from_utf8_lossy(value).into_owned();
                entry_map.insert(name, value_str);
            }
        }

        if !entry_map.is_empty() {
            entries.push(entry_map);
        }
    }


    // Reverse to get chronological order because we start seek end of the journal
    entries.reverse();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write to JSON
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &entries)?;

    println!("Collected {} journal entries", entries.len());

    Ok(())
}

pub fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {

    let walker = WalkBuilder::new(src)
        .hidden(false)
        .build();

    for entry in walker {
        let entry = entry?;
        let src_path = entry.path();

        let relative = src_path
            .strip_prefix(src)
            .unwrap_or(src_path);
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
