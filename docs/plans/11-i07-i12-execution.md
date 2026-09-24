---
title: Integrated native performance I07-I12 execution checkpoint
status: complete
date: 2026-09-20
adrs: [ADR-0074]
phase: 1
evidence: Implemented I07-I12; Tested 70 isolated units with force-validation; final integration open
---

# I07-I12 execution checkpoint

[Plan 11](11-integrated-native-performance.md) owns scope. This checkpoint records
the implementation of I07-I12 and its development evidence. The starting source
capture is `build/plan11/i07-i12-start-20260920T034119Z/`. Other workspace changes
predate this execution and are not attributed to these packages.

Targeted isolated units are sufficient for implementation and deletions. No
integration, component, solver, parity or performance campaign is run here. The
complete I00-I17 implementation/deletion boundary still precedes I18 functional
assessment and I19 measurements. Existing enforcement is retained; this work adds
no workflow controls.

## Implemented boundary

### I07 — Demand, obligations and terminals

`pse-compiler/src/native/model.rs` provides complete-publication and explicit
inspection demand. Backward selection includes producer inputs, pre/postcondition
dependencies, required effects and the selected producers' evidence. Source-to-P3
inspection omits later algorithms. Artifact inspection retains an explicit
nonpublication contract; a partial artifact cannot silently become a complete
publication.

`pse-rules/src/invariants.rs` separates immutable registry/query templates, bound
obligations and successful outcomes. It uses the existing native SQL syntax cache
and exact current source binding, rather than introducing another SQL engine or
serialized plan cache. A retained native obligation is checked before query binding.
The engine's bounded weak index owns no plans; bounded successful receipts retain
only checked or witnessed inputs and their allocation ownership. Registry,
implementation, effective policy, checker, referenced inputs and absence all
participate. Plan-derived receipts cannot create a SessionContext ownership cycle.
Unknown declarations, observed inputs and unsupported providers receive fresh work.
Witnessed selections compare the retained binding's source evidence as well as its
provider owner, so a changed witness invalidates reuse at the same provider address.

Findings use a flat native union-all, with a direct branch for a singleton. Full
reports alone add distinct findings and identity ordering. `exists`, exact bag
`count_rows`, bounded arrival-order `sample`, streaming and completed retention have
different terminal contracts. Samples report observed truncation. A negative
existence answer requires exhaustion; required effects always drain. Violations
prevent value release while later required effects still settle.

Settled zero-violation receipts release logical projection/predicate protection.
The physical wrapper then exposes the native child's existing equivalence/order
and statistics. Pending checks suppress exact statistics that could erase required
work. Field evidence is not promoted into unproved global keys or foreign keys.

### I08 — Compiler indexes and retained inventories

`passes/p3/config/lookup.rs` supplies the finite configuration algorithm's typed
equality/IN indexes over actual checked chunks. Native Arrow row encoding preserves
declared primary-key order; SQL null selection and duplicates remain explicit.
Generated appends update existing indexes immediately, so later expansion can see
earlier generated declarations. Configuration selections no longer construct a
session and execute a query per binding. Actual row tokens and source lists remain
attached to the source occurrence.

`passes/native_outputs.rs` accumulates supported chunks, retaining the original
prefix. It compacts only for a contiguous consumer or final output. Configuration
bindings similarly compact at their actual completion boundary. No old growing
prefix union/concatenation route remains in these callers.

`quantity_relations/inventory.rs` retains one complete physical selection, including
absent families, in the actual assembly. Concurrent callers share typed completion;
the index lock is released before awaiting. Failed completion is retried by a fresh
owner. Selection bookkeeping and decoded inventory have retained reservations.
`documents.rs` retains one actual checked document owner and its parsed result.
Prepared P4/P7 paths share indexed row owners. P4 source graphs retain reverse-node
indexes once per graph rather than reconstructing them per scalar expression.

### I09 — Rule epochs and affected representatives

`ReusableGroup` prepares related roots with one cache store and advances its epoch
once for the whole group. Rule candidate values and support use that group. Source
tracing substitutes the actual retained producer where both values and witnesses
need it. A failed group cannot resume into a later round.

`strata/rounds.rs` deduplicates incoming assertions, anti-joins existing identities,
and ranks facts only for affected semantic keys. Unchanged facts are preserved.
The representative's original payload and derivation survive selection. Exact
payload comparison also detects a signed-zero change when a head has no derivation
column; the existing lossless scalar codec supplies comparison fields without
replacing output values. New assertion identities and support edges participate in
termination even when ordinary value equality would hide their change.

