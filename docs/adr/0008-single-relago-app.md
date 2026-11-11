# ADR 0008: One Relago app for problems and logs; reporter window merged into it

- **Status:** Accepted
- **Date:** 2026-10-22
- **Deciders:** Relago maintainers
- **Related:** [RFC 0003](../rfc/0003-relago-app.md); [RFC 0001](../rfc/0001-daemon-redesign.md) Q7; [ADR 0001](0001-problem-centric-daemon.md), [ADR 0003](0003-problem-visibility.md)

## Context

Users want to see what Relago collected: the problems it detected and the system logs around them, in a viewer similar to GNOME Logs. Today the only UI is a reporter window, which pops up for a single crash and shows a debug dump of the journal. A separate per-user agent process listens for crash signals.

Building a viewer as a *third* UI next to the reporter window and the agent would split the user experience across three places and triplicate the D-Bus client code.

## Decision

1. We will build **one desktop application, Relago** (`uz.xinux.relago.Relago`, crate `relago-app`), with these views:
   - **Problems**: the problem store, through the daemon,
   - **Logs**: a full, GNOME Logs–style journal browser,
   - **Sent reports**,
   - **Settings**.
2. The **reporter window is merged into the app.** "Report…" opens the problem's page in the app. The `reporter` subcommand and the separate reporter process are removed.
3. **Problem data comes only through the daemon's D-Bus API.** The app never reads the store files.
4. **Raw logs are read directly from the journal by the app, as the user.** journald enforces permissions, and the app never escalates privileges.
5. The **app's background mode replaces the `relago gui` agent** (receiving signals, showing notifications). D-Bus activation starts it when needed.
6. The daemon's D-Bus interface (`uz.xinux.relago.Daemon1`) is a **public, versioned API** (RFC 0003 §5). The app is one client of it, not a privileged one.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| One app, reporter merged, logs + problems (chosen) | One place for everything; one D-Bus client; problems and logs linked both ways | Larger app; reporting flow depends on the app |
| Separate viewer + small reporter window | Reporter stays minimal | Two UIs, duplicated client code, inconsistent UX |
| Problems view only, logs only around problems | Smaller scope | Users still need GNOME Logs for everything else; weaker "what happened before the crash" story |
| Extend GNOME Logs upstream | No new app | Different language/toolkit stack; Relago problems are Xinux-specific |

## Consequences

- `crates/gui` becomes `crates/app`. A new `relago-journal` crate holds the journal code shared by the daemon's detectors, the report builder and the app (`Entry`, reader, scrubber, NDJSON export).
- The Logs view has no dependency on the daemon, so it can be built immediately, in parallel with the daemon redesign (RFC 0003 §9, app phase 1).
- RFC 0001 Q7 (per-user agent) is resolved: the agent is merged into the app's background mode.
- The daemon API needs paging, filters, occurrence and evidence queries, and update/remove signals (RFC 0003 §5), and stable problem IDs (the fingerprint).
