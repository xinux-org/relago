#![warn(rust_2018_idioms)]

//! Follow future journal log messages and print up to 100 of them.
use anyhow::anyhow;
use systemd::journal::{self, Journal, JournalEntryField, JournalSeek};
use tracing::error;

pub mod fields {
    pub const MESSAGE: &str = "MESSAGE";
    pub const MESSAGE_ID: &str = "MESSAGE_ID";
    pub const PRIORITY: &str = "PRIORITY";
    pub const UNIT: &str = "_SYSTEMD_UNIT";
    pub const PID: &str = "_PID";
    pub const UID: &str = "_UID";
    pub const EXE: &str = "_EXE";
    pub const COMM: &str = "_COMM";
    pub const CMDLINE: &str = "_CMDLINE";
    pub const TRANSPORT: &str = "_TRANSPORT";
    pub const BOOT_ID: &str = "_BOOT_ID";
    pub const ERRNO: &str = "ERRNO";
    pub const CODE_FILE: &str = "CODE_FILE";
    pub const SYSLOG_IDENTIFIER: &str = "SYSLOG_IDENTIFIER";

    // Coredump-specific fields
    pub const COREDUMP_PID: &str = "COREDUMP_PID";
    pub const COREDUMP_EXE: &str = "COREDUMP_EXE";
    pub const COREDUMP_COMM: &str = "COREDUMP_COMM";
    pub const COREDUMP_SIGNAL: &str = "COREDUMP_SIGNAL";
    pub const COREDUMP_SIGNAL_NAME: &str = "COREDUMP_SIGNAL_NAME";
    pub const COREDUMP_FILENAME: &str = "COREDUMP_FILENAME";
    pub const COREDUMP_UID: &str = "COREDUMP_UID";
    pub const COREDUMP_CMDLINE: &str = "COREDUMP_CMDLINE";
}

#[derive(Debug, Debug, Clone)]
pub struct JournalEntry {
    pub message: String,
    pub priority: Option<u8>,
    pub unit: Option<String>,
    pub pid: Option<u32>,
    pub exe: Option<String>,
    pub comm: Option<String>,
    pub transport: Option<String>,
    pub errno: Option<i32>,
}

pub fn get_field(journal: &mut Journal, field: &str) -> Option<String> {
    let entry = journal.get_data(field).ok()??;
    entry
        .value()
        .map(String::from_utf8_lossy)
        .map(|v| v.into_owned())
}

/// Extract a structured [`JournalEntry`] from the current journal position.
pub fn extract_entry(journal: &mut Journal) -> Option<JournalEntry> {
    let message = get_field(journal, fields::MESSAGE)?;

    Some(JournalEntry {
        message,
        priority: get_field(journal, fields::PRIORITY).and_then(|s| s.parse().ok()),
        unit: get_field(journal, fields::UNIT),
        pid: get_field(journal, fields::PID).and_then(|s| s.parse().ok()),
        exe: get_field(journal, fields::EXE),
        comm: get_field(journal, fields::COMM),
        transport: get_field(journal, fields::TRANSPORT),
        errno: get_field(journal, fields::ERRNO).and_then(|s| s.parse().ok()),
    })
}
pub struct JournalTail {
    journal: Journal,
}

impl JournalTail {
    /// Open the journal and seek to the tail (only new entries).
    pub fn open() -> anyhow::Result<Self> {
        let mut journal = journal::OpenOptions::default()
            .open()
            .map_err(|e| anyhow!("Could not open journal: {e}"))?;

        journal
            .seek(JournalSeek::Tail)
            .map_err(|_| anyhow!("Could not seek to end of journal"))?;

        // Tail points past the last entry, step back to it
        journal.previous()?;

        Ok(Self { journal })
    }

    pub fn add_match(mut self, field: &str, value: &str) -> anyhow::Result<Self> {
        self.journal
            .match_add(field, value)
            .map_err(|e| anyhow!("Failed to add match {field}={value}: {e}"))?;
        Ok(self)
    }

}


impl Iterator for JournalTail {
    type Item = JournalEntry;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.journal.next() {
                Ok(0) => {
                    if let Err(err) = self.journal.wait(None) {
                        error!(error = %err, "Failed to wait on journal");
                        return None;
                    }
                }
                Ok(_) => {
                    if let Some(entry) = extract_entry(&mut self.journal) {
                        return Some(entry);
                    }
                }
                Err(err) => {
                    error!(error = %err, "Failed to read next journal entry");
                    return None;
                }
            }
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting journal-logger");

    let tail = JournalTail::open()?;
    for entry in tail {
        println!("{entry:#?}");
    }

    Ok(())
}
