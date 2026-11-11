# ADR 0011: One detector contract with closed outputs and declared capabilities

- **Status:** Proposed
- **Date:** 2026-11-05
- **Deciders:** Relago maintainers
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §5.1, §7.2; [RFC 0002](../rfc/0002-detectors.md) §3, §3.9, §6, open point 2; [RFC 0004](../rfc/0004-nix-problems.md) §7, §8, §12 K1; [ADR 0002](0002-json-file-problem-store.md) (store rules), [ADR 0007](0007-report-structure.md) (closed `details`)

## Context

RFC 0002 §3.1 defines four detector types: Collector, Assembler, Probe and Correlator. The simple detectors fit them: CRASH-1, MEM-1, KERN-2, HW-1, HW-2, DISK-1, DISK-3, NIX-3, NIX-16 and NIX-17. About half of the specified detectors need something the types do not offer:

- **Persistent state.** SVC-1 and SVC-2 keep pending recovery records, KERN-1 remembers seen pstore records, NIX-12 keeps a 30-day series. Each writes to `state.json` on its own.
- **Timers in a Collector.** The Collector trait has no state and no tick, but SVC-1 and SVC-2 need a 60 s recovery timer.
- **Changing existing problems.** CRASH-2 raises an existing problem's severity, NIX-5 reclassifies an OOM event, DISK-2, NIX-9 and NIX-14 resolve problems. The Correlator's `Correlation` result is not specified, and a Probe can only emit new events.
- **Inputs beyond one type.** SESS-1 needs logind journal entries as well as events. DISK-2 and NIX-14 need two schedules. NIX-9 runs after an activation, and NIX-6 triggers DISK-2.
- **I/O outside the context.** PWR-1 reads the previous boot's journal, NIX-2 reads a whole failed run, NIX-3 and NIX-9 read `/run` and the Nix profiles. `Ctx` offers none of these.
- **Central tables.** Correlation rules (RFC 0002 §3.4, RFC 0004 §7), suppression windows (RFC 0002 §3.5, RFC 0004 §8) and the unit claim table (RFC 0004 K1) are fixed lists. Adding a detector means editing them.
- **Notification special cases.** SESS-1 holds its notification until the next login (now the default delivery rule of RFC 0002 §3.9), NIX-8 suppresses NIX-1's, and manual Nix rebuilds never notify.
- **A new kind of source.** The Nix structured log socket (RFC 0004 K8) is neither journal entries nor a probe.

Left as is, each of these becomes detector-specific code in the daemon. Two forces pull against each other:

- Without a firm contract, detectors reach into the store, the policy and each other, and the framework becomes hard to test and to reason about.
- With rigid detector types, every new scope (application problems, new log sources) needs a framework change.

Two constraints are fixed by other decisions. Report `details` are closed per kind for privacy (ADR 0007). Detectors are in-tree only and the traits stay internal (RFC 0002 open point 2).

## Decision

We will define **one detector contract** that is strict about what a detector may affect and flexible about how it gets its inputs and does its work.

### 1. One trait

```rust
pub trait Detector: Send {
    fn descriptor(&self) -> &Descriptor;
    fn on_input(&mut self, input: DetectorInput, ctx: &mut Ctx) -> Vec<DetectorOutput>;
    fn on_tick(&mut self, now: Instant, ctx: &mut Ctx) -> Vec<DetectorOutput> { vec![] }
}

pub enum DetectorInput<'a> {
    Entry(&'a Entry),            // journal entry matching a subscription
    Schedule(ScheduleId),        // a declared schedule fired
    Event(&'a RawEvent),         // another detector's event, before correlation
    Window(WindowChange),        // a suppression window opened or closed
}

pub enum DetectorOutput {
    Emit(RawEvent),
    Update(ProblemId, ProblemChange),
    Resolve(ProblemId),
    Link(EventRef, EventRef),
    Claim(EventRef),
}
```

Collector, Assembler, Probe and Correlator stay as descriptive labels for common combinations of subscriptions. They are no longer separate traits.

### 2. Strict: what every detector must satisfy

