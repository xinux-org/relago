pub mod window;
use notify_rust::Notification;
use serde::Serialize;
use anyhow::anyhow;

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}

pub fn modal(error: Modal) -> Option<()> {
    let error_string = serde_json::to_string(&error).expect("Modal cooked");

    Notification::new()
        .summary("Error")
        .body(format!("{}", error_string).as_str())
        .icon(&error.exe)
        .show().expect("Notify cooked");
    window::open(error);

    Some(())
}
