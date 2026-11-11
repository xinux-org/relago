# ADR 0007: Report structure

- **Status:** Accepted
- **Date:** 2026-10-24
- **Deciders:** Relago maintainers, report server team
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §5.5 (detector scope), §8, §9.2 (server API); [RFC 0002](../rfc/0002-detectors.md) §3.3, §5; [ADR 0004](0004-opt-in-automatic-micro-reports.md) (consent), [ADR 0006](0006-data-formats.md) (formats)

## Context

Today a report is a directory zipped by `ReportBuilder`, containing:
- `system_info.json` (with the hostname and network interfaces),
- `journal_report.json.zlib` (the whole system journal, NDJSON or a JSON array depending on the flags),
- optionally a raw copy of `/etc/nixos` including `.git`,
- optionally `meta.json` and log files.

The layout is not versioned and the server has to guess what it gets. The contents are not tied to the problem that triggered the report, and nothing limits what personal data goes in.

The redesign introduces two report tiers:
- **micro reports:** small, automatic under consent (ADR 0004),
- **full reports:** large, on request.

It also adds 18 detectors with 15 problem kinds, each with different data (RFC 0001 §5.5, RFC 0002 §2). The server team needs one stable, versioned contract for both tiers and all kinds. The client needs rules that make leaking data structurally hard.

## Decision

### 1. One envelope, two tiers

Both tiers share the same **envelope**: common fields that describe the report, the problem, the system and the client. Problem-specific data goes in a `details` object whose shape is fixed per `kind`.

- **Micro report:** the envelope as one JSON document, schema `relago.micro/1`.
- **Full report:** an archive that **contains the micro report** (`micro.json`) plus larger attachments. A full report is therefore always a superset of the micro report, and the server processes both with the same code path for the common part.

### 2. Micro report (`relago.micro/1`)

```jsonc
{
  "schema": "relago.micro/1",
  "report_id": "7f1c…",                 // random UUIDv4 per report; not stable per machine
  "created_at": 1727700000000000,       // µs since epoch (ADR 0006)
  "client": { "version": "0.1.0", "send_mode": "auto" },   // auto | manual; "auto" only if ADR 0004 (reopened) keeps automatic sending

  "problem": {
    "fingerprint": "fp_v1:3f9a…",
    "kind": "coredump",                 // ProblemKind, see §4 (RFC 0002 §2)
    "detector": "CRASH-1",
    "severity": "critical",
    "owner": "user",                    // user | system; never the uid
    "occurrences": {
      "count": 14,
      "first_seen": 1727600000000000,
      "last_seen":  1727700000000000,
      "versions": ["130.0", "131.0"],
      "looping": false                  // CRASH-2
    },
    "subject": {                        // what is affected; exactly one variant
      "type": "exe",                    // exe | unit | device | mount | kernel | generation
      "package": { "name": "firefox", "version": "131.0" },
      "exe": "bin/firefox"              // store path and hash stripped
    }
  },

  "details": {                          // shape fixed by problem.kind, see §4
    "signal": { "number": 11, "name": "SIGSEGV" },
    "frames": [ { "fn": "…", "module": "libxul.so" } ],
    "comm": "firefox",
    "unit": "app-gnome-firefox-1234.scope",
    "core_available": true
  },

  "system": {
    "os": "xinux",
    "nixos_version": "25.11.20260901.abcdef0",
    "system_hash": "b3…",               // sha256 of the /run/current-system target path
    "kernel": "6.12.40",
    "arch": "x86_64",
    "boot_age_s": 5400,                 // seconds since boot, bucketed to 5 min
    "desktop": "GNOME",                 // XDG_CURRENT_DESKTOP, if a session is involved
    "session_type": "wayland"
  },

  "journal": [                          // ≤ 50 lines, scoped per kind (§5), scrubbed (§6)
    { "t": 1727699999000000, "p": 3, "unit": "app-gnome-firefox-1234.scope", "m": "…" }
  ],

  "context": "optional text written by the user (≤ 4 KiB), shown to them before sending"
}
```

Rules:
- **Required fields:** `schema`, `report_id`, `created_at`, `client`, `problem` (all but `occurrences.versions` and `looping`), `details`, `system.os`, `system.kernel`, `system.arch`. Everything else is optional.
- **No machine identifiers in the body.** There is no hostname, machine-id, boot-id, uid, username, IP or MAC address. The device is identified only by the `Reporter-ID` transport header (RFC 0001 §9.2), so the body can be stored or shared without identifying the machine.
- **Size limits:** the target is ≤ 16 KiB and the **hard limit is 64 KiB**. When a report exceeds the target, the builder trims in this order: `journal` lines (oldest first), then `details.frames` beyond 10, then `context` (truncated, with the user warned in the UI). A report still above 64 KiB is a bug: it is not sent, and a `relago-internal` problem is recorded.

