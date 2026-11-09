# ADR 0002: Store problems as JSON files

- **Status:** Accepted
- **Date:** 2026-10-17
- **Deciders:** Relago maintainers
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §7, Q1; [ADR 0001](0001-problem-centric-daemon.md)

## Context

The problem-centric daemon (ADR 0001) needs local, persistent state:
- problems and their recent occurrences,
- the journal cursor,
- pending "did it recover?" evaluations,
- ignore lists,
- per-user consent.

The expected volume is small (hundreds of problems, <= 20 occurrences each). There is exactly one writer, the daemon. The data must survive crashes and upgrades, and developers must be able to inspect it easily.

## Decision

We will store state as **JSON files** under `/var/lib/relago` (`StateDirectory=relago`, mode `0700`):

- `problems/<fingerprint>.json`: one file per problem, including a ring buffer of its last occurrences.
- `state.json` (journal cursor, pending evaluations, schema version), `ignores.json`, `consent.json`.

Rules (RFC 0001 §7.2):
- **Single writer:** only the daemon writes, and it keeps all problems in memory.
- **Atomic writes:** temp file + `fsync` + `rename` + directory `fsync`.
- **Flushing:** write-behind at most once per second.
- **Ordering:** the journal cursor is written *after* the problem data it covers.
- **Corrupt files** are quarantined to `problems/.corrupt/`. The daemon never refuses to start because of one bad file.
- **Versioning:** every file carries a `schema` version and is upgraded on load.
- **Limits:** retention caps and a hard limit of 1 000 problems.
- **Encapsulation:** all access goes through a `ProblemStore` trait.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| JSON files (chosen) | No new native dependency; human-readable (`jq`); trivial backups; easy to test | We implement atomicity, retention and indexing ourselves; querying is in-memory only |
| SQLite (`rusqlite`) | Transactions, queries, mature | C dependency; opaque to users; more than we need at this volume |

## Consequences

- Crash-safety must be tested explicitly (leftover `.tmp`, truncated files).
- All problems are loaded into memory at start. This is fine at ≤ 1 000 problems, and the hard cap enforces it.
- Queries such as "problems for uid X with severity ≥ H" are in-memory scans. That is acceptable at this size.
- **Revisit if:** we need cross-problem queries on large histories, multiple writers, or the store exceeds the cap regularly. The `ProblemStore` trait keeps a move to SQLite local to one module. A superseding ADR would record that.
