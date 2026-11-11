# ADR 0013: Problem fingerprint: kind plus per-kind identity parts, with variants and regression handling

- **Status:** Proposed
- **Date:** 2026-11-05
- **Deciders:** Relago maintainers, report server team
- **Related:** [RFC 0001](../rfc/0001-daemon-redesign.md) §6.1, §7.1, §9.2, Q6; [RFC 0002](../rfc/0002-detectors.md) §3.3, §5; [RFC 0004](../rfc/0004-nix-problems.md) §5, §9, §14; [ADR 0007](0007-report-structure.md) (report fields), [ADR 0009](0009-store-viewer-data.md) (store layout)

## Context

The fingerprint decides whether a new occurrence belongs to an existing problem. It is the mechanism that stops the "every crash is a new window" behaviour, and it is shared with the server: the server answers "known" only because every machine computes the same fingerprint for the same bug. A fingerprint must therefore be deterministic. A random UUID cannot be used, because two identical crashes would get two different values.

The documents disagree on what goes into it:

- RFC 0001 §6.1 defines `sha256(kind || package.name || exe basename || signal || top-5 frame function names)`. That only fits coredumps; most kinds have no package, executable, signal or stack.
- RFC 0002 §5 and RFC 0004 §5 start every fingerprint with the **detector ID**. Detector IDs are internal labels and were all renamed on 2026-11-04; with the detector ID as input, that rename would have split the history of every problem, locally and on the server.
- Several specs use inputs that change with every build (frame offsets in CRASH-1, source line numbers in KERN-2), or that must never leave the machine (the executable name in NIX-16, derivation names in NIX-5).
- `a || b` with plain string concatenation is ambiguous: `"ab" || "c"` equals `"a" || "bc"`.

Any fingerprint makes two kinds of mistakes, and both matter:

- **False merge:** two different bugs get the same fingerprint. The second bug is hidden inside the first: it only increments a counter, it is not notified again within 24 h, and if the first problem is `known` or fixed, the second is silenced as Info.
- **False split:** one bug gets several fingerprints. The user sees duplicates, and the server undercounts the bug.

## Decision

### 1. Formula and encoding

```
fingerprint = "fp_v1:" + lowercase_hex( sha256( canonical_json( ["fp_v1", kind, part₁, part₂, …] ) ) )
```

- `kind` is the problem kind of ADR 0007 §4, not the detector ID.
- The parts are the **identity parts** of that kind (§3), in the listed order, each a string.
- `canonical_json` is a JSON array without whitespace, UTF-8, NFC-normalised. A missing part is `null`, never an empty string, so "absent" and "empty" differ.
- On disk the fingerprint is used as a directory name with `:` replaced by `_` (`problems/fp_v1_3f9a…/`, ADR 0009). The D-Bus problem ID and the report field keep the `fp_v1:` form.

### 2. Rules for identity parts

Every identity part must be:

1. **Stable across occurrences.** No timestamps, PIDs, boot IDs, journal cursors, counts, memory addresses or offsets, or device names that change between boots. Template instances are stripped (`foo@bar.service` → `foo@.service`). Numbers are stripped from message text.
2. **Version-independent**, so one bug across updates stays one problem. No package versions, store hashes or source line numbers. A kind may include a version only when the version *is* the problem (`older_generation`, `outdated_system`, `reboot_needed`, `driver_mismatch`).
3. **Machine-independent.** No hostname, username, uid, home path or serial number.
4. **Distinguishing.** Enough to separate different bugs of the same kind (signal, frames, unit result).
5. **Allowed in a report.** The fingerprint is sent in every micro report, and a hash of guessable inputs can be reversed by trying candidates. A value that may not be sent in the clear (a private executable or derivation name, a file path) is replaced by a **keyed private part**: `"priv:" + hex(HMAC-SHA256(machine_secret, value))[0..16]`. The `machine_secret` is 32 random bytes generated at first start, stored in `state.json`, never sent, and kept across upgrades. Such problems deduplicate locally but never across machines, which is correct for private software.

### 3. Identity parts per kind

Frames are normalised by §5 before use.

