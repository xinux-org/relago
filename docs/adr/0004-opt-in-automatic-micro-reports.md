# ADR 0004: Opt-in automatic micro reports; full reports always ask

- **Status:** Proposed
- **Date:** 2026-10-17
- **Deciders:** Relago maintainers
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §6.4, Q4; [ADR 0007](0007-report-structure.md) (micro reports); [ADR 0003](0003-problem-visibility.md)

## Context

Today nothing is sent unless the user clicks "Send Report" in a window that pops up for every crash, and what is sent is a full system dump. The UI claims the report is collected anonymously, which is not true.

With micro reports (ADR 0007), a single report is small, scoped to the crash, and built from an allow-list. That makes automatic sending acceptable, *if the user agreed*. Asking for every crash is the main source of the "Relago keeps popping up" complaint.

## Decision

- **Consent is opt-in, per user.** When the first High/Critical problem for a user appears, the Relago app (ADR 0008) shows a one-time consent dialog ("Automatically send small crash summaries to Xinux?"), with a preview of an example micro report. The answer is stored per uid in `consent.json`.
- **Opted in:** micro reports for that user's problems are sent automatically, only for the first occurrence of each fingerprint. Later occurrences update counts. The notification is informational ("Firefox crashed; a summary was sent").
- **Opted out or undecided:** nothing is sent until the user clicks "Report…".
- **Full reports always require explicit confirmation** in the Relago app, regardless of consent, with per-category opt-in.
- **System-owned problems** (ADR 0003) are auto-sent only under the consent of an admin, never under a non-admin's consent.
- Consent can be changed any time in the Relago app's settings (ADR 0008) and via `relago consent on|off`. The NixOS module can set a system default and lock it for managed installs.
- The "collected anonymously" wording is removed. The consent dialog states exactly what is sent.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Opt-in auto micro, full always asks (chosen) | Few prompts after one decision; informed consent; matches GNOME/ABRT practice | Fewer reports than opt-out; needs a consent UI |
| Always ask per problem | Strongest control | Recreates the popup fatigue we are fixing |
| Auto by default, opt-out | Most data | Weak consent for data leaving the machine; poor fit for a privacy-respecting distro |

## Consequences

- New UI: consent dialog, consent toggle in the Relago app's settings (ADR 0008), `relago consent` CLI.
- The micro-report builder must be allow-list based, with snapshot tests proving that excluded fields never appear (RFC 0001 §12).
- Report volume depends on opt-in rate. The server should not assume full coverage.
- Managed deployments (schools, government) can enforce a policy through the NixOS module.
