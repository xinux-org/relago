pub mod window;

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
    relm4::RelmApp::new("uz.xinux.relago.Reporter")
        .visible_on_activate(false)
        .run::<App>(());
}
