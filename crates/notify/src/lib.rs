pub mod window;
use notify_rust::Notification;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}

pub fn modal(error: Modal) -> Result<(), Box<dyn std::error::Error>> {
    let error_string = serde_json::to_string(&error)?;

    Notification::new()
        .summary("Error")
        .body(format!("{}", error_string).as_str())
        .icon(&error.exe)
        .show()?;

    window::open(error);

    Ok(())
}
