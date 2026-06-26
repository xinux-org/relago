use super::messages::CmdCrashOut;
use super::model::App;
use futures_util::FutureExt;
use relm4::ComponentSender;
use report::{ReportBuilder, JournalMode};
use reqwest::blocking::multipart;
use utils::config::CONFIG;
use utils::setup_key;

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("Failed to read reporter ID at {path}: {source}")]
    MissingId {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Client error (HTTP {status}): {body}")]
    Client { status: u16, body: String },

    #[error("Server error (HTTP {status}): {body}")]
    Server { status: u16, body: String },
}

pub fn run(sender: ComponentSender<App>, context: Option<String>) {
    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                let uuid_path = CONFIG.get().data_dir.join("uuid");
                let key_path = CONFIG.get().keys.join("key.pub");
                for path in [&uuid_path, &key_path] {
                    if !path.exists() {
                        out.emit(
                            CmdCrashOut::UploadFailed(UploadError::MissingId {
                                path: path.display().to_string(),
                                source: std::io::Error::new(
                                    std::io::ErrorKind::NotFound,
                                    "registration file missing",
                                ),
                            })
                            .into(),
                        );
                        return;
                    }
                }

                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.05,
                        message: "Reading journal entries…".into(),
                    }
                    .into(),
                );

                let keys = format!("{}/server.pub", CONFIG.get().keys.display());
                let nix_config = CONFIG.get().nix_config.to_string_lossy().into_owned();
                let tmp_dir = CONFIG.get().tmp_dir.to_string_lossy().into_owned();

                let rep_file = tokio::task::spawn_blocking(move || {
                    ReportBuilder::new(&tmp_dir)
                        .system_info()
                        .journal(JournalMode::All)
                        .nixos_config(&nix_config)
                        .encrypt(&keys)
                        .build()
                })
                .await;

                let rep_file = match rep_file {
                    Ok(r) => r,
                    Err(e) => {
                        out.emit(CmdCrashOut::Error(format!("Report task failed: {e}")).into());
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
                        println!("ZIP FILE: {zip_path}");
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

                let result = match tokio::task::spawn_blocking(move || upload(path, context)).await
                {
                    Ok(res) => res,
                    Err(e) => {
                        out.emit(CmdCrashOut::Error(format!("Upload task failed: {e}")).into());
                        return;
                    }
                };

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
                    Err(e) => out.emit(CmdCrashOut::UploadFailed(e).into()),
                }
            })
            .drop_on_shutdown()
            .boxed()
    });
}

pub fn run_setup_key(sender: ComponentSender<App>) {
    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.1,
                        message: "Registering reporter…".into(),
                    }
                    .into(),
                );

                let result = tokio::task::spawn_blocking(setup_key::init).await;

                let output = match result {
                    Err(e) => CmdCrashOut::Error(format!("Setup task failed: {e}")),
                    Ok(Err(e)) => CmdCrashOut::Error(format!("Setup failed: {e}")),
                    Ok(Ok(())) => CmdCrashOut::SetupKeyDone,
                };
                out.emit(output.into());
            })
            .drop_on_shutdown()
            .boxed()
    });
}

pub fn upload(file_path: String, context: Option<String>) -> Result<(), UploadError> {
    let server = CONFIG.get().server.clone();
    let uuid_path = CONFIG.get().data_dir.join("uuid");
    let uuid = std::fs::read_to_string(&uuid_path)?
        .trim()
        .to_owned();

    let mut form = multipart::Form::new().file("report", file_path)?;
    if let Some(context) = context {
        form = form.text("context", context);
    }

    let url = format!("{}/upload/report", &server);

    let res = reqwest::blocking::Client::new()
        .post(&url)
        .header("Reporter-ID", &uuid)
        .multipart(form);
    let res = res.send()?;
    let status = res.status();
    let body = res.text()?;

    if status.is_success() {
        Ok(())
    } else if status.is_client_error() {
        Err(UploadError::Client {
            status: status.as_u16(),
            body,
        })
    } else {
        Err(UploadError::Server {
            status: status.as_u16(),
            body,
        })
    }
}
