use super::messages::CmdOut;
use relm4::gtk::{self};
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}

#[derive(Default)]
pub struct App {
    pub computing: bool,
    pub task: Option<CmdOut>,
}

pub struct Widgets {
    pub button: gtk::Button,
    pub button_close: gtk::Button,
    pub progress: gtk::ProgressBar,
    pub label: gtk::Label,
    pub label_pct: gtk::Label,
    pub scroll: gtk::ScrolledWindow,
}
