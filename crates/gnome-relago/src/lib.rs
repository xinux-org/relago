pub mod window;
use std::error::Error;

use notify_rust::Notification;
use window::Modal;
use zbus::{conn, interface};

use window::model::App;

struct ReportService;

#[interface(name = "org.relago.ReportHandler")]
impl ReportService {
    async fn report(&self, data: Modal) {
        let _notif = Notification::new()
            .summary("Crash detected")
            .body(&(data.message))
            .icon("dialog-error")
            .show();

        let app_id = format!("org.relm4.Reporter.p{}", std::process::id());

        let _app = relm4::RelmApp::new(&app_id)
            .with_args(vec![])
            .run::<App>(data);
    }
}

pub async fn start_listener() -> Result<conn::Connection, Box<dyn Error>> {
    let service = ReportService;

    let conn = conn::Builder::session()?
        .name("org.relago.ReportService")?
        .serve_at("/org/relago/ReportService", service)?
        .build()
        .await?;
    Ok(conn)
}