- **Closed outputs.** A detector affects the system only through `DetectorOutput`. The pipeline applies the outputs and may refuse or downgrade them (shadow mode, flood limits, suppression windows). A detector never writes to the store, the policy or another detector directly.
- **Closed `details`.** Every emitted event carries a `Details` variant of a declared kind, with the closed schema of ADR 0007.
- **All I/O through `Ctx`.** `Ctx` provides the clock, journal queries (including previous boots), filesystem roots (`/sys`, `/proc`, `/run`, `/nix/var`, all overridable in tests), the store read-only, the detector's own state, and its config. Nothing else.
- **Bounded and isolated.** Flood limits, memory caps and panic isolation (RFC 0002 §3.6) are applied by the framework, without the detector's cooperation.
- **Declared up front.** The `Descriptor` holds the detector's ID, scope prefix, kinds, subscriptions, claims, windows it opens, correlation rules, notification hints, config schema, state schema version and default rollout.

### 3. Flexible: what a detector chooses

- **Subscriptions.** Any combination of journal matches, schedules (on start, once per boot, every *N*, after a window closes), other detectors' events, and window changes. A dependency between detectors is a subscription: DISK-2 subscribes to `no-space` events from NIX-2 and NIX-4 instead of being called by them.
- **Own state.** Each detector has a private, versioned state value. The daemon persists it with the store's atomic-write rules (ADR 0002), restores it on start, and upgrades it by schema version.
- **Declared cross-detector behaviour.** Claims, windows and correlation rules are declared in the descriptor of the detector that needs them. The central tables of RFC 0002 §3.4–§3.5 and RFC 0004 §7–§8 become generated views of the descriptors (`relago detectors list`).
- **Notification hints** from a closed set: default, never notify, and supersede another detector's notification for the same incident. Holding a notification until an eligible session exists is not a hint: it is the default delivery rule for every problem (RFC 0002 §3.9, rule 4).

### 4. Extension rules

- **Adding a detector** touches only its own module, its `Details` variant and schema, and its fixtures. Detector-specific code anywhere else in the daemon is a defect in the contract, to be fixed by extending the contract.
- **A new input source** (such as the Nix structured log) is added on the daemon side as a new `DetectorInput` variant. Existing detectors are unaffected.
- **A new kind** is an optional, versioned schema addition under ADR 0007 §7.
- **A new `DetectorOutput` variant or notification hint** changes the contract and needs a new ADR.

### 5. Naming

The types are `DetectorInput`, `DetectorOutput` and `ProblemChange`. We avoid "Effect", which suggests an effect system; "Action", already used for notification buttons and polkit actions; and "Request", already used for HTTP requests and `requested_by`.

### 6. Execution model

Detectors run **concurrently with each other**, and **never concurrently with themselves**.

- **One task per detector.** Each detector is owned by its own task with a bounded inbox. All of its inputs (entries, schedule ticks, events, window changes) arrive through that inbox, in order. A detector is never called from two places at once, so `&mut self` is safe and detectors need no locks. The trait requires `Send`, not `Sync`.
- **One journal reader, fan-out.** The daemon keeps a single journal reader (RFC 0001 §5.1). It reads each entry once and delivers a shared reference to the inbox of every detector whose subscription matches.
- **Blocking I/O off the async runtime.** Detector tasks run on a blocking thread pool, because `Ctx` I/O (journal queries, `statvfs`, sysfs and pstore reads) is blocking. A slow detector delays only itself.
- **Deterministic order before correlation.** The pipeline tags every `DetectorOutput` with the ordering key of the input that produced it: the journal cursor for entries, the firing time for schedules and ticks. Detectors do not handle the key themselves. The correlation window (RFC 0002 §3.4) already buffers for 5 s. It sorts outputs by that key before applying claims, links and merges, so results do not depend on which detector finished first.
- **Boot batch.** Detectors scheduled once per boot run as one batch at start. Their outputs are correlated together when the whole batch has finished, not within the 5 s window; a detector still running after 60 s is correlated on its own (RFC 0002 §3.9, rule 3).
- **Backpressure.** When a detector's inbox is full, the reader waits. The per-detector flood limits (RFC 0002 §3.6) keep this rare. A detector that stays blocked beyond a timeout is treated like a panicked detector: it is disabled until restart and a `relago-internal` problem is recorded.
- **Cursor low-water mark.** The persisted journal cursor advances only to the oldest entry that every subscribed detector has finished, and only after the resulting problem data is written (RFC 0001 §7.2). A crash therefore never skips an entry that a slow detector had not processed. Re-processed entries are de-duplicated by `dedup_key` (RFC 0002 §3.4).
- **Detector state** is saved by the daemon between calls, never during one, so a snapshot always reflects whole inputs.

Rejected for the execution model:

