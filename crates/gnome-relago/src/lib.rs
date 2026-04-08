pub mod window;
use std::{error::Error, time::Duration};

use notify_rust::Notification;
use window::Modal;
use zbus::{conn, proxy};

use window::model::App;

#[proxy(
    interface = "org.relago.DaemonService",
    default_service = "org.relago.DaemonService",
    default_path = "/org/relago/DaemonService"
)]
trait DaemonService {
    async fn pop_crash(&self) -> zbus::Result<Option<Modal>>;
    async fn has_pending(&self) -> zbus::Result<bool>;
}

pub async fn start_listener() -> Result<conn::Connection, Box<dyn Error>> {
    let conn = zbus::Connection::system().await?;
    let proxy = DaemonServiceProxy::new(&conn).await?;

    loop {
        match proxy.has_pending().await {
            Ok(true) => {
                match proxy.pop_crash().await {
                    // show notification
                    Ok(m) => match m {
                        Some(modal_data) => {
                            let _notif = Notification::new()
                                .summary("Crash detected")
                                .body(&(modal_data.message))
                                .icon("dialog-error")
                                .show();

                            println!("Showing crash notification: {:?}", modal_data);

                            let app_id = format!("org.relm4.Reporter.p{}", std::process::id());

                            let _app = relm4::RelmApp::new(&app_id)
                                .with_args(vec![])
                                .run::<App>(modal_data);
                        }
                        _ => {
                            println!("Mana yana Option error");
                        }
                    },
                    Err(_e) => {
                        println!("Ma naxuy error");
                    } // call your existing notification/window code here
                }
            }
            Ok(false) => {}
            Err(e) => eprintln!("Failed to poll daemon: {}", e),
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
