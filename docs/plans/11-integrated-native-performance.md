---
title: Integrated work reuse and native execution performance
status: in-progress
date: 2026-09-19
adrs: [ADR-0047, ADR-0052, ADR-0055, ADR-0065, ADR-0066, ADR-0068, ADR-0069, ADR-0070, ADR-0071, ADR-0072, ADR-0073, ADR-0074]
phase: 1
evidence: Proposed implementation; Interface-checked pinned mechanisms and current source
---

# Integrated work reuse and native execution performance

**Historical execution plan:** [Plan 13](13-rust-computation-architecture.md) is the
active successor. I00–I17 implementation and incomplete I18/I19 receipts are retained
below; their unresolved obligations are carried by the Plan 13 manifest. Resume from
the [W19 repair checkpoint](13-w19-repair-checkpoint.md), not an earlier source seal.

## Context

Implement the complete target in the [integrated design review](../design_review/reviews/design_review_integrated-native-performance_2026-09-19.md),
including T01–T14, C01–C16, additional opportunities A01–A09, policy amendments,
deletions, and unresolved assessment obligations. The target preserves existing
modeling outcomes while removing repeated semantic admission, assembly construction,
plan analysis, tuple transport, numerical callback allocation, and storage observation.

Implementation is in progress. The [execution inventory](11-execution-inventory.md)
records the implemented I00–I03 boundary and targeted development evidence; the
complete target remains unqualified until I18/I19. The user has authorized changing
necessary rules and policies to realize the target. Those changes are deliverables
in I00 and I17, not permission questions or reasons to retain the current architecture.
ADR-0074 and blueprint revision 42 record the initial amendment. Accepted ADRs,
dependency pins and historical assessment receipts remain unchanged.

### Mandatory implementation and testing sequence

**Targeted unit tests are sufficient behavioral verification for every implementation
item, including deletions, throughout I00–I17. Run the full array of integration
testing only after all implementation scope and all deletions are complete and the
repository has fully pivoted to the target design.** This sequence is an explicit
user requirement, based on practical experience that completing the design pivot
before broad testing is substantially more efficient.

During implementation, use targeted unit tests for the changed contracts and negative
controls. Scoped compilation, static checks and pure generation may support those
edits. Do not run integration, component, end-to-end, compiler workflow, real Delta
publication/recovery, linked-solver, fresh-store Python or full-suite campaigns between
packages, after an individual deletion, or to keep intermediate states broadly green.
A smaller integration subset is still integration testing and waits for the same
complete-scope barrier. Test names, `--lib`, recipe labels and tiny datasets do not
change what a test actually executes.

Implement and register the eventual integration tests alongside their changes; they
may be compiled, but remain **not run by design** until I17 is closed. An individual
package's implementation/deletion completion requires its targeted unit evidence,
not an integration receipt. I17 checks that the entire target implementation and
deletion inventory is complete; it does not require the deferred suite to have run.
Then I18 executes the full applicable integration/functional assessment, and I19
performs performance qualification. Final acceptance still requires those later
results against the zero-failure baseline.

This rule governs every package, Development exit, library probe, inherited Plan 10
obligation and command selection below. If a control requires an integrated journey,
write it during its package and defer execution to I18. Do not turn that control into
a prerequisite for proceeding with the remaining implementation or deletions.

### Execution relationship and scope

This plan is the active successor execution sequence to [Plan 10](10-native-contract-consolidation.md).
I00 switched the repository's active execution pointers.
Plan 10's [inventory](10-execution-inventory.md), source receipts, case manifest,
and partial campaigns remain evidence; neither its source closure nor its existing
unit receipts establish acceptance of this target. Do not complete Plan 10 first
using an architecture this plan replaces. Carry its unresolved acceptance obligations
forward explicitly, preserving existing implemented capabilities and deletion results.
[Plan 09](09-native-caching-and-pivot-completion.md) remains the historical qualified
scope; its retained behavior and cache/publication contracts are regression obligations.

The implementation is a direct replacement. Remove replaced APIs, callers, tuple
transport, repeated admission routes, and the scalar callback's old evaluator path.
There is no compatibility engine, alternate production legacy path, or historical
data migration. The scalar numerical program and Arrow batch evaluator are both
target representations with distinct consumers and shared admitted semantics.

No new simulator functionality, Hessian implementation, independent dependency
engine, proof database, cache framework, workflow language, or crate is required.
Existing libraries remain eligible under ADR-0066; no dependency or licence ban is
introduced. Broader simulator work retained by Plan 07 remains outside this plan.

### Planning evidence and source boundary

**Interface-checked:** `just doctor` reported ready on 2026-09-19; `just --list`
and `just assessment-list` were inspected. The working tree has extensive pre-existing
staged and unstaged work at HEAD `a46f358bdfc2ca27f9f240ab6c045b63141c3ee9`.
HEAD alone does not identify the inspected source. I00 must inventory the actual
working tree and preserve concurrent edits; no reset, cleanup, or stash is part of
this plan. Recheck affected source at each handoff instead of applying old line numbers.

**Implemented, by source inspection:** `Witness::capture/matches` invokes
`EngineSession::bound_state`; compiler algorithms still use `native/layout.rs`
and nested role sessions; numerical stages construct batches during evaluation;
cache streams reattach buffer ownership; exact snapshot reads observe current head;
successful publication settlement calls reconciliation. These are mechanism findings,
not measurements of their individual cost.

Some paths in the review are shorthand. Exact Delta snapshot and resident ownership
is in `crates/pse-catalog/src/cache_service/{snapshot,resident}.rs`; generic native
cache policy is in `crates/pse-engine/src/cache_service/`. Delta operation composition
is `crates/pse-catalog/src/delta/operation.rs`. Use these owners in execution packets.

The historical command `just assessment build/assessment/2026-09-19-local-full`
reached a user-interrupted `just test`: dev build, nextest `ci`, explicit
`pse-relations/force-validate`, no fail-fast, no retries, **zero failure baseline**.
Of 1,085 selected tests, 1,020 passed, 24 had assertion failures, five timed out,
three were interrupted, and 33 were not started. These are retained review/receipt
observations, not rerun results. Earlier overlapping attempts must not be added to
these counts. The complete carry-forward matrix is in Verification below.

### Library authority and selected native mechanisms

Use the local [DataFusion skill](../../.codex/skills/datafusion/SKILL.md),
[Delta Lake skill](../../.codex/skills/deltalake/SKILL.md), and
[tracing skill](../../.codex/skills/datafusion-tracing/SKILL.md) before probing.
Do not use Context7 for DataFusion, Arrow, Parquet, object_store, or Delta questions
in this work. For other libraries, Context7 discovery is allowed; pin-sensitive
claims still require the consumer's exact source, feature profile, or a bounded probe.

`Cargo.toml` and `Cargo.lock` currently resolve DataFusion 55.1.0, Arrow/Parquet
59.3.0, object_store 0.13.2, datafusion-tracing 55.0.0, PyO3 0.29.2 and
pyo3-arrow 0.19.0. Delta is the unpublished capture
`58f07cd62bfbce3649a7e1c87c696288068ae184`, with kernel
`8ba063f8f84fec222000f66d40d70911d7c79675`, plus the repository's source overlay.
The consumer enables Delta `datafusion`, `rustls`, and `nanosecond-timestamps`.
The skill's broader documentation profile does not establish consumer feature support.

