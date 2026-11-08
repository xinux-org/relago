# ADR 0003: Problems are visible to their owner; system problems to `wheel`

- **Status:** Accepted
- **Date:** 2026-10-16
- **Deciders:** Relago maintainers
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §6.2, §9.1, Q3; [RFC 0002](../rfc/0002-detectors.md) §3.3 (`Owner`)

## Context

Xinux machines can have several users. Problems come in two flavours:

- **User-owned:** a crash of a process run by a user (coredump with that uid, OOM kill in `user-<uid>.slice`, a failing user service).
- **System-owned:** system service failures, kernel/hardware/storage problems, NixOS activation and upgrade failures.

Today the signal is broadcast on the system bus. Every session's agent reacts, so user A sees user B's crashes, including executable names and messages. Access to the daemon is gated only by membership in the `relago` group, which nobody is added to.

## Decision

- Every problem has an `Owner`: `User(uid)` or `System`, determined by its detector (RFC 0002).
- **User-owned** problems are visible to, and notified to, **only that uid**.
- **System-owned** problems are visible to, and notified to, **members of `wheel`**.
- The daemon identifies D-Bus callers by their Unix uid (`GetConnectionUnixUser`) and filters `ListProblems`/`GetProblem` accordingly. It does not rely on group membership for authorization.
- Signals (`ProblemAdded`) carry the owner uid and no sensitive details. Clients, such as the Relago app (ADR 0008), fetch details via methods, which are filtered.
- Actions affecting system scope (system-wide ignore, registration) require polkit authorization (admin).

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Owner + `wheel` (chosen) | Privacy between users; admins learn about system problems | Needs uid-based filtering and polkit |
| Everyone sees everything | Simplest | Leaks one user's activity to others |
| Owner only; system problems never notified | Maximum privacy | Nobody is told when the system itself is broken |

## Consequences

- The `relago` group is no longer an authorization mechanism. The D-Bus policy allows any local user to call the daemon, and the daemon filters.
- Single-user machines, the common case, see everything, because the user is in `wheel`.
- Multi-seat or shared machines with non-admin users get privacy by default.
- Micro-report consent for system problems follows the admin who acts on them (ADR 0004).
