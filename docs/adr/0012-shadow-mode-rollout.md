# ADR 0012: New detectors ship in shadow mode for one release

- **Status:** Proposed
- **Date:** 2026-11-05
- **Deciders:** Relago maintainers
- **Related:** [RFC 0002](../rfc/0002-detectors.md) §3.7, §5, open point 1; [RFC 0004](../rfc/0004-nix-problems.md) §5, §13.4; [ADR 0004](0004-opt-in-automatic-micro-reports.md) (consent), [ADR 0005](0005-notification-threshold.md) (threshold)

## Context

Relago is being redesigned largely because of popup fatigue: the current version notifies for every journal entry it matches. Most new detectors could bring that back:

- **Unverified inputs.** Only the systemd message IDs are verified against the shipped systemd (RFC 0002 §4.1). Kernel, smartd, Nix and application message patterns are written from memory, and a wrong pattern fires on harmless events.
- **Unknown noise.** For many detectors it is not known how often harmless events trigger them: users hold the power button (PWR-1), corrected hardware errors are frequent (HW-2), some drivers warn constantly (KERN-2), some GNOME user services fail harmlessly (SVC-2).
- **Data quality.** Under automatic consent (ADR 0004), false problems would also be sent to the server.

RFC 0002 §3.7 proposed shadow mode for all new detectors, with a short list of very-low-noise exceptions, and RFC 0004 §13.4 did the same for the Nix detectors. The individual detector specs disagreed and marked most detectors active.

## Decision

1. **Every new detector ships in shadow mode for its first release.** In shadow mode a detector runs fully: its problems are fingerprinted, scored and stored with `state = shadow`, but they are **never notified and never sent**. Counts are available through `relago detectors stats` and `GetDetectorStats()` (RFC 0002 §3.7).
2. **Exceptions**, active from their first release because their noise is very low:
   - from RFC 0002: CRASH-1, SVC-1, MEM-1, MEM-2, KERN-1;
   - from RFC 0004: NIX-8, NIX-10, NIX-17, and the amendments to NIX-2.
3. **Promotion** to active is a change of the detector's default rollout in a later release, decided by the maintainers per detector from its shadow counts and fixture results. A detector can also stay in shadow longer, for example HW-1 for drivers without fixtures, or NIX-14 while the clock is not synchronised.
4. **Local override.** An admin can force a detector active or into shadow with `[detectors.<ID>] shadow = …` in `config.toml` or `services.relago.detectors.<ID>.shadow` in the NixOS module (RFC 0002 §3.8). VM tests run every detector active.
5. **Changes to the exception list** need a new ADR.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Shadow for one release, very-low-noise exceptions (chosen) | Protects against the popup problem the redesign exists to fix; one predictable rule; tuning happens on real machines before users are disturbed | The first release notifies for only a few detectors; value of the others is delayed by a release |
| All detectors active from the start | Full coverage immediately | Unverified patterns and unknown noise reach users directly; risks recreating popup fatigue |
| Shadow only for detectors that can raise High or Critical problems and parse free text or have Medium or higher noise | Fewer detectors delayed; detectors that never notify gain nothing from shadow anyway | Needs a judgement per detector and per release; harder to apply consistently |
| Everything in shadow, including core detectors | Maximum caution | The first release does nothing visible; the core detectors are already well understood |

## Consequences

- In the first release, notifications come only from CRASH-1, SVC-1, MEM-1, MEM-2 and KERN-1. Problems of every other detector are recorded with `state = shadow` but do not notify; for example low disk space (DISK-2) and session crashes (SESS-1) wait for promotion.
- The rollout line of every detector in RFC 0002 §5 and RFC 0004 §5 follows this ADR.
- **Promotion needs data that is not yet collected centrally.** Shadow counts stay on each machine unless RFC 0002 open point 1 (counts in micro reports) is accepted, which depends on the consent model of ADR 0004. Until then, promotion relies on the team's machines, the fixture farm and VM tests, and counts that volunteers share.
- Shadow problems are hidden in the Relago app by default and shown through the `shadow` state filter (RFC 0003 §4.1).
- **Signals that this was wrong:** detectors stay in shadow for many releases because nobody has data to promote them, or promoted detectors still turn out noisy, which means the counts or the promotion review are not good enough.