| Mechanism | Interface-checked evidence | Implementation consequence |
|---|---|---|
| Retained context and query state | DataFusion skill `df.storage-reuse`; exact 55.1.0 `execution/context/mod.rs`, `SessionContext::state/state_ref` | Context clones share state; `state()` clones query state and refreshes query start. Clone at a real query boundary; freeze provider selections separately. Never hold the state lock across awaits. |
| State reconstruction | Exact 55.1.0 `execution/session_state.rs`, builder `build` | `prepared_plans` is initialized empty. Do not rebuild stable assembly or claim SQL prepared registrations survive rebuilding automatically. |
| Native cache extension | Exact 55.1.0 `dataframe/mod.rs`, `DataFrame::cache`; existing `session/cache.rs` | A registered `CacheFactory` is called first; only the fallback collects into `MemTable`. Extend the actual factory and completion machinery, and admit its returned plan. |
| Streaming and memory | Skill `df.consume` | Stream creation and polling can fail; sort/hash working sets, retained results, and background work need their own budgets. A stream or native pool does not bound RSS. |
| Expressions and pushdown | Skills `df.expressions`, `df.pushdown`, `arrow.schema` | Bind/coerce before dependent field reconstruction; preserve qualifiers and metadata. Exact pushdown must preserve all matches; inexact pushdown needs residuals and safe limit placement. |
| Selection and relation semantics | Skills `arrow.select`, `df.relations` | Null masks, duplicate gather indices, nullable indices, zero-column row counts, bag semantics, and ordering must be explicit. Array construction alone proves no domain validity. |
| Delta operation session | Skill `delta.session`; current `DeltaOperationContext` | Retain composed Delta/PSE planners and concrete `SessionState`; use `RequireSessionState`. An arbitrary trait wrapper or default session may lose caller functions and planners. |
| Delta historical provider | Skills `delta.open`, `delta.read` | Load the selected version before creating its provider. An already loaded snapshot can override a provider builder's version option. Reusing log state does not guarantee vacuumed data exists. |
| Native write and settlement | Skills `delta.write`, `delta.commit`, `delta.replay` | Use native builders and the physical-input bridge. Success supplies committed state; an error can occur after visible commit. A transaction marker is not automatic append deduplication. |
| CDF and retention | Skills `delta.cdf`, `delta.features`, `delta.retention` | Preserve observed interval, images, residual filtering, protocol/feature limits, and read leases. Do not checkpoint a requested end beyond the observed range. Set vacuum mode and dry-run explicitly. |
| Repository Delta additions | [Delta native interfaces](../../tooling/delta-native-seams.md), patch and generated provenance | Cache injection, snapshot extent estimates and bounded CRC replay are local overlay interfaces, not unmodified upstream APIs. Verify against the overlay before use. |
| Trace completion | Tracing skill `tracing.metrics` and lifecycle routes | Native metric recording follows recorder lifetime; instrument every actual producer path. Disabled, sampled, interrupted, and complete evidence remain distinguishable. |
| Python attachment | Context7 `/pyo3/pyo3` [parallelism guidance](https://github.com/pyo3/pyo3/blob/main/guide/src/parallelism.md), checked against PyO3 0.29.2 `marker.rs` | `Python::detach` requires `Ungil` closure and result; detached work cannot use Python APIs. pyo3-arrow's C callback calls `reader.next()` from its consumer, so attachment must be qualified on the actual route. |

These are **Interface-checked** contracts. The skills contain earlier, bounded probe
receipts; this planning pass did not rerun them or execute a new performance campaign.
The exact source resolves the compact skill table's oversimplified description of
`DataFrame::cache`. Remaining composition probes are assigned to their implementation
packages, with decisions and failure outcomes below; they are not a prerequisite
profiling project.

## Decisions

### D01 — Extend existing owners and separate lifetimes

| Responsibility | Existing owner to extend | Identity and expiry |
|---|---|---|
| Schemas, predicates, preservation contracts | `pse-schema` registry/resolved handles; `pse-relations` prepared validation | Actual immutable registry and exact declaration; external equivalence receives a complete check before interning. |
| Stable native capabilities | `EngineFactory`, assembly/function/rule owners | Actual implementations and semantic settings; replace on incompatible capability change. |
| Model environment | `EngineSession`, retained `SessionContext`, shared runtime | Compatible model/workspace lifetime; no private full-budget runtime for nested work. |
| Selected inputs | Provider bindings, role scopes, catalog views | Exact immutable table/Arrow owners or Delta versions, explicit absence, and relevant policies; a mutable provider address is insufficient. |
| Intrinsic preparation | Existing admission/traversal/preparation machinery | Actual producer and expression owners plus relevant binding/implementation context; retain owners while identity is used. |
| Completed work | Operation completion and native cache machinery | Pure successful completion and complete selected dependencies; finite model retention. Transient errors and partial streams never become successful entries. |
| Numerical structure | `pse-numerics` admitted program | Opcode/kernel/derivative/guard semantics, structure, bindings and numerical policy. |
| Mutable execution | Native execution context, worktables, solver workspace | Attempt or explicit rule epoch; fresh cancellation, authorization, query properties, metrics, and mutable operator state. |
| Durable visibility | `pse-catalog` Delta versions and publication control | Exact members and successful control publication; uncertain commits settle before replay. |
| Memory | Native reservations and `pse-ids` safe ownership adapters | Adequate allocation extent through the last reader, including array/slice/FFI escape. |

No new parallel registry represents these responsibilities. Compact indexes and
numerical instructions are derived layouts, never editable model authority.

### D02 — Evidence proves one property of one selection

Retain local value evidence, relational obligation completion, canonical evidence,
allocation ownership, and durable publication evidence separately. `FieldCheckedBatch`
proves local fields/values, not keys, foreign references, completeness, permission,
canonical identity, or publication. New native values still need a producer contract
or admission. A Delta CHECK property does not validate all old files or every Arrow
conversion. Every deleted check must name the exact evidence replacing it.

Cloning preserves local evidence; slicing/filtering preserve per-value predicates
but not completeness; exact projection preserves retained fields but not removed-field
obligations. Checked concatenation preserves local predicates, not cross-chunk
uniqueness. Gather, cast, join, computation and schema adaptation need explicit
field/preservation rules, including nullable indices and nested metadata. Preserve
raw versus checked API distinctions without inventing a generic proof language.

Completion states distinguish not-started, running, completed-valid,
completed-invalid, cancelled, resource-refused, and uncertain-commit. Reusable
successful values enter model retention only after all applicable obligations settle.
A deterministic invalidity witness may support rejection under an exact selection;
it is not a successful result or a reusable transient error. Sampling is never
evidence of complete absence of violations.

### D03 — Native composition, explicit finite boundaries and demand

Compose scans, filters, joins, grouping, source support and declarative checks before
execution so DataFusion sees their relationships. Preserve native analysis and
optimizer phases. Stage names identify provenance and contracts, not mandatory
materialization points. Finite graph/domain algorithms materialize only their actual
inputs and return checked chunks through typed relation ports.

An `ExecutionPlan` has one schema. A heterogeneous finite producer therefore retains
one existing operation/completion owner and exposes one native adapter per declared
relation. Ports expose real dependencies, effects and selected demand. Concurrent
ports within an attempt join the same producer; `schema()` and `scan()` do not run
the body. Rewritten inputs, bindings or demand create a new completion identity.
No universal tuple schema or second stage scheduler is introduced.

Demand is the closure of requested values, required validation, effects and provenance.
Complete publication adds every member in its declared artifact profile. Existence,
exact count, bounded sample, complete findings, stream, and retained result are
different terminal contracts. Only a sufficient violation witness can finish a
rejection early; successful whole-input validation consumes the required scope.

### D04 — Reuse immutable meaning; re-admit live execution

Default to immutable relation/producer granularity, with complete positive and
negative dependencies. Include referenced/absent/empty relations, membership,
multiplicity, required policies, implementation identity, lexical/outer bindings,
volatility, and rule epochs where relevant. Preserve Plan 09's qualified consumed-input
reuse rather than replacing it with pointer or display-name matching.

Same retained owner is an in-process fast path, not a complete reuse proof. Unknown
mutable providers are non-cacheable unless they supply an admitted immutable revision
contract. Capture ambient/stable/volatile dependencies before folding; fresh task
state cannot refresh an already folded timestamp. Prefer reusing pre-evaluation
structure when an ambient value cannot safely enter the key.

Default to model-level **completed** pure reuse and attempt-local in-flight sharing.
Cross-attempt shared fills remain disabled until interested-consumer ownership,
cancellation and quota controls pass; their absence does not block this selected
default. A later optional implementation must use the same cache, not add a service.
Every cache hit still passes current authorization, resource, cancellation and effect
admission. Reusable physical state requires qualified reset/reentrancy; otherwise
retain logical preparation and rebuild attempt-local physical state.

### D05 — Scalar solver execution and batch execution share semantics

Derive a compact typed instruction layout from admitted numerical structure. Resolve
opcode, constant payload, input/output slots, guarded region, source/equation mapping,
derivative capability and output closure at preparation. Reserve mutable workspace
once per solve. Keep immutable program, bound-parameter epoch, variable-point validity,
computed-output mask, and cancellation ownership separate.

Use this scalar route for repeated objective, gradient, residual and Jacobian
callbacks within the current supported backend contract. Keep DataFusion physical
expressions and Arrow kernels for batch/scenario consumers. Both routes dispatch
through the same admitted operation meanings; unsupported scalar kernels fail during
preparation unless an explicit contracted scalar implementation is provided. No
silent one-row DataFrame fallback or new derivative promise is allowed.

Preserve guarded branches, ordered arithmetic, finite checks at required intermediate
nodes, source-local errors, explicit contraction policy, and no unwind through Ipopt.
Check cancellation at callback entry and bounded instruction/expensive-kernel
boundaries. Do not infer bitwise equivalence from algebraic equality.

### D06 — Ownership and storage reuse remain exact about their limits

Attach a safe allocation lease once at the producer/chunk escape boundary and carry
it through internal checked handoffs. An array cloned away from its batch must keep
the lease. Track scratch, cache retention and exports separately with incremental
totals. Eliminate the global pointer ledger only when every surviving escape route
has adequate extent and lifetime coverage. Unknown foreign backing capacity requires
an honest extent contract or a checked reserved copy.

Exact Delta snapshot keys contain store/table generation and root, selected version,
load capability, interpretation/options and relevant maintenance identity. Current-head
resolution is a separate freshness-governed operation. A pinned miss loads its version
directly; no latest-head probe is required merely to discover a cache hit. Maintenance,
authorization, read leases and actual missing-file errors remain effective.

Native writes return exact committed state on unequivocal success. Carry validating
write evidence and the durable mapping to publication, preserving cross-relation
obligations. Keep parent conflict detection, read sets, complete membership and the
control-record visibility boundary. Ambiguous errors retain reconciliation; neither
concurrent member writes nor transaction markers create atomic multi-table publication.

### D07 — Policy reconciliation is implementation work

Use the [ADR skill](../../.codex/skills/adr/SKILL.md) and current decision rules for
the formal changes. I00 allocates one focused integrated lifetime/execution decision
with `just adr-new integrated-native-performance`; its numeric ID is allocated then,
not predicted here. Link the integrated review, update proposed overlapping records,
and amend governed blueprint sections with a revision row before behavior changes.
If distinct contracts require more than one record, separate their governed scope
rather than duplicate assertions. Add allocated IDs to this plan's front matter.

Keep the new decision **Proposed** until the required design-review/status process is
satisfied. The integrated review's current verdict is Revise, not an Accept receipt.
Decision PRs use the repository's `adr`/`needs-review` metadata and blueprint amendment
route. The user's authorization already covers these amendments; formal status handling
does not require another direction-setting approval question.

| Policy/change | Current authority and live status | Required amendment and owner |
|---|---|---|
| Context lifetime | Blueprint §14.3/§14.3.1; ADR-0068 and ADR-0073 proposed | Retained compatible model assembly/context, immutable revision views, fresh query/attempt state; I00/I03. |
| Reuse granularity and prior measurement prerequisite | ADR-0042 **superseded by ADR-0068**; stale register R-01/R-22 and blueprint §14.3 | Permit trustworthy producer/invariant/index scope immediately. Replace stale triggers; keep adoption of an additional dependency framework a separate evidence-based decision; I00/I04/I14. |
| Named-stage materialization and tuple transport | Blueprint §3.3.3/§14.3.1; proposed ADR-0068/0073 | Native relational composition and contracted finite ports; no universal stage capture or independent scheduler; I00/I06. |
| Bound preparation/completion lifetime | Proposed ADR-0070/0073; Plan 10 guidance | Replace blanket prohibitions with exact semantic keys and bounded model retention; I00/I04/I14. |
| Repeated boundary validation | Blueprint §5.4/§14.3.1; ADR-0052 proposed | New/untrusted values and unresolved obligations are checked; exact established evidence survives supported handoffs; I00/I02/I07/I13. |
| Allocation mechanism | Blueprint §14.3/§14.3.2; ADR-0055/0073 proposed; ADR-0046 already superseded | Specify adequate extent and last-reader guarantees, permitting chunk ownership and once-only attachment. Preserve fallible admission, fix inaccurate no-ledger statements; I00/I05. |
| Numerical layout and claims | Blueprint §18.2; ADR-0047 **accepted** | Implement the guarded supported instruction/workspace scope. Preserve accepted ordered/guarded semantics; do not edit ADR-0047's accepted argument. A genuinely conflicting change would require a successor, not a waiver; I00/I11/I17. |
| IPC hot-path description | Blueprint §5.4, §20 and proposed ADR-0068 | Describe actual Delta/Arrow streams; IPC remains where used for encoding/spill. Hash framing is unchanged; I00/I10/I17. |
| Cache families and replay defaults | Native and Delta cache policies; proposed ADR-0070 | Finite resource partitions, source-specific freshness, qualified replay/checksum support, metadata/file-state distinction; I12/I14. |
| Workflow/test concurrency | Blueprint §18.8; runtime/thread policy; `.config/nextest.toml` | Bounded CPU/I/O/memory/solver budgets, nested permit safety, explicit deterministic serialization only where necessary; I14/I16. |
| Force-validation and performance modes | `AGENTS.md`, `.claude/rules/rust.md`, justfile | Keep explicit force-validation for correctness tests. Add separately named production-equivalent benchmark mode with recorded feature graph; I00/I19. |
| Python generated-contract checks | Blueprint §21.5; Python rules/import fixtures | Move exhaustive generated-class lint to generation/quality; keep runtime registration, package/native compatibility and dynamic-contract admission; I00/I15. |
| Assurance/reporting | Blueprint §23/§24, testkit, validation runner | Contract observation by default, explicit diagnostic capture, independent case outcomes, truthful failed/unsupported/interrupted statuses; I16. |
| Execution guidance | `AGENTS.md`, Plan 10/index, docs/dev guidance, role sources | Make targeted unit tests sufficient for all implementation/deletion packages; defer every integration campaign until the complete target pivot and deletion barrier. Point to Plan 11; amend canonical role/rule sources and regenerate aliases when needed; I00/I17. |

ADR-0046 and ADR-0067 are already superseded by ADR-0068; do not supersede them
again as active decisions. ADR-0052, ADR-0055, ADR-0068, ADR-0070 and ADR-0073 are
currently proposed and can be revised through their actual status path. Recheck
statuses before edits. Use `PSE_DESIGN_EDIT=1` deliberately for the blueprint
amendment and explain it in its decision/design PR. Accepted records stay immutable
apart from supported supersession metadata. Run the index generator, not hand edits
inside the ADR index block.

## Plan

### Dependency order and package contract

All packages are **Proposed**. Each implementation receipt must identify changed
declarations, producers, consumers, generated outputs, deleted paths, exact commands,
negative controls and remaining limitations. Completion requires callers to use the
replacement; a compiled unused mechanism is insufficient. Package ownership below
means source responsibility, not authorization to overwrite concurrent edits.

All **Development exit** controls in I00–I17 mean targeted unit tests and applicable
non-executing checks. Where their described scenario also needs real-consumer
integration evidence, implement that test now and schedule its execution in I18.
Passing targeted units is sufficient to advance and close the implementation package;
its deferred integration receipt is an I18 acceptance obligation.

| Package | Deliverable | Prerequisites | Review targets |
|---|---|---|---|
| I00 | Scope ledger, policy/decision amendments and command ownership | None | §10, T14 |
| I01 | Binding, witness and retained-extent correctness foundation | I00 | T03/T09 |
| I02 | Registry-owned schema/validation evidence and preservation | I01 | T01 |
| I03 | Retained assembly/context and immutable selected views | I01 | T02/T13 |
| I04 | Distinct plan facts and relevant preparation witnesses | I02/I03 | T03 |
| I05 | Chunk ownership and typed completion foundation | I01/I02/I03 | T09/T13 |
| I06 | Native composition and finite checked output ports | I04/I05 | T04 |
| I07 | Demand, shared obligations and terminal contracts | I06 | T05 |
| I08 | Bulk compiler construction and retained inventories | I06/I07 | T06 |
| I09 | Affected-key rule updates and epoch state | I04/I05/I07 | T07 |
| I10 | Semantic indexes and canonical work reuse | I02/I04/I05 | T11 |
| I11 | Guarded scalar program and solve workspace | I02/I04/I05 | T08 |
| I12 | Exact Delta snapshots and qualified native cache families | I03/I05 | T10/T13 |
| I13 | Evidence-aware native writes and publication settlement | I07/I12 | T10 |
| I14 | Model result retention and bounded scheduling integration | I06/I07/I09/I11/I12/I13 | T13 |
| I15 | Python startup, stream and metadata boundary | I02/I05/I12/I14 | T12 |
| I16 | Typed diagnostics, fixtures, observations and assessment tooling | I04–I15 | T14 |
| I17 | Full deletion, caller, policy and source barrier | I00–I16 | All |
| I18 | Complete functional assessment and grouped remediation | I17 | §9, all |
| I19 | Performance characterization and final acceptance/outcome | I18 | §9, all |

The order is a dependency graph, not a requirement to serialize independent work.
After shared contracts are fixed, compiler/rules, canonical indexes, numerics and
Delta work can progress independently. I16 tooling can be developed earlier against
disposable fixtures; its closure waits for all producer contracts. Within a shared
file, finish its contract change before dependent callers are converted.

### I00 — Establish scope, formal policy changes and reproducible handoffs

**Own:** `docs/plans/`, applicable proposed ADRs/register, the dedicated blueprint
amendment, `AGENTS.md`, `.claude/rules/`, `justfile`,
`scripts/validation{,_scope}.py`, `xtask/src/architecture_acceptance*`.

1. Record actual staged/unstaged/untracked task source, pins, overlay hashes,
   current commands and available disk before implementation. Preserve historical
   `build/assessment/` and `build/plan10/` artifacts and the canonical regression seed.
2. Create the Plan 11 execution inventory and acceptance-case manifest from this
   plan. Each case has a stable ID, source, actual binary/test identity, feature mode,
   owning package, positive/negative oracle, and required receipt. Copy references
   to Plan 10 A01–A16 and L01–L18 coverage; do not duplicate or overwrite old receipts.
3. Implement D07's governance amendments, allocate decision IDs, and reconcile the
   register and active-plan pointers. Retain historical incomplete verdicts.
4. Define focused unit namespaces and recipe ownership for this plan. Reuse
   `check-library`, `check-package`, `check-test`, `unit-package`, `py-unit`, and
   contract-only codegen recipes. Add a short recipe only where none fits. Extend
   the existing acceptance/seal tools to select Plan 11 without modifying Plan 10
   evidence or making an old source seal qualify new code. Make their prerequisites
   enforce unit-only behavioral verification during implementation and prevent any
   integration campaign from starting before the complete I17 barrier.
5. Fix the assessment scope before execution: required checks, advisory reports,
   genuinely deferred checks and other-environment exclusions are distinct. Preserve
   every selected outcome. No new timing attribution study or broad suite runs here.

**Development exit:** front matter/index/register/docs checks pass; a disposable
receipt test rejects missing cases, stale source, interrupted logs and duplicate
identities. The ledger contains every review target, carried failure and deletion.

### I01 — Correct binding/scoping and the retained-extent contract first

**Own:** `pse-engine/src/session/{admission,scalar,field_transfer,physical_fields}.rs`,
compiler `passes/native_construction.rs`, `pse-ids/src/owned_buffer.rs`, affected
lambda/witness/ownership unit fixtures.

1. Resolve names, native lambdas and coercions before field reconstruction that
   needs them. Keep original declared meaning available for post-rewrite comparison;
   preserve qualifiers and semantic metadata through normalization.
2. Project witness/source scopes before reusing short aliases. Test two same-named
   relation roles and repeated witness composition without relying on globally unique
   display names. Admit actual transformed/factory output fields.
3. Resolve the 2,624-versus-408 retained-capacity counterexample by identifying
   allocation owner/extent provenance. Distinguish visible bytes, backing capacity,
   alignment and a reservation's supported extent. Foreign hidden capacity is not
   inferred from an arbitrary view's memory-size method.
4. Preserve the current leaf-expression-pushdown mitigation. Its removal requires
   a pin-specific source-span/Delta-conversion reproducer proving full field and
   expression preservation; a faster name-only rewrite does not justify it.

**Delete/replace:** invalid schema-before-binding ordering, ambiguous growing alias
scope, and the incorrect extent assumption. Do not reduce admitted invalid-input
coverage or relax equality/timeout assertions.

**Development exit:** nested lambda, repeated alias, hostile metadata, shared-parent,
hidden capacity, alignment and resource-refusal units pass with force-validation.
Preparation error paths retain typed causes. These are isolated controls, not heater
or complete Delta workflow runs.

### I02 — Retain schemas, predicate preparation and exact local evidence

**Own:** `pse-schema/src/{arrow,resolved_contract}.rs`, registry ownership,
`pse-relations/src/{columnar,validate}/`, schema/code generators and checked callers.

1. Derive Arrow schemas, resolved nested traversal, prepared predicates and constant
   reflection batches once per immutable registry/declaration owner. Extend the
   existing prepared-validator cache; avoid another serialization-key cache.
2. Add explicit checked preservation operations for clone, slice, filter, exact
   projection and checked chunk concatenation. Keep field evidence tied to the exact
   contract handle. Mark relational completeness/key evidence separately.
3. Admit newly computed/cast/joined values only where their producer has not already
   established the required contract. Audit each current admission site into retained,
   preserved, missing-check, or untrusted-ingress treatment.
4. Compile nested row-domain traversal once, reuse native masks/predicates, and retain
   compact row/path coordinates. Format paths only for findings. Preserve parent-null,
   dictionary, offsets, unions, empty arrays and zero-column row counts.
5. Use scalar predicate evaluation only when the real prepared predicate supports it;
   do not simulate a scalar capability by silently constructing a one-row relation.
   Change generators and regenerate through the pure contract recipes.

**Delete/replace:** duplicate field/metadata serialization, prepared-cache scans on
same-owner hits, successful-row path strings, repeated value scans at certified handoffs.

**Development exit:** counters show no extra local predicate execution for unchanged
checked handoffs; changed values, foreign declarations, nullable gathers, cast loss and
unsupported mappings still reject. Cross-chunk duplicate-key controls prove local
evidence has not been promoted into relational validity.

### I03 — Retain compatible native assembly and selected provider views

**Own:** `pse-engine/src/session/{factory,assembly,engine_session,execution,reuse,config,policy}*`,
`pse-engine/src/provider/`, generic cache state binding, `pse-runtime` composition.

1. Retain actual registry/functions/planners/rules/settings in assembly owners and
   reuse the compatible model `SessionContext`. Derive immutable selection/catalog
   views without mutating same-name tables under active workflows.
2. Retain effective semantic policy per selection. Compare stored witnesses directly;
   `Witness::matches` must not call a builder to reconstruct the state it compares.
   Separate capability identity from complete view identity and consumed-input identity.
3. Replace nested `candidate_roles` factory assembly with selected role binding over
   shared capabilities. Keep store-generation/cache namespaces and pool/disk owners.
4. Produce query `SessionState` and task/attempt services at real execution boundaries,
   preserving fresh time, cancellation, authorization and metrics. Preserve SQL
   prepared definitions where used; builder reconstruction is not their lifecycle.
5. Keep Delta-specific planner composition in catalog/runtime, preserving the
   storage-independent engine boundary and `RequireSessionState` path.

**Delete/replace:** nested full registries/factories, repeated policy inventories,
catalog reconstruction inside witness comparisons, context UUID as semantic proof.

**Development exit:** one stable assembly across compatible revisions, unaffected
table reuse, changed relevant UDF/policy/provider invalidation, same-address mutable
provider refusal, old active selection isolation, fresh query-time values, preserved
prepared SQL, and distinguishing caller-UDF/planner tests. No lock crosses an await.

### I04 — Derive intrinsic native facts once and bind only relevant context

**Own:** engine `session/{traversal,admission,preparation,freshness,reuse,contract}*`,
native cache admission, requirement preparation interfaces, scoped graph consumers.

1. Split producer-intrinsic fields/dependencies/effects/volatility from lexical,
   outer-binding, role, worktable and live-admission checks. Retain producer owners
   alongside in-process identity; never key surviving facts by a reclaimed address.
2. Prepare related output/requirement roots together. Share actual intrinsic DAG facts,
   memoize expression fields against exact schema/binding context, and refresh changed
   nodes after native rewrites. Do not hard-code a universal traversal count.
3. Record stable/volatile and ambient requirements before simplification. A folded
   time/random/external function cannot become an apparently pure cross-attempt value.
4. Preserve input and actual returned-output admission around custom `CacheFactory`
   and operation factories. Skip only unchanged-owner derivations with valid evidence.
5. Qualify physical reset/reentrancy per operator family and epoch. Retain existing
   dynamic-filter disabling until isolation/reset controls establish a safe route.

**Delete/replace:** full ancestor-effect paths in intrinsic-fact keys, repeated root
walks that derive the same fact, unconditional whole-view preparation invalidation.

**Development exit:** diamond graphs with distinct parents derive facts by distinct
structure, while lexical/recursive/outer scopes and effects stay distinct. Cover
changed factories, volatile folding, absent sources, epoch replacement, cancellation,
deep plans and invalid reconstruction. Counts measure semantic work, not a fragile
fixed number of optimizer passes.

### I05 — Replace ownership bookkeeping and establish typed producer completion

**Own:** `pse-ids/src/owned_buffer.rs`, retained extent helpers, checked relation
ownership, engine `operation/{completion,ownership}.rs`, `session/cache*`.

1. Implement producer/chunk owners with fallible preallocation admission and explicit
   retained extents. Attach safe leases once to actual escaping buffers, including
   nested children, validity, offsets and dictionaries; carry already leased storage.
2. Replace repeated process-global scans and allocation-map resums with owner-local
   state and incremental totals. Avoid retaining an entire multi-output bundle merely
   because one small port/slice survives. Make a reserved small copy an explicit
   retention policy where it releases a much larger parent.
3. Make completion distinguish success, invalidity, cancellation, refusal, partial
   stream and uncertain effects. Preserve attempt-local once-only execution and
   last-waiter abandonment. New attempts do not inherit poisoned transient results.
4. Cache owned checked chunks; resident reads clone existing owners without deep
   rewrapping. Spill decoding establishes its own admitted output lease; a stream
   owner alone is insufficient when a returned array escapes the stream.
5. Inventory every escape from batch, projection, scalar/list/dictionary view, cache,
   native operation, numerical output and C stream before deleting the global ledger.
   Keep genuine live resource checks at export/admission even when semantics are reused.

**Delete/replace:** global weak pointer inventory, repeated `retain_allocations` plus
`retain_owner` on certified cache reads, repeated full-map sums and result-only leases.
Deletion happens with the final caller cutover; no legacy ledger remains as fallback.

**Development exit:** array/slice/dictionary survives parent/query/cache drop, final
reader releases the correct charge, concurrent claims do not reassign ownership,
eviction respects exports, foreign hidden extents fail/copy honestly, cancellation
releases scratch, and a fresh attempt retries an abandoned pure computation safely.

### I06 — Compose native relational work and replace finite tuple transport

**Own:** compiler `native/{mod,model,execution,value,layout}*`, `Algorithm` contracts
and callers, engine operation adapters, catalog `artifact.rs`, affected inspection
fixtures and generator declarations.

1. Classify each existing algorithm segment from its real body: relational composition,
   finite domain/graph/numerical work, or effect settlement. Move known-input relational
   queries out of execution callbacks into inspectable native logical plans.
2. Compose adjacent relational segments through their field/effect contracts. Retain
   materialization only for required finite inputs, fixed points, deliberate completed
   result reuse, requested artifacts or effect boundaries.
3. Extend the existing operation completion with checked per-relation outputs. Each
   port adapter exposes native dependencies and streams its selected chunks; demand
   and selected inputs participate in completion identity. Declare whether outputs
   can complete independently or require one atomic valid bundle.
4. Ensure `with_new_children`/rewrites/reset cannot retain a completion for changed
   inputs. Keep preparation/introspection free of hidden execution. Within one attempt,
   multiple ports/partitions join one admitted producer.
5. Convert every caller of tuple encode/member/pack/unpack, including inspection and
   tests. Preserve actual source support and pass-attempt attribution as typed data.

**Delete/replace:** `crates/pse-compiler/src/native/layout.rs` and tuple-only helpers,
generic `LargeList<Struct>` transport/UNNEST callers, mandatory whole-stage captures,
nested factory creation. Legitimate domain lists/structs and semantic UNNEST remain.

**Development exit:** pre-execution plans expose representative relational chains;
two finite ports execute one body; changed input/demand cannot collide; projected
outputs keep leases; schema/scan has no effects; invalid output and cancelled producer
cannot publish a partial valid bundle. Use tiny fake/checked inputs, not full compiler
or durable publication journeys before I17.

### I07 — Prepare demand and obligations together; specialize terminal results

**Own:** compiler artifact/model demand, `passes/native_construction.rs`, engine
requirement/protection/completion contracts, rules `invariants/{program,native}*`,
catalog artifact publication membership.

1. Represent requested values and obligatory validation/effect/provenance closure
   within current operation contracts. Avoid accumulating every intermediate output.
   Full artifact publication explicitly selects every profile member and obligation.
2. Retain invariant templates by registry/implementation/source shape, bound instances
   by complete selection, and completed outcomes separately. Include absent/empty
   relations and referenced right-hand inputs; consume a postcondition later only
   when its exact obligation selection is unchanged.
3. Share actual expensive producers among values, support, findings and provenance
   using I05/I06 completion. Cloning a plan or adding a view is not execution sharing.
4. Add existence/count/sample/full-findings/stream/retain terminal choices. Build flat
   union-all branches for existential queries; preserve dedup/sort for full reports
   whose contract requires them. Keep exact counts exact.
5. Release projection/predicate barriers only for settled whole-input obligations.
   Pending checks and effects retain protection. Publish established constraints,
   functional dependencies/order and correctly scoped statistics back to planning.

**Delete/replace:** full collection/sort/DISTINCT for existential consumers,
left-deep findings union, repeated pre/post check execution for identical selections,
unconditional accumulated-output execution and obsolete settled barriers.

**Development exit:** same obligation executes once; changed reference/absence/empty
input reruns; downstream filter cannot hide an invalid pending row; no-violation
success waits for completion; early rejection does not skip required effects;
diagnostic truncation is explicit; source-to-P3 demand is smaller while complete
publication remains complete. Independent assessment cases still all run.

### I08 — Batch compiler construction and retain physical/domain inventories

**Own:** `passes/p3/config.rs`, `passes/native_outputs.rs`, P4/P7/P8/P9 relational
callers, `quantity_relations/inventory.rs`, document/path/reverse-node derivations.

1. Replace per-instance/per-binding queries with relational joins/grouping per
   configuration relation or dependency expansion layer. Use a typed derived index
   only for the finite algorithm's repeated lookup needs.
2. Accumulate checked chunks/builders append-only. Concatenate at an actual contiguous
   consumer, not after each generated row. Later expansion rounds retain an indexed
   view of earlier generated rows; final-only assembly must not hide dependencies.
3. Retain physical inventory by all actual selected inputs, including absent families.
   Reuse across compatible attempts, not just within a stage. Charge inventories,
   parsed documents and path/reverse-node indexes to their retained owners.
4. Keep correspondence, source support and provenance relational. Preserve duplicate,
   ordering, null and metadata semantics while batching. Use actual generated field
   constructors instead of fabricated aliases or row-position recovery.

**Delete/replace:** per-selection session/query creation, repeated unions of growing
configuration, prefix concatenation, repeated inventory/document/path scans.

**Development exit:** query and concat counts scale with relations/expansion rounds,
not row count; generated dependencies remain visible; changed physical declarations
invalidate while unrelated edits reuse; values and support match independent small
fixtures with duplicate/reordered/missing inputs. Full heater/mixer comparisons wait.

### I09 — Update rule facts by affected keys with explicit epoch ownership

**Own:** `pse-rules/src/strata/rounds.rs`, relational assertion/fact/head/support
construction, worktable/provider epoch interfaces and rule-specific units.

1. Retain immutable rule templates and separate mutable epoch inputs. Compute candidate
   assertions, changed facts and support within one completion scope.
2. Identify affected semantic keys; recompute representatives/conflicts/support for
   those groups and replace their derived results. A newly lower-ranked assertion
   can change a fact's actual payload and provenance even when its key already exists.
3. Use native semi/anti joins and aggregate/window capabilities where they express
   the contract. A fresh hash join is not persistent membership state. If the current
   lookup need requires an index, bind it to the existing worktable with exact key/null
   semantics, epoch, finite memory, admission and invalidation.
4. Preserve signed zero, payload metadata, bag/set semantics, negative dependencies,
   truth/conflict rules, empty owners and exact settled cardinalities. Treat changed
   negative inputs/retractions as invalidation or explicit recomputation; do not
   assume monotonic append unless proved by the rule contract.

**Delete/replace:** repeated whole-accumulation ranking/grouping for unchanged groups,
duplicate head/support execution and unnecessary result collection for existence.

**Development exit:** duplicate assertion, changed representative, signed zero,
support conflict, negative dependency and epoch-active-reader controls match clean
finite rule evaluation. Work counters include index build and affected-group scans;
no unsupported claim that all work is proportional only to delta size.

### I10 — Reuse semantic indexes and canonical evidence without changing identity

**Own:** `pse-quantity/src/{registry,index}*`, MathIR canonicalization/load/topology
and operator bindings, `pse-ids/src/{canon,frame}*`, canonical callers.

1. Derive immutable quantity/opcode/conversion/binder indexes once per owner. Bind
   inference reuse to operands, indexes, policies and invariant-checker identity.
   Share graph validation/order only when the required binding/edge contract matches.
2. Carry canonical evidence within the trusted workflow; do not trust a hash supplied
   alongside arbitrary rows. Untrusted reopening still establishes graph, binding,
   ordering, normalization, metadata and canonical identity.
3. Generalize the existing frame writer over `FrameSink` so hash-only consumers avoid
   the final combined preimage allocation. Preserve byte-identical fixed fields,
   length prefixes, component ordering and version constants. IPC component lengths
   and required component buffers remain explicitly accounted.
4. Retain preimage bytes only for callers requesting them. Remove sorts/gathers only
   when exact producer evidence establishes the required order/layout.

**Delete/replace:** repeated immutable registry/binder scans, unused full preimage
allocation, duplicate trusted canonical reconstruction; retain validation formerly
performed by an ignored traversal result.

**Development exit:** frozen bytes/hash vectors and independent framing oracle agree;
cyclic/bad-bound/forged canonical inputs reject; registry/policy changes invalidate;
hash-only output allocates no combined preimage. No format migration or new hash
contract is smuggled into this optimization.

### I11 — Implement the guarded scalar numerical program and solve workspace

**Own:** `pse-numerics/src/{expressions,stages,arena,finite,operation}*`, admitted
opcode/kernel derivative interfaces, `pse-backend-native/src/driver/` and callbacks.

1. Inventory the actual supported operations and derivative capabilities. Lower them
   to typed instructions/slots with region guards and exact source/equation mappings;
   bind kernel implementations once. Make unsupported scalar capability a preparation
   error. Keep batch/scenario adapters over the same operation contract.
2. Reserve contiguous per-solve slots and output buffers once, with precomputed bounds.
   Update free variables in place; separate constants, parameters, point-dependent
   intermediates and derivative outputs. The shared program contains no solve token.
3. Compute requested output dependency closures. At an unchanged point, reuse only
   completed valid slots and extend the output mask for new demand. A parameter edit
   invalidates dependent constant/parameter work; policy or binding changes invalidate
   the appropriate preparation. Point identity follows declared floating semantics.
4. Map objective/gradient/constraint/Jacobian callbacks to their own demanded outputs.
   Preserve sparse coordinate ordering and current derivative scope. Do not calculate
   every residual and derivative just to answer an objective request.
5. Execute only active scalar branches; maintain compatible batch masks. Inline
   required finite checks and failure-node retention. Check cancellation between bounded
   instruction blocks and expensive kernels and preserve Ipopt intermediate cancellation,
   typed terminal status and catch-unwind boundaries.

**Delete/replace:** per-node one-row batches/arrays in the scalar callback route,
per-callback reservation/export scaffolding, eager full-output evaluation, stale
cancellation captured in reusable numerical preparation. Retain legitimate batch stages.

**Development exit:** scalar/batch/independent oracle cases cover every supported
operation, inactive invalid branches, derivatives, ordered arithmetic and permitted
tolerances. Same-point objective then Jacobian reuses the valid prefix without claiming
uncomputed outputs. Parameter refresh, cancellation then new solve, non-finite inputs,
allocation refusal and bounded cancellation all pass. Repeated supported callbacks
perform no steady-state allocation after workspace setup, except explicitly contracted
kernel scratch with a bound. Solver-linked integration waits for I18.

### I12 — Separate exact Delta reuse from freshness and qualify native cache policy

**Own:** catalog `cache_service/{snapshot,resident,policy,settings}*`, Delta provider/
load/lease/maintenance integration, generic engine cache policy and runtime settings.

1. Key exact snapshots independently of observed head. Retain actual root/store/table
   generation, exact version, load requirement, semantic interpretation and maintenance
   epoch. Implement separate current-head lookup with explicit freshness/refresh rules.
2. Remove latest-head observations from pinned cache-hit and resident-hit paths. On
   a pinned miss, load the version directly. Retain read leases, access checks, replacement
   invalidation, epoch-guarded insertion and truthful missing-log/data errors.
3. Reuse immutable selected semantic settings and commit-returned snapshots. Distinguish
   metadata-only state from file-complete state; do not serve a scan from insufficient
   cached capability or treat snapshot presence as a retention guarantee.
4. Allocate finite native metadata/statistics/predicate/listing/snapshot/resident/in-flight
   budgets from the shared policy. Include per-reader and concurrent-load multiplication
   in admission, not just one nominal allowance. Enable only families with sound source
   identity/freshness; mutable Delta directory listings are not exact snapshot state.
5. Qualify the existing local CRC/checksum overlay, checkpoint and replay routes.
   Native checksums may be unavailable without an eligible seed and do not enumerate
   all active files. Keep bounded native replay on absent/unsupported acceleration;
   checksum failure after success must not rerun a data write.
6. Preserve eligible CDF change-key reuse with inclusive observed bounds, images,
   deletes, schema changes and negative dependencies. Unsupported mapping/history
   invalidates incremental reuse and uses ordinary exact snapshot recomputation,
   without pretending missing history was observed.

**Delete/replace:** latest head in immutable snapshot/resident keys, repeated settings
reconstruction, head probes solely for pinned hits and accidental feature/default claims.

**Development exit:** key/load-policy units, metadata-versus-files capability controls,
generation/maintenance invalidation and budget arithmetic pass. Fake-store counters
prove pinned hits avoid head I/O. Real append, vacuum, corruption/CDF and overlay replay
journeys are authored now and executed after I17. Any overlay edit changes the patch
and regenerates vendor source via `just delta-source`, never direct vendor edits.

### I13 — Carry write evidence into coherent publication and settle known success

**Own:** catalog `delta/{operation,contract,field_check,layout,publish,settlement,provider}*`,
`delta/dml/`, `publication.rs`, artifact membership and receipt consumers.

1. Retain native WriteBuilder/update/delete/merge and the physical-input bridge;
   preserve the concrete selected PSE/Delta session and live effect admission.
2. Produce a completion record binding the exact input selection, contract/mapping,
   established local checks, committed member version and native operation outcome.
   Use it to remove only repeated checks on those exact rows. External/reopened data
   still needs unresolved admission; cross-relation publication checks still run.
3. On unequivocal native success, carry returned state, metrics and operation identity
   directly into settlement/cache admission. Replace success-path full control-row/log
   searches with the verified record or exact version-addressed receipt.
4. Preserve optimistic parent/read-set conflict rules and the control publication
   as the only coherent multi-member visibility boundary. A native update with no
   matching row is a conflict, not a successful publication.
5. On hook error, lost response or cancellation with unknown commit outcome, inspect
   committed state and reconcile before any replay. Retain known data success when
   only checksum/observation work fails. Avoid sequential append replay based solely
   on application transaction markers.
6. Preserve exact DML affected-row answers. Use trustworthy native metrics/known exact
   counts; keep an accounted correct count fallback where statistics are insufficient.
   Skip inverse casts only for a proven lossless mapping or exact reusable conversion
   evidence, retaining hostile unsigned/nested/null/metadata loss controls.

**Delete/replace:** unconditional success reconciliation scans, member receipt searches
when exact versions are known, duplicate already-proved local checks, avoidable counts
with available exact evidence. Keep uncertain-outcome recovery and necessary counts.

**Development exit:** isolated settlement/state-machine tests cover known success,
no-op conflict, uncertain outcome, receipt mismatch, partial member set and failed
acceleration; mapping/count oracles remain exact. Author the existing actual-boundary
fault matrix for final execution; do not claim stubbed settlement proves durable recovery.

### I14 — Integrate bounded model retention, concurrency and fan-out

**Own:** runtime budgets, engine execution/cache admission, catalog independent member
opens, compiler output consumers, rules epoch services, test workflow runtime policy.

1. Promote only eligible pure completed results into bounded model retention using
   exact dependency selection and I05 ownership. Preserve current live admission on
   hits. Cache eviction releases unpinned ownership; retained exports remain charged.
2. Keep cross-attempt fills attempt-local by default; share completed results across
   attempts. Within an attempt, coalesce compatible producer ports with explicit
   completion. If cross-attempt fill sharing is selected, implement waiter interest,
   fill-owned cancellation/resources and last-consumer behavior before enabling it.
3. Bound independent member opens/output execution with shared CPU, I/O, memory,
   spill and solver budgets. Assemble catalogs/results in deterministic declared order
   after concurrent work. Preserve required effect and publication dependencies.
4. Ensure nested queries inherit admission or release outer permits before waiting
   for children. Budget per-reader cache, prefetch and blocking native operators.
   A saturated parent/child queue must not deadlock.
5. Replace blanket one-worker/one-partition workflow policy with configured general
   budgets and explicit deterministic tiny fixtures. Coordinate process-level nextest
   concurrency so each test does not allocate the machine's entire capacity.
6. Select fan-out deliberately: retain expensive shared results once, stream to bounded
   consumers with a slow-consumer policy, or re-execute cheap pure work. Expose exact
   or labelled-estimate native statistics without promoting estimates to validity.

**Delete/replace:** serial independent loops, blanket serial test groups where no
semantic need exists, repeated pure completions and independent full-budget runtimes.

**Development exit:** saturated nested scheduling completes; one cancelled waiter
cannot poison another live attempt; late stream error never mints success; mixed
small/large inputs respect aggregate budgets and required ordering; eviction plus
live tiny export has honest retention. Resource refusal remains typed and bounded.

### I15 — Make Python startup and columnar transfer use retained native owners

**Own:** `pse-py/src/inspection/{runtime,stream,errors,settings}*`, known attached
entry points, `python/pse/{__init__,governance,_transfer}.py`, quality/codegen wiring.

1. Reuse the canonical immutable process registry owner. Preserve compatible shared
   runtime settings and live handle admission instead of assembling another registry.
2. Move exhaustive generated-contract lint from normal import into code generation,
   static quality and dedicated invalid-contract units. Keep lightweight extension/
   native-package compatibility, extension registration and dynamic/untrusted admission.
   Preserve the existing ban on `typing.Any`; moving the check does not weaken it.
3. Keep Arrow streams the bulk route and `structure_rows` an explicit convenience.
   Retain one-use, close, cancellation, error and last-reader semantics.
4. Detach only known Python-attached blocking Rust-only operations using exact PyO3
   bounds. Qualify the C-stream consumer route with two readers/threads and an independent
   progress control. Do not add an attachment cycle merely because a closure lacks
   `Python<'_>`. If necessary, use one bounded prefetch owner with backpressure and
   explicit cancellation, never an unbounded queue or thread per batch.
5. Replace repeated schema field removal with one native zero-column projection only
   after the consumer's native metadata representation is shown to preserve duplicate
   keys exactly. If the available API cannot, add a narrow lossless native adapter;
   do not round-trip through a Python dictionary or weaken mismatch detection.

**Delete/replace:** redundant registry assembly, import-time scans of generated classes,
repeated per-field schema reconstruction, any unqualified blocking attachment path.

**Development exit:** static generated/dynamic contract controls, exact nested/duplicate
metadata, compatibility failure, one-use/early-close/error/cancel and exported-owner
unit tests pass. Implement and compile the actual consumer/attachment and fresh-store
integration cases now; execute them in I18. Interpreter attachment behavior is recorded
there for the actual build; free-threaded or other Python builds require separate
evidence.

### I16 — Complete truthful diagnostics, fixture coverage and assurance tooling

**Own:** `pse-diagnostics`, engine assurance/testkit, `tests/support/`, conformance
fixture generator, numerical fixtures, validation scripts/tests, justfile and xtask
acceptance/coverage reporting; relevant Python assertions.

1. Use one typed diagnostic projection for semantic code, resource class and leaf
   cause. Cover transparent/shared/context/aggregate wrappers and preserve original
   leaves. Replace display-spelling assertions with typed identity without hiding
   missing causes or changing expected semantic classification.
2. Default functional helpers to Contract observation. Request Diagnostic text/JSON,
   previews and full findings only in tests asserting them. Preserve capture-cap
   refusal/truncation and off/complete/sampled/interrupted coverage explicitly.
3. Generate literals from their declared semantic fields, including tagged enums.
   Emit independent outcomes for every invariant case and both valid/violating inputs;
   a first failure cannot abort the remaining registry cases.
4. Instrument existing seams for assembly construction, intrinsic facts, new-value
   admission, requirement execution/reuse, producer completion, rule rows, numerical
   closures and Delta requests. Native plans/metrics and tracing are evidence of work;
   independent functional oracles still establish values. Disable previews by default.
5. Finish no-fail-fast assessment continuation and persistent receipts. Capture exact
   argv, exit status, source identity/drift, mode, test/case totals and interruptions.
   Dependent setup failure marks affected work blocked/not-run while independent gates
   continue. Do not manufacture a green aggregate from missing or truncated receipts.
6. Repair unsafe-audit package enumeration for the virtual workspace; aggregate actual
   supported invocations with their failures. Triage shear/machete using real/macro
   consumers. Preserve regression seed licensing and advisory provenance. No blanket
   allowances or suppressed tool errors close the inventory.
7. Keep API-reference doc lint's current R-20 deferral explicit until genuinely
   implemented. Its exit-2 stub is unsupported evidence, never a pass. Report advisory
   dependency findings under ADR-0066 separately from required product/quality gates;
   tool execution failures are still failures. Any change to required assessment scope
   is recorded before the campaign, with its authority and unresolved capability.

**Delete/replace:** forced full-plan rendering in ordinary value tests, display-string
diagnostic identity, abort-on-first invariant loops, swallowed audit exits and stale
test root-shape assumptions about wrapped native commands.

**Development exit:** disposable runner tests prove all selected case IDs survive
failure/interruption, unsupported is distinct from pass, native cause matrix is
complete and rendering-off value tests do not request incidental evidence. Actual
source/receipt matching, requested capture completeness and test enumeration agree.

### I17 — Close the complete implementation and deletion barrier

**Own:** execution inventory, case manifest, source seal, deletion/caller audits,
final policy/docs/generator reconciliation and all packages' unresolved entries.

1. Verify I00–I16 landed in producers **and** every consumer/generator/test/doc path.
   Close the deletion ledger below, including indirect consumers and inspection tools.
2. Reconcile blueprint claims with actual supported scalar, batch, Delta and Python
   behavior, current decision status and explicit deferred capabilities. Source-qualified
   implementation can be Implemented; no broad test/measurement label is inferred.
3. Check generated contract equivalence, pins/overlay provenance, crate/import boundaries
   and selected isolated unit receipts. Targeted unit tests are sufficient behavioral
   evidence for this implementation/deletion barrier. Verify that every required change,
   caller conversion and deletion is finished and the target design is fully in place
   before permitting any integration tests, compilation fixtures, publication/solver
   journeys or broad benchmarks.
4. Seal actual source bytes, symlinks/modes, relevant configs/locks and case manifest.
   Preserve old Plan 10 receipts separately. A source seal is an entry check for I18,
   not a fix for failing tests or proof of functional acceptance.

**Exit:** no replacement TODO, dead compatibility caller, missing required mechanism,
or unowned deletion remains. Every proposed control is an actual registered test case:
targeted unit controls have run; integration controls are implemented and may be
compiled, but are explicitly **not run by design until I18**. No integration pass is
required to close I17, and none is claimed by its source seal.
Optional accelerations remain explicitly disabled/unclaimed if unqualified; none may
stand in for a missing mandatory target mechanism.

### I18 — Run the complete local functional assessment and resolve findings together

**Own:** existing reusable assessment runner and per-case durable results, with every
package owning remediation of its failures.

1. Enter only after I17 confirms all scope, including every deletion, is implemented
   and the target pivot is complete. Refresh the editable extension once from sealed
   source; execute the full applicable integration suite and the complete
   current-environment assessment in a new Plan 11 directory, no fail-fast and no
   retries. Include exact case counts, standalone engineering cases, linked solver,
   Rust modes/features, Python, generation and governance described below.
2. Execute all positive/adversarial journeys authored during implementation, including
   fresh-store reopen, actual fault-injected commits, exported arrays and clean-versus-
   reused compiler/rule/numerical outputs. Collect all independent failures together.
3. Resolve defects by shared root cause. Preserve original receipts and reproducers;
   invalidate gates affected by repairs, then continue in a linked new receipt. Do not
   rerun unaffected broad gates just to accumulate green results or increase timeouts
   to conceal structural problems.
4. Reconcile case manifest versus enumeration/execution, source drift and all carried
   Plan 10 obligations. Failed, interrupted, unsupported, excluded and not-run are
   distinct. All required executable checks must have zero failures against zero.

**Exit:** complete required functional evidence for the final source. Advisory or
explicitly deferred capability reports remain visible and do not become invented
passes. Product acceptance stays open for any unresolved required case.

### I19 — Characterize performance and issue the final independent acceptance

**Own:** existing native cache/consolidation benchmarks, engineering inspection,
numerical callback characterization, final review, plan inventory and Outcome.

1. Execute the measurement matrix below after functional scope is qualified. Record
   cold/warm/changed-input/eviction/contention, force-validation and production-equivalent
   modes separately, including hardware, budgets, feature graph and observation cost.
2. Report preparation, execution, transfer, storage/recovery, callback and end-to-end
   time separately with structural counts, allocation/retention, native pool peaks
   and process RSS. Identify regressions and limits; do not promise an unmeasured
   speedup or use incomparable historical runs as a ratio baseline.
3. If tuning changes behavior/keys/policy or code, invalidate and rerun affected
   correctness controls and measurements. Fix material regressions; do not mark the
   plan done merely because a timing report exists.
4. Assess G1–G7 independently against the acceptance rows below, including extension
   locality and truthful supported capability claims. Update decision evidence through
   the proper lifecycle and record supported/unsupported boundaries.
5. Mark the plan done only when every required package, deletion and acceptance claim
   is closed. Fill Outcome with what was built, a real corrected mistake, deliberate
   deviations and exact evidence links. Preserve the historical incomplete campaigns.

### Deletion and consolidation ledger

Each row requires a caller search, applicable compilation and targeted unit-test
evidence for its replacement. That is sufficient to close the deletion during
implementation; integration verification of the completed pivot waits for I18.
A search match can be a legitimate domain operation; review meaning instead of
banning tokens.

| ID | Replace/delete | Owner | Surviving responsibility |
|---|---|---|---|
| L01 | Invalid field-before-binding and broad witness alias scope | I01/I04 | Native coercion, field meaning and lexical correctness |
| L02 | Duplicate schema serialization, cache-key derivation and successful-row path formatting | I02 | Exact external admission and precise failure paths |
| L03 | Repeated local admission at certified handoffs | I02/I06/I07/I13 | New/untrusted values and unresolved relational/durable checks |
| L04 | Nested factory/registry rebuild and witness-through-bound-state comparison | I03 | Immutable role views and fresh query state |
| L05 | Ancestor-path intrinsic fact rediscovery and independent duplicate root walks | I04 | Scope/effect/epoch-sensitive admission and native rewrite phases |
| L06 | Compiler generic tuple layout/pack/member/UNNEST route and its fixtures | I06 | Typed checked finite ports and legitimate domain nested values |
| L07 | Unconditional relational stage capture and all-intermediate output execution | I06/I07 | Finite input materialization, explicit artifacts/effects/provenance |
| L08 | Repeated invariant execution, existential full sort/DISTINCT/collect, settled barriers | I07 | Complete validation, promised full reports and pending protection |
| L09 | Per-row queries, repeated generated-prefix union/concat, inventory/path scans | I08 | Expansion dependencies and exact source support |
| L10 | Full rule representative recomputation for unchanged keys | I09 | Affected representatives, conflicts, negative dependencies and support |
| L11 | Repeated quantity/binder scans and unrequested combined canonical preimage | I10 | Exact frame bytes and untrusted canonical admission |
| L12 | Scalar callbacks' one-row stage/batch evaluator and captured solve token | I11 | Guarded scalar slots plus legitimate shared-semantic batch evaluation |
| L13 | Global pointer ledger, repeated allocation-map sweeps and cache-read buffer wrapping | I05 | Adequate pre-admitted extent and last-reader escape lease |
| L14 | Latest-head coupling/probes for exact snapshot/resident hits | I12 | Explicit freshness, maintenance and missing-file behavior |
| L15 | Success-path publication reconciliation scans and redundant write checks | I13 | Verified completion, exact counts, conflicts and ambiguous recovery |
| L16 | Blanket serial independent work and per-task full-machine budgets | I14 | Aggregate CPU/I/O/memory/spill/solver control and required ordering |
| L17 | Python duplicate registry/import lint/schema field-removal loop | I15 | Compatibility, dynamic admission, exact metadata and stream lifecycle |
| L18 | Forced Diagnostic value helpers, display-code assertions, aborted case loops, masked tools | I16 | Requested complete evidence, typed causes, truthful assessment |

### Complete review traceability

`W` and `P` below mean the two input reviews, as defined by the integrated review.
Additional opportunities refer to that review's A01–A09, not Plan 10 acceptance IDs.

| Target / integrated finding | Input findings and additional opportunities | Packages | Acceptance |
|---|---|---|---|
| T01 / C03 | W F03/F06; P P02; A08 | I02/I05/I07/I13 | V02/V05/V10 |
| T02 / C04 | W F05; P P03; A01/A07 | I03/I04/I12 | V03/V04/V09 |
| T03 / C01/C05 | W F01/F04/F08; P P04 and MathIR traversal lead | I01/I04/I07 | V01/V04/V06 |
| T04 / C06 | W F03/F13; P P05; A03/A06 | I06/I07 | V05/V06 |
| T05 / C07 | W F06–F08/F13; P P04/P07; A03/A05 | I07 | V06 |
| T06 / C08 | W F07; P P06; A07 | I08 | V07 |
| T07 / C09 | P P07 | I09 | V07 |
| T08 / C10 | P P01; A04/A07 | I11 | V08 |
| T09 / C02 | W F02; P P09; A06 | I01/I05 | V01/V05 |
| T10 / C11 | W §4.8; P P08, Delta double-cast/resident-head leads; A08 | I12/I13 | V09/V10 |
| T11 / C12 | P P11, IndexSet and MathIR traversal leads; A07 | I10 | V11 |
| T12 / C14 | P P12, Python transfer lead | I15 | V12 |
| T13 / C13 | W F10/§4.7; P P03/P10; A01/A02/A05/A06/A09 | I03/I05/I12/I14 | V03/V05/V13 |
| T14 / C15 | W F09/F11/F12; P §9 coverage | I16 | V14 |
| C16 / policy and qualification | W §9/§10; P §9/§10; integrated §6/§9/§10 | I00/I17/I18/I19 | V15/V16 |

The integrated review's corrections are binding planning constraints: preserve actual
factory-output admission, last-reader leases, full anti-join dependencies, affected
representatives, query-time freshness, native snapshot semantics and uncertain-commit
recovery. No implementation packet may revert to the unsafe shortcuts identified in
its §3 while claiming to implement the corresponding input finding.

## Verification

All target assertions below are **Proposed** until linked to actual source and named
receipts. **Interface-checked** means inspected pinned API/source. **Implemented**
requires the replacement and caller/deletion evidence. **Tested** names the command,
mode, selected/executed counts and **zero failure baseline**. **Measured** includes
workload and environment conditions. No formal proof or target speedup is claimed.

### Development versus integration command boundary

**Before I17 closes, targeted unit tests are the only behavioral tests to execute
and are sufficient for implementation/deletion progress.** Compile-only, static and
pure-generation commands remain available as needed. Do not invoke a convenience
aggregate such as `just test`, `just ci-fast`, `just ci-pr`, `just py-test`,
`just architecture-acceptance` or `just assessment` while any target implementation
or deletion remains unfinished. Review transitive recipe/fixture behavior, not just
the command's label. Full integration is mandatory afterward, not between slices.

| Stage | Existing command surface | Meaning and limit |
|---|---|---|
| Planning and I00 discovery | `just doctor`, `just --list`, `just assessment-list`, `just lib-outline <doc>` | Readiness/scope/document discovery; not product correctness |
| I01–I16 compilation | `just check-library <pkg>`, `just check-package <pkg>`, `just check-test <pkg> <target>` | Compile only; do not execute product/fixture journeys |
| I00–I17 isolated units | `just unit-package <pkg> '<reviewed-filter>'`; existing `dev-*`/`unit-*` recipes only after inspecting their selection | Sufficient behavioral verification for implementation and deletions; explicit force-validation, no integration hidden under `--lib` or fixture setup |
| I00–I17 Python units | `just py-unit <selection>` | Targeted units only; no component, fresh publication or inspection fixture |
| I01–I17 static and pure generation | `just codegen-contracts`, `just codegen-contracts-check`, `just family-check`, scoped formatting/lints/types, `just docs`, `just adr-lint` | No direct generated edits; full physical fixture generation waits for the barrier |
| I17 barrier | Extended `just architecture-seal` and `just architecture-preflight` selecting Plan 11 | Complete source/deletion inventory and targeted unit evidence; integration is not a prerequisite. Current recipes are Plan 10-specific and need I00/I17 updates |
| I18 integration/functional campaign | `just assessment <new-directory>` with the I16 phase-aware scope | Full applicable suite only after every implementation/deletion item is complete; no-fail-fast receipts and no silent omission of original selected gates |
| I19 characterization | Existing `just bench-smoke`, `just bench-cache`, `just bench-consolidation-native`, `just engineering-inspection <new-directory>` plus I00/I11-owned numerical/performance recipes | Reuse campaign-owned runs; a new production-equivalent mode is Proposed until implemented and listed by `just --list` |

No existing recipe completely covers Plan 11's scalar selective callback measurements
or its new barrier. I00 assigns those gaps to the existing justfile/xtask/benchmark
owners instead of prescribing an unreviewed long Cargo command. Keep pinned tools
and normal build caches; no routine `cargo clean`, fresh target directories, wheel
builds, or integration runs during implementation.

Before the final campaign, extend the existing runner with dependency-aware functional
and measurement phases. Functional failures do not suppress independent functional
checks; measurements that need an unqualified workflow receive a truthful blocked
status and run after remediation. Preserve one aggregate scope and receipt chain.

### Required functional and adversarial acceptance

Each row requires targeted unit controls during implementation and its applicable
final real-consumer journey in I18, after the full pivot and all deletions. These
final acceptance obligations are not extra pre-I17 package gates. Exact executable
case IDs are registered in I00 and implemented with the owning package; the
descriptions here are specifications, not claims that new tests already exist.

| ID | Required oracle and negative controls | Owners |
|---|---|---|
| V01 | Native lambda/coercion/field phases and repeated aliases preserve declared meaning; hostile metadata rejects. Owner-aware hidden/sliced extent resolves the existing mismatch without weakening limits. | I01; C01/C02 |
| V02 | Registry/schema/prepared predicate reuse avoids repeated work; valid preservation retains evidence. Changed/cast/foreign/new values reject appropriately; local checks never certify keys/references/completeness. | I02; C03 |
| V03 | Compatible revisions retain one assembly; immutable old/new views coexist. Changed relevant providers/functions/policies invalidate; unrelated edits retain eligible work. Fresh query time, live permission/quota and prepared SQL survive correctly. | I03/I14; C04 |
| V04 | Distinct shared DAG facts scale with actual structure while lexical/outer/worktable/effect scopes remain distinct. Actual factory output is admitted; volatile folding, changed epoch, rewrite/reset/dynamic-filter controls prevent unsafe reuse. | I04; C05 |
| V05 | Typed finite ports share one admitted attempt completion, expose dependencies and preserve source/field meaning; changed inputs/demand cannot collide. Array/slice/dictionary/FFI exports keep adequate lease through final reader and eviction. | I05/I06; C02/C06 |
| V06 | Requested closure includes all required validation/effects/provenance and publication membership. Complete dependency keys include absent/empty/reference targets. Existence/sample/full report contracts and settled barrier release cannot hide an invalid row. | I07; C07 |
| V07 | Bulk compiler construction preserves generated dependencies and exact positive/negative support. Affected rule groups update actual representative/payload/conflict/provenance, including duplicates, signed zero and negative-input changes. Clean versus reused outputs agree. | I08/I09; C08/C09 |
| V08 | Guarded scalar/batch/independent numerical values, failures, derivatives and ordering agree under declared policy. Inactive invalid branches stay unevaluated; parameter/point/output masks, attempt cancellation, workspace allocations and linked solver results are correct. | I11; C10 |
| V09 | Pinned Delta reuse survives unrelated append without head I/O; root replacement/maintenance/insufficient load capability invalidates. Missing vacuumed data still fails. CDF bounds/images/features, checksum eligibility and budgets are truthful. | I12; C11 |
| V10 | Exact write evidence and loss-aware mapping justify every skipped check. Native session/functions remain intact. Known success avoids redundant scans; parent conflicts, post-commit hook errors, lost responses, partial members and exact DML counts settle correctly. | I13; C11 |
| V11 | Quantity/binder/index reuse respects full context; trusted canonical reuse preserves byte-identical hashes. Cycles, malformed bindings and self-consistent forged rows reject. Hash-only framing avoids only the unnecessary combined buffer. | I10; C12 |
| V12 | Python static governance retains coverage; extension compatibility and untrusted contracts still check. Exact metadata including duplicate keys, stream use/close/error/cancel, actual attachment behavior and last-reader ownership hold through real consumers. | I15; C14 |
| V13 | Nested saturated CPU/I/O/resource scheduling terminates without oversubscription or permit deadlock. Waiter cancellation, partial fills, slow consumers, fan-out and eviction preserve independent work and resource admission. | I14; C13 |
| V14 | Every selected invariant/gate has an outcome; typed leaves and resource codes survive wrappers. Contract-mode value tests avoid full rendering; requested evidence limits remain visible. Audit refusal, interruption and source drift cannot pass. | I16; C15 |
| V15 | Complete current-environment campaign closes carried failures and required unrun gates, with unchanged-source or explicitly invalidated/requalified receipts. All required executable gates have zero failures against zero. | I17/I18; C16 |
| V16 | All target mechanisms/deletions/policies are reconciled; cold/warm/changed/eviction/contention measurements disclose conditions. G1–G7 are independently assessed; deferred capabilities and unmeasured claims remain explicit. | I19; C16 |

### Plan 10 acceptance carry-forward

Plan 10's case manifest remains source evidence. Map each actual retained case to the
new manifest and retain its identity/history; replace a case only with an equal or
stronger oracle and an explicit explanation of the changed production contract.

| Plan 10 obligations | Plan 11 preservation/extension |
|---|---|
| A01 foreign contract admission; A02 nested constraints; A03 native values/generation | V02/V10/V11; keep every declared family and hostile foreign declaration |
| A04 exact arithmetic and nested Python transfer | V07/V08/V12; retain existing P4 extrema/nonintegral/zero controls |
| A05 operation/effect/lease lifecycle; A06 complete native adapter hooks | V04/V05/V06/V13; retain scalar/aggregate/window/provider hook matrix and native identity |
| A07 complete dependencies and reuse | V03/V04/V06/V09; retain Plan 09 consumed-input and negative-dependency controls |
| A08 compiler outcomes/provenance; A09 graph/numerical correctness | V07/V08/V11; retain current template/engineering capabilities and independent oracles |
| A10 allocation ownership; A11 Delta mapping; A12 recovery/retention | V05/V09/V10; actual faults and exports remain required |
| A13 Python settings/reports/streams; A14 engine/diagnostic/testkit boundaries | V12/V14 plus `engine-boundary-check`, import/metadata and constructor controls |
| A15 measurements; A16 complete review/deletion closure | V15/V16; retain prior L01–L18 deletions alongside this plan's ledger |

### Retained failures and unrun gates

These are the integrated review/W §9's historical findings, carried as open
verification obligations. A focused prior repair or now-ready doctor does not turn
the old campaign into an unchanged-source passing run.

| Historical assertion group | Count | Remediation and required final receipt |
|---|---:|---|
| Diagnostic display spelling | 11 | I16 typed identity assertions; V14 |
| Rule cause traversal | 3 | I16 full leaf traversal, including actual failing wrapper path; V14 |
| Numerical resource classification | 1 | I11/I16 nested resource classification; V08/V14 |
| Numerical fixture semantic metadata | 2 | I02/I11/I16 declared literal construction; V02/V08 |
| Retained allocation extent | 1 | I01/I05 owner-aware capacity; V01/V05 |
| Tagged enum fixture generation | 1 | I16 declaration-driven literal and independent case results; V14 |
| Diagnostic plan capture bound | 1 | I16 requested observation and explicit capacity outcome; V14 |
| Unresolved native lambda | 1 | I01/I04 phase order; V01/V04 |
| Deferred DDL root shape | 1 | I16 assert command/effect contract through legitimate wrappers; V04/V14 |
| Implicit physical observation | 1 | I16 explicit observation request or execution-contract assertion; V14 |
| Ambiguous witness qualification | 1 | I01/I08 scoped source binding; V01/V07 |
| Native timeouts/interruption/not-started | 5 / 3 / 33 | I18 complete selected run; preserve status distinctions and investigate slow remaining workflows |

The retained assessment completed 46 checks before interruption: raw 37 successful
exits and nine failures, with 27 further gates not run. The old masked unsafe-audit
success is invalid evidence. Keep its strict failing follow-up and original logs.

| Outstanding check family | Required disposition |
|---|---|
| Environment/type follow-ups | Preserve historical failures; recheck final environment and types after extension refresh. |
| Licence annotation on property regression seed | Preserve reproducer and add the correct annotation through its source owner; no deletion to clear lint. |
| `doc-lint` phase-0 stub | Report R-20 deferred/unsupported honestly; no API-doc validation claim from exit 2. If implemented, run its real gate and update scope/authority. |
| Stale architecture preflight | Replace with Plan 11 source-qualified barrier after implementation/deletion, then actual acceptance; resealing alone closes no failure. |
| quick-xml advisories, proc-macro-error2 status, vendor wildcard findings | Re-inventory exact dependency paths and current advisory data during I16; record remediation or ADR-0066 advisory disposition. No exploitability claim from a label, no blanket suppression. |
| Shear/machete | Prove real versus macro-expanded consumers; remove only genuinely unused dependencies and qualify the changed graph. |
| Unsafe audit virtual-manifest refusal | Supported per-package enumeration and honest aggregate results; tool refusal remains failure. |
| Rust remaining modes/doctests/features | Complete workspace/default/no-default/release, doctests, declared feature combinations and coverage as selected by the final scope. |
| Native solver, Python and engineering | Pinned Linux solver execution, refreshed editable Python unit/component cases, Rust/Python reopen and `heater-ftpx`, `heater-fctp`, `mixer-ftpx`, `mixer-fctp` cases with every outcome retained. |
| Benchmarks and measurements | I19's complete matrix, with smoke execution distinct from timing qualification. |

Use `just assessment-list` as the executable inventory and reconcile it with this
table. Include `just test`, `just test-release`, `just doctest`, `just doctest-release`,
governance/family/engine-boundary/codegen checks, both Clippy modes, Python quality,
`just native-solver-test`, feature checks, coverage, fresh inspections and the named
engineering cases. Keep their command-owned feature/profile details.

Current-environment scope excludes Windows/macOS/other Python versions, wheel/sdist
builds, remote GitHub configuration, alternate toolchains, rebuilt solver images,
and unbounded mutation campaigns. IDAES parity uses another Python environment and
is excluded from this local campaign under the existing scope; do not claim numerical
IDAES parity from native solver success. Existing regression outcomes remain required.
No exclusion, unsupported result or advisory finding may be relabelled as pass.

### Measurement matrix

All rows are **Proposed measurements**, executed in I19. Extend the existing
`native_consolidation`, `native_cache` and engineering fixtures; use an additional
focused scalar callback fixture only for the uncovered selective-output workload.

| Workload | Separate observations | Structural acceptance / comparison |
|---|---|---|
| Registry/local admission | Cold schema/predicate preparation, checked handoff, new/foreign rows; nested shapes and null rates | Work follows distinct contracts and new chunks; unchanged checked handoffs do not rescan values. |
| Model preparation | Cold, unchanged repeat, relevant/unrelated source edit, changed UDF/policy, diamond DAG | Assembly builds, intrinsic facts, contextual checks, bound requirements and retained plan bytes; native phases remain visible. |
| Source-to-P3 and full heater/mixer compilation | Composition/preparation/execution, demanded outputs, provenance and publication | Query/capture/concat/inventory counts by model size; no universal tuple transport or per-row preparation. |
| Rule fixpoint | Cold index construction, small/large candidate deltas, representative change and negative-input invalidation | Rows examined, affected groups, index build/probe work, rounds, result/support cardinality and memory. |
| Scalar numerics | Program preparation, workspace setup, repeated same/new points, objective then derivatives, parameter refresh | Instruction/slot/output counts, allocations per callback, cancellation latency and independent value/Jacobian checks. |
| Batch/scenario numerics and solves | Declared batch sizes, shared DAG/guard shapes, actual supported solver cases | Throughput, preparation, callback and end-to-end solve time; same semantic contracts, no new Hessian claim. |
| Ownership and fan-out | Multiple ports, cache hit/eviction, tiny slice pinning large parent, FFI array surviving parent | Scratch/completed/pinned/exported bytes, copies, pool peaks and RSS separately; adequate last-reader lease. |
| Delta reads/replay | Cold/warm exact version, unrelated append, refreshed head, metadata/files capability, CRC eligible/ineligible | Head/log/file/list/byte counts, replay/load latency, maintenance/retention failures and acceleration eligibility. |
| Delta writes/publication | Known success, ambiguous post-commit error, concurrent members/parent conflict, exact DML count | Native write/settlement/read-back I/O, publication latency and correct visible member set. |
| Python | Cold import, repeated handle creation, columnar stream, requested rows, two-reader progress | Static lint removed from import, registry reuse, metadata correctness, attachment/backpressure and ownership cost. |
| Concurrency and observation | Small/large workflows under declared CPU/I/O/partition budgets, contention, Contract versus requested Diagnostic | Total resource use, deadlock/cancellation controls, observation overhead and complete evidence coverage. |

Record exact source/locks/overlay, host/CPU, compiler/profile, feature graph,
validation mode, dataset shape, cache state, thread/partition/solver settings,
memory/spill budgets, observation mode, repetitions and dispersion. Correctness
runs use explicit force-validation. The separately named production-equivalent
measurement may omit it only after I00's policy/recipe change and an actual recorded
feature graph confirms the mode; Cargo feature unification must not silently retain it.

Report absolute and comparable relative results. Historical partial timings are
context, not a universal baseline. No 100×, seconds-level or arbitrary twofold threshold
is promised. Select any stable timing regressions after representative evidence;
ordinary CI can enforce deterministic work-count properties independently of timing.
Do not retain or resurrect the old production evaluator/tuple pipeline to manufacture
a before/after benchmark. Existing bounded semantic oracles remain test-only.

### Independent gate closure

| Gate | Required evidence |
|---|---|
| G1 authority | V02/V04/V05/V11: registry, selected providers, native plans and Delta control remain authorities; derived indexes/instructions add no editable truth. |
| G2 semantic fidelity | V01/V07/V08/V10/V12: fields/nulls/multiplicity/order/source identity, guarded values, loss-aware durable mapping and Python transfer agree. |
| G3 validity | V02/V05/V06/V10: every removed check names exact producer/preservation/completion evidence; invalid new/untrusted inputs still reject. |
| G4 hidden behavior | V03/V04/V06/V13: effects, ambient values, permissions, cancellation and resources retain live attempt boundaries. |
| G5 consistency/recovery | V05/V09/V10/V13: last reader, conflicting parent, partial members, ambiguous commits and maintenance remain correct. |
| G6 transformation/reuse | V03/V04/V06/V07/V08/V11: complete dependency keys, epoch/reset qualification and clean-versus-reused equivalence. |
| G7 truthful claims | V14/V15/V16: complete receipts, unsupported boundaries, architecture wording and measurements accurately describe the delivered scope. |

Evaluate each as pass, fail, unresolved or genuinely not applicable with supporting
receipts. A passing microbenchmark cannot offset a required unresolved gate.

## Open items

The architectural direction is selected. These are bounded implementation choices
with defaults and explicit resolution points, not reasons to restart architecture
selection or request permission to change necessary policy.

| Choice / remaining clarification | Selected default | Resolution and failure behavior |
|---|---|---|
| Finite multi-output native adapter | Existing operation/completion plus typed relation ports | I06 proves rewrite/demand identity and once-only execution; extend this seam if needed, never restore tuple transport. |
| Physical preparation reuse | Reuse only qualified reset/reentrant operators; otherwise immutable logical preparation | I04's operator matrix proves behavior; unqualified mutable state remains attempt-local. |
| Foreign backing allocation extent | Owned explicit extent or checked reserved copy | I01/I05 must demonstrate adequate lease for every escape. No guessed hidden capacity. |
| Cross-attempt in-flight coalescing | Disabled; bounded completed reuse is required | I14 may enable only after interested-consumer cancellation/resource controls; record default and limits. |
| CRC/checksum and native cache capacities | Sound finite budgets; acceleration only for qualified source/profile | I12 consumes local overlay, feature and replay controls; missing CRC uses bounded native log replay, not a legacy path. |
| CDF incremental eligibility | Supported observed ranges only | I12 invalidates/recomputes exact snapshot state when history/features cannot support incremental proof. |
| Scalar kernel coverage | Every currently supported solver operation gets an admitted scalar implementation or explicit preparation refusal | I11's capability matrix must not silently reduce previously supported modeling outcomes; resolve required gaps before I17. |
| Python C-callback attachment and schema projection | Qualify actual caller and preserve native metadata exactly | I15 narrow API/source/probe work chooses detach/prefetch/native adapter without unsafe attachment assumptions or dictionary conversion. |
| API doc-lint/advisory assessment status | Existing explicit deferral/advisory authority, truthful separate reporting | I00/I16 register scope before the campaign; no zero-baseline waiver for required tests, lints or broken tools. |
| Timing thresholds and final capacities | No unmeasured speed promise | I19 derives tuning from comparable results; changed semantics/policy reopens affected checks. |

If an exact library clarification remains after the skill and source, add a bounded
discriminating probe with its lock/features, positive and negative controls and receipt
to the owning package. Before I17, execute it only if it is an isolated unit-level
contract probe. Implement integration-dependent probes now and defer their execution
to I18 under the same rule as other integration cases. Resolve implementation choices
using the pinned contract/source and targeted units; record remaining integration
claims for final qualification. Do not promote a discovery index,
interface match or source search into a runtime guarantee.

At each checkpoint, leave the next package, implemented/deleted boundary, exact
commands/modes/results, case manifest state and remaining gates in the repository
inventory. Preserve task-relevant dirty work and all interrupted receipts. A new
agent resumes that handoff rather than rediscovering or requalifying the whole tree.

## Outcome (recorded after implementation)

### What was built

I00–I15 are implemented with targeted development evidence recorded in the
[execution inventory](11-execution-inventory.md), the
[I07-I12 checkpoint](11-i07-i12-execution.md) and the
[I13-I15 checkpoint](11-i13-i15-execution.md). I16/I17 are implemented with 332 passing targeted units after I18 corrections, including all deletions; see
[their execution checkpoint](11-i16-i19-execution.md). I18/I19 and the full target's
functional/performance qualification remain open. Fill the final outcome after I19.

### A mistake made and corrected

Native expression normalization initially rebuilt an UNNEST dependency mapping before
validating the offered mapping. The forged-mapping unit exposed that loss of evidence;
the original mapping is now checked before rewriting. Resource-bound SessionState
reconstruction also discarded native prepared SQL; retained model context cloning now
owns SQL planning and definitions, with scoped runtimes applied at execution binding.

The I04 stable-time control exposed a missing nondeterministic read classification;
repeated reads now refresh pre-fold values. The I06 shared-producer control exposed
DataFusion cooperative wrappers changing physical addresses; reuse now follows
unchanged actual cache owners while changed inputs receive fresh completion.

### Deviations from the plan, deliberate

No additional campaign controls or agent infrastructure are being added after I00,
per the user's execution direction. The existing enforcement remains. Workspace
Clippy uses the normal `just clippy` recipe in both feature modes; behavioral checks
remain targeted units until I17. Further deviations belong here and in their owning ADR.
