use super::messages::CmdOut;
use relm4::gtk::{self, prelude::*};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    #[serde(rename = "Unit")]
    pub unit: String,
    #[serde(rename = "Executable")]
    pub exe: String,
    #[serde(rename = "Message")]
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
