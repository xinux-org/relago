//! Follow future journal log messages and print up to 100 of them.
use anyhow::anyhow;
use gnome_relago::window::Modal;
use systemd::journal::{self, JournalSeek};
use zbus::{conn, proxy};

use crate::crash::{CoredumpCrash, Crash, OomCrash, ServiceFailureCrash};
use crate::registry::PluginRegistry;

#[proxy(
    interface = "org.relago.ReportHandler",
    default_service = "org.relago.ReportService",
    default_path = "/org/relago/ReportService"
)]
trait ReportService {
    async fn report(&self, data: Modal) -> zbus::Result<()>;
}

#[derive(Debug)]
enum ReportRes {
    Done,
}

pub async fn run() -> anyhow::Result<()> {
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

    // journal.previous()?;

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
                Some(Crash::Coredump(ref r)) => {
                    let modal_data = Modal {
                        unit: r.unit.as_deref().unwrap_or("unknown").to_string(),
                        exe: r.exe.clone(),
                        message: format!("Process crashed with a coredump."),
                    };

                    tokio::spawn(async move {
                        match zbus::Connection::session().await {
                            Ok(conn) => {
                                let proxy = ReportServiceProxy::new(&conn).await.unwrap();
                                if let Err(e) = proxy.report(modal_data).await {
                                    eprintln!("Failed to send crash report to GUI: {}", e);
                                } else {
                                    println!("Crash report sent to Gnome service.");
                                }
                            }
                            Err(e) => {
                                eprintln!("Could not connect to Session Bus: {}. Is a GUI session running?", e);
                            }
                        }
                    });
                }

                Some(Crash::ServiceFailure(r)) => {
                    println!("Service failed: {:?}", r);
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

    // Ok(())
}
