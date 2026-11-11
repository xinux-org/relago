# ADR 0005: Notify for High and Critical problems only by default

- **Status:** Accepted
- **Date:** 2026-10-18
- **Deciders:** Relago maintainers
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §6.2, §6.3, Q5; [ADR 0003](0003-problem-visibility.md)

## Context

Problems are assigned one of four severities (RFC 0001 §6.2):

- **Critical:** a user's application crashed or was OOM-killed; data-loss risk.
- **High:** a system service failed and stayed failed, or keeps failing; kernel/GPU/storage errors.
- **Low:** transient failures that recovered on their own; corrected hardware errors.
- **Info:** known or ignored problems.

Notifying for everything reproduces the current popup fatigue. Notifying only for Critical hides broken system services from admins.

## Decision

By default, desktop notifications are shown for **High and Critical** problems only. Low and Info problems are recorded in the store and visible in the problem list, but produce no notification.

The rate limits of RFC 0001 §6.3 still apply on top of the threshold:
- one notification per fingerprint per 24 h,
- at most 3 per 10 min per user,
- a boot grace period of 120 s,
- ignore lists.

The threshold is configurable in `config.toml` (`[policy] notify_min_severity`) and in the NixOS module.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| High and above (chosen) | Admins learn about persistent service failures; transient noise stays silent | Depends on correct High/Low classification (recovery detection) |
| Critical only | Quietest | Persistent system failures go unnoticed |
| Everything except Info | Most complete | Noisy; restarts that healed themselves still interrupt the user |

## Consequences

- Correct severity classification becomes important, especially "failed and did not recover" for services (RFC 0002 SVC-1). Misclassification shows up as either noise or missed problems.
- Detectors in shadow mode (RFC 0002 §3.7, [ADR 0012](0012-shadow-mode-rollout.md)) never notify, whatever their severity.
