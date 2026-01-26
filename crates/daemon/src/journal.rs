#![warn(rust_2018_idioms)]

//! Follow future journal log messages and print up to 100 of them.
use anyhow::anyhow;
use systemd::journal::{self, Journal, JournalEntryField, JournalSeek};
use tracing::error;

const KEY_UNIT: &str = "_SYSTEMD_UNIT";
const KEY_MESSAGE: &str = "MESSAGE";
const KEY_PRIORITY: &str = "PRIORITY";
const KEY_CODE_FILE: &str = "CODE_FILE";
const KEY_ERRNO: &str = "ERRNO";
const KEY_SYSLOG_RAW: &str = "SYSLOG_RAW";
const KEY_STDOUT_ERR: &str = "_TRANSPORT=stdout";

const MAX_MESSAGES: usize = 5;

#[derive(Debug)]
struct JournalItem {
    message: String,
    // message_id: String,
    priority: String,
    unit: Option<String>,
    errno: Option<String>,
    std_err: Option<String>,
}

fn from_journal_fields(mut journal: Journal) -> JournalItem {
    let keys = vec![KEY_UNIT, KEY_MESSAGE];

    // let a =  journal.get_data(field)
    todo!()
}

// TODO: Need handle errror when illegal journal field
fn get_field_from_journal(journal: &mut Journal, j_field: &str) -> Option<String> {
    let Some(Some(entry)) = journal.get_data(j_field).ok() else {
        return None;
    };

    // TODO: need match option
    return entry
        .value()
        .map(String::from_utf8_lossy)
        .map(|v| v.into_owned());
}
struct J {
    journal: Journal,
}

impl J {
    pub fn new(mut journal: Journal) -> anyhow::Result<Self> {
        // Seek to end of current log to prevent old messages from being printed
        journal
            .seek(JournalSeek::Tail)
            .map_err(|_| anyhow!("Could not seek to end of journal"))?;

        // JournalSeek::Tail goes to the position after the most recent entry so step back to
        // point to the most recent entry.
        journal.previous()?;

        Ok(Self { journal })
    }
}

impl Iterator for J {
    type Item = JournalItem;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.journal.next() {
                Ok(0) => {
                    if let Err(err) = self.journal.wait(None) {
                        error!(error = %err, "failed to wait on journal");
                        return None;
                    }
                }
                Ok(_) => {
                    let message = get_field_from_journal(&mut self.journal, KEY_MESSAGE)?;

                    let unit = get_field_from_journal(&mut self.journal, KEY_UNIT);
                    // let unit = "someUNit".to_string();

                    let priority = get_field_from_journal(&mut self.journal, KEY_PRIORITY)?;
                    let errno = get_field_from_journal(&mut self.journal, KEY_ERRNO);
                    let std_err = get_field_from_journal(&mut self.journal, KEY_STDOUT_ERR);

                    println!("{}", priority);

                    let jr = JournalItem {
                        message,
                        priority,
                        unit,
                        errno,
                        std_err,
                    };

                    return Some(jr);
                }
                Err(err) => {
                    error!(error = %err, "failed to get next on journal");
                    return None;
                }
            }
        }
    }
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting journal-logger");

    // Open the journal
    let journal = journal::OpenOptions::default()
        .open()
        .expect("Could not open journal");

    let j = J::new(journal)?;

    let mut a = j.into_iter();
    while let Some(log) = a.next() {
        println!("{log:#?}");
    }

    Ok(())
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    main()
}
