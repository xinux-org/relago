use anyhow::Context;
use clap::{Arg, ArgAction, Args, Command, FromArgMatches, arg, command};

use daemon::journal;
use gui::start_listener;
use std::process;
use utils::{
    config::{CONFIG, Config, ConfigLayer},
    setup_key,
};

const CONFIG_FILE: &str = "/var/lib/relago/config.toml";

pub fn run() -> anyhow::Result<()> {
    match Config::get_config(CONFIG_FILE) {
        Ok(config) => {
            CONFIG.set(move || config.clone());
        }
        Err(e) => {
            println!("An error occurred: {e}");
            process::exit(1)
        }
    }

    let tmp_dir = CONFIG.get().tmp_dir.to_string_lossy().into_owned();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let matches = command!() // requires `cargo` feature
        .subcommand(
            Command::new("exec")
                .about("Run daemon")
                .arg(Arg::new("exec").required(true).action(ArgAction::Append)),
        )
        .subcommand(Command::new("daemon").about("Run daemon").arg(arg!([NAME])))
        .subcommand(Command::new("gui").about("Run notification-report"))
        .subcommand(
            Command::new("report")
                .about("Report journal entries to JSON file")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("DIR")
                        .help("Output directory for report"), // .default_value(CONFIG.get().tmp_dir.as_os_str().to_owned()),
                )
                .arg(
                    Arg::new("recent")
                        .short('r')
                        .long("recent")
                        .value_name("NUM")
                        .help("Report only N most recent entries (from tail)"),
                )
                .arg(
                    Arg::new("nixos-config")
                        .long("nixos-config")
                        .value_name("PATH")
                        .help("Path to NixOS configuration directory (e.g., ~/nix-conf)"),
                )
                .arg(
                    Arg::new("encrypt-key")
                        .short('e')
                        .long("encrypt-key")
                        .value_name("PATH")
                        .help("Path to PGP public key file for encrypting the report"),
                ),
        )
        .subcommand(ConfigLayer::augment_args(
            Command::new("configure").about("Manage configuration via CLI"),
        ))
        .subcommand(
            Command::new("reporter")
                .about("Launch crash reporter GUI")
                .arg(
                    Arg::new("unit")
                        .short('u')
                        .long("unit")
                        .value_name("UNIT")
                        .help("Unit name")
                        .default_value("test"),
                )
                .arg(
                    Arg::new("exe")
                        .short('e')
                        .long("exe")
                        .value_name("EXE")
                        .help("Executable name")
                        .default_value("test"),
                )
                .arg(
                    Arg::new("message")
                        .short('m')
                        .long("message")
                        .value_name("MESSAGE")
                        .help("Crash message")
                        .default_value("Coredump"),
                ),
        )
        .subcommand(Command::new("setup-key").about("Setup GPG keys"))
        .get_matches();

    match matches.subcommand() {
        Some(("exec", sub_matches)) => {
            let r = sub_matches
                .get_many::<String>("exec")
                .unwrap_or_default()
                .map(std::string::String::as_str)
                .collect::<Vec<_>>();

            let cmd = r
                .first()
                .ok_or_else(|| anyhow::anyhow!("no exec argument provided"))?;
            cmd_exec(cmd)?;
        }
        Some(("report", sub_matches)) => {
            let output_dir = sub_matches
                .get_one::<String>("output")
                .unwrap_or(&tmp_dir)
                .to_owned();

            let mut builder = report::ReportBuilder::new(&output_dir).system_info();

            match sub_matches
                .get_one::<String>("recent")
                .and_then(|s| s.parse::<usize>().ok())
            {
                Some(n) => builder = builder.journal(report::JournalMode::Recent(n)),
                None => builder = builder.journal(report::JournalMode::All),
            }

            if let Some(path) = sub_matches.get_one::<String>("nixos-config") {
                builder = builder.nixos_config(path);
            }

            if let Some(key) = sub_matches.get_one::<String>("encrypt-key") {
                builder = builder.encrypt(key);
            }

            builder.build()?;
        }
        Some(("daemon", _sub_matches)) => {
            println!("Relago daemon application is started without fuckery!!!");
            let runtime = tokio::runtime::Runtime::new()?;

            runtime.block_on(async {
                match journal::run().await {
                    Ok(()) => {
                        println!("Started");
                    }
                    Err(e) => {
                        eprintln!("Daemon error: {e}");
                    }
                }
            });
        }
        Some(("gui", _sub_matches)) => {
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(async {
                println!("GUI Agent started. Listening for crash signals...");

                match start_listener().await {
                    Ok(_conn) => {
                        // CRITICAL: This keeps the block_on from returning.
                        // Without this, the program would exit immediately.
                        std::future::pending::<()>().await;
                    }
                    Err(why) => {
                        eprintln!("Failed to start D-Bus listener: {why:?}");
                        std::process::exit(1);
                    }
                }
            });
        }
        Some(("reporter", sub_matches)) => {
            let modal = gui::window::Modal {
                unit: sub_matches
                    .get_one::<String>("unit")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_owned()),
                exe: sub_matches
                    .get_one::<String>("exe")
                    .cloned()
                    .unwrap_or_else(|| "unknown".to_owned()),
                message: sub_matches
                    .get_one::<String>("message")
                    .cloned()
                    .unwrap_or_else(|| "A crash was detected.".to_owned()),
            };
            gui::start_gui(modal);
        }
        Some(("configure", sub_matches)) => {
            Config::save_config(CONFIG_FILE, ConfigLayer::from_arg_matches(sub_matches)?)?;
        }
        Some(("setup-key", _sub_matches)) => {
            setup_key::init()?;
        }
        _ => {
            println!("`No subcommand argument spesified`");
        }
    }

    Ok(())
}

fn cmd_exec(cmd: &str) -> anyhow::Result<()> {
    let output = process::Command::new(cmd)
        .output()
        .context("Failed to execute command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{}", stderr);
    }

    Ok(())
}
