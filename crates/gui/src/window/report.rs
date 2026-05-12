use super::messages::CmdCrashOut;
use super::model::App;
use futures_util::FutureExt;
use relm4::ComponentSender;
use report::create_report;
use reqwest::blocking::multipart;
use utils::config::CONFIG;

pub fn run(sender: ComponentSender<App>, context: Option<String>) {
    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.05,
                        message: "Reading journal entries…".into(),
                    }
                    .into(),
                );

                let keys = format!("{}/key.pub", CONFIG.get().keys.display());
                let nix_config = CONFIG.get().nix_config.to_string_lossy().into_owned();

                let rep_file = tokio::task::spawn_blocking(move || {
                    create_report(
                        CONFIG.get().tmp_dir.clone().to_str().unwrap(),
                        Some(CONFIG.get().nix_config.clone().to_str().unwrap()),
                        None,
                        Some(&keys),
                    )
                })
                .await;

                let rep_file = match rep_file {
                    Ok(r) => r,
                    Err(e) => {
                        let _ = out.send(CmdOut::Error(format!("Report task failed: {e}")));
                        return;
                    }
                };

                let path = match rep_file {
                    Err(e) => {
                        out.emit(
                            CmdCrashOut::Error(format!("Failed to collect report: {e}")).into(),
                        );
                        return;
                    }
                    Ok(f) => {
                        out.emit(
                            CmdCrashOut::Progress {
                                fraction: 0.3,
                                message: "Report collected, compressing…".into(),
                            }
                            .into(),
                        );

                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                        let zip_path = f.file.display().to_string();

                        out.emit(
                            CmdCrashOut::Progress {
                                fraction: 0.55,
                                message: format!(
                                    "Compressed → {}",
                                    zip_path.split('/').last().unwrap_or("report.zip")
                                ),
                            }
                            .into(),
                        );

                        zip_path
                    }
                };

                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.65,
                        message: format!("Uploading {:.1} KB…", size as f64 / 1024.0),
                    }
                    .into(),
                );

                let result = tokio::task::spawn_blocking(move || upload(path, context))
                    .await
                    .unwrap();

                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.9,
                        message: "Finalizing…".into(),
                    }
                    .into(),
                );

                tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                match result {
                    Ok(_) => out.emit(CmdCrashOut::Finished { bytes: size }.into()),
                    Err(e) => out.emit(CmdCrashOut::Error(format!("Upload failed: {e}")).into()),
                }
            })
            .drop_on_shutdown()
            .boxed()
    });
}

pub fn upload(file_path: String, context: Option<String>) -> anyhow::Result<()> {
    let server = CONFIG.get().server.clone();

    let mut form = multipart::Form::new().file("report", file_path)?;

    if let Some(context) = context {
        form = form.text("context", context);
    };

    reqwest::blocking::Client::new()
        .post(format!("{}/upload/report", &server))
        .multipart(form)
        .send()?;

    Ok(())
}
