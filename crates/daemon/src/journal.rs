//! Follow future journal log messages and print up to 100 of them.
use std::sync::Arc;

use anyhow::anyhow;
use gui::window::Modal;
use systemd::journal::{self, JournalSeek};
use tokio::sync::Mutex;
use zbus::interface;
use zbus::object_server::SignalEmitter;

use crate::crash::{CoredumpCrash, Crash, OomCrash, ServiceFailureCrash};
use crate::registry::PluginRegistry;

pub struct CrashQueue {
    pub pending: Arc<Mutex<Vec<Modal>>>,
}

impl Clone for CrashQueue {
    fn clone(&self) -> Self {
        Self {
            pending: Arc::clone(&self.pending),
        }
    }
}

#[interface(name = "org.relago.DaemonService")]
impl CrashQueue {
    #[zbus(signal)]
    async fn crash_detected(signal_emitter: &SignalEmitter<'_>, modal: Modal) -> zbus::Result<()>;

    async fn pop_crash(&self) -> Option<Modal> {
        self.pending.lock().await.pop()
    }

    async fn has_pending(&self) -> bool {
        !self.pending.lock().await.is_empty()
    }
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

    let shared_pending: Arc<Mutex<Vec<Modal>>> = Arc::new(Mutex::new(vec![]));

    let queue_for_dbus = CrashQueue {
        pending: Arc::clone(&shared_pending),
    };

    let conn = zbus::connection::Builder::system()?
        .name("org.relago.DaemonService")?
        .serve_at("/org/relago/DaemonService", queue_for_dbus)?
        .build()
        .await?;

    let emitter = SignalEmitter::new(&conn, "/org/relago/DaemonService")?;

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
                Some(Crash::Coredump(ref r)) => {
                    let modal_data = Modal {
                        unit: r.unit.as_deref().unwrap_or("unknown").to_string(),
                        exe: r.exe.clone(),
                        message: format!("Process crashed with a coredump."),
                    };

                    emitter.crash_detected(modal_data).await?;

                    println!("Crash queued for gnome agent.");
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
