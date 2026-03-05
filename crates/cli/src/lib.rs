use clap::{arg, command, Arg, ArgAction, Command, Parser, Subcommand};

// use daemon::core;
use daemon::*;
use nixlog::error as NixErr;
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
        .subcommand(Command::new("fetch").about("Fetch data").arg(arg!([INTERFACE])))
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
        Some(("fetch", sub_matches)) => {
            println!("Fetcher should take data from journal might be around here");
            let _ = fetcher::run();
        }
        Some(("daemon", sub_matches)) => {
            // Daemon started
            // println!("daemon");
            // dbus-send --system --type=signal /com/example com.example.signal_name string:"hello world"
            let _ = fetcher::run();
            let _ = core::run();
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
