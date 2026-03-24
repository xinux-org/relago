pub mod window;

use notify_rust::Notification;
use std::process::Command;

pub fn modal(unit: &str, exe: &str, message: &str) -> anyhow::Result<()> {
    let exe_path = std::env::current_exe()?;
    let wayland = find_wayland_display().unwrap_or_else(|| "wayland-1".to_string());

    println!("{}", exe_path.to_str().unwrap());

    Command::new(exe_path.to_str().unwrap())
        .args([
            // "--user",
            // "--machine=1000@",
            // &format!("--setenv=WAYLAND_DISPLAY={wayland}"),
            // "--setenv=XDG_RUNTIME_DIR=/run/user/1000",
            // "--setenv=DISPLAY=:0",
            // "--",
            "reporter",
            "-u",
            &format!("{unit}"),
            "-e",
            &format!("{exe}"),
            "-m",
            &format!("{message}"),
        ])
        .spawn()?;

    Notification::new()
        .summary("Crash detected")
        .body(message)
        .icon("dialog-error")
        .show()?;

    Ok(())
}

fn find_wayland_display() -> Option<String> {
    std::fs::read_dir("/run/user/1000/").ok()?.find_map(|e| {
        let e = e.ok()?;
        let name = e.file_name();
        let name = name.to_str()?;
        (name.starts_with("wayland-") && !name.ends_with(".lock")).then(|| name.to_string())
    })
}
