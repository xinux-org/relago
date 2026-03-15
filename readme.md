# Relago

System crash reporting and diagnostics tool for Linux.

## Commands

### Report Command

Generate a comprehensive system report including journal entries, system information, and NixOS configuration.

**Default output:** `/tmp/relago/report_YYYY-MM-DD_HH-MM-SS/`

#### Basic Usage

```bash
# Report all journal entries (default)
cargo run -- report

# Report with NixOS configuration
cargo run -- report --nixos-config ~/configuration-path

# Report last N entries only
cargo run -- report --recent 100

# Report with custom output directory
cargo run -- report --output /custom/path

# Combine options
cargo run -- report --nixos-config ~/configuration-path --recent 500 --output /custom/path
```

#### Short Flags

```bash
# Recent entries (short flag)
cargo run -- report -r 100

# Output directory (short flag)
cargo run -- report -o /custom/path
```

#### Report Structure

The report creates a timestamped directory containing:

```
report_YYYY-MM-DD_HH-MM-SS/
├── journal_report.json.zlib    # Compressed systemd journal entries
├── system_info.json            # CPU, RAM, disks, network information
└── nixos-config/               # NixOS configuration (if --nixos-config provided)
    ├── flake.nix
    ├── systems/
    └── modules/
```

#### Examples

```bash
# Full system report with NixOS config
cargo run -- report --nixos-config ~/configuration-path

# Recent entries only, custom location
cargo run -- report -r 50 -o /var/reports

# Quick diagnostic with last 10 entries
cargo run -- report -r 10
```