- **All detectors on one thread** (RFC 0002 §3.2 today): simple ordering, but one slow detector or blocking `Ctx` call stalls every other detector.
- **Detectors called from any thread** (`Sync` detectors): more parallelism, but every detector needs internal locking, and outputs are no longer reproducible in tests.

### 7. Testing

Detectors are functions of their state, their input and `Ctx`, so every test is deterministic with a fake context.

1. **Unit tests** with a `DetectorBench`: one detector with a fake `Ctx` (fake clock, journal backed by `journalctl -o export` files, temporary filesystem root, in-memory state). Feed inputs, advance the clock, snapshot the outputs (`insta`).
2. **Fixture scenarios** per detector, in `crates/collect/tests/fixtures/<ID>/<scenario>/`: `input.export`, optional `sysfs/`, `script.toml` (clock steps, ticks, restart points) and `expected.json` (the `DetectorOutput` list). At least one positive and two negative scenarios per detector (RFC 0002 §6.1).
3. **Contract tests**, run automatically for every registered detector:
   - descriptor is valid, IDs are unique, default config parses against its schema;
   - emitted kinds match the declared kinds, and every `details` validates against its JSON schema;
   - no identifier (hostname, username, uid, IP, machine ID) appears in any output;
   - the same inputs give the same outputs, regardless of how detector calls are interleaved;
   - saving and restoring state midway gives the same outputs as an uninterrupted run;
   - floods stay within bounds, and fuzzed journal entries never cause a panic;
   - no two detectors declare conflicting claims on the same input.
4. **Pipeline tests** with a `PipelineBench`: several detectors through the real suppression, correlation and claim steps, to test the declared cross-detector behaviour. They run on a deterministic scheduler that can replay detector completions in any order, so ordering bugs show up as reproducible differences, not flaky tests.
5. **Unchanged**: the constant and pattern drift checks (RFC 0002 §4.3, RFC 0004 §13.1), NixOS VM tests, and shadow mode in production (RFC 0002 §3.7).

## Options considered

| Option | Pros | Cons |
|---|---|---|
| One trait, closed outputs, declared capabilities (chosen) | Every detector is self-contained; cross-detector behaviour is visible in one place per detector; contract tests cover all detectors automatically; new sources and scopes need no detector changes | `Descriptor` and `Ctx` grow; the pipeline must apply declarations generically; more up-front framework work |
| Keep the four types of RFC 0002 §3.1, special-case the rest in the daemon | Least framework work now | About half the detectors become special cases; central tables keep growing; behaviour is hard to test in isolation |
| Four types, each extended with the missing hooks | Familiar names | Hooks duplicated across types; detectors that combine types (SVC-1, NIX-1) still do not fit one |
| Detectors with direct access to the store, policy and journal | Maximum flexibility | Untestable without a live system; any detector can leak data or break another; no enforceable privacy guarantee |
| Dynamically loaded plugins (shared libraries, WASM, scripts) | Third parties could add detectors | Conflicts with closed `details` and the privacy allow-list; needs a stable ABI; out of scope per RFC 0002 open point 2 |

## Consequences

- RFC 0002 §3.1–§3.5 and §6 are rewritten to implement this contract. Each detector's "Type" line in RFC 0002 §5 and RFC 0004 §5 becomes a list of subscriptions, state and declarations.
- The correlation and window tables in RFC 0002 §3.4–§3.5 and RFC 0004 §7–§8, and the claim table of RFC 0004 K1, move into the descriptors of the detectors they belong to. The RFC tables remain as documentation, generated from or checked against the descriptors.
- `Ctx` gains journal queries, configurable filesystem roots and per-detector state. The test fakes grow with it, once.
- RFC 0002 §3.2 (the single-thread statement and the thread groups in its pipeline diagram) is updated to the execution model of §6. RFC 0001 §7.2 gains the cursor low-water mark.
- The daemon needs per-detector task supervision: inbox sizes, a blocked-detector timeout, and restart-safe state snapshots. These are framework code, written once.
- The pipeline must resolve declarations deterministically, for example two detectors claiming the same event. Conflicts are rejected at start and by the contract tests.
- No detector code exists yet, so nothing has to migrate. The existing `crash-event` macro can remain an implementation helper inside detectors (RFC 0001 §5.2).
- **Signals that this was wrong:** pull requests that add a detector also change framework files; `DetectorOutput` or the notification hints need frequent additions; detector-specific branches appear in the daemon or the policy; contract tests are routinely skipped.