`RoundInputs::replace_many` validates all replacements before changing any input
and refuses turnover while readers remain active. Facts, assertions, deltas and
support advance together after settlement. A new evaluation starts from its actual
inputs; no monotonic receipt is carried across a changed negative dependency or
retraction. Native joins still build their own hash tables. Native plan observations
include this work; this implementation does not claim persistent membership indexes
or work proportional only to delta size.

### I10 — Semantic indexes and trusted structure

Quantity registries retain complete-key, unit-symbol, opcode and directed-conversion
indexes. Binder sets index actual complete bound-index values by identity while
preserving value comparison. The bounded inference cache includes ordered operands,
types, index sets, exact floating bits and actual immutable checker ownership.
Mutable checkers and failures are not reused. Canonicalization uses this cache for
ordinary operators, kernel contracts and equation residual inference.

`PreparedMath` is an immutable, unforgeable wrapper around the full binding-aware
loader result. Trusted canonicalization can reuse its established all-node topology
walk. Physical inference, normalization, environments and selection claims still
undergo their actual checks. Raw/reopened rows still take full admission; supplied
hashes cannot establish canonical validity.

`pse-ids/src/canon/stages.rs` writes the same frame into the existing `FrameSink`
contract. Hash-only work streams directly into BLAKE3 and allocates no combined
preimage. Requested preimages retain the exact previous byte layout. IPC component
buffers, lengths and their conservative admission remain present. No identity
version, component order or canonical format changes.

### I11 — Guarded scalar numerical execution

`pse-numerics/src/scalar.rs` lowers actual admitted native bindings into typed slots.
Each solve owns preallocated values, validity bits, traversal stack and input slots.
Constants, parameters and point-dependent work have separate invalidation masks.
Point comparison uses floating bits, including signed zero. Same-point demand can
extend to an uncomputed derivative without invalidating completed residual work.
Parameter refresh validates the full schema and values before updating anything.

| Admitted scalar capability | Semantics and derivative boundary |
|---|---|
| Float64, exact Int64 constants, Boolean constants/guards | Integer conversion must be exact; nonnullable declared input layout; runtime finite checks |
| Ordered `+`, `-`, `*`, `/`, unary negative | Authored association is retained; existing exact first-derivative lowering |
| Comparison, Boolean operations and searched CASE | Boolean typing, short-circuiting and active-branch execution; variable-dependent derivative guards remain refused |
| `abs`, `exp`, `ln`, `log10`, `sqrt`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `tanh`, `power` | Actual native implementation identity and arity; existing admitted sparse first derivatives; no Hessian claim |
| Other kernels/functions or input representations | Explicit scalar preparation refusal; no one-row Arrow fallback inside callbacks |

Batch/scenario evaluation remains available over the same admitted operations.
The reusable program owns no solve cancellation token. Requests check their current
token at entry and between bounded traversal blocks. Mathematical source nodes and
residual/sparse-derivative ordinals survive failure reporting, with typed causes.
Unsupported scalar capability remains a typed preparation cause.

The native backend owns scalar slots and output vectors once per solve. Objective,
gradient, constraint and Jacobian callbacks demand their own outputs; sparse
coordinate order, intermediate cancellation and catch-unwind boundaries remain.
The callback route no longer creates one-row batches, evaluates every result, or
decodes numerical list arrays on each invocation.

### I12 — Exact Delta state and native cache policy

Snapshot keys contain actual store namespace, exact version, verified local root/
retention generation and load requirement. Observed head is absent. A pinned hit
does no head lookup; a pinned miss loads its version directly. An unpinned request
observes current head once. Metadata-only state cannot satisfy query capability.
Commit-returned snapshots use their actual capability and do not replay head.

Local read leases establish a durable token plus root identity. Cooperating
maintenance renews the token before effects; failed/no-op maintenance may invalidate
reuse conservatively. A replaced root cannot adopt the old held lock as evidence.
Idle entries hold no read lease, while active consumers retain their reservations.

Native metadata/statistics caches additionally use the verified source generation,
so identical paths and object metadata in a replacement root cannot alias an older
file entry. Unqualified remote/maintenance views bypass these families. Mutable
directory listings remain uncached. Resident keys retain immutable semantic settings
once; a changed physical child loses reuse eligibility unless the native owner check
establishes the same producer through transparent scheduling wrappers.

Metadata, statistics, snapshot, resident, in-flight and working budgets are finite.
Predicate admission counts all active readers and the conservative active/prefetched
reader multiplier. The default per-reader predicate cache remains zero pending I19
fanout qualification; explicitly configured finite caches use aggregate admission.
CRC/checksum acceleration remains opt-in. Native replay is bounded to one log buffer
per snapshot load and deployment concurrency remains bounded.

