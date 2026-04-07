use futures_util::stream::StreamExt;
use window::model::{App, Modal};
use zbus::{proxy, Connection};

pub mod window;

#[proxy(
    interface = "org.relago.gnome.Report",
    default_service = "org.relago.gnome",
    default_path = "/org/relago/gnome"
)]
trait GnomeReport {
    #[zbus(signal)]
    fn modal_signal(&self, modal: Modal) -> zbus::Result<()>;
}

pub async fn start_listener() -> zbus::Result<()> {
    let connection = Connection::system().await?;
    let proxy = GnomeReportProxy::new(&connection).await?;
    let mut modal_stream = proxy.receive_modal_signal().await?;

    while let Some(msg) = modal_stream.next().await {
        let args: ModalSignalArgs = msg.args().expect("Error parsing message");
        let modal = args.modal.clone();
        println!("{:?}", modal);
        let app_id = format!("org.relago.Reporter.p{}", std::process::id());
        relm4::RelmApp::new(&app_id).run::<App>(modal);
    }

    panic!("Stream ended unexpectedly");
}