| Kind | Identity parts |
|---|---|
| `coredump` | package name (keyed private part if the executable is not from the Nix store), executable basename (same rule), signal name, top-5 frames |
| `unit_failure` | scope (`system` / `user`), unit name with instance stripped, `UNIT_RESULT`, exit status |
| `session_crash` | component package, fingerprint of the linked coredump |
| `oom_kill` | killer (`oomd` / `kernel`), victim (app ID, else unit with instance stripped, else `comm`), constraint (kernel only, else `null`) |
| `kernel_panic` | panic reason with numbers stripped, `RIP` function, top-5 call-trace functions |
| `kernel_oops` | class (warning / bug / oops / gpf), function, top-3 call-trace functions |
| `gpu_hang` | driver, engine or ring, Xid code or ecode class, GPU vendor:device ID |
| `hw_error` | source (mce / aer / edac), class, bank or PCI vendor:device or memory controller |
| `fs_error` | filesystem, error class, mount class |
| `low_space` | mount class |
| `unclean_shutdown` | class (hard-reset / failed-resume) |
| `disk_health` | class, device model |
| `activation_failure` | sorted failed unit names with instances stripped, sorted failed snippet names |
| `upgrade_failure` | class, derivation `pname` for `build-failure` (keyed private part if not public, RFC 0004 §9), else `null` |
| `older_generation` | NixOS version of the newest generation |

The kinds proposed in RFC 0004 §14 follow the same rules when their schemas are added (ADR 0010):

| Kind | Identity parts |
|---|---|
| `nix_build_oom` | derivation `pname` if known (keyed private part if not public), RAM bucket |
| `bootloader_failure` | bootloader, class |
| `reboot_needed` | NixOS version of the current system |
| `outdated_system` | scope (`outdated` / `end-of-life`), release |
| `foreign_binary` | executable basename as a keyed private part |
| `driver_mismatch` | kernel module version, client version |

This replaces RFC 0001 §6.1 and the "Fingerprint" lines of RFC 0002 §5 and RFC 0004 §5. CRASH-2 has no kind and no fingerprint of its own. Detectors that specialise `unit_failure` (NIX-10, NIX-13, NIX-15) use their own kind once it exists, else the `unit_failure` parts.

### 4. Precision

Kinds with stack frames (`coredump`, `kernel_panic`, `kernel_oops`) record how precise the fingerprint is, from the frames it was built from:

- `full`: at least two of the top-5 frames have function names,
- `modules`: frames are known only by module name (unsymbolised; common on NixOS until RFC 0001 Q6 is resolved),
- `none`: no frames at all.

Precision is stored with the problem and sent as the optional `problem.fingerprint_precision`. A problem with precision `modules` or `none` is coarse and may merge different bugs, so §7 never silences its new variants.

### 5. Frame normalisation

1. Skip frames of crash-handling code at the top of the stack: `abort`, `raise`, `__pthread_kill_implementation`, `__assert_fail`, `g_assertion_message*`, `g_log_*`, `std::panicking::*`, `core::panicking::*`, `rust_panic`, signal trampolines. The list lives in `relago-proto` with tests.
2. A symbolised frame contributes its function name with template and generic arguments removed (`std::vector<int>::at` → `std::vector::at`).
3. An unsymbolised frame contributes its **module basename only** (`libxul.so`). Offsets change with every build and are never used.
4. Frames from the dynamic loader and `libc` start-up (`_start`, `__libc_start_main`) are dropped at the bottom.

### 6. Variants

Each occurrence also gets a **variant signature**: a second hash over information that is too unstable or too detailed for identity but separates sub-cases.

| Kind | Variant parts |
|---|---|
| `coredump` | up to 30 normalised frames; the normalised assertion or abort message, if the crash logged one |
| `kernel_panic`, `kernel_oops` | the full normalised call trace |
| `unit_failure`, `upgrade_failure`, `activation_failure` | the first error line of the failed run, normalised |
| other kinds | none |

Normalising a message replaces numbers, hex values, quoted strings, paths and the scrubber's placeholders with fixed tokens, after the ADR 0007 §6 scrubbing.

Variants do not create new problems. The problem keeps the list of variants it has seen (count and first and last seen per variant), the app shows them, and the micro report carries the occurrence's variant as the optional `problem.variant`. The server can use variants to split a problem it considers merged.

### 7. Known and fixed problems, and regressions

RFC 0001 §9.2 currently lowers every future occurrence of a `known` problem to Info. With a false merge, that silences a new bug. Instead:

- An occurrence of a `known` problem is lowered to Info **only if** its variant has been seen before **and** the fingerprint precision is `full`. Otherwise it keeps its normal severity, is notified under the usual rules, and is sent as a new occurrence with its variant.
- When the server reports a resolution of `fixed_in_update`, it may include `fixed_in`, the package or NixOS version that contains the fix. An occurrence on a version at or above `fixed_in`, or, without `fixed_in`, on any version not seen before the resolution, is a **regression**: the problem returns to state `new`, its notification window is reset, and the next micro report sets the optional `problem.regression = true`.

### 8. Versioning

