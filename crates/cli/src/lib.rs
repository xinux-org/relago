use clap::{arg, command, Arg, ArgAction, Command};

use report;
use std::{io::BufRead, path::PathBuf};
use subprocess::Exec;
use utils::config::CONFIG;

pub fn run() -> anyhow::Result<()> {
    let tmp_dir: PathBuf = CONFIG.get().tmp_dir.to_path_buf();

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
        .subcommand(
            Command::new("report")
                .about("Report journal entries to JSON file")
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .value_name("DIR")
                        .help("Output directory for report")
                        .default_value(tmp_dir.as_os_str().to_owned()),
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
            let rep: String = sub_matches
                .get_one::<String>("output")
                .unwrap_or(&tmp_dir.into_os_string().into_string().unwrap())
                .to_owned();

            let nixos_config = sub_matches
                .get_one::<String>("nixos-config")
                .map(|s| s.as_str());

            // Check if `--recent` argument added
            let recent_entries = sub_matches
                .get_one::<String>("recent")
                .and_then(|s| s.parse::<usize>().ok());

            // report::create_report(rep, nixos_config, recent_entries)?;
            report::run(rep.as_str(), nixos_config, recent_entries)?
        }
        Some(("daemon", _sub_matches)) => {
            // Daemon started
            // println!("daemon");
            // dbus-send --system --type=signal /com/example com.example.signal_name string:"hello world"

            // let _ = fetcher::run();
            // let _ = core::run();

            println!("Relago daemon application is started without fuckery!!!");
            let _ = daemon::journal::run();
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

                // let _ = NixErr::process_nix_error(&collected_output);
            }
        }
        Err(e) => {
            print!("{}", e)
        }
    }

    Ok(())
}
