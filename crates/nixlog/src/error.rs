use nom::{
    bytes::complete::{tag, take_until, take_while1},
    character::complete::multispace1,
    IResult,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use subprocess::Exec;

const LOG_DIR: &str = "/tmp/relago";

#[derive(Debug)]
pub struct NixBuildError {
    pub drv_path: String,
    pub short_log: String,
    pub full_log: Option<String>,
    pub log_file: Option<PathBuf>,
}

impl NixBuildError {
    pub fn from_output(output: &str) -> Option<Self> {
        let (_, command) = parse_nix_log_command(output).ok()?;
        let (_, drv_path) = parse_drv_path(command).ok()?;

        Some(NixBuildError {
            drv_path: drv_path.to_string(),
            short_log: output.to_string(),
            full_log: None,
            log_file: None,
        })
    }

    /// Run `nix log <drv>` for get logs
    pub fn fetch_full_log(&mut self) -> anyhow::Result<()> {
        let cmd = format!("nix log {}", self.drv_path);
        let capture = Exec::shell(&cmd).capture()?;

        if capture.success() {
            self.full_log = Some(capture.stdout_str());
        } else {
            tracing::warn!("nix log failed: {}", capture.stderr_str());
        }

        Ok(())
    }

    /// Save to relagodata directory
    pub fn save(&mut self, log_dir: &str) -> anyhow::Result<PathBuf> {
        fs::create_dir_all(log_dir)?;

        // Extract drv hash
        let drv_hash = self
            .drv_path
            .trim_start_matches("/nix/store/")
            .split('-')
            .next()
            .unwrap_or("unknown");

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let filename = format!("{}-{}.log", drv_hash, timestamp);
        let path = PathBuf::from(log_dir).join(&filename);

        let content = self.format_log();
        fs::write(&path, &content)?;

        self.log_file = Some(path.clone());
        tracing::info!("Saved crash log to: {}", path.display());

        Ok(path)
    }

    fn format_log(&self) -> String {
        let mut content = String::new();

        content.push_str("=== RELAGO CRASH REPORT ===\n\n");
        content.push_str(&format!("Derivation: {}\n", self.drv_path));
        content.push_str(&format!("Timestamp: {}\n", chrono::Utc::now().to_rfc3339()));

        if let Ok(version) = fs::read_to_string("/run/current-system/nixos-version") {
            content.push_str(&format!("NixOS Version: {}\n", version.trim()));
        }
        if let Ok(gn) = fs::read_link("/run/current-system") {
            content.push_str(&format!("Generation: {}\n", gn.display()));
        }

        content.push_str("\n\n========================== SHORT LOG ==========================\n\n");
        content.push_str(&self.short_log);

        if let Some(ref full_log) = self.full_log {
            content
                .push_str("\n\n========================== FULL LOG ==========================\n\n");
            content.push_str(full_log);
        }

        content
    }
}

fn parse_nix_log_command(input: &str) -> IResult<&str, &str> {
    let (input, _) = take_until("For full logs, run:")(input)?;
    let (input, _) = tag("For full logs, run:")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, command) = take_while1(|c| c != '\n')(input)?;
    Ok((input, command.trim()))
}

fn parse_drv_path(input: &str) -> IResult<&str, &str> {
    let (input, _) = take_until("/nix/store/")(input)?;
    let (input, path) = take_while1(|c: char| !c.is_whitespace())(input)?;
    Ok((input, path))
}

pub fn process_nix_error(output: &str) -> anyhow::Result<()> {
    if let Some(mut error) = NixBuildError::from_output(output) {
        tracing::info!("Detected nix build error: {}", error.drv_path);

        
        if let Err(e) = error.fetch_full_log() {
            tracing::warn!("Failed get log: {}", e);
        }


        
        match error.save(LOG_DIR) {
            Ok(path) => {
                println!("\n[relago] Crash report saved: {}", path.display());
            }
            Err(e) => {
                tracing::error!("Failed to save log: {}", e);
            }
        }
    } else {
        tracing::debug!("Skip");
    }

    Ok(())
}