Existing checksum seed/corruption, checkpoint/replay, CDF history/image and maintenance
journeys remain authored for I18. The exact-version journey now explicitly checks an
old pinned cache hit after an append. No vendor overlay was edited, no missing CDF
history is treated as observed, and a checksum failure cannot retry a completed write.

## Library evidence

Interface-checked against the local DataFusion and Delta skills and pinned sources:
DataFusion 55.1.0; Arrow/Parquet 59.3.0; object_store 0.13.2; delta-rs capture
`58f07cd62bfbce3649a7e1c87c696288068ae184`; delta-kernel capture
`8ba063f8f84fec222000f66d40d70911d7c79675`. No Context7 lookup was used for these families.

The composition uses native union, aggregate/window, semi/anti join, native count,
Arrow row encoding, exact native UDF owners, `DefaultCache`, memory reservations,
Delta `load_version`, load capabilities and the caller's required native session.
DataFusion's file metadata entry validates size and last-modified time; the added
source namespace is therefore necessary for table replacement. A one-branch union
was corrected after its isolated native-planning test refused it.

The test-only `allocation-counter` dependency is pinned once at 0.8.1 and used by
the numerical callback and canonical frame allocation oracles. No production
allocator or unsafe implementation was added.

## Verification

Tested: **70 isolated units passed, zero failures**, against a zero-failure baseline.
Every selection ran through `just unit-package` with
`pse-relations/force-validate` enabled. The exact command arrays, filters and exit
codes are recorded in `build/plan11/i07-i12-final/units.json`; individual logs are
named for their package in that directory. Excluded tests were not executed.

| Package | Passed | Selected unit scope |
|---|---:|---|
| `pse-engine` | 20 | Assembly and terminal contracts, complete source selection and changed witnesses, obligations, effect barriers, shared group execution, round replacement, cache budgets and file namespaces |
| `pse-catalog` | 11 | Snapshot keys/capabilities/pins, zero fake-store I/O on a pinned hit, local lease generations, settings and four CDF images |
| `pse-rules` | 8 | Flat union, affected representative/signed-zero/negative conflict, completed input constraints and isolated invariant SQL binding/findings |
| `pse-compiler` | 3 | Demand closure, incremental duplicate-preserving lookup, original supported chunks |
| `pse-numerics` | 14 | Native binding matrix, independent derivative oracle, guards, same-point demand, parameter refresh, cancellation, refusal and allocation counts |
| `pse-quantity` | 9 | Complete binder/index values and inference cache checker identity/eligibility |
| `pse-mathir` | 1 | Prepared/raw canonical agreement, root environment and forged disconnected-cycle refusal |
| `pse-ids` | 1 | Independent frozen frame layout/hash and zero hash-only framing allocations |
| `pse-templates` | 3 | Actual path owners, coordinates, invalid parent/tuple and finite enumeration budget |

The allocation units observed zero allocations/bytes in hash-only frame assembly
and in 128 successive scalar point updates with residual and derivative requests
after workspace setup. These are scoped allocation controls, not timing or complete
solver-performance claims.

Interface-checked: `just check` passed for the workspace/all targets
(`build/plan11/i07-i12-final/check.log`). Normal `just clippy` passed with default
and no-default features, `-D warnings`, baseline zero, in
`build/assessment/20260920T053414.038234Z/`. Normal `just quality` passed in
`build/assessment/20260920T051803.849912Z/`. `just family-check` and `just adr-lint`
passed; `just py-sync` rebuilt the editable extension and generated its API stub.
`just fmt-check`, `just docs` and the final `just doctor` passed. The latter confirms
the editable environment matches the current source and lockfiles after rebuilding.

`just codegen-check` was refused by the existing I17 campaign barrier before any
generation comparison. The existing pure `just codegen-contracts-check` passed
Rust, Python and documentation generation equivalence in
`build/assessment/20260920T052108.096298Z/`. No generated schema tree was hand edited.
The upstream `proc-macro-error2 2.0.1` future-Rust compatibility notice remains
distinct from current compiler or Clippy errors.

Full integration and performance remain **not run by design**. Package completion
is not final product acceptance; I13-I17 and the I18/I19 campaigns remain open.

## Outcome

The implementation retains actual immutable owners and uses explicit invalidation
where the source changes. Two errors found by isolated units were corrected: a
singleton findings union, and a fixture that expected only nonempty retained chunks
although an initial declared empty chunk is intentional. The cache review also
extended root-generation separation to native file caches.

Deliberate conservative boundaries are qualified input-owner reuse, no remote
retention proof inferred from URLs, no steady-state scalar fallback, no persistent
rule membership claim, and disabled default predicate/CRC acceleration until the
later integrated qualification supports it. These boundaries preserve the target
contracts without claiming measurements that have not been performed.
