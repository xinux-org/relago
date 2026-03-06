use anyhow::anyhow;
use serde_json;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use systemd::journal::{self, JournalSeek};

const TMP: &str = "/tmp/relago/journal_export.json";

pub fn run() -> anyhow::Result<()> {
    export_to_file(TMP)
}
/// This function for exporting entries to file
pub fn export_to_file(path: &str) -> anyhow::Result<()> {
    println!("Exporting all journal entries...");

    let mut reader = journal::OpenOptions::default()
        .open()
        .map_err(|e| anyhow!("Could not open journal: {e}"))?;

    reader
        .seek(JournalSeek::Head)
        .map_err(|e| anyhow!("Could not seek to head of journal: {e}"))?;

    let mut entries: Vec<BTreeMap<String, String>> = Vec::new();
    let mut count = 0;

    loop {
        match reader.next_entry()? {
            Some(entry) => {
                entries.push(entry);
                count += 1;

                if count % 1000 == 0 {
                    eprint!("\rProcessed {} entries...", count);
                }
            }
            None => {
                break;
            }
        }
    }

    eprintln!("\rTotal entries collected: {}", entries.len());

    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to JSON file
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &entries)?;

    println!("Exported {} entries to: {}", entries.len(), path);

    Ok(())
}

/// This function for exporting only recent N entries to file
/// if recent entries not needed you can commit this function
/// or just will not add extra arguments for export command like `-r number`
pub fn export_recent(path: &str, num_entries: usize) -> anyhow::Result<()> {
    println!("Exporting {} recent journal entries...", num_entries);

    let mut reader = journal::OpenOptions::default()
        .open()
        .map_err(|e| anyhow!("Could not open journal: {e}"))?;

    // Seek to end
    reader
        .seek(JournalSeek::Tail)
        .map_err(|e| anyhow!("Could not seek to tail: {e}"))?;

    let mut entries: Vec<BTreeMap<String, String>> = Vec::new();

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

    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write to JSON
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &entries)?;

    println!("Exported {} entries to: {}", entries.len(), path);

    Ok(())
}
