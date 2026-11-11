# ADR 0010: Detect Nix problems beyond activation and upgrades

- **Status:** Proposed
- **Date:** 2026-10-13
- **Deciders:** Relago maintainers
- **Related:** [RFC 0004](../rfc/0004-nix-problems.md); supersedes the "NIX-4 Nix build failures: not planned" part of the RFC 0001 §5.5 scope decision; [ADR 0005](0005-notification-threshold.md), [ADR 0007](0007-report-structure.md)

## Context

The first-release scope (RFC 0001 §5.5) limited NixOS coverage to NIX-1 (activation failed), NIX-2 (automatic upgrade failed) and NIX-3 (booted an older generation), and ruled out NIX-4 "Nix build failures" as too noisy: most build failures are normal developer work, not system bugs.

Nix is where Xinux differs most from other distributions, and it fails in more ways than these three: a bootloader that was not updated, a broken `nix-daemon`, builds killed for lack of memory or disk, a system that stopped receiving updates or runs an end-of-life release, failed Home Manager activations, generic Linux binaries that cannot run, and driver mismatches after an update. RFC 0004 lists them and shows that most need no system change and little noise control.

The noise argument of that scope decision holds for arbitrary `nix build` failures. It does not hold for system rebuilds and automatic runs.

## Decision

We will add the **N1 detectors of RFC 0004 §5** to Relago's scope, after the first release (after Phase 2c of RFC 0001 §11):

- NIX-2 extended (failure classes, `pname`, build output tail in full reports only),
- NIX-5 build killed for lack of memory, NIX-6 build failed for lack of disk space,
- NIX-8 bootloader update failed, NIX-9 reboot needed,
- NIX-10 `nix-daemon` down, NIX-13 garbage collection failed,
- NIX-14 system out of date, NIX-14b NixOS release end-of-life,
- NIX-15 Home Manager activation failed,
- NIX-16 generic Linux binary cannot run, NIX-17 GPU driver mismatch.

NIX-4 is **redefined** as "manual system rebuild failed" and moved to the second Nix phase (N2). It is recorded but never notified, because the user already sees the error in the terminal.

We will **not** detect failures of arbitrary `nix build`, `nix develop` or `nix-shell` runs.

Derivation names leave the machine only if their `pname` exists in nixpkgs (RFC 0004 §9, K4). Build output is never part of a micro report.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| N1 of RFC 0004, phased after the first release (chosen) | Covers the Nix failures desktop users actually hit; most need no system change; first-release scope unchanged | Eleven more detectors to fixture and tune; new `details` kinds |
| Keep the first-release scope as is (NIX-1 to NIX-3 only) | No extra work | Misses the most Xinux-specific problems, including C-severity ones (NIX-8, NIX-10) |
| All Nix problems, including arbitrary builds | Maximum coverage | Developer build failures flood the store; derivation names leak private projects |

## Consequences

- ADR 0007 §4 gains the `details` kinds listed in RFC 0004 §14; the JSON schemas change before any N1 detector ships.
- The RFC 0001 §3 non-goal on Nix build/evaluation failures and its §5.5 NixOS table point to RFC 0004.
- A fixture farm (NixOS VM tests on both shipped nixpkgs versions) and a pattern drift check against the pinned Nix/nixpkgs sources are required before the first N1 detector is merged (RFC 0004 §13.1).
- Relago's build gains a generated nixpkgs `pname` index (RFC 0004 K4).
- The source for manual rebuilds (Nix structured log or a rebuild wrapper) is **not** decided here; it is needed only for N2 (RFC 0004 §12 K8, open point 1).
- If shadow-mode counts show that an N1 detector is noisy, it stays in shadow; removing it from scope requires a new ADR.