The prefix (`fp_v1`) changes when the formula, the identity parts of any kind, or the normalisation rules change in a way that changes existing values. A new prefix starts new problems; the daemon keeps the old ones until they age out (RFC 0001 §7.2). Adding a new kind does not change the prefix.

## Collision cases

The case from the discussion: Firefox crashes, and the fingerprint were the executable alone. Later, perhaps after a fix, Firefox crashes again for a different reason, with a different error message.

| # | Case | With the executable alone | With this ADR |
|---|---|---|---|
| 1 | Two different Firefox bugs, both SIGSEGV, symbols available | **False merge**: one problem; the second bug is never notified | Different top frames → different fingerprints → two problems |
| 2 | Bug fixed in Firefox 131; in 132 a different bug crashes in the same function with the same top frames | **False merge** into a problem that is `known` / fixed; silenced as Info | Same fingerprint, but a new variant (deeper frames or abort message differ) and a version at or above `fixed_in`: treated as a **regression** (§7), notified, `regression = true` sent |
| 3 | The same bug comes back in 133 (the fix was incomplete) | Merged and silenced | Same fingerprint and variant, version above `fixed_in`: regression (§7) |
| 4 | Two different bugs, no symbols (precision `modules` or `none`) | Merged | Still merged if the module lists match. Precision is coarse, so new variants are never silenced, and the server sees the precision and variants and can split |
| 5 | Several different abort-based crashes (`abort`, `raise` on top) | Merged | Crash-handling frames are skipped (§5), so the real frames decide; the assertion message is a variant |
| 6 | Same bug reached through different callers (top frames equal, callers differ) | Merged (correct) | Top-5 may differ in the callers → **false split**. Accepted: top-5 is the usual trade-off; the server can merge refs |
| 7 | Same bug, but a new compiler inlines differently and function names change | Merged (correct) | **False split** across versions. Accepted; rare, and the server can merge refs |
| 8 | `nginx.service` fails with exit status 1, once for a config error, once for a busy port | Merged | Merged too (same unit, result and status), but two variants from the first error line; both visible in the app and the report |
| 9 | Two private programs, both called `main`, crash with no symbols | Merged, and the name leaks in the hash | Executable becomes a keyed private part; same name on this machine still merges. Different frames or modules separate them. Never compared across machines |
| 10 | One app OOM-killed by systemd-oomd and by the kernel | — | Two problems (killer is an identity part). Chosen because the remedies differ (pressure policy versus memory limit) |
| 11 | A detector is renamed or split | History splits if the detector ID is an input | No effect: the kind is the input, not the detector ID |

## Options considered

| Option | Pros | Cons |
|---|---|---|
| Kind + per-kind identity parts, keyed private parts, precision, variants, regression rule (chosen) | Deterministic and shared with the server; survives detector renames; no private names in clear or guessable form; new bugs inside an old problem are not silenced | More rules to implement and test; variants and regressions need server support |
| Random UUID per problem | Trivial | No deduplication at all: every occurrence is a new problem |
| Executable (or unit) alone | Simple, very stable | Merges all bugs of an application into one problem (cases 1–3) |
| Detector ID + parts (RFC 0002 today) | Already written | A detector rename or split changes every fingerprint |
| Hash of the full stack or full message | Never merges different bugs | Splits on every version, address layout or wording change; the popup problem returns |
| Send raw data, let the server cluster | Best clustering quality | No local deduplication while offline or before consent; more data leaves the machine |

## Consequences

- **RFC changes**: RFC 0001 §6.1 points to this ADR; RFC 0001 §7.1 adds `machine_secret` to `state.json` and `fingerprint_precision` and the variant list to `problem.json`; RFC 0001 §9.2 replaces the "lower to Info" rule with §7 and adds `fixed_in` to `resolution`. The "Fingerprint" lines of RFC 0002 §5 and RFC 0004 §5 drop the detector ID and reference §3; CRASH-1 loses the frame offsets, KERN-2 the source line, NIX-5 and NIX-16 use keyed private parts.
- **Report fields**: `problem.fingerprint_precision`, `problem.variant` and `problem.regression` are optional additions to `relago.micro/1`, allowed by ADR 0007 §7.
- **Server**: it must accept `fixed_in` in resolutions, may merge several fingerprints into one `ref`, and may use variants to split.
- **Tests**:
  - golden vectors (fixed inputs → fixed fingerprint string) for every kind, so an accidental change to encoding or normalisation fails CI;
  - one fixture per collision case above;
  - a test that no fingerprint input outside the allow-list of §2 rule 5 is used in the clear.
- **Signals that this was wrong**: users or the server see many problems that are obviously the same bug (too much splitting), or regressions are reported for problems the server considers unrelated (too much merging); either means the identity parts of that kind need a new prefix.
