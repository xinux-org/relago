use super::messages::CmdOut;
use super::model::App;
use futures_util::FutureExt;
use relm4::ComponentSender;
use report::create_report;
use reqwest::blocking::multipart;
use std::{sync::Arc};
use utils::config::CONFIG;

pub fn run(sender: ComponentSender<App>, context: Option<String>) {
    let tmp_dir = Arc::new(CONFIG.get().tmp_dir.to_string_lossy().into_owned());

    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                out.send(CmdOut::Progress {
                    fraction: 0.05,
                    message: "Reading journal entries…".into(),
                })
                .unwrap();

                let keys = format!("{}/key.pub", CONFIG.get().keys.display());

                let rep_file = tokio::task::spawn_blocking(move || {
                    create_report(
                        Arc::try_unwrap(tmp_dir).unwrap().as_str(),
                        Some(CONFIG.get().nix_config.clone().to_str().unwrap()),
                        None,
                        // Some(CONFIG.get().public_key)
                        // Some("~/keys/gpg-pub.asc")
                        Some(&keys.clone()),
                    )
                })
                .await
                .unwrap();

                let path = match rep_file {
                    Err(e) => {
                        out.send(CmdOut::Error(format!("Failed to collect report: {e}")))
                            .unwrap();
                        return;
                    }
                    Ok(f) => {
                        out.send(CmdOut::Progress {
                            fraction: 0.3,
                            message: "Report collected, compressing…".into(),
                        })
                        .unwrap();

                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                        let zip_path = format!("{}.zip", f.file.display());

                        out.send(CmdOut::Progress {
                            fraction: 0.55,
                            message: format!(
                                "Compressed → {}",
                                zip_path.split('/').last().unwrap_or("report.zip")
                            ),
                        })
                        .unwrap();

                        zip_path
                    }
                };

                let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

                out.send(CmdOut::Progress {
                    fraction: 0.65,
                    message: format!("Uploading {:.1} KB…", size as f64 / 1024.0),
                })
                .unwrap();

                let result = tokio::task::spawn_blocking(move || upload(path, context))
                    .await
                    .unwrap();

                out.send(CmdOut::Progress {
                    fraction: 0.9,
                    message: "Finalizing…".into(),
                })
                .unwrap();

                tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                match result {
                    Ok(_) => out.send(CmdOut::Finished { bytes: size }).unwrap(),
                    Err(e) => out
                        .send(CmdOut::Error(format!("Upload failed: {e}")))
                        .unwrap(),
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
