pub mod window;

use notify_rust::Notification;
use std::process::Command;

pub fn modal(unit: &str, exe: &str, message: &str) -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?;

    let wayland = std::fs::read_dir("/run/user/1000/")
        .ok()
        .and_then(|mut d| {
            d.find_map(|e| {
                let e = e.ok()?;
                let name = e.file_name();
                let name = name.to_str()?;
                if name.starts_with("wayland-") && !name.ends_with(".lock") {
                    Some(name.to_string())
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| "wayland-1".to_string());

    Command::new("systemd-run")
        .args([
            "--user",
            "--machine=1000@",
            &format!("--setenv=WAYLAND_DISPLAY={}", wayland),
            "--setenv=XDG_RUNTIME_DIR=/run/user/1000",
            "--setenv=DISPLAY=:0",
            &format!("--setenv=RELAGO_UNIT={}", unit),
            &format!("--setenv=RELAGO_EXE={}", exe),
            &format!("--setenv=RELAGO_MESSAGE={}", message),
            "--",
            exe_path.to_str().unwrap(),
            "reporter",
        ])
        .spawn()?;

    Notification::new()
        .summary("Crash detected")
        .body(message)
        .icon("dialog-error")
        .show()?;

    Ok(())
}
