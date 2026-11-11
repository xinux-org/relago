# ADR 0009: Store journal excerpts and sent reports for the viewer; per-problem directories

- **Status:** Accepted
- **Date:** 2026-10-20
- **Deciders:** Relago maintainers
- **Amends:** [ADR 0002](0002-json-file-problem-store.md) (layout only; the JSON-file decision and its rules stay)
- **Related:** [RFC 0003](../rfc/0003-relago-app.md) §4, §8; [ADR 0006](0006-data-formats.md), [ADR 0007](0007-report-structure.md), [ADR 0008](0008-single-relago-app.md)

## Context

The Relago app (ADR 0008) shows problems together with their log evidence and the reports sent about them. Two facts make the current store design (ADR 0002) insufficient:

- **journald rotates and vacuums logs.** A cursor recorded for a crash two weeks ago may no longer resolve, so the viewer would have nothing to show.
- **Nothing records what was sent.** Users cannot check what Relago transmitted about them. That is a transparency gap, especially with automatic micro reports (ADR 0004).

ADR 0002 stores each problem as one file, `problems/<fingerprint>.json`, and loads every problem into memory at start. Putting excerpts and report copies into that file would inflate start-up time and memory.

## Decision

1. **Per-problem directory**, replacing the single file:

   ```
   /var/lib/relago/problems/<fingerprint>/
   ├── problem.json                 summary + occurrence ring buffer (as in ADR 0002, loaded at start)
   ├── excerpts/<occurrence>.json   scrubbed journal excerpt of one occurrence (loaded on demand)
   └── sent/
       ├── <report_id>.micro.json   exact micro report that was sent
       └── <report_id>.manifest.json  manifest of a sent full report (archive not kept)
   ```

   `problem.json` gains a `sent` index (`report_id`, tier, `sent_at`, `server_status`, `send_mode`). Each occurrence stores **all** evidence cursors (`cursors`) instead of one.

2. **Journal excerpts.** For each occurrence, store the scrubbed excerpt, which is the same data as the micro report `journal` field (≤ 50 lines, ADR 0007 §5–6). Excerpts are kept for the **first** occurrence and the **3 most recent**; excerpts of other occurrences are deleted as the ring buffer moves.
3. **Sent reports.** Keep every sent micro report (≤ 64 KiB, ADR 0007) and the manifest of every sent full report. Full-report archives are never kept.
4. **Formats** follow ADR 0006: all of these are bounded, so plain JSON.
5. **Size budget.** The whole store is capped at **64 MiB** (configurable). When over budget, the daemon evicts in this order:
   1. excerpts of resolved/ignored problems,
   2. sent-report copies older than 90 days,
   3. whole problems by oldest `last_seen`.

   The 1 000-problem cap from ADR 0002 stays.
6. **Atomic operations.** The rules of ADR 0002 (temp + rename, quarantine, write-behind) apply to every file. A problem directory is created by writing `problem.json` last, so a directory without `problem.json` is garbage and is removed at start.
7. **Access.** These files are served only through the D-Bus API (`GetExcerpt`, `ListSentReports`, `GetSentReport`), with ADR 0003 visibility. The store remains `0700 relago`.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Per-problem directory, lazy excerpts and report copies (chosen) | Start-up still loads only small summaries; viewer always has evidence; full transparency | More files; eviction logic needed |
| Everything inside `problem.json` | One file per problem | Large files loaded at start; rewrites of big files on each occurrence |
| No excerpts; rely on journal cursors only | Smallest store | Evidence disappears after journal rotation |
| Keep full-report archives too | Complete record | Can be hundreds of MiB each; sensitive data kept at rest |

## Consequences

- ADR 0002's layout (`problems/<fingerprint>.json`) is superseded by this layout. Its other rules are unchanged. RFC 0001 §7.1 is updated.
- Worst case, the store now uses tens of MiB instead of a few. The 64 MiB budget bounds it.
- "Clear history" (RFC 0001 Q8) must delete excerpts and sent-report copies too.
- Tests: eviction order, budget enforcement, cleanup of directories without `problem.json`, and that excerpts are scrubbed before being written.
