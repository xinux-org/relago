use fluent_zero::t;
use futures_util::FutureExt;
use relm4::ComponentSender;
use report::{
    JournalMode, ReportBuilder, setup_key,
    upload::{UploadError, upload},
};
use utils::config::CONFIG;

use super::{App, CmdCrashOut};

pub fn run(sender: &ComponentSender<App>, context: Option<String>) {
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
                println!("CmdCrashOut::Progress run");

                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.05,
                        message: t!("progress-reading-journal").into(),
                    }
                    .into(),
                );
                println!("CmdCrashOut::Progress fraction: 0.05, run");

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
                        println!("CmdCrashOut::Progress fraction: 0.3, run");
                        out.emit(
                            CmdCrashOut::Progress {
                                fraction: 0.3,
                                message: t!("progress-compressing").into(),
                            }
                            .into(),
                        );

                        tokio::time::sleep(std::time::Duration::from_millis(300)).await;

                        let zip_path = f.file.display().to_string();
                        let zip_file_name = zip_path.split('/').next_back().unwrap_or("report.zip");
                        out.emit(
                            CmdCrashOut::Progress {
                                fraction: 0.55,
                                message: t!("progress-compressed", { "file" => zip_file_name }).into(),
                            }
                            .into(),
                        );
                        zip_path
                    }
                };

                println!("CmdCrashOut::Progress Uploading, run");
                let size = std::fs::metadata(&path).map_or(0, |m| m.len());
                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.65,
                        message: t!("progress-uploading", { "kb" => format!("{:.1}", size as f64 / 1024.0) }).into(),
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
                        message: t!("progress-finalizing").into(),
                    }
                    .into(),
                );

                tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                match result {
                    Ok(()) => out.emit(CmdCrashOut::Finished { bytes: size }.into()),
                    Err(e) => out.emit(CmdCrashOut::UploadFailed(e).into()),
                }
            })
            .drop_on_shutdown()
            .boxed()
    });
}

pub fn run_setup_key(sender: &ComponentSender<App>) {
    sender.command(|out, shutdown| {
        shutdown
            .register(async move {
                out.emit(
                    CmdCrashOut::Progress {
                        fraction: 0.1,
                        message: t!("progress-register").into(),
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
