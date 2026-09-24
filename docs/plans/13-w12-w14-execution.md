---
title: Rust computation W12–W14 execution packet
status: complete
date: 2026-09-23
adrs: [ADR-0076, ADR-0081]
phase: 1
evidence: Tested — W12-W14 isolated units and scoped static checks; W19-W20 acceptance remains open
---

# Rust computation W12–W14 execution

**Historical scoped packet.** Its results and remaining-work statements describe
that execution boundary. Use the [W19 repair checkpoint](13-w19-repair-checkpoint.md)
for the current implementation, source seal and outstanding qualification.


Implements [Plan 13](13-rust-computation-architecture.md) after W00–W11. The shared
starting tree is captured in `build/plan13/w12-w14/starting-source.json` and
`starting-status.txt`. Preserve concurrent work and all earlier acceptance receipts.

## Decisions and ownership

W12 → W13 → W14. Registry/codegen own durable declarations; catalog owns exact
providers, publication and recovery; compiler owns pure semantic values; runtime
owns admission, asynchronous work and updates; engine/columnar own resources.

Use Salsa 0.28.4, DataFusion 55.1.0, Arrow 59.3.0 and existing graph pins. Delta
reference authority is the skill plus vendored commit
`58f07cd62bfbce3649a7e1c87c696288068ae184`, never published-crate Context7 results.
The maintainer selected automatic qualified CDF with exact endpoint fallback.
No new dependency or Salsa persistence feature is needed.

## W12 — selected complete durable products

1. Registry profiles select authored release/case, compiled model/problem,
   diagnostics, requested runtime results and explicit inspection. Required empty
   relations remain present. Profiles follow consumer products, not all stage outputs.
   Separate requested, computed/validated and stored sets.
2. Declare generated artifact descriptors: version/profile, requested roots, exact
   release vector, semantic and implementation identity, schema/algorithm/target
   contracts, value assumptions and reconstruction contract. Publish a descriptor
   alongside products; the control row owns actual output versions. Refuse local
   handles, incompatible identities and opaque reconstruction. Eviction never loads latest.
3. Replace implicit complete-publication demand with an explicit profile. Preserve
   obligation/provenance closure for unwritten dependencies. Pure compiler views can
   reconstruct exactly; unresolved support and effectful outcomes require durable
   members. Reopening validates the same closure.
4. Keep Delta-aware sessions, projection by name, exact snapshots, native write
   evidence, stable attempts, expected parents and typed control transactions. Known
   commits retain versions; uncertain outcomes reconcile before retry. Partial
   member writes remain unpublished candidates protected by attempt retention.
5. Delete unconditional intermediate destinations/members and direct callers (L11).
   Full public consumer orchestration remains W15.

## W13 — complete exact release updates

1. Stage validated edits or exact endpoints outside Salsa. All routes produce typed
   changes and a complete target. Resolve latest to exact first. Recheck expected base
   at the update barrier; no partial apply or implicit rebase. Compare fields before
   setters. Storage rewrites update provenance without recomputing unchanged mathematics.
2. Implement endpoint comparison first: keys, deletes, selectors, nested/float meaning
   and required empty members. Use a complete retained base or load its exact version.
3. Try qualified forward CDF [old+1,new]; unchanged members bypass it. Check history,
   enablement/features/schema and retained files. A clamped end is not coverage. Collect
   keys from all images without positional pairing, then compare exact endpoint rows.
   Unsupported/unavailable CDF falls back; missing endpoints refuse specifically.
4. Declare whole-release checkpoints and intervals. Publish after full admission;
   restart requires an admitted baseline. Preserve successful admission if checkpoint
   observation fails. Feed replay windows into existing retention/maintenance.
5. Preserve provider/store/capability generations and immutable old readers. Delete
   CDF-only correctness and missing-history-as-unchanged behavior (L12).

## W14 — aggregate resources and truthful cancellation

1. Extend SharedRuntime without per-session pools. Carry run scopes through admission,
   graph/Salsa, DataFusion, solver and outputs. Include source/decode overlap, maps,
   keys/tombstones, metadata/payload, graph copies/workspace, caches/spill, encoding
   and foreign buffers. Reserve before allocation and charge shared owners once.
2. Share owned CPU admission across queries/updates, phases and solvers. Bound queued
   requests and phase keys. Memory failure refuses without deadlocking permits. Nested
   work borrows its parent's scope; partition/foreign fan-out fits admitted policy.
   Add shared I/O at store registration; LimitStore gates are separate per store.
3. Use Salsa LRU/trigger_lru_eviction for values, memory_usage/heap_size for reporting,
   explicit entity/key/revision/generation limits for lifetime, and native graph
   workspace estimates. Retired generations remain charged through readers.
4. Generalize leased typed payloads without compiler/native dependency cycles.
   Preserve leases through solver preparation, slices/dictionaries and FFI. Unknown
   foreign backing capacity uses reserved ingress copying.
