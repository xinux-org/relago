use clap::{arg, command, Arg, ArgAction, Command, Parser, Subcommand};

use daemon::*;
use nixlog::error as NixErr;
use notify::modal;
use report;
use std::io::{BufRead, Read};
use subprocess::Exec;

pub fn run() -> anyhow::Result<()> {
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
                .arg(Arg::new("exec").action(ArgAction::Append)),
        )
        .subcommand(Command::new("daemon").about("Run daemon").arg(arg!([NAME])))
        .subcommand(Command::new("notify").about("Run notification"))
        .subcommand(
            Command::new("report")
                .about("Report journal entries to JSON file")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("DIR")
                        .help("Output directory for report")
                        .default_value("/tmp/relago"),
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
                ),
        )
        .subcommand(
            Command::new("reporter")
                .about("Launch crash reporter GUI")
                .arg(
                    Arg::new("unit")
                        .long("unit")
                        .value_name("UNIT")
                        .help("Unit name"),
                )
                .arg(
                    Arg::new("exe")
                        .long("exe")
                        .value_name("EXE")
                        .help("Executable name"),
                )
                .arg(
                    Arg::new("message")
                        .long("message")
                        .value_name("MESSAGE")
                        .help("Crash message")
                        .default_value("Coredump"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("exec", sub_matches)) => {
            let r = sub_matches
                .get_many::<String>("exec")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            match cmd_exec(&r[0]) {
                Err(_) => println!("Cooked"),
                Ok(_) => println!("exec"),
            }
        }
        Some(("report", sub_matches)) => {
            let rep = sub_matches
                .get_one::<String>("output")
                .map(|s| s.as_str())
                .unwrap_or("/tmp/relago");

            let nixos_config = sub_matches
                .get_one::<String>("nixos-config")
                .map(|s| s.as_str());

            // Check if `--recent` argument added
            let recent_entries = sub_matches
                .get_one::<String>("recent")
                .and_then(|s| s.parse::<usize>().ok());

            // report::create_report(rep, nixos_config, recent_entries)?;
            report::run(rep, nixos_config, recent_entries)?
        }
        Some(("daemon", sub_matches)) => {
            // Daemon started
            // println!("daemon");
            // dbus-send --system --type=signal /com/example com.example.signal_name string:"hello world"

            // let _ = fetcher::run();
            // let _ = core::run();

            println!("Relago daemon application is started without fuckery!!!");
            let _ = daemon::journal::run();
        }
        Some(("reporter", _)) => {
            let unit = std::env::var("RELAGO_UNIT").unwrap_or_default();
            let exe = std::env::var("RELAGO_EXE").unwrap_or_default();
            let message =
                std::env::var("RELAGO_MESSAGE").unwrap_or_else(|_| "Coredump".to_string());

            let modal = notify::window::Modal { unit, exe, message };
            let app_id = format!("org.relm4.Reporter.p{}", std::process::id());

            relm4::RelmApp::new(&app_id)
                .with_args(vec![])
                .run::<notify::window::model::App>(modal);
        }
        _ => println!("`None`"),
    }

    Ok(())
}

fn cmd_exec(cmd: &str) -> anyhow::Result<()> {
    let cm = Exec::shell(cmd);

    match cm.clone().capture() {
        Ok(capture) => {
            if !capture.success() {
                let mut collected_output = String::new();

                let v = cm.stream_stderr()?;
                let reader = std::io::BufReader::new(v);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            // println!("Line :{}", l);
                            collected_output.push_str(&l);
                        }
                        Err(e) => print!("Error:{}", e),
                    }
                }

                let _ = NixErr::process_nix_error(&collected_output);
            }
        }
        Err(e) => {
            print!("{}", e)
        }
    }

    Ok(())
}
