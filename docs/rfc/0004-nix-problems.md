# RFC 0004: Nix and NixOS problems

| | |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-10-13 |
| **Depends on** | [RFC 0001](0001-daemon-redesign.md) (catalogue §5.5, severity §6.2), [RFC 0002](0002-detectors.md) (detector framework, NIX-1 to NIX-3) |
| **Decisions** | Bound by [ADR 0003](../adr/0003-problem-visibility.md) (owner), [ADR 0004](../adr/0004-opt-in-automatic-micro-reports.md) (consent), [ADR 0005](../adr/0005-notification-threshold.md) (threshold), [ADR 0007](../adr/0007-report-structure.md) (kinds, scrubbing). **Revisits** the RFC 0001 §5.5 scope decision ("NIX-4 Nix build failures: not planned") and the RFC 0001 §3 non-goal "Detecting Nix build/evaluation failures". Where this RFC and an ADR differ, the ADR wins. Until [ADR 0010](../adr/0010-nix-problem-scope.md) is accepted, the first-release scope of RFC 0001 §5.5 also wins over this RFC |
| **Affects** | `crates/collect`, `crates/proto` (new `details` kinds), `module.nix`, Xinux profile |

## 1. Summary

The first release covers three NixOS problems: failed activation (NIX-1), failed automatic upgrade (NIX-2) and booting an older generation (NIX-3). Nix is the part of Xinux that no other distribution's crash reporter understands, and it fails in many more ways than these three:

- builds and evaluations fail, run out of memory or disk, or hang;
- the store, its database or `nix-daemon` break;
- the bootloader is not updated, or the running system needs a reboot to be consistent;
- the system stops receiving updates, or runs an end-of-life release;
- Home Manager activation fails;
- applications fail in NixOS-specific ways (generic Linux binaries, missing GSettings schemas, driver/kernel mismatches after a switch).

This RFC:

1. lists the Nix-related problems selected for the first Nix phase (§4), with source, noise and a scope recommendation; the others are in [`docs/plan.md`](../plan.md);
2. defines the **log sources** for Nix, including the ones the journal does not have (§3);
3. specifies the **detectors proposed for the first Nix phase** (§5), and sketches the rest (§6);
4. adds **correlation, suppression, privacy and notification rules** specific to Nix (§7–§10);
5. compares **all solution options** for the eight shared building blocks behind these detectors (§12), and gives a **delivery plan** (§13).

## 2. Conventions

- IDs continue the NIX group of RFC 0001 §5.5. NIX-1 to NIX-3 keep their RFC 0002 specs. **NIX-4 is redefined** (§5) and NIX-5 onwards are new.
- Everything in RFC 0002 §2 applies: message formats below are **unverified** and come from memory of Nix, nixpkgs and kernel sources. No detector is merged before its patterns have real fixtures (RFC 0002 §6).
- "Manual" means started by a person in a terminal (`nixos-rebuild`, `nh`, `nix build`, `home-manager switch`). "Automatic" means started by a systemd unit (`nixos-upgrade.service`, `nix-gc.service`, `home-manager-<user>.service`).

## 3. Log sources for Nix

The core difficulty: **most Nix output never reaches the journal.** A manual `nixos-rebuild switch` prints build and evaluation errors to the user's terminal. `nix-daemon` sends them to its client over the socket and logs almost nothing itself.