### 3. Full report (`relago.full/1`)

A stream `tar` → `zstd` → OpenPGP (ADR 0006) named `report-<report_id>.tar.zst.pgp`:

```
manifest.json      relago.full/1      index of the archive (always first entry in the tar)
micro.json         relago.micro/1     the micro report for the same problem (always present)
system.json        relago.system/1    extended system info (§3.2)
journal.ndjson     relago.journal/1   extended journal (§3.3), one entry per line
core.zst           —                  optional: core file, zstd-compressed
nixos-config.tar   —                  optional: redacted configuration (§3.4)
```

#### 3.1 `manifest.json`

```jsonc
{
  "schema": "relago.full/1",
  "report_id": "7f1c…",                 // same as micro.json
  "created_at": 1727700000000000,
  "requested_by": "server",             // server (want_full) | user
  "server_ref": "…",                    // from POST /v2/problems, if any
  "consented": ["system", "journal", "core"],     // categories the user ticked
  "config": {                           // optional, only with nixos-config.tar (§3.4)
    "root": "/etc/nixos", "git_head": "…",
    "originals": [ { "path": "configuration.nix", "sha256": "…" } ]   // pre-redaction files
  },
  "files": [
    { "path": "micro.json",     "schema": "relago.micro/1",   "sha256": "…" },
    { "path": "system.json",    "schema": "relago.system/1",  "sha256": "…" },
    { "path": "journal.ndjson", "schema": "relago.journal/1", "lines": 18342, "truncated": false, "sha256": "…" },
    { "path": "core.zst",       "schema": null,               "size": 48211123, "sha256": "…" }
  ]
}
```

- `manifest.json` is the **first** tar entry, so the server knows the contents before reading the rest. A trailing manifest is rejected because the server must be able to route the stream early.
- To make that possible, the builder works in two passes, both streaming and neither holding a file in memory:
  1. Each file is written to a private temp directory (mode `0700`), computing its `sha256`, size and line count on the fly.
  2. The final `tar` → `zstd` → PGP stream is produced with the manifest first, then the staged files.

  The temp directory is removed after upload or on failure.
- Every file present in the archive is listed in the manifest, and every listed file is present. Unknown extra files are an error.

#### 3.2 `system.json` (`relago.system/1`)

The micro `system` block plus:
- CPU model and core count,
- RAM, swap and zram sizes,
- GPU vendor:device IDs and driver names,
- storage devices (model, transport, size bucket; no serial numbers),
- filesystem types and usage buckets of the standard mounts (`/`, `/nix`, `/boot`, `/home`, `/var`),
- the list of enabled NixOS services by name only, if the configuration category is consented.

It does **not** include the hostname, network interface names or addresses, serial numbers, or the user list. The collector is allow-list based, like `Details`.

#### 3.3 `journal.ndjson` (`relago.journal/1`)

One object per line: `{"t", "p", "unit", "pid", "comm", "m"}`, with optional `"id"` (the `MESSAGE_ID`) and `"k": true` for kernel messages.

- **Scope:** the boot in which the problem occurred, limited to the affected unit, its dependencies, the kernel, and the systemd manager, from 10 min before the first evidence cursor to 2 min after the last.
- **Caps:** 200 000 lines or 64 MiB uncompressed, whichever is hit first. The newest lines are kept, and `truncated: true` is set in the manifest.
- The same scrubbing as the micro journal applies (§6).

#### 3.4 Optional attachments

- **`core.zst`:** only for CRASH-1/SESS-1, only if `COREDUMP_FILENAME` is still available, and only if the user ticked "core file". Capped at 256 MiB compressed; if larger, it is omitted and the manifest says so.
- **`nixos-config.tar`:** only if ticked. It is built from the configuration directory:
  - **Excluded:** `.git/`, any path matching `secret*`, `*.age`, `*.key`, `*.pem`, `id_*`, `*password*`, and files > 1 MiB.
  - **Rewritten:** filesystem UUIDs and labels in `hardware-configuration.nix` are replaced with placeholders.
  - The file list is shown to the user before sending.
  - The manifest's optional `config` block records the config root, its git `HEAD` if it is a repository, and the sha256 of each **original** (pre-redaction) file, so support can prepare a patch against the exact base (RFC 0001 §8.2). Relago never applies such patches (RFC 0001 §3).

### 4. `details` per kind

`details` is the serialised `Details` variant of the detector (RFC 0002 §3.3). Each variant is a closed schema: unknown fields are rejected by the builder's own schema test. The first-release variants and their fields are the "Micro-report details" of each detector in RFC 0002 §5. Examples:

| Kind (detector) | `subject.type` | Key `details` fields |
|---|---|---|
| `coredump` (CRASH-1) | `exe` | `signal`, `frames`, `comm`, `unit`, `core_available` |
| `unit_failure` (SVC-1/SVC-2) | `unit` | `result`, `exit_code`, `exit_status`, `restart_count`, `recovered`, `user_unit` |
| `session_crash` (SESS-1) | `exe` | `component`, `linked` (fingerprints of CRASH-1/HW-1), `session_type` |
| `oom_kill` (MEM-1/MEM-2) | `exe` or `unit` | `killer` (`oomd`/`kernel`), `app_id`, `constraint`, `anon_rss_kib` |
| `kernel_panic` (KERN-1) | `kernel` | `reason`, `rip_fn`, `frames`, `taint`, `crashed_kernel` |
| `kernel_oops` (KERN-2) | `kernel` | `class`, `function`, `location` (WARN), `frames`, `taint`, `third_party` |
| `gpu_hang` (HW-1) | `device` | `driver`, `pci_id`, `engine`, `code`, `reset_ok`, `comm` |
| `hw_error` (HW-2) | `device` or `kernel` | `source`, `class`, `bank`/`pci_id`/`mc`, `count` |
| `fs_error` (DISK-1) | `mount` | `fs`, `class`, `mount_class`, `device_model`, `transport`, `count` |
| `low_space` (DISK-2) | `mount` | `mount_class`, `fs`, `size_bucket`, `free_percent`, `generations` |
| `unclean_shutdown` (PWR-1) | `kernel` | `class`, `prev_uptime_s`, `during_sleep`, `last_signals` |
| `disk_health` (DISK-3) | `device` | `class`, `device_model`, `firmware`, `attribute` |
| `activation_failure` (NIX-1) | `generation` | `failed_units`, `failed_snippets`, `aborted`, `from_version`, `to_version` |
| `upgrade_failure` (NIX-2) | `generation` | `class`, `drv_name`, `input`, `consecutive`, `first_error` |
| `older_generation` (NIX-3) | `generation` | `booted_gen`, `newest_gen`, `booted_version`, `newest_version`, `evidence` |

### 5. Micro journal scope per kind

Micro reports include at most 50 lines:
- CRASH-1/SESS-1/MEM-1/MEM-2: the crashing PID and its unit, ±60 s;
- SVC-1/SVC-2: the failed invocation (`_SYSTEMD_INVOCATION_ID`);
- KERN-2/HW-1/HW-2/DISK-1: the assembled kernel block only;
- NIX-1/NIX-2: the unit's error lines only;
- KERN-1/DISK-2/PWR-1/DISK-3/NIX-3: no journal lines.

### 6. Scrubbing (applies to every free-text field)

Applies to journal `m`, `context`, `first_error`, and message-derived strings, always in the builder, before preview:
- `/home/<name>/…` → `~/…`; the current usernames → `<user>`; the hostname → `<host>`.
- `/nix/store/<hash>-` → `/nix/store/…-`.
- IPv4/IPv6 addresses → `<ip>`, MAC addresses → `<mac>`, e-mail addresses → `<email>`, UUIDs → `<uuid>`.
- Long hex/base64 tokens (≥ 32 chars) → `<token>`.

The preview (RFC 0001 §8.3) shows the scrubbed result, byte-for-byte what will be sent.

### 7. Versioning and compatibility

- Schema IDs are `relago.<name>/<major>`. Adding optional fields keeps the major version. Removing or renaming a field, changing its type, or making it required bumps the major version.
- Readers ignore unknown optional fields in `/N` documents from newer minor client versions. Writers never emit fields outside the schema.
- The JSON Schemas live in `crates/proto/schemas/*.json`. The server imports them from the Relago repository, and client tests validate every generated report against them.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Envelope + per-kind `details`, full report embeds micro (chosen) | One parser for common fields; 15 kinds without schema sprawl; full ⊇ micro | Closed per-kind schemas must be maintained with each detector |
| Flat micro report per kind (separate schema each) | Simple per kind | 15 schemas with duplicated common parts; server code per kind |
| Free-form `details` map | Easy to add fields | Detectors can leak anything; server cannot rely on shape |
| Keep current ZIP layout, add files | Minimal change | Unversioned, no manifest, not streamable, carries today's privacy problems |

## Consequences

- `relago-proto` owns the Rust types and JSON Schemas for `micro`, `full`, `system`, `journal`, and every `details` variant. Adding a detector means adding a `details` variant and a schema.
- Tests:
  - every fixture from RFC 0002 §6 also produces a micro report that is validated against the schema and snapshot-tested,
  - a dedicated test asserts that no identifier (hostname, username, IP, machine-id, uid) appears in any generated report,
  - the size-limit trimming is tested.
- The server must reject reports with an unknown major schema version, a manifest that doesn't match the archive, or a micro report above 64 KiB.
- The current `ReportBuilder` layout (`system_info.json`, `journal_report.json.zlib`, `meta.json`) is retired with the server's `/v1` API.