5. One waiter leaving preserves shared work; last-waiter cancellation requests stop.
   Publication settles independently. Blocking jobs retain permits until return.
   Stale phase output cannot install; old solves retain release labels. Delete detached
   leases, duplicate gates and value-LRU-only claims (L13).

## Verification

Baseline: zero failures. Every Rust unit command explicitly enables
`pse-relations/force-validate`. Add `unit-rust-durability-resources` selecting isolated
profile/descriptor/settlement, update-normalization and owner/scheduler units.

W12: required empty/missing members, selected products, support closure, descriptor
mismatches/eviction, stable attempts, known versus uncertain outcomes. W13: route
equivalence, key changes/deletes, images/ranges, selectors/schema, no-op updates,
stale/partial admission and checkpoints. W14: saturation/nesting, one/all-waiter
cancellation, noncooperative completion, partial fill, metadata growth, shared backing,
retired readers and multi-store I/O.

Run affected foundation/computation/Delta boundary units, compilation, scoped Clippy
(including linked solver interfaces), family/format/ADR checks and pure generation.
Author/compile real publication/recovery/history/vacuum/solver/FFI journeys for W19;
never execute them before W18. RSS/spill/throughput measurements remain W20.
Receipts name commands, tests, counts, source and conditions. W15–W20 remain open.

## Outcome

### What was built

**Implemented:** W12 introduces registry-owned product roots and transitive support
closure, explicit intermediate inspection, generated artifact descriptors, current-format
admission and exact descriptor comparison on reopening. Catalog product selection requires
present empty members, preserves retained validation/source plans, and adds the descriptor
through the ordinary publication protocol. Native driver demand follows selected roots;
it no longer creates a destination for every computed intermediate. Publication attempts,
expected parents, exact member versions, uncertain settlement and Delta-aware scans stay
under the existing catalog owners. Failed partial member writes remain candidates until
the control row commits; attempt and publication retention retain their previous duties.

**Implemented:** W13 adds generated primary-key and row equality helpers, bounded complete
endpoint comparison, qualified CDF candidate collection from all images, and a serialized
expected-base installation. The baseline includes semantic input identity, so concurrent
direct edits cannot overwrite each other merely because their stored release is unchanged.
The native reader reuses the existing complete-phase decoder and obligation templates.
Unsupported or unavailable CDF falls back; authorization, cancellation, resource and unknown
failures propagate. Checkpoints are constructed after successful whole-release admission,
use the existing typed catalog publication protocol and retain replay windows including
both endpoint versions. A failed checkpoint observation leaves the admitted release intact;
a restarted database must first admit the exact checkpoint target. Checkpoint retry handles
and decoded artifact descriptors carry allocation owners.

**Implemented:** W14 moves the reusable typed lease into columnar ownership, retains it
through numerical/solver preparation, and adds release/input/generation/revision labels to
solver outcomes (relation version 2). CPU admission is shared by compiler updates/queries
and native partitions; nested work borrows the parent permit. Bounded query/update and
phase admission refuses excess work. Generation reservations include explicit metadata
allowances alongside bounded keys/entities/revisions; Salsa heap reports supplement rather
than replace these controls. One waiter leaving preserves shared work, the last waiter
requests cancellation, and a running blocking closure keeps its permits until it returns.
One I/O gate covers registered stores; nested wrappers borrow the same scoped owner, body
reads retain it, and idle listings release capacity between items. Replacing a store keeps
old readers alive under their actual identity.

Public compile/execute/publish orchestration and Python consumer migration remain W15.
L11 closes its W12 unconditional destination generation here but remains open for W15
caller reconciliation. L12's CDF-only admission and L13's detached typed solver ownership
are removed. Earlier graph projections, semantic Salsa queries and actual relational
execution retain their W00-W11 responsibility boundaries.

### Library functionality and bounds

| Library / exact evidence | Reused capability and application |
|---|---|
| Salsa 0.28.4 skill and local macro API | Existing tracked queries, compare-before-set, LRU eviction, `heap_size`, `memory_usage().heap_size_of_fields`; explicit database lifetime limits remain necessary |
| Rust graph skill / existing W06-W11 adapters | Existing complete projections, library graph algorithms and conservative workspace admission remain unchanged; no new graph scheduler or store |
| DataFusion 55.1.0 / Arrow 59.3.0 skill | Native memory reservations, retained immutable sources, generated checked batches, native projections/unnest/distinct and existing relational obligations |
| Delta pinned commit `58f07cd62bfbce3649a7e1c87c696288068ae184` skill and vendored source | Exact snapshots, `DeltaCdfTableProvider`, native actions/history, native write evidence and the existing PSE control-row settlement; no published-crate Context7 reference |
| object_store 0.13.2 / Tokio 1.53.1 | Existing `ObjectStore`/multipart/stream interfaces, owned semaphore permits, task-local `scope`/`try_with`, and owned blocking tasks. Context7 scoped-task documentation was checked against pinned Tokio source. Native `LimitStore` has a private per-store gate, so the shared wrapper only supplies cross-store admission |

