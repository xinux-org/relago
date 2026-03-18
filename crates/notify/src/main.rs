pub mod window;

use notify_rust::Notification;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Modal {
    pub unit: String,
    pub exe: String,
    pub message: String,
}

fn main() -> anyhow::Result<()> {
    let unit = "".to_string();
    let exe = "daxshat".to_string();

    let rr = Modal {
        unit,
        exe,
        message: "Coredump error".to_string(),
    };

    let error_string = serde_json::to_string(&rr).expect("Modal cooked");

    Notification::new()
        .summary("Error")
        .body(format!("{}", error_string).as_str())
        .icon(&rr.exe)
        .show()
        .expect("Notify cooked");

    window::open(rr);

    Ok(())
}
