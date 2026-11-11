# ADR 0006: Data formats: JSON for bounded data, NDJSON for streams

- **Status:** Accepted
- **Date:** 2026-10-22
- **Deciders:** Relago maintainers, report server team
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §7, §8; [ADR 0002](0002-json-file-problem-store.md); [ADR 0007](0007-report-structure.md)

## Context

Relago produces structured data in several places: the local problem store, micro reports, and full reports. Full reports can contain very large amounts of journal data.

Today the formats are inconsistent. `relago report` writes the journal as NDJSON when collecting all entries, but as one large JSON array with `--recent`. A large JSON array must be held in memory in full, both when building it (the client) and when parsing it (the server), unless both sides use awkward streaming parsers. For big reports this causes high memory use and slow processing on both sides. The current pipeline also compresses twice, and reads the whole report into memory for encryption.

## Decision

We choose the format by whether the data is **bounded**:

1. **Bounded data uses JSON** (one UTF-8 object per file or request body):
   - micro reports (`POST /v2/problems`, ≤ ~16 KiB, schema `relago.micro/N`),
   - full-report `manifest.json` and `system.json`,
   - local store files (`problems/<fingerprint>/*.json` in the layout of ADR 0009, `state.json`, `ignores.json`, `consent.json`; ADR 0002).
2. **Unbounded, record-oriented data uses NDJSON**: one JSON object per line, `\n`-terminated, UTF-8, no pretty-printing:
   - journal entries in full reports (`journal.ndjson`),
   - any future unbounded sequence (e.g. occurrence history, if the cap in ADR 0002 is ever lifted).
3. **The full report container** is `tar` → `zstd` → OpenPGP (SEIPDv2). Every stage is streamed: the client never holds the whole report in memory, and the server can decrypt, decompress, unpack and parse `journal.ndjson` line by line. There is no additional compression inside the archive.
4. **Every JSON document and NDJSON stream is versioned.**
   - JSON documents carry a `schema` field.
   - An NDJSON stream is described in `manifest.json` (`{"path": "journal.ndjson", "schema": "relago.journal/1", "lines": N}`), so individual lines stay small and the reader knows the record type before it starts.
5. **Encoding rules:**
   - Journal field values that are not valid UTF-8 are encoded as `{"b64": "…"}`.
   - Timestamps are integers in microseconds since the Unix epoch.
   - A reader must skip a line it cannot parse, and count it, rather than rejecting the whole stream.

## Options considered

| Option | Pros | Cons |
|---|---|---|
| JSON for bounded + NDJSON for streams (chosen) | Constant-memory streaming where size is unbounded; simple JSON where it is not; both readable with standard tools (`jq`, `jq -c`) | Two formats to know; NDJSON is not a single valid JSON document |
| JSON everywhere | One format | Large arrays require full buffering or streaming parsers; memory spikes on client and server |
| NDJSON everywhere | One format, always streamable | Awkward for single documents (manifest, micro report, config-like state); no benefit for small data |
| Binary (CBOR, MessagePack, protobuf) | Smaller, faster | Not human-readable; harder to debug and inspect reports; zstd already removes most of the size difference |

## Consequences

- The report builder writes the journal directly to the tar/zstd/PGP stream, one entry at a time. `collect_journal_all` and `collect_journal_recent` merge into one NDJSON writer. Reading the whole file into memory for encryption goes away.
- The server must parse `journal.ndjson` as a stream and tolerate bad lines. The micro report is an ordinary JSON body.
- `manifest.json` becomes part of the public report contract, together with `relago.micro/N` (ADR 0007).
- Local store files remain plain JSON (ADR 0002 unchanged).
