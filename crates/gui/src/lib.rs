pub mod window;

use relm4::{adw, gtk::gio};
use window::Modal;
use zbus::proxy;

use crate::window::model::App;

#[proxy(
    interface = "org.relago.DaemonService",
    default_service = "org.relago.DaemonService",
    default_path = "/org/relago/DaemonService"
)]
pub trait DaemonService {
    #[zbus(signal)]
    fn crash_detected(&self, modal: Modal) -> zbus::Result<()>;
    async fn pop_crash(&self) -> zbus::Result<Option<Modal>>;
    async fn has_pending(&self) -> zbus::Result<bool>;
}

pub fn start_gui() {
    let app = adw::Application::new(
        Some("uz.xinux.relago.Reporter"),
        gio::ApplicationFlags::empty(),
    );

    relm4::RelmApp::from_app(app)
        .visible_on_activate(false)
        .run::<App>(());
}