Metadata uses a conservative allowance: 1024 bytes per configured entity/query key,
an additional configured payload allowance and 4096 fixed bytes. Generated row/index,
phase decode, checkpoint overlap and numerical workspace allowances are separately
reserved. These are admission estimates, not measured allocator capacity or a hard RSS
limit. Shared Salsa field heap reports may count shared storage more than once and do
not create duplicate native reservations. No hard cancellation deadline is claimed.

### Verification receipts

Baseline: zero failures. Rust units use nextest's default profile with explicit
`pse-relations/force-validate`. Logs and source hashes are retained under
`build/plan13/w12-w14/`. These are scoped implementation receipts, not W19/W20 qualification.

- **Tested:** `just unit-rust-foundations`: 87 passed, 0 failed; 254 deliberately unselected.
- **Tested:** `just unit-rust-computation`: 32 passed, 0 failed; 188 deliberately unselected.
- **Tested:** `just unit-package pse-catalog 'test(delta::contract::tests::) or test(delta::layout::tests::) or test(delta::settlement::delta_boundary_unit::)'`:
  10 passed, 0 failed; 84 deliberately unselected. Native declaration/codec/settlement units only.
- **Tested:** `.venv/bin/python -m unittest scripts.tests.test_implementation_phase`: 7 passed, 0 failed.
- **Tested:** `just unit-rust-durability-resources`: 30 passed, 0 failed; 412 deliberately unselected.
- **Tested:** `just unit-native-callbacks-solver`: 1 passed, 0 failed; 1 deliberately unselected;
  `pse-backend-native/ipopt,pse-relations/force-validate`, pinned linked interface, no solver invocation.
- **Interface-checked:** `just clippy-rust-foundations` and `just lint-native-data-solver`:
  exit 0, affected crates/all targets, explicit force-validation, `-D warnings`;
  the latter checks both runtime and backend with `pse-runtime/ipopt`.
- **Interface-checked:** `just family-check`, `just fmt-rust-check`, `just lint-toml`: exit 0.
- **Interface-checked:** `just check`: exit 0; locked workspace/all targets, dev profile, compile only.
- **Tested:** `just codegen-rust-contracts-check`, `just codegen-python-check`,
  `just codegen-docs-check`: exit 0; exact fresh generation, no product execution.
  A temporary Git index includes intent-to-add for four new generated files, allowing the
  existing strict tracked-tree check without modifying the real staging area.
- **Interface-checked:** `just quality`, `just adr-lint`, `just docs`: exit 0;
  Python/static/governance metadata and documentation only.
- **Interface-checked:** `just py-sync`, `just doctor`: exit 0; editable native extension,
  actual compiled API stubs and environment ready. The last Rust fixture edit changed uv's
  source freshness key during the first refresh, so refresh was repeated after source froze.

Final log/source digests and commands are in `build/plan13/w12-w14/final-verification.json`
and `final-source.json`. Cargo retains the upstream `proc-macro-error2 2.0.1` future
incompatibility notice; mdBook retains its large-search-index notice. Neither is a new
product lint finding. No RSS, timing, FFI stress or real solver qualification is inferred.

Two added native journeys are authored and compiled: exact product reopening with a cold
factory and descriptor mismatch refusal; old provider/new commit with CDF versus endpoint
agreement. Existing duplicate delivery, competing parent, partial member/lost response,
maintenance and solver ownership fixtures remain carried acceptance cases. None of those
journeys has been executed here. `13-acceptance-cases.toml` adds 15 isolated case groups and
two W19-owned integration cases without changing inherited declarations.

### A mistake made and corrected

The first inspection fixture requested instance bindings, which P9 can extend; expecting
only P3 was incorrect. The fixture now requests P3-only template declarations. Generated
key equality initially emitted redundant constant boolean expressions; the generator was
fixed and all outputs regenerated. The first I/O gate also acquired twice through a nested
wrapper; scoped permit borrowing plus nested get/list/multipart tests corrects the deadlock.
The new missing-member unit initially attempted to construct an empty artifact, correctly
refused by its constructor; it now requests an additional absent member from a valid artifact.
The solver feature check exposed stale unleased test callers, which now carry a reserved
owner and generated selection label. Quality additionally exposed the stale REUSE annotation path after the earlier schema-to-codegen
move; the same licence annotation now follows the templates. ADR evidence metadata requires
the exact charter label, with command/conditions in `verification`, which was corrected.
Final receipts supersede those failed intermediate runs.

### Deviations from the plan, deliberate

CDF is collected automatically when qualified but does not yet skip full endpoint
comparison. Full comparison checks candidate coverage, including an incorrectly empty feed;
this implementation claims correctness and fallback, not an acceleration measurement.
W20 owns qualification of a faster path. No Salsa serialization feature or new third-party
library is introduced. The old aggregate `dev-delta-boundaries` recipe is guarded until W18;
its affected isolated modules were selected through the existing `unit-package` recipe,
without changing or bypassing that guard. Formal acceptance of ADR-0081 and the authorized
blueprint revision 48 remains under the decision/design PR workflow. W15-W20 remain open.