| # | Source | What it contains | Readable by `relago`? | Used by |
|---|---|---|---|---|
| S1 | Journal of Nix-related **units** (`nixos-upgrade`, `nix-gc`, `nix-optimise`, `nix-daemon`, `home-manager-<user>`, `nixos-rebuild-switch-to-configuration`) | Full stdout/stderr of automatic runs | Yes (`systemd-journal` group) | NIX-1, NIX-2, NIX-7, NIX-10, NIX-11, NIX-13 |
| S2 | **Build logs** `/nix/var/log/nix/drvs/<2 chars>/<rest>.drv.bz2` | The builder's full output, per derivation, for every build the daemon ran (manual or automatic) | Yes (world-readable, verify) | NIX-2 enrichment, NIX-4 |
| S3 | **Nix structured log** (`json-log-path` setting; present in the shipped Nix 2.34, [manual](https://nix.dev/manual/nix/2.34/command-ref/conf-file#conf-json-log-path)) | A JSON copy of all log events (activities, build results, errors) written to a file or Unix socket | Yes, if the module points it at a Relago socket | NIX-4, NIX-5, NIX-6 (preferred source) |
| S4 | **Xinux rebuild wrapper** (`nixos-rebuild`/`nh` wrapped by the Xinux profile, output also sent to the journal with `systemd-cat -t xinux-rebuild`) | Manual system rebuilds: start, end, exit code, stderr | Yes, through the journal | NIX-4, NIX-5, Rebuild window (§8) |
| S5 | **Filesystem state**: `/run/current-system`, `/run/booted-system`, `/nix/var/nix/profiles/`, `statvfs` | Generations, versions, what changed since boot, store size | Yes | NIX-3, NIX-8, NIX-9, NIX-12 |
| S6 | **Kernel and application messages** in the journal | OOM kills of build processes, driver mismatches, app stderr (GNOME/KDE send app output to the journal) | Yes | NIX-6, NIX-14, NIX-15, NIX-16 |
| S7 | **`nix-daemon` client queries** (path validity, store info) | Whether an output is valid, store version | Yes (`allowed-users = *` by default) | NIX-4 (S2 fallback), NIX-11 |

**Proposal:** S1, S2, S5 and S6 need no system change and are used from the first Nix phase. For manual builds, S3 is preferred if the Nix version in nixpkgs supports it; S4 is the fallback. Open point 1 (§11) decides.

S2 on its own is not enough to detect failures: a log file exists for both successful and failed builds. It is used to **enrich** a failure found elsewhere (NIX-2, NIX-4), or, as a last resort, with an S7 validity check (log exists, outputs not valid, no build running → failed).

## 4. Catalogue

This catalogue lists the problems selected for the first Nix phase (N1) and the ones already covered (NIX-1, NIX-3). Items assigned to N2, Later or No are listed in [`docs/plan.md`](../plan.md), with their original rows; the rest of this RFC still refers to them by ID.

**Legend** as in RFC 0001 §5.5. **Phase:** N1 = first Nix phase (§5), N2 = second (§6), Later, No.

### Builds and evaluation

| ID | Problem | Detection | Sev. | Noise | Effort | Phase |
|---|---|---|---|---|---|---|
| NIX-2 | Automatic upgrade failed (existing) | S1; **add** the failing derivation's log tail from S2 | H → C | Low | S | **N1** (enrichment) |
| NIX-5 | Build killed for lack of memory | MEM-1/MEM-2 whose victim is in `nix-daemon.service` or runs as a `nixbld*` user (S6) | L manual / H automatic | Low | S | **N1** |
| NIX-6 | Build failed for lack of disk space | `No space left on device` in S1/S3/S4, linked to DISK-2 | H | Low | S | **N1** (as NIX-2/NIX-4 class) |

### Store and `nix-daemon`

| ID | Problem | Detection | Sev. | Noise | Effort | Phase |
|---|---|---|---|---|---|---|
| NIX-10 | `nix-daemon` down (failed, crash-looping or socket broken) | SVC-1/CRASH-1/CRASH-2 on `nix-daemon.service` / `.socket`, specialised | C | Low | S | **N1** |
| NIX-13 | Garbage collection or store optimisation failed | SVC-1 on `nix-gc.service` / `nix-optimise.service`, specialised wording | H | Low | S | **N1** (as SVC-1 specialisation) |

### Generations, boot and reboot

| ID | Problem | Detection | Sev. | Noise | Effort | Phase |
|---|---|---|---|---|---|---|
| NIX-1 | Activation failed (existing) | — | H / C | Low | — | (done) |
| NIX-3 | Booted an older generation (existing) | — | L / H | Low | — | (done) |
| NIX-8 | **Bootloader update failed**: system switched, but boot entries were not written | `Failed to install bootloader`, `bootctl` / `grub-install` / `systemd-boot-builder` errors inside the Activation window | **C** | Very low | S | **N1** |
| NIX-9 | **Reboot needed**: kernel, initrd, kernel modules or systemd changed since boot | S5: compare `kernel`, `initrd`, `kernel-modules`, `systemd` links of `/run/booted-system` and `/run/current-system` | I; L after 14 days | None | S | **N1** |

### Freshness and lifecycle

| ID | Problem | Detection | Sev. | Noise | Effort | Phase |
|---|---|---|---|---|---|---|
| NIX-14 | **System out of date** | S5: date in `nixos-version` of `/run/current-system` older than 30 days (L), 90 days (H) | L / H | Low | S | **N1** |
| NIX-14b | **NixOS release end-of-life** | Release (`25.05`, …) past its support end. Dates in a table shipped with Relago, updated with each Relago release | H | None | S | **N1** |

### Home Manager and user profiles

| ID | Problem | Detection | Sev. | Noise | Effort | Phase |
|---|---|---|---|---|---|---|
| NIX-15 | **Home Manager activation failed** (NixOS module) | SVC-1 on `home-manager-<user>.service`, specialised; class `file-conflict` from `Existing file '…' is in the way of '…'` | H | Low | S | **N1** |

### NixOS-specific application failures

| ID | Problem | Detection | Sev. | Noise | Effort | Phase |
|---|---|---|---|---|---|---|
| NIX-16 | **Generic Linux binary cannot run** (downloaded program, AppImage, game) | NixOS `stub-ld` message in app output: `Could not start dynamically linked executable: …` (verify text) | H (user) | Low | S | **N1** |
| NIX-17 | **GPU driver mismatch after an update** (NVIDIA kernel module ≠ userspace) | Kernel `NVRM: API mismatch: the client has the version X, but … this kernel module has the version Y`; linked to NIX-9 | H | Very low | S | **N1** |

## 5. First Nix phase (N1): specifications

Spec fields follow RFC 0002 §3.10.

### NIX-2 (amended): Automatic upgrade failed, with build log

- **Change:** for class `build-failure`, keep the **last 30 lines** of the builder's output, scrubbed (§9). Nix already prints the last `log-lines` lines (default 25 in the shipped Nix 2.34, [manual](https://nix.dev/manual/nix/2.34/command-ref/conf-file#conf-log-lines)) under the error, so they are usually in the S1 journal; the S2 log file is the fallback (§12, K3).
- **New classes:** `oom` (NIX-5 linked), `no-space` (already exists; now linked to DISK-2 and NIX-6).
- **Micro-report details:** adds `pname` only if public (§9). The log tail is stored locally and shown in the app; it is sent **only in a full report**, after the user's preview.

### NIX-5: Build killed for lack of memory

- **Type:** Correlator.
- **Input:** MEM-1/MEM-2 events where the victim's cgroup is `system.slice/nix-daemon.service` (`task_memcg=` in the kernel `oom-kill:` line), or the victim uid is in the `nixbld` range.
- **Effect:** the MEM event becomes NIX-5 (not a user-app OOM). If a NIX-2/NIX-4 failure follows within 10 min, it is linked and its class set to `oom`.
- **Owner:** `System`.
- **Fingerprint:** `NIX-5 || derivation name if known (from the next NIX-2/NIX-4) || RAM bucket`.
- **Severity:** L if no automatic run is active; H inside `nixos-upgrade`.
- **Suggested fix:** lower `nix.settings.max-jobs` / `cores`, enable zram.
- **Micro-report details:** RAM and swap totals, `max-jobs`, `cores`, derivation name (public only).
- **Rollout:** **shadow** for its first release ([ADR 0012](../adr/0012-shadow-mode-rollout.md)), then promoted on its shadow counts.

### NIX-6: Build failed for lack of disk space

- **Type:** a class of NIX-2/NIX-4, plus a link to DISK-2.
- **Rule:** `No space left on device` in a failed run → class `no-space`; a DISK-2 probe runs immediately (not at its next 10 min tick) for `/nix/store` and `/tmp` (builds run in `/tmp` unless `build-dir` is set).
- **Severity:** H.
- **Suggested fix:** `nix-collect-garbage -d`, then retry; `boot.tmp.useTmpfs` sizing if `/tmp` is a tmpfs.

### NIX-8: Bootloader update failed

- **Type:** Collector, active only inside the Activation window (RFC 0002 §3.5).
- **Match:** `SYSLOG_IDENTIFIER` of the activation unit or `switch-to-configuration`; patterns (verify per NixOS version and bootloader): `Failed to install bootloader`, `bootctl: …` errors, `grub-install: error: …`, Python tracebacks from `systemd-boot-builder.py`.
- **Parse:** bootloader (`systemd-boot`, `grub`, `other`), first error line, whether `/boot` was full (DISK-2 link: `No space left on device`).
- **Owner:** `System`.
- **Fingerprint:** `NIX-8 || bootloader || class (no-space | permission | efi-vars | other)`.
- **Severity:** **C**. The system now runs a configuration that the next boot will not start, or the boot menu is broken.
- **Suggested fix:** free `/boot` (`boot.loader.systemd-boot.configurationLimit`), then `nixos-rebuild boot`.
- **Correlation:** a NIX-1 in the same window is linked; NIX-8 is notified and NIX-1 is not, to avoid two popups.
- **Micro-report details:** bootloader, class, first error line (scrubbed), `/boot` free percentage.
- **Rollout:** active from its first release (very low noise; exception in [ADR 0012](../adr/0012-shadow-mode-rollout.md)).

### NIX-9: Reboot needed

- **Type:** Probe, `OnStart` and after each Activation window closes.
- **Method:** compare the targets of `kernel`, `initrd`, `kernel-modules` and `sw/lib/systemd` (verify paths) under `/run/booted-system` and `/run/current-system`.
- **Owner:** `System`.
- **Fingerprint:** `NIX-9 || current system's NixOS version`.
- **Severity:** I. Raised to L after 14 days. Raised to H if NIX-17 or NIX-9b happens while it is open.
- **Resolution:** resolved automatically on the next boot.
- **Rollout:** **shadow** for its first release ([ADR 0012](../adr/0012-shadow-mode-rollout.md)), then promoted on its shadow counts. Its Info and Low problems would not notify anyway.

### NIX-10: `nix-daemon` down

- **Type:** specialisation of SVC-1/CRASH-1/CRASH-2 for `nix-daemon.service` and `nix-daemon.socket`.
- **Severity:** **C**. Without it, no user can build, install or update anything.
- **Notification text:** names the consequence ("Software installation and updates are broken"), not the unit.
- **Rollout:** active from its first release (very low noise; exception in [ADR 0012](../adr/0012-shadow-mode-rollout.md)).

### NIX-13: Garbage collection or optimisation failed

- **Type:** specialisation of SVC-1 for `nix-gc.service` and `nix-optimise.service`.
- **Severity:** H if it failed 2 runs in a row (one failure can be a lock or a shutdown), and a DISK-2 for `/nix/store` is open or within 10 % of its threshold; else L.
- **Rollout:** **shadow** for its first release ([ADR 0012](../adr/0012-shadow-mode-rollout.md)), then promoted on its shadow counts.

### NIX-14 / NIX-14b: System out of date, release end-of-life

- **Type:** Probe, `OnStartOncePerBoot` and `Every(24 h)`.
- **Method:** parse `/run/current-system/nixos-version` (`25.11.20260901.abcdef0`): release `25.11`, date `2026-09-01`. Look up the release in the built-in support table.
- **Rules:** NIX-14: date older than 30 days → L, 90 days → H. NIX-14b: release past its support end → H; warning (L) 30 days before.
- **Owner:** `System`.
- **Fingerprint:** `NIX-14 || release`, `NIX-14b || release`.
- **Resolution:** resolved when the version changes.
- **Suggested fix:** links to the Xinux upgrade instructions; mentions NIX-2 if automatic upgrades are failing.
- **Micro-report details:** release, age in days (bucketed), whether `system.autoUpgrade` is enabled.
- **Rollout:** **shadow** for its first release ([ADR 0012](../adr/0012-shadow-mode-rollout.md)), then promoted on its shadow counts; it stays in shadow while the clock looks wrong (time not synchronised).

### NIX-15: Home Manager activation failed

- **Type:** specialisation of SVC-1 for `home-manager-<user>.service`.
- **Parse:** class `file-conflict` (`Existing file '(.+)' is in the way of`), `build-failure`, `activation`, `other`.
- **Owner:** `User(uid)` from the unit instance name. The admin (`wheel`) does **not** see it unless it is also a system problem (ADR 0003).
- **Severity:** H.
- **Suggested fix** (`file-conflict`): `home-manager.backupFileExtension = "backup";`.
- **Micro-report details:** class only, and the conflicting file's path relative to home **without file name** (privacy, §9).
- **Rollout:** **shadow** for its first release ([ADR 0012](../adr/0012-shadow-mode-rollout.md)), then promoted on its shadow counts.

### NIX-16: Generic Linux binary cannot run

- **Type:** Collector.
- **Match:** journal entries of user units/scopes whose `MESSAGE` starts with the stub-ld text (verify the exact text in nixpkgs `nixos/modules/config/stub-ld.nix`).
- **Owner:** `User(_UID)`.
- **Fingerprint:** `NIX-16 || exe basename`.
- **Severity:** H on the first occurrence per binary (the program silently "does nothing" for the user); later occurrences are folded.
- **Suggested fix:** `appimage-run` for AppImages, `steam-run`, or `programs.nix-ld.enable`.
- **Micro-report details:** none beyond the kind. The exe name is **not** sent: it is not a Nix package and may be private. Counts only.
- **Rollout:** **shadow** for its first release ([ADR 0012](../adr/0012-shadow-mode-rollout.md)), then promoted on its shadow counts.

### NIX-17: NVIDIA driver mismatch

- **Type:** Collector.
- **Match:** kernel `NVRM: API mismatch: the client has the version (\S+), but` … `this kernel module has the version (\S+)`.
- **Owner:** `System`.
- **Fingerprint:** `NIX-17 || module version || client version`.
- **Severity:** H. Linked to NIX-9: the text says "Restart to finish the update", which is almost always the fix.
- **Rollout:** active from its first release (very low noise; exception in [ADR 0012](../adr/0012-shadow-mode-rollout.md)).

## 6. Second Nix phase (N2): sketches

The catalogue rows of these items are in [`docs/plan.md`](../plan.md) (section 2).

Full specs are written when the phase starts.

- **NIX-4 / NIX-5-eval: manual system rebuild failed.** Source S3 or S4 (open point 1). Only the **system** closure (`nixos-rebuild`, `nh os`) and Home Manager. The class list is NIX-2's. Severity L, never notified: the user is looking at the error in the terminal. It goes to the problem list, so the app and full reports can show it.
- **NIX-7 classes** (substituter, hash mismatch, hung build) and **NIX-7f** (deprecation warnings): new classes and counters on NIX-2/NIX-4.
- **NIX-9b, NIX-9c:** correlations on NIX-9 and PWR-1.
- **NIX-11:** store corruption patterns from S1, plus DISK-1 `csum` errors on mount class `nix`. A weekly `nix-store --verify --check-contents` run is **not** started by Relago (it is root-only and expensive); the module can offer it as an opt-in timer whose output NIX-11 reads.
- **NIX-12:** store growth forecast. Needs a small time series in `state.json` (one value per day, 30 days).
- **NIX-17b, NIX-18:** NixOS packaging bugs from application output. Shadow first (unknown noise). They are directly useful to Xinux, which maintains these packages.

## 7. Correlation rules (additions to RFC 0002 §3.4)

| Signals seen together | Result |
|---|---|
| MEM-1/MEM-2 with victim in `nix-daemon.service` or a `nixbld*` user | **NIX-5**, not a user-app OOM |
| NIX-5 then NIX-2/NIX-4 within 10 min | NIX-2/NIX-4 class `oom`, linked to NIX-5; only NIX-2/NIX-4 is notified |
| NIX-2/NIX-4 class `no-space` | Immediate DISK-2 probe; linked |
| NIX-8 and NIX-1 in the same Activation window | Keep both; notify **NIX-8** only |
| PWR-1 for a boot that ended inside the Activation window | **NIX-9c** |
| NIX-17 or NIX-9b while NIX-9 is open | Link to NIX-9; NIX-9 raised to H |
| SVC-1 on `nix-daemon.service` / `.socket`, `nix-gc.service`, `nix-optimise.service`, `home-manager-*.service` | Added to the claim table of RFC 0002 §3.4: claimed by NIX-10, NIX-13 and NIX-15 before SVC-1 emits. `nixos-upgrade.service` is already claimed by NIX-2 |

## 8. Suppression windows (additions to RFC 0002 §3.5)

| Window | Starts | Ends | Effect |
|---|---|---|---|
| **Rebuild** | S3 build activity or S4 wrapper start for a system rebuild | End + 10 s | MEM-2 in `nix-daemon.service` becomes NIX-5. CRASH-1 and SVC events of build processes (`nixbld*` uids) are dropped: test suites crash on purpose |
| **Upgrade** | `nixos-upgrade.service` start | Its end + 10 s | Same as Rebuild |

## 9. Privacy

Nix data is more revealing than it looks. Rules, on top of ADR 0007 §6:

- **Derivation names** are sent only if the package name (`pname`) is **public**, that is, it exists in nixpkgs. A cache lookup cannot decide this: a derivation that failed to build is, by definition, not in any binary cache. The options are compared in §12 (K4). Otherwise the name is replaced by `<private>`. Open point 2.
- **Build log tails** are never part of a micro report. A public `pname` does not make the log public: the user may have overridden `src` with a private fork. Tails go only into full reports, scrubbed, at most 30 lines, after the preview.
- **Flake inputs** are reduced to their type and public-ness (`github`, `gitlab`, `git`, `path`, `tarball`; `public`/`private`). URLs are never sent: `git+ssh://…/acme/internal` reveals the organisation, and tarball URLs can contain tokens.
- **Paths from Home Manager conflicts** and stub-ld binaries are never sent (§5).
- The **configuration** itself is only in full reports, under ADR 0007 §3.4 and RFC 0001 §3 rules. No N1/N2 detector reads `/etc/nixos`.

## 10. Notifications

ADR 0005 applies, with one extra rule:

> A failure the user **started by hand and saw in their terminal** is recorded but never notified, whatever its severity.

So NIX-4/NIX-15b never notify. NIX-2, NIX-8, NIX-10, NIX-13, NIX-15, NIX-16 and NIX-17 do. NIX-9 and NIX-14 are visible in the app and become a notification only at H.

## 11. Open points

| # | Point | Proposal |
|---|---|---|
| 1 | Source for manual rebuilds | §12 K8: Nix structured log if supported, else a wrapper that runs the rebuild as a transient unit. Spike in M0 (§13) |
| 2 | How to decide a `pname` is public | §12 K4: index of nixpkgs `pname`s generated at Relago build time |
| 3 | NixOS release support dates | §12 K5: computed from the release number, with a shipped override table |
| 4 | Do NIX-16/NIX-18 (application output) belong here or in a future "application problems" RFC? | Here, because the cause and the fix are NixOS-specific |
| 5 | Should NIX-9 say "reboot needed" on its own, outside of any problem? | It is useful but closer to a system-update feature. Keep it as an I problem now; move it to the Xinux update mechanism (RFC 0001 §3) once that exists |

## 12. Solutions

The catalogue is covered by eight shared building blocks (K1–K8). Each detector in §5–§6 is a small parser or rule on top of them. For every block, all options found are listed; the recommended one is marked.

| Block | Covers |
|---|---|
| K1 Unit claims | NIX-2, NIX-10, NIX-13, NIX-15 |
| K2 Nix error classifier | NIX-2, NIX-4, NIX-6, NIX-7*, NIX-11, NIX-15 |
| K3 Build output tail | NIX-2, NIX-4 |
| K4 Public-name gate | every detector that sends Nix names |
| K5 System probe | NIX-3, NIX-9, NIX-12, NIX-14, NIX-14b |
| K6 Pattern collectors | NIX-9b, NIX-16, NIX-17, NIX-17b, NIX-18 |
| K7 Build-process identification and windows | NIX-5, NIX-8, NIX-9c, §8 |
| K8 Manual-build source | NIX-4, NIX-15b |

### K1: Unit claims

A failure of `nixos-upgrade`, `nix-daemon`, `nix-gc`, `home-manager-<user>` must become its own kind, not a generic SVC-1.

| Option | Pros | Cons |
|---|---|---|
| **a. Static claim table in SVC-1 (recommended):** unit glob → claiming detector, consulted before SVC-1 emits | One place, no race, already how NIX-2 works | Adding a unit needs a code change |
| b. Separate Collectors on `UNIT=`, SVC-1 dropped by correlation (§7) | No change to SVC-1 | Depends on the 5 s window; two events for one failure if correlation overflows |
| c. User-configurable unit → kind mapping | Flexible | Kinds have closed schemas (ADR 0007); arbitrary units cannot fill them |

### K2: Nix error classifier

| Option | Pros | Cons |
|---|---|---|
| **a. Regex table on Nix's `error:` lines (recommended for text sources)** | Works on every text source (S1, S4); easy to fixture | Nix rewords messages between versions; needs the drift check (§13) |
| **b. Structured fields from the Nix JSON log (recommended when S3 is used)** | Activity types and result codes, no text parsing | Only for S3; format is documented as unstable |
| c. Exit codes only (Nix 2.34: 100 build failure, 101 timeout, 102 hash mismatch, 104 not deterministic; [manual](https://nix.dev/manual/nix/2.34/command-ref/nix-store/realise#special-exit-codes-for-build-failure)) | Trivial | Too coarse: no `no-space`, `download` or evaluation classes; only failed builds get these codes |
| d. Heuristic or ML over the whole log | Might find new classes | Unpredictable, untestable, not explainable to users |

Both a and b produce the **same** class enum, so detectors do not care which source was used.

### K3: Build output tail

| Option | Pros | Cons |
|---|---|---|
| **a. Parse the `Last N log lines:` block Nix prints under the error (recommended)** | Already in the journal (S1) and in S3/S4; no file access | Only N lines (`log-lines`, default 25, [manual](https://nix.dev/manual/nix/2.34/command-ref/conf-file#conf-log-lines)); absent if a user set `log-lines = 0` |
| **b. Read `/nix/var/log/nix/drvs/…bz2` directly (fallback)** | Full log, works for any build the daemon ran | Needs a bz2 reader; permissions and layout to verify; disabled by `keep-build-log = false` |
| c. Run `nix log <drv>` as a subprocess | Handles remote stores and settings correctly | Spawns the Nix CLI from the daemon; slower; depends on the CLI version |

### K4: Public-name gate

| Option | Pros | Cons |
|---|---|---|
| **a. `pname` index of nixpkgs, generated at Relago build time from the nixpkgs Xinux pins (recommended)** | Offline; deterministic; no network call | ~1–2 MiB (a compact set such as an FST); a private package that reuses a nixpkgs name passes (low risk: the name is already public) |
| b. Send `sha256(pname)`; the server matches it against its own nixpkgs list | Client stays small; server learns only names it already knows | Guessable private names can be brute-forced; needs server work |
| c. `narinfo` lookup on public caches | Exact for *successful* outputs | **Does not work for failures:** a derivation that failed is not in any cache |
| d. Never send derivation names | Safest | NIX-2/NIX-4/NIX-5 lose most of their value for Xinux maintainers |
| e. Ask the user each time | User decides | Only possible for full reports; micro reports are automatic (ADR 0004) |

### K5: System probe

| Question | Options | Recommendation |
|---|---|---|
| Where versions and generations come from | a. Read `/run/*-system/nixos-version` and the profile symlinks directly; b. run `nixos-version --json` / `nix-env --list-generations` | **a**; b only in debugging tools |
| What changed since boot (NIX-9) | a. Compare `kernel`, `initrd`, `kernel-modules`, systemd links; b. compare whole closures (`nix store diff-closures`) | **a**: cheap and exactly the reboot-relevant parts |
| Store growth (NIX-12) | a. `statvfs` used bytes of the `/nix` filesystem, daily; b. `du` of the store; c. sum of path sizes from the Nix DB | **a**: free and constant time. If `/nix` shares a filesystem with `/`, the trend is labelled "root filesystem" |
| Support end dates (NIX-14b) | a. Rule from the release number (`YY.05` → 31 Dec YY, `YY.11` → 30 Jun YY+1, verify against the NixOS policy); b. shipped table; c. fetched from the Relago server | **a with a b override** for exceptions; no network dependency |
| "System date" for NIX-14 | a. Date in `nixos-version`; b. last successful `nixos-upgrade` run; c. `flake.lock` modification time | **a**, with b as context. c needs reading `/etc/nixos` (privacy) |

### K6: Pattern collectors (kernel and application output)

| Option | Pros | Cons |
|---|---|---|
| **a. RFC 0002 Collectors on journal text (recommended)** | Existing framework; GNOME and KDE send app stderr to the journal | Misses apps started from a terminal |
| b. Wrap applications (wrapper scripts, `LD_PRELOAD`) to capture errors | Catches terminal launches | Invasive; changes every package; breaks reproducibility |
| c. Probe `.desktop` files and user units for dead store paths (NIX-18b) | Finds the problem before the user clicks | Reads home directories; Later |

### K7: Build-process identification and windows

Needed to tell a build that ran out of memory (NIX-5) from a user app, and to drop crashes of test suites during builds.

| Option | Pros | Cons |
|---|---|---|
| **a. Cgroup: `task_memcg=` in the kernel `oom-kill:` line, `_SYSTEMD_UNIT=nix-daemon.service` (recommended, primary)** | Precise; covers all builds the daemon runs | Builds as root without the daemon (rare) are missed |
| **b. Build uids: the `nixbld` group range, and the `auto-allocate-uids` range (fallback)** | Works when cgroup data is missing | Ranges are configurable; read them from the system config |
| c. Process ancestry through `/proc` | Exact at the moment of the event | The process is already dead when the event is read |

### K8: Manual-build source

The open question with the most options. It only matters for NIX-4/NIX-15b, which are never notified (§10).

| Option | Pros | Cons |
|---|---|---|
| **a. Nix `json-log-path` → Unix socket owned by `relago` (recommended if supported)** | Structured; covers every client, with no wrapper to bypass | Present in the shipped Nix 2.34; whether it accepts unprivileged writers is still to verify (§13.1); every user's Nix writes to it, so events can be **spoofed**: the daemon must take the owner from `SO_PEERCRED`, never from the message |
| **b. Wrapper that runs the rebuild as a transient unit (`systemd-run --pipe --wait --unit=xinux-rebuild-…`) (recommended fallback)** | The rebuild then looks exactly like `nixos-upgrade`: K1 claims it, K2 classifies it; one code path | The environment changes (SSH agent for private flakes, working directory) and must be passed through; bypassed by calling the real binary |
| c. Wrapper that tees output to the journal (`systemd-cat -t xinux-rebuild`) | Simplest | Exit code and boundaries must be logged separately; bypassable |
| d. Infer from build logs + validity checks (S2 + S7) | No system change | Racy; misses evaluation errors (no log) |
| e. Nix `post-build-hook` | Built into Nix | Runs only on **success**; useless for failures |
| f. Shell hook (`precmd`) recording the exit code of `nixos-rebuild` | No Nix change | Per-shell, fragile, user-level |
| g. Run `nix-daemon` verbosely (`-vvv`) into the journal | No new component | Huge log volume; still no client-side evaluation errors |
| h. eBPF/audit tracing of `nix` processes | Catches everything | Heavy, privileged, out of proportion |

### Per-detector alternatives

| Detector | Options | Recommendation |
|---|---|---|
| NIX-8 bootloader failed | a. Error patterns inside the Activation window; b. compare boot entries in `/boot` with generations; c. `bootctl list` | **a**. b fails because NixOS mounts the ESP `umask=0077` (root only); c needs root |
| NIX-10 `nix-daemon` down | a. K1 claim on unit failure; b. periodic connect to the daemon socket | **a**, plus b as a cheap check every 10 min (also catches a hung daemon) |
| NIX-11 store corruption | a. Error patterns in S1; b. opt-in weekly `nix-store --verify --check-contents` timer, output read by NIX-11; c. open `db.sqlite` read-only and run `integrity_check` | **a**, **b** for managed installs. c is unsafe (WAL needs write access, can disturb the daemon) |
| NIX-16 generic binary | a. stub-ld message; b. `ENOENT` exec errors for ELF interpreters | **a**. It needs `environment.stub-ld.enable` (default on since 24.05, verify); the Xinux profile keeps it on |
| NIX-17 driver mismatch | a. Kernel `NVRM: API mismatch`; b. compare `/proc/driver/nvidia/version` with the userspace version | **a**; b as enrichment |

## 13. Delivery plan

### 13.1 Fixture farm first

Every pattern in this RFC is unverified, so verification comes before detectors:

- **NixOS VM tests that cause each failure on purpose** (the table in §14), run against **both nixpkgs versions Xinux ships** (stable and unstable). Each run records `journalctl -o export` and the build log; the recordings become RFC 0002 §6 fixtures.
- **Pattern drift check:** a flake check, like RFC 0002 §4.3, that searches the Nix and nixpkgs sources pinned by Xinux for every literal pattern we depend on (stub-ld text, `Failed to install bootloader`, `is in the way of`, `Last … log lines`, …). A nixpkgs bump that rewords a message fails CI instead of silently breaking detection.
- The farm also answers open point 1: whether `json-log-path`, which the shipped Nix 2.34 has, accepts unprivileged writers.

### 13.2 Milestones

| Milestone | Work | Delivers |
|---|---|---|
| **M0: decisions** | ADR 0010 (N1 scope); new `details` kinds in ADR 0007; K8 spike | Unblocks the rest |
| **M1: fixture farm** | §13.1 | Verified patterns, regression tests |
| **M2: simple wins** | K1, K2 (text), K5 | NIX-10, NIX-13, NIX-15, NIX-2 classes, NIX-9, NIX-14, NIX-14b |
| **M3: build output** | K3, K4 | NIX-2 tail and `pname`, NIX-6 |
| **M4: correlation** | K6, K7 | NIX-5, NIX-8, NIX-16, NIX-17 (end of N1) |
| **M5: N2** | K8, K2 (structured), more K5 | NIX-4, NIX-7*, NIX-7f, NIX-9b, NIX-9c, NIX-11, NIX-12, NIX-17b, NIX-18 |

M2 starts after the main roadmap's Phase 2c (NIX-1, NIX-2 and the Activation window) has landed.

### 13.3 Source references still needed

Every identifier below is marked "verify" or comes from memory. As in RFC 0002 §4.1 and §4.4, each needs a link to its source, pinned to the Nix, nixpkgs or Home Manager version Xinux ships. The fixture farm (§13.1) confirms the behaviour, and the reference records where it is defined.

| Identifier | Used in | Source to cite |
|---|---|---|
| The Nix version that introduced `json-log-path` (the setting itself is referenced in §3) | S3, K8 | Nix release notes |
| Build log layout `/nix/var/log/nix/drvs/…` and its permissions | S2, K3 | Nix source |
| `Last N log lines:` block | K3 | Nix source |
| `nixbld` uid range, `auto-allocate-uids` | NIX-5, K7 | Nix manual (`nix.conf`) |
| stub-ld message text; `environment.stub-ld.enable` default | NIX-16 | `nixos/modules/config/stub-ld.nix` in nixpkgs |
| `Failed to install bootloader`, `systemd-boot-builder.py` errors, `grub-install` errors | NIX-8 | the NixOS bootloader modules in nixpkgs |
| ESP mounted with `umask=0077` | NIX-8 alternatives | the NixOS installer and filesystem modules |
| `kernel`, `initrd`, `kernel-modules`, `sw/lib/systemd` links in a system closure | NIX-9, K5 | `nixos/modules/system/activation/top-level.nix` |
| `modprobe: FATAL: Module … not found` | NIX-9b | kmod source |
| `Existing file '…' is in the way of`; `home-manager.backupFileExtension` | NIX-15 | Home Manager source |
| `NVRM: API mismatch` | NIX-17 | `open-gpu-kernel-modules` |
| Store corruption messages (`database disk image is malformed`, `path '…' is not valid`, …) | NIX-11 | SQLite error strings; Nix source |
| NixOS release support periods | NIX-14b, K5 | the NixOS release policy |

### 13.4 Rollout

- **Active from the first release of each milestone** (very low noise): NIX-8, NIX-10, NIX-17, the NIX-2 changes.
- **Shadow for one release** (RFC 0002 §3.7, [ADR 0012](../adr/0012-shadow-mode-rollout.md)): all others, promoted on their shadow counts.
- **Never notified:** NIX-4 and NIX-15b (§10), however accurate.

## 14. Required decisions and follow-ups

This RFC only becomes effective with:

- **[ADR 0010](../adr/0010-nix-problem-scope.md)** (Proposed), which supersedes the "NIX-4 not planned" part of the RFC 0001 §5.5 scope decision and accepts the N1 scope;
- **new `details` kinds** in ADR 0007 §4 and `crates/proto/schemas`: `nix_build_oom` (NIX-5), `bootloader_failure` (NIX-8), `reboot_needed` (NIX-9), `nix_daemon_down` (NIX-10, or reuse `unit_failure` with a flag), `outdated_system` (NIX-14/NIX-14b), `home_manager_failure` (NIX-15), `foreign_binary` (NIX-16), `driver_mismatch` (NIX-17), and new fields on `upgrade_failure` (`pname`, new classes; `log_tail` in the full report only);
- updating the RFC 0001 §3 non-goal and the §5.5 NixOS table to point here;
- fixtures per RFC 0002 §6, and VM tests:

| Test | Triggers | Asserts |
|---|---|---|
| `upgrade-build-fail` | `nixos-upgrade` with a derivation that `exit 1`s | NIX-2 `build-failure` with log tail |
| `build-oom` | Build `stress-ng --vm` inside a memory-limited `nix-daemon` | NIX-5, not MEM-2 for a user app |
| `bootloader-full` | Fill a tiny `/boot`, then switch | NIX-8 C, NIX-1 linked, one notification |
| `reboot-needed` | Switch to a config with another kernel | NIX-9 I, resolved after reboot |
| `hm-conflict` | Pre-create a file Home Manager manages | NIX-15 `file-conflict` for that user only |
| `stub-ld` | Run a generic dynamically linked binary from a user unit | NIX-16 H once, then folded |
