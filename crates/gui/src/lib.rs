pub mod window;

use futures_util::StreamExt;
use notify_rust::Notification;
use std::error::Error;
use window::Modal;
use zbus::{conn::Connection, proxy};

use window::model::App;

#[proxy(
    interface = "org.relago.DaemonService",
    default_service = "org.relago.DaemonService",
    default_path = "/org/relago/DaemonService"
)]
trait DaemonService {
    #[zbus(signal)]
    fn crash_detected(&self, modal: Modal) -> zbus::Result<()>;
    async fn pop_crash(&self) -> zbus::Result<Option<Modal>>;
    async fn has_pending(&self) -> zbus::Result<bool>;
}

pub async fn start_listener() -> Result<(), Box<dyn Error>> {
    let conn = Connection::system().await?;
    let proxy = DaemonServiceProxy::new(&conn).await?;

    let mut stream = proxy.receive_crash_detected().await?;

    println!("Agent is idling");

    while let Some(signal) = stream.next().await {
        match signal.args() {
            Ok(args) => {
                let modal_data = args.modal;

                println!("Signal received! Crash in unit: {}", modal_data.unit);

                // Trigger your notification and UI
                let _notif = Notification::new()
                    .summary("Crash detected")
                    .body(&modal_data.message)
                    .icon("dialog-error")
                    .show();

                spawn_ui(modal_data);
            }
            Err(e) => eprintln!("Failed to parse signal arguments: {}", e),
        }
    }

    Ok(())
}

// Note: It creates new UI with new process id
fn spawn_ui(modal_data: Modal) {
    let app_id = format!("org.relm4.Reporter.p{}", std::process::id());

    let _app = relm4::RelmApp::new(&app_id)
        .with_args(vec![])
        .run::<App>(modal_data);
}
