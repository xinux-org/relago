pub mod compress;

use anyhow::anyhow;
use serde_json::Serializer;
use serde::ser::SerializeSeq;
use serde::Serializer as _;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufWriter;
use systemd::journal::{self, JournalSeek};
use compress as cmp;
use std::path::Path;

// TODO: Maybe good move to config ?
const TMP: &str = "/tmp/relago/journal_report.json";

pub fn run() -> anyhow::Result<()> {
    report_to_file(TMP)
}
/// This function for reporting entries to file
pub fn report_to_file(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();
    println!("Reporting all journal entries...");

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
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

    println!("Reported {} entries to: {}", count, path.display());

    let dest = path.parent().unwrap_or(Path::new("/tmp/relago"));
    cmp::compress(path, dest)?;
    Ok(())
}

pub fn report_recent(path: impl AsRef<Path>, num_entries: usize) -> anyhow::Result<()> {
    let path = path.as_ref();
    println!("Reporting {} recent journal entries...", num_entries);

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
        std::fs::create_dir_all(parent)?;
    }

    // Write to JSON
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &entries)?;

    println!("Reported {} entries to: {}", entries.len(), path.display());

    Ok(())
}
