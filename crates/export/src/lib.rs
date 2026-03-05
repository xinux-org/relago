use anyhow::anyhow;
use systemd::journal::{self, Journal, JournalEntryField, JournalSeek};

pub fn run() -> anyhow::Result<()> {
    println!("exporting...");

    let mut reader = journal::OpenOptions::default()
        .open()
        .expect("Could not open journal");

    reader
        .seek(JournalSeek::Head)
        .expect("Could not seek to end of journal");

    let mut i = 0;
    loop {
        loop {
            let rs = reader.next_entry()?;

            match rs {
                Some(t) => {
                    let r = t.keys();
                    println!("KEYS: {:?}", r);
                }
                None => {
                    println!("none");
                    return Ok(());
                }
            }

            i += 1;
            if i >= 6 {
                eprintln!("done.");
                return Ok(());
            }
        }

        reader.wait(None)?;
    }
    Ok(())
}
