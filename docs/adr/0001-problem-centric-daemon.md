# ADR 0001: Redesign the daemon around a problem store

- **Status:** Accepted
- **Date:** 2026-10-10
- **Deciders:** Relago maintainers
- **Related:**

## Context

The current daemon reads the journal and, for every matching entry, emits a D-Bus signal. The per-user GUI agent then opens a new reporter window for each signal. The reporter window, running as the desktop user, collects the whole journal and `/etc/nixos`, encrypts the result, and uploads it.

The codebase review traced most of the serious bugs to this design:

- **Wrong or missing detection:** detection keys on field presence, and only coredumps reach the user.
- **Lost crashes:** crashes that happen while no GUI is running are lost.
- **Permissions:** privileged work runs as the desktop user, so it either fails with permission errors or exposes keys and reports
- **Report size and privacy:** reports are huge and privacy-invasive.

Fixing these one at a time would keep the flawed flow: journal entry → signal → window → full report.

## Decision

We will redesign the daemon around a **persistent problem store**:

1. **Detectors** turn journal entries and system probes into problem events (specified in RFC 0002).
2. Events are **fingerprinted and de-duplicated** into problems with occurrence counts.
3. A **severity** and **notification policy** decides whether the user is told.
4. The **daemon owns all privileged work**: journal access, keys, report building, encryption, upload. It runs as the `relago` system user.
5. The GUI becomes a **thin D-Bus client** (`uz.xinux.relago.Daemon1`). The daemon no longer depends on GUI crates; shared types live in a new `relago-proto` crate.
6. The migration is **incremental**, not a full rewrite. Phase 0 hotfixes land first on the current code.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Problem-centric redesign, incremental (chosen) | Fixes the root cause; every phase ships on its own | Larger total effort than patching; temporary coexistence of old and new code |
| Patch the current design issue by issue | Smallest steps | Flow stays flawed; dedup/policy have nowhere to live; privilege problems remain |
| Rewrite from scratch in one go | Clean result | Long period with no releases; high risk; blocks bug fixes |
| Adopt ABRT | Existing project | Fedora-centric, heavy, no NixOS awareness, different server protocol |

## Consequences

- New crates: `relago-proto`, `relago-collect`. `daemon` stops depending on `gui`.
- The D-Bus interface changes from `uz.xinux.relago.DaemonService` to `uz.xinux.relago.Daemon1`. The old interface stays until Phase 6.
- The `relago` system user needs `systemd-journal` group membership; the desktop user no longer needs access to `/var/lib/relago`.
- NixOS VM tests become part of `nix flake check`.
- **Signal that this was wrong:** users still see repeated windows for the same problem, the fingerprinting or policy design needs revisiting.
