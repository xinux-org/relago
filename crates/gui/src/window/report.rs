use super::messages::CmdOut;
use super::model::App;
use futures_util::FutureExt;
use relm4::ComponentSender;
use report::create_report;
use reqwest::blocking::multipart;
use utils::config::CONFIG;

pub fn run(sender: ComponentSender<App>, context: Option<String>) {
    let tmp_dir = CONFIG.get().tmp_dir.to_string_lossy().into_owned();

    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                let _ = out.send(CmdOut::Progress {
                    fraction: 0.05,
                    message: "Reading journal entries…".into(),
                });

                let keys = format!("{}/key.pub", CONFIG.get().keys.display());
                let nix_config = CONFIG.get().nix_config.to_string_lossy().into_owned();

                let rep_file = tokio::task::spawn_blocking(move || {
                    create_report(
                        &tmp_dir,
                        Some(nix_config.as_str()),
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
                        let _ = out.send(CmdOut::Error(format!("Failed to collect report: {e}")));
                        return;
                    }
                    Ok(f) => {
                        let _ = out.send(CmdOut::Progress {
                            fraction: 0.3,
                            message: "Report collected, compressing…".into(),
                        });

                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                        let zip_path = format!("{}.zip", f.file.display());

                        let _ = out.send(CmdOut::Progress {
                            fraction: 0.55,
                            message: format!(
                                "Compressed → {}",
                                zip_path.split('/').last().unwrap_or("report.zip")
                            ),
                        });

                        zip_path
                    }
                };

                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                let _ = out.send(CmdOut::Progress {
                    fraction: 0.65,
                    message: format!("Uploading {:.1} KB…", size as f64 / 1024.0),
                });

                let result =
                    match tokio::task::spawn_blocking(move || upload(path, context)).await {
                        Ok(r) => r,
                        Err(e) => {
                            let _ =
                                out.send(CmdOut::Error(format!("Upload task failed: {e}")));
                            return;
                        }
                    };

                let _ = out.send(CmdOut::Progress {
                    fraction: 0.9,
                    message: "Finalizing…".into(),
                });

                tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                match result {
                    Ok(_) => {
                        let _ = out.send(CmdOut::Finished { bytes: size });
                    }
                    Err(e) => {
                        let _ = out.send(CmdOut::Error(format!("Upload failed: {e}")));
                    }
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
