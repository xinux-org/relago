use clap::{arg, command, Arg, ArgAction, Args, Command, FromArgMatches};

use notify::window::{model::App, Modal};
use std::{env, io::BufRead, process};
use subprocess::Exec;
use utils::config::{Config, ConfigLayer, CONFIG};

const CONFIG_FILE: &str = "/var/lib/relago/config.toml";

pub fn run() -> anyhow::Result<()> {
    match Config::get_config(CONFIG_FILE) {
        Ok(config) => {
            CONFIG.set(move || config.clone());
        }
        Err(e) => {
            println!("An error occurred: {}", e);
            process::exit(1)
        }
    }

    let tmp_dir = CONFIG.get().tmp_dir.clone();

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
        .get_matches();

    match matches.subcommand() {
        Some(("exec", sub_matches)) => {
            let r = sub_matches
                .get_many::<String>("exec")
                .unwrap_or_default()
                .map(|v| v.as_str())
                .collect::<Vec<_>>();
            match cmd_exec(r[0]) {
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
        Some(("daemon", sub_matches)) => {
            // Daemon started
            // println!("daemon");
            // dbus-send --system --type=signal /com/example com.example.signal_name string:"hello world"

            // let _ = fetcher::run();
            // let _ = core::run();

            println!("{:?}", sub_matches.try_get_raw("NAME"));
            println!("Relago daemon application is started without fuckery!!!");
            let _ = daemon::journal::run();
        }
        Some(("reporter", sub_matches)) => {
            let options = ["unit", "exe", "message"];

            let vals = options.map(|x| {
                match sub_matches
                    .get_one::<String>(x)
                    .map(|s| s.as_str())
                    .to_owned()
                {
                    Some(y) => y,
                    None => "None",
                }
            });

            let modal = Modal {
                unit: vals[0].to_string(),
                exe: vals[1].to_string(),
                message: vals[2].to_string(),
            };

            let app_id = format!("org.relm4.Reporter.p{}", std::process::id());

            relm4::RelmApp::new(&app_id)
                .with_args(vec![])
                .run::<App>(modal);
        }
        Some(("configure", sub_matches)) => {
            Config::save_config(CONFIG_FILE, ConfigLayer::from_arg_matches(sub_matches)?)?
        }
        _ => {
            println!("`None`")
        }
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
