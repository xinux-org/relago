# Relago

Automatic crash reporting tool for XinuxOS. It monitors your system for crashes, collects diagnostic information, and lets you submit reports through a simple GUI.

## What it does

Relago runs as a background daemon that watches the systemd journal for crash events. When it detects a crash, it sends a desktop notification and opens a GUI window where you can review the crash details and choose to send a report.

Detected crash types:
- Coredumps (application crashes)
- Service failures (systemd unit failures)
- Out-of-memory kills (OOM events)

When you send a report, Relago collects:
- System information (CPU, memory, disk usage, network interfaces)
- Systemd journal entries
- NixOS/XinuxOS configuration files (if available)

If the user clicks "Send Report" in the GUI, everything is compressed into a ZIP file and uploaded to the report server. If the user clicks "Cancel", the report is discarded.

## Installation

```bash
# clone the repository
git clone https://github.com/xinux-org/relago
cd relago

# build in release mode (requires GTK4, libadwaita, systemd dev libraries)
cargo build --release

# binary location after build
./target/release/relago
```

## Commands

### daemon

```bash
# before running daemon, create the config directory and file
sudo mkdir -p /var/lib/relago
sudo touch /var/lib/relago/config.toml

# start the crash monitoring service
# watches systemd journal for coredumps, service failures, and OOM events
# when crash detected: extracts details -> shows desktop notification -> opens reporter GUI
cargo run -- daemon

# start with a custom instance name
cargo run -- daemon myinstance
```

### report

```bash
# generate a diagnostic report (saved as ZIP)
# collects: system info, journal entries, nixos config
cargo run -- report

# save report to specific directory
cargo run -- report -o /path/to/output

# only include last 100 journal entries (faster, smaller report)
cargo run -- report -r 100

# include nixos configuration in the report
cargo run -- report --nixos-config /etc/nixos/xinux-config

# combine options
cargo run -- report -o ./my-report -r 500 --nixos-config ~/nixos-config
```

### reporter

```bash
# open the crash reporter GUI window
# shows crash details, send button, upload progress
cargo run -- reporter

# specify crash details (usually called automatically by daemon)
cargo run -- reporter -u nginx.service -e nginx -m "Segmentation fault"

# -u : systemd unit name
# -e : executable name
# -m : crash message
```

### notify

```bash
# run the notification system component
cargo run -- notify
```

### exec

```bash
# execute a shell command
cargo run -- exec "ls -la"
```

## Testing crash detection

To manually trigger a crash for testing, use the [crash](https://github.com/xinux-org/crash) project.

```bash
# install crash tool
git clone https://github.com/xinux-org/crash
cd crash
cargo build --release

# first, start relago daemon in one terminal
cargo run -- daemon

# then, in another terminal, run crash to trigger a test crash
./target/release/crash
```

## Configuration

Configuration file: `/var/lib/relago/config.toml`

```toml
# number of threads for compression
parallel_compression = 4

# temporary directory for report generation
tmp_dir = "/tmp/relago"

# data storage directory
data_dir = "/var/lib/relago/data"

# path to nixos configuration
nix_config = "/etc/nixos/xinux-config"

# report upload server URL
server = "https://example.com"
```

## Report structure

```
report_2024-01-15_10-30-45/
├── system_info.json          # CPU, memory, disk, network info
├── journal_report.json.zlib  # compressed systemd journal entries
└── nixos-config/             # nixos configuration (if included)
```
