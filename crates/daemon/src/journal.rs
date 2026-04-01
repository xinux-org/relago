//! Follow future journal log messages and print up to 100 of them.
use anyhow::anyhow;
use std::thread;
use systemd::journal::{self, JournalSeek};

use crate::crash::{CoredumpCrash, Crash, OomCrash, ServiceFailureCrash};
use crate::registry::PluginRegistry;

pub fn run() -> anyhow::Result<()> {
    let mut registry = PluginRegistry::new();
    registry
        .register(
            CoredumpCrash::filters(),
            CoredumpCrash::detect,
            Crash::Coredump,
        )
        .register(
            ServiceFailureCrash::filters(),
            ServiceFailureCrash::detect,
            Crash::ServiceFailure,
        )
        .register(OomCrash::filters(), OomCrash::detect, Crash::Oom);

    // ── Open journal ──────────────────────────────────────────────────────────

    let mut journal = journal::OpenOptions::default()
        .open()
        .map_err(|e| anyhow!("could not open journal: {e}"))?;

    // Seek to tail — only follow new entries from this point forward.
    journal
        .seek(JournalSeek::Tail)
        .map_err(|e| anyhow!("journal seek failed: {e}"))?;

    journal.previous()?;

    registry.install_filters(&mut journal)?;

    loop {
        match journal.next() {
            // 0 means "no new entries yet" — block until journald wakes us.
            Ok(0) => {
                journal
                    .wait(None) // None = block indefinitely
                    .map_err(|e| anyhow!("journal wait failed: {e}"))?;
            }

            Ok(_) => match registry.run(&mut journal) {
                Some(ref cr @ Crash::Coredump(ref r)) => {
                    let _ = handle_crash(cr);
                    println!("Core dumped: {:?}", r);
                }

                Some(Crash::ServiceFailure(_r)) => {
                    // if r.job_result == "done" {
                    //     continue;
                    // }

                    println!("Service failed");
                }

                Some(Crash::Oom(r)) => {
                    println!("Out of memory: {:?}", r);
                }

                None => {
                    println!("entry passed filter but no plugin matched");
                }
            },

            Err(e) => {
                return Err(e.into());
            }
        }
    }
}

fn handle_crash(cr: &Crash) -> anyhow::Result<()> {
    match cr {
        Crash::Coredump(_dump) => {
            thread::spawn(move || {
                println!("Handler called inside thread");
            });
        }
        Crash::ServiceFailure(_r) => {
            println!("Service failed");
        }
        Crash::Oom(_r) => {}
    }
    Ok(())
}
