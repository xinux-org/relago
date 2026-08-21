pub(crate) mod locales;
pub mod window;

use crate::locales::{CACHE, DEFAULT_LC, LOCALES, try_detect_language};
use crate::window::App;
use crate::window::Modal;

use fluent_zero::{set_lang, t};
use futures_util::StreamExt;
use notify_rust::Notification;
use relm4::adw;
use relm4::gtk::gio;
use zbus::Connection;
use zbus::proxy;

#[proxy(
    interface = "org.relago.DaemonService",
    default_service = "org.relago.DaemonService",
    default_path = "/org/relago/DaemonService"
)]
pub trait DaemonService {
    // crash_detected async or sync ??
    #[zbus(signal)]
    async fn crash_detected(&self, modal: Modal) -> zbus::Result<()>;
    async fn pop_crash(&self) -> zbus::Result<Option<Modal>>;
    async fn has_pending(&self) -> zbus::Result<bool>;
}

pub async fn start_listener() -> anyhow::Result<()> {
    let conn = Connection::system().await?;
    let proxy = DaemonServiceProxy::new(&conn).await?;
    let mut stream = proxy.receive_crash_detected().await?;
    println!("Agent is idling");

    set_lang(try_detect_language().unwrap_or_else(|| DEFAULT_LC.clone()));

    while let Some(signal) = stream.next().await {
        match signal.args() {
            Ok(args) => {
                let modal_data = args.modal;
                println!("Signal received! Crash in unit: {}", modal_data.unit);

                if let Err(e) = Notification::new()
                    .summary(&t!("notification-crash-detected"))
                    .body(&modal_data.message)
                    .icon("dialog-error")
                    .show()
                {
                    eprintln!("Failed to show notification: {e}");
                }

                reporter(&modal_data);
            }
            Err(e) => eprintln!("Failed to parse signal arguments: {e}"),
        }
    }
    Ok(())
}

fn reporter(modal: &Modal) {
    let exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("cannot locate own executable to launch reporter: {e}");
            return;
        }
    };

    let spawn = tokio::process::Command::new(exe)
        .arg("reporter")
        .arg("-u")
        .arg(&modal.unit)
        .arg("-e")
        .arg(&modal.exe)
        .arg("-m")
        .arg(&modal.message)
        .spawn();

    match spawn {
        Ok(mut child) => {
            tokio::spawn(async move {
                if let Err(e) = child.wait().await {
                    eprintln!("Failed to wait on reporter GUI: {e}");
                }
            });
        }
        Err(e) => eprintln!("Failed to launch reporter GUI: {e}"),
    }
}

pub fn start_gui(modal: Modal) {
    let app = adw::Application::new(
        Some("org.relago.Reporter"),
        gio::ApplicationFlags::NON_UNIQUE,
    );

    relm4::RelmApp::from_app(app)
        .with_args(vec![])
        .run::<App>(modal);
}
