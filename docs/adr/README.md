# Architecture Decision Records

This directory records significant architecture decisions for Relago, one decision per file.

- **RFCs** ([`docs/rfc/`](../rfc/)) are proposals and designs under discussion. They can be long and change often.
- **Unselected plans** ([`docs/plan.md`](../plan.md)) lists detectors that were considered but are not in the current scope.
- **References** ([`docs/references.md`](../references.md)) collects the external sources the documents rely on, pinned to the shipped versions.
- **ADRs** (`docs/adr/`) record a decision: short and dated. ADRs are the source of truth. Where an RFC and an ADR differ, the ADR wins, and work items (issues) are created from ADRs, not the other way round.

## Status

| Status | Meaning |
|---|---|
| **Proposed** | Written and under discussion; not yet agreed. Also used when an accepted decision is **reopened**: the status line says when and why. |
| **Accepted** | Agreed and merged. From then on the ADR is not rewritten; only its status line changes. |
| **Deprecated** | No longer applies, without a replacement. |
| **Superseded by ADR NNNN** | Replaced by a newer ADR. |

An ADR may also **amend** an earlier one in a narrow way (for example, ADR 0009 changes only the store layout of ADR 0002). The amending ADR has an **Amends** line, and the amended ADR's status line points to it.

## Process

1. Copy [`template.md`](template.md) to `NNNN-short-title.md` using the next free number.
2. Set the status to **Proposed**, add the ADR to the index below, and open a PR.
3. While the PR is open, the ADR can be corrected freely.
4. On merge after agreement, set the status to **Accepted** in the ADR and in the index.
5. To change an accepted decision, write a new ADR that supersedes or amends it, and update the old one's status line. To reopen it for discussion instead, set it back to **Proposed** with the date and reason, and mark the RFC questions that depend on it as open.

## Conventions

- **No issue tracker references.** ADRs and RFCs do not cite issue numbers or review lists. Describe the problem in words.
- **Related line**: RFCs (with sections) and other ADRs only.
- **External identifiers** (systemd message IDs, journal fields, kernel or Nix message texts, configuration options) link to their source, pinned to the version in the nixpkgs revision of `flake.lock`. Identifiers that have no reference yet are listed in the RFC's "Source references still needed" section ([RFC 0002 §4.4](../rfc/0002-detectors.md#44-source-references-still-needed), [RFC 0004 §13.3](../rfc/0004-nix-problems.md#133-source-references-still-needed)).
- **Detector IDs** use a group prefix and a number: CRASH, SVC, SESS, MEM, KERN, HW, DISK, PWR, NIX, NET, SEC (for example `CRASH-1`, `NIX-14b`). The catalogue is [RFC 0001 §5.5](../rfc/0001-daemon-redesign.md#55-problem-catalogue).
- **Units** are binary: KiB, MiB, GiB. Timestamps are integer microseconds since the Unix epoch (ADR 0006).

## Index

| ADR | Title | Status | Date |
|---|---|---|---|
| [0001](0001-problem-centric-daemon.md) | Redesign the daemon around a problem store | Accepted | 2026-09-30 |
| [0002](0002-json-file-problem-store.md) | Store problems as JSON files | Accepted; amended by 0009 | 2026-09-30 |
| [0003](0003-problem-visibility.md) | Problems are visible to their owner; system problems to `wheel` | Accepted | 2026-09-30 |
| [0004](0004-opt-in-automatic-micro-reports.md) | Opt-in automatic micro reports; full reports always ask | Proposed (reopened 2026-11-05) | 2026-09-30 |
| [0005](0005-notification-threshold.md) | Notify for High and Critical problems only by default | Accepted | 2026-09-30 |
| [0006](0006-data-formats.md) | Data formats: JSON for bounded data, NDJSON for streams | Accepted | 2026-09-30 |
| [0007](0007-report-structure.md) | Report structure | Accepted | 2026-09-30 |
| [0008](0008-single-relago-app.md) | One Relago app for problems and logs; reporter window merged into it | Accepted | 2026-09-30 |
| [0009](0009-store-viewer-data.md) | Store journal excerpts and sent reports for the viewer; per-problem directories | Accepted | 2026-09-30 |
| [0010](0010-nix-problem-scope.md) | Detect Nix problems beyond activation and upgrades | Proposed | 2026-10-13 |
| [0011](0011-detector-contract.md) | One detector contract with closed outputs and declared capabilities | Proposed | 2026-11-05 |
| [0012](0012-shadow-mode-rollout.md) | New detectors ship in shadow mode for one release | Proposed | 2026-11-05 |
| [0013](0013-problem-fingerprint.md) | Problem fingerprint: kind plus per-kind identity parts, with variants and regression handling | Proposed | 2026-11-05 |

### By topic

| Topic | ADRs | Main RFC |
|---|---|---|
| Daemon and problem store | 0001, 0002, 0009 | [RFC 0001](../rfc/0001-daemon-redesign.md) |
| Detectors | 0010, 0011, 0012, 0013 | [RFC 0002](../rfc/0002-detectors.md), [RFC 0004](../rfc/0004-nix-problems.md) |
| Visibility and notifications | 0003, 0005 | [RFC 0001](../rfc/0001-daemon-redesign.md) §6 |
| Reports and consent | 0004, 0006, 0007 | [RFC 0001](../rfc/0001-daemon-redesign.md) §8 |
| Relago app | 0008, 0009 | [RFC 0003](../rfc/0003-relago-app.md) |

## Pending ADRs

Discussed or provisionally decided in an RFC, but not yet recorded as an ADR:

| Topic | Where discussed |
|---|---|
| Server API `/v2` (micro reports, occurrence updates, resolution, `fixed_in`) | [RFC 0001](../rfc/0001-daemon-redesign.md) Q2, §9.2; ADR 0013 §7 |
| First-release detector scope (18 detectors) | [RFC 0001](../rfc/0001-daemon-redesign.md) §5.5 |
| Boundary with support-issued patches: Relago shows resolution status only | [RFC 0001](../rfc/0001-daemon-redesign.md) §3, §9.2 |
| Stack trace source | [RFC 0001](../rfc/0001-daemon-redesign.md) Q6 |
| Retention and privacy of the local store, "Clear history" | [RFC 0001](../rfc/0001-daemon-redesign.md) Q8; [RFC 0003](../rfc/0003-relago-app.md) §10 |
| Default severities of HW-2 and PWR-1 | [RFC 0002](../rfc/0002-detectors.md) §5 |
| Shadow-mode counts in micro reports, needed to promote detectors (ADR 0012) | [RFC 0002](../rfc/0002-detectors.md) §7, open point 1 |
| Which boot the report's `system` block describes (amends ADR 0007) | [RFC 0002](../rfc/0002-detectors.md) §3.9, §7, open point 6 |
| Source for manual Nix rebuilds (structured log or wrapper) | [RFC 0004](../rfc/0004-nix-problems.md) §11, §12 K8 |
