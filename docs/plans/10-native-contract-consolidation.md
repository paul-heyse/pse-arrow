---
title: Native contract compilation and consolidation hard pivot
status: in-progress
date: 2026-09-18
adrs: [ADR-0071, ADR-0072, ADR-0073]
phase: 1
evidence: Proposed
---

# Native contract compilation and consolidation hard pivot

**Historical execution plan:** [Plan 13](13-rust-computation-architecture.md) now owns
the replacement architecture and all unresolved acceptance inherited through Plan 11.
Use the [current repair checkpoint](13-w19-repair-checkpoint.md) to resume. The scope,
failed/incomplete receipts and dated status statements below retain their original
meaning; they do not certify the current working tree.

## Context

This is the full implementation plan for the recommendations in the
[native consolidation reassessment](../design_review/reviews/design_review_native-consolidation-reassessment_2026-09-18.md).
It covers all twelve independent findings R1–R12, all seventeen dispositions of the
initial review, and the supporting recommendations in review §§2–11. The initial
review is an opportunity inventory; its rejected shortcuts are not implementation
instructions. The coverage matrices below identify each selected replacement.

**Status: grouped N18 repairs are implemented; N16/N17 are closed for a source-qualified continuation.** The
[execution inventory](10-execution-inventory.md) records source-qualified progress.
Plan 09 remains a completed predecessor
with its own source-qualified receipts. This plan adds the reviewed consolidation;
it does not reopen earlier execution strategies or Plan 07's unscheduled simulator scope.

**Outcome:** one compilation of resolved semantic contracts into native Arrow fields,
DataFusion checks/plans/functions, Delta enforcement/storage mappings and generated
interfaces; shared native execution services; relational compiler correspondence;
compact graph/numerical preparation; native resource accounting; and one implementation
of the associated diagnostics, configuration and boundary conversions.

**Hard pivot:** update all active callers and delete replaced implementations. No
compatibility modules, predecessor readers, fallback engines, old-data migration,
dual stores or transition period. Retain useful domain algorithms and the effects,
ownership and recovery semantics needed by the target; predecessor representations
are not acceptance oracles. Change existing policies or crate restrictions where they
prevent the target, rather than retain redundant code to satisfy them.

**Functional boundary:** preserve currently implemented process-model authoring,
typed compilation, quantities, graph analyses, numerical kernels, native/Delta execution,
publication/reopen and Python inspection. Do not add a new end-to-end solver workflow,
new physics, sweep service, backend, distributed execution or remote destructive
coordination. Useful numerical components may remain independently callable without
pretending they are already connected to a product workflow.

**Verification order is mandatory:** compilation, pure generation and explicitly
selected isolated unit tests during implementation. Finish **all** code, callers,
declarations, fixtures and deletions in N00–N16, then close N17's implementation
inventory. **Only N18 may execute integration, storage/compiler/solver journeys or
performance qualification.** A journey under `--lib` is still integration. Do not
increase timeouts or retries to obtain acceptance.

### Baseline and evidence

The planning baseline is `main` at
`a46f358bdfc2ca27f9f240ab6c045b63141c3ee9`, with the two review documents,
their evidence manifests and book entries present as uncommitted documentation.
Preserve that work. The reassessment's
[evidence manifest](../design_review/evidence/native-consolidation-reassessment-2026-09-18.json)
records inspected source hashes and P1/P2 characterization probes. Refresh evidence
for changed paths when execution starts; do not repeat unrelated discovery by default.

**Interface-checked during planning:** `just metadata-resolve`, current consumers,
the DataFusion/Delta skills, selected sources and command recipes. Pins are DataFusion
**55.1.0**, Arrow/Parquet **59.3.0**, object_store **0.13.2**, delta-rs
**58f07cd62bfbce3649a7e1c87c696288068ae184**, and kernel
**8ba063f8f84fec222000f66d40d70911d7c79675**. Delta uses the workspace source
overlay in `vendor/delta-rs/PROVENANCE.json`; its additions are not upstream APIs.
Installed PyArrow is **25.0.1**, resolved PyO3 **0.29.2**, and Rust **1.98.1**.
No family upgrade is required.

**Tested in the preceding review, not implementation qualification:** P1 characterizes
Arrow claims/slices and reproduces the copied integer-division guard's panic; P2 invokes
the actual Python transfer classifier and demonstrates map-child metadata misreporting.
These are regression inputs, not evidence that fixes exist.

`just doctor` during planning reports **one environment-freshness failure, baseline
zero**. Planning does not rebuild the extension. No workspace compilation, test suite,
integration journey or benchmark was run to create this plan. The
[planning evidence](../design_review/evidence/native-contract-consolidation-plan-2026-09-18.json)
records current pins, source continuity and documentation checks. Implementation evidence
will be recorded by N00/N18, not relabeled onto this planning manifest.

### Library capability references

Use the [DataFusion skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/SKILL.md) and
[Delta Lake skill](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/deltalake/SKILL.md), including its
source overlay (`.codex/skills/deltalake/content/overlays/pse-native-cache-seams.md`, local reference).
Do not use Context7 for Rust Arrow, DataFusion or Delta. It remains available for
PyArrow/PyO3 and other libraries. Verify version-sensitive claims at the selected pin.

| Capability | Reference / implementation constraint |
|---|---|
| Native local checking | `datafusion_expr::expr::Expr`, `datafusion_physical_expr_common::physical_expr::PhysicalExpr`, `SessionState::create_physical_expr`; [expressions](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/content/topics/expressions.md). Compile under the actual preparation context. |
| Native operators | [Logical planning](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/content/topics/logical-planning.md), [ExecutionPlan](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/content/traits/ExecutionPlan.md). Audit provided methods, reconstruction, reset and child statistics. |
| Function adapters | [ScalarUDFImpl](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/content/traits/ScalarUDFImpl.md), [AggregateUDFImpl](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/content/traits/AggregateUDFImpl.md). Preserve capabilities only where adapter preconditions hold. |
| Sources/providers | [Custom providers](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/content/topics/custom-table-providers.md). Use native memory/source/stream adapters; private, consumptive WorkTable APIs do not implement multi-reader epochs. |
| Resources | `datafusion_execution::memory_pool::{MemoryConsumer, MemoryReservation}`; [sessions/runtime](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/datafusion/content/topics/sessions-and-runtime.md). Arrow claims and the native bridge are infallible accounting, not allocation admission. |
| Delta operations | [Builders](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/deltalake/content/catalogs/operations.md), [DataFusion integration](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/deltalake/content/topics/datafusion.md). Operations are on `DeltaTable`; no `DeltaOps`. |
| Delta validation/storage | [Constraints](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/deltalake/content/topics/protocol-and-constraints.md), [schema/types](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/deltalake/content/topics/schema-and-types.md). Invoke public operations, not private `validation_predicates`. |
| Retention/transaction evidence | [Maintenance](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/deltalake/content/topics/maintenance.md) and the selected-source overlay. Transaction actions are witnesses, not automatic retry deduplication. |

Resolved Arrow pool and DataFusion Arrow-bridge features are currently off. N13 may
enable `arrow-array/pool` and `datafusion-execution/arrow_buffer_pool` with the ownership
protocol below. Signatures alone do not prove feature reachability or public import paths.

## Decisions

These are **Proposed target decisions** selected for execution. Defaults are concrete;
the bounded selection tests below resolve library-dependent variants without a second
architecture exercise or execution track.

### D01 — Resolved contracts and exact admitted handles

`pse-schema` owns an immutable `ResolvedRelationContract` and registry binding; the
name is a target API. Include native Arrow fields, nullability, keys, references, checks,
enum domains, resolved extension parameters and operation-relevant policies. Resolve
referenced definitions as a shared graph; cyclic relation references must not expand
descriptors recursively. Exact graph comparison uses visited-pair memoization as needed.

Classify semantic versus presentation/documentation metadata by declared role. Unknown
metadata remains semantic. Keep full physical/transfer fields; excluding documentation
from execution identity must not falsely report that every metadata byte survived.

Generated adapters bind their complete expected contract once per actual registry owner.
Foreign admission compares exactly, including transitive enum/extension definitions.
Digests index candidates or reject mismatches; matches are not proof of equivalence.
After admission, sealed owner-bound handles permit cheap pointer/handle checks. No
public ordinal, digest or pointer address can fabricate admission. Interning is bounded
by the actual immutable registry lifetime, not a global unbounded cache.

### D02 — One semantic compiler, distinct evidence levels

| Product | Owner | Native form / meaning |
|---|---|---|
| Resolved declarations/fields | `pse-schema` | Arrow fields plus domain declarations; no session ownership |
| Prepared local predicates | `pse-relations` | Native expressions/physical expressions under supplied context |
| Structured local violations | `pse-relations`, schema in `pse-schema` | Arrow rows with rule, relation/key, nested path and observed value |
| Relational obligation templates | `pse-relations`, declaration in `pse-schema` | Native aggregate/anti-join/projection templates bound to actual relations |
| Requirements and completion | `pse-engine` | Operation barriers and established evidence; no publication authority |
| Durable lowering/storage | `pse-catalog::delta`, mapping declared in `pse-schema` | Delta constraints/builders and exact execution/storage translation |
| Publication validity | `pse-catalog` | Completed obligations and exact member-version selection |

Physical Arrow validity, field admission, local values, relational validity and publication
completeness remain distinct. Advertise optimizer constraints only after their source
establishes them. Semantic checks respect null-parent visibility and active alternatives;
imported buffers first satisfy Arrow structural safety.

### D03 — Native columns, lossless literals and semantic hashing

Bulk work uses arrays, masks and builders. `ScalarValue` serves dynamic scalar APIs,
not mandatory row-by-row materialization. Keep generated typed rows/borrowed views and
domain newtypes with actual consumers. Remove the universal `Cell` graph.

One field-directed literal codec in `pse-schema` preserves raw float bits for lossless
literals. Canonical identity is a separate `pse-ids` kernel with explicit NaN,
signed-zero, metadata and ordering policies. Rust calls the kernel directly; its UDF
is an adapter. One frame writer owns length/tag/version encoding across identity paths.

Use native `DataType` plus a checked canonicalization descriptor. Retain Arrow
`RowConverter` for current ordering. Explicit unsupported canonical encodings do not
ban those Arrow types elsewhere; add encodings when a real consumer requires them.
Self-description becomes Arrow batches built within schema, avoiding a schema/relations
cycle. If a semantic preimage format changes, version it deliberately and update all
current consumers/fixtures; no historical decoder or migration is required.

### D04 — Two new crates and explicit composition

| Boundary | Target ownership / dependency direction |
|---|---|
| `pse-schema` | Declarations, resolved handles, literals, generator traversal; never depends on engine/catalog |
| `pse-relations` | Admitted batches/views and prepared predicates/obligations; accepts native preparation services |
| `pse-ids` | Identity and minimal native allocation ownership; may use modular DataFusion resources, never engine/catalog |
| `pse-diagnostics` | Leaf code/class vocabulary and causal classification; no schema/relations/numerics/structural dependency |
| **New `pse-engine`** | `EngineFactory`, bound `EngineSession`, native adapters, preparation/traversal/operation families, generic cache/resource policy; no Delta/catalog import |
| `pse-catalog` | Publication/provider selection, leases, durable identities, Delta planners/operations and snapshot adapters; composes engine |
| Domain/compiler/rules/numerics | Algorithms use engine services; durable orchestration may legitimately use catalog |
| `pse-runtime` | Product composition and the actual shared runtime/pool |
| **New `pse-testkit`** | Dev-only fixtures/faults/assertions around production assembly; no production normal/build dependency |

Split current `SnapshotSession`: generic bound-plan operations move to engine; selected
publication/read leases remain in catalog. Native sources/streams retain actual owner
guards supplied by catalog without a Delta type in engine signatures. Moving all fields
unchanged does not remove the coupling.

Narrow `AlgorithmContext` to engine services, declared inputs, required physical inventory/
documents, registry, cancellation and native resources. Keep compiler-specific port types
in the domain layer. Engine operation scaffolding does not import `CompilerError`; native
external errors retain concrete domain causes. Repurpose the unimplemented diagnostic
crate and remove its unused high-level dependencies. No `pse-codes`, `pse-delta`, catalog
compatibility re-exports or second production factory at completion.

### D05 — Operation families and complete native delegation

Ordinary relational work uses native operators. Custom families are finite algorithms,
requirement/lease barriers, shared computations and durable commands. Share mechanics
within those families; declare identity, consumption, cardinality, reset, cancellation
and effects explicitly. No generic workflow DSL or second scheduler.

Audit required and provided native methods: fields, simplification, statistics, ordering,
groups/sliding/window/reversal, volatility and reconstruction. Rewrap rewritten native
functions. Transparent wrappers forward modern child statistics and native metrics;
unknown facts stay conservative. Validation/effects cannot disappear behind projection
or limit. Epoch inputs retain immutable multi-reader ownership and replacement exclusion;
native streams replace transport boilerplate, not that protocol.

### D06 — Scoped dependencies and correctly keyed reuse

Share bounded traversal mechanics with purpose-specific context: provider/implementation
owner, demand, correlation/worktable scope, invocation/epoch, region and effect ancestry.
Optimizer transfer has one demand list per actual child; semantic evidence also retains
requirements, absence, row membership, multiplicity, keys and negative facts. Hidden cache
producers remain available to evidence without reopening the optimizer boundary. Unknown
extensions keep whole-input evidence.

Cache syntax by SQL/dialect/parser options; bind plans under actual provider/function/
rule/schema owners and semantic settings. Preserve existing bounded cache lifetimes.
Generic eviction/accounting can be shared while snapshot/resident/result keys and freshness
remain distinct. Configuration classification is explicit/versioned, never a blanket
prefix exemption for unknown options. Hits still re-admit the current resource request.

### D07 — Relational correspondence and specialized graphs

P7/P8/P9/template correspondence/provenance uses native matching, grouping, joins and
support relations. Carry source keys alongside values. Materialize shared inputs where
that bounds repeated producers/preparation; do not prescribe one giant query per pass.
P6's native joins are a foundation to consolidate.

Retain indexed graph algorithms with typed argument, payload, guard and binding edges.
Provide dependency versus evaluation views and borrowed binding overlays, eliminating
whole-graph clones. Default to the current admitted graph with shared adjacency utilities.
Petgraph is eligible where it simplifies the implementation and preserves ordered traversal,
cycle witnesses and guards; conversion merely to change library names is unnecessary.

### D08 — Compact numerical preparation and distinct arithmetic roles

Share checked kernels with separate evaluate/fold/lower/differentiate/quantity bindings.
Non-foldable does not mean unevaluable. Select native math by actual domain/precision/error
behavior; use an explicit kernel/UDF where needed. A UDF is not automatically differentiable.

Keep node identity through value/gradient lowering, share derivative nodes and sparse
bindings, then emit native projection stages/reusable physical expressions. Compute shared
pure nodes once within an evaluation region; keep unsafe branch-local work guarded.
Inspect physical preparation for renewed inlining: aliases alone prove nothing. Prepare
once and bind changing Arrow values in callbacks; no query planning inside callbacks.

### D09 — Native reservation authority and narrow ownership adapters

Replace `MemoryReserver`, custom reservations and `PoolReserver` with native DataFusion
consumers/reservations. Today's production wrapper already uses the native pool. A small
`Arc` owner can hold a native reservation across buffers/FFI/readers; it is not another
budget abstraction. Use one deployment pool for shared allocations and separate attribution.

Fallible admission precedes allocation. Retained allocation capacity, visible/copy estimates
and RSS remain different metrics. Deduplicate shared backing allocations; charge copies.
Remove forecasts whose only purpose was Cell decoding.

Use Arrow claims only through a tested allocation-aware transfer that consumes pre-acquired
native capacity and preserves last-owner release. `try_grow` then `claim` is insufficient.
Where the infallible claim API cannot represent safe transfer, the selected target is the
native reservation owner, not the legacy reservation trait or unaccounted foreign buffers.

### D10 — Native Delta operations with complete effects and retention

Keep native providers/snapshots/CDF/builders/caches and the selected overlay. One
`DeltaOperationContext` carries actual session, resolved contract, lossless storage
descriptor, resources, commit metadata, attempt semantics and maintenance policy.
Builders stay operation-specific; no second DML AST.

Derive local/durable checks from D02. Native casts perform mechanical conversion; declared
UInt64/decimal, fixed-binary, list-child, dictionary and extension mapping preserves
meaning that Delta's physical schema cannot encode directly.

Use one bounded exact-version action reader for receipts/recovery/post-commit observations.
Distinguish commit failure from committed data with failed metrics observation. Publication
selects exact members after obligations; retries reconcile complete request/transaction/
receipt evidence. One retention declaration combines age floors, selected/protected versions,
read leases, CDF intervals and unresolved attempts. Ordinary commits keep automatic log
cleanup off. Native keep-versions/Full-Lite vacuum/log cleanup uses the full effective set,
including maintenance fences/cache invalidation. Acceleration failure never replays a write.

### D11 — Shared configuration, diagnostics and generated boundaries

One effective resolver applies defaults, explicit overrides and limits, then semantic,
operational and inspection projections. Preserve typed native configuration and actual
caller-installed functions/rules/extensions. Generate Python conversion/validation/report
projections from declarations; do not copy property lists between layers.

One vocabulary declares failure classes and specific codes. Borrowed classification is
separate from owned/shared attachment. Preserve concrete causes and native External/Shared/
Context/Collection structure. No `Cow<DataFusionError>` or stringified-cause substitute.

One structural field traversal feeds distinct Rust/Python/JSON Schema/Markdown policies.
Use native JSON serialization. Keep useful static typing/authoring validation; reflection
alone does not replace them. Python uses native nested fields and exact comparisons inside
its four-state transfer classifier. Reports have named fields, durations a declared width/
unit, and IDs one cross-language text admission rule.

### D12 — Reconcile decisions as part of implementation

N00 records new crate boundaries, metadata/identity, resource and Python contracts in
proposed ADRs before their implementation. Allocate numbers with `just adr-new` and append
them to `adrs`; the empty list avoids invented identifiers. Update stale direction and
dependency/crate governance. Supersede accepted ADRs rather than rewrite their rationale.
Blueprint changes use the dedicated design route with revision rows/stable citations.
The reassessment's “Revise” verdict is not silently relabeled “Accept”: the corrected
target receives its own scoped acceptance evidence. This work reconciles governance to
the selected design; it does not justify compatibility code or request a new policy veto.
This planning edit changes no blueprint or ADR.

## Plan

### Packages and dependency order

Owners below are responsibility boundaries, not an instruction to spawn agents. Execute
one coherent sequence; coordinate independent edits if parallel work is later authorized.
The status column records current implementation; integrated acceptance remains N18 work.

| Package | Work | Requires | Status |
|---|---|---|---|
| N00 | Inventory, decisions, command support and test classification | This plan | Implemented; unit-tested |
| N01 | Confirmed arithmetic/Python defects | N00 | Implemented; unit-tested |
| N02 | Resolved contracts and admitted handles | N00 | Implemented; unit-tested |
| N03 | Native validation/obligation compiler | N02 | Implemented; unit-tested |
| N04 | Native values, canonical framing, generation/renderers | N02, N03 | Implemented; unit-tested |
| N05 | Diagnostic leaf and causal classification | N00 | Implemented; unit-tested |
| N06 | Engine extraction, effective settings, production/testkit assembly and native execution assurance | N02, N03, N05 | Implemented; isolated units and static checks |
| N07 | Operation families/providers/epochs/metrics | N06 | Implemented; isolated units and static checks |
| N08 | Complete function adapter delegation | N03, N06 | Implemented; isolated units and static checks |
| N09 | Scoped dependencies and reuse keys | N07, N08 | Implemented; isolated units/static checks |
| N10 | Native compiler correspondence/provenance | N04, N07, N09 | Implemented; isolated units/static checks |
| N11 | Shared graphs/operator capability bindings | N01, N02, N07 | Implemented; isolated units/static checks |
| N12 | Compact numerical/derivative preparation | N08, N11 | Implemented; isolated units/static checks |
| N13 | Native memory ownership/transfer | N04, N06, N07 | Implemented; isolated units/static checks |
| N14 | Delta operation/storage/read/retention consolidation | N03, N07, N09, N13 | Implemented; isolated units/static checks |
| N15 | Python settings/reports/IDs/transfer integration | N01, N04, N05, N06, N13, N14 | Implemented; isolated Rust/Python units/static checks |
| N16 | Remaining fixtures/callers/dependencies/docs | N01–N15 | Implemented, including grouped campaign-02 repairs |
| N17 | Complete implementation/deletion barrier | N00–N16 | Closed; current source receipt required before continuation |
| N18 | Final integration/measurements/acceptance | N17 closed | In progress: repaired shared engine and fixtures require requalification; remaining gates and review unexecuted |

Working order: N00 → N01 → N02–N05 → N06–N09 → N10–N14 → N15–N17 → N18.
N01 prevents known defects entering shared abstractions; N02–N04 remove semantic
duplication early. N05 can precede N03 where useful. N06 starts testkit support;
N16 completes fixture migration rather than postponing its foundation.

### N00 — Establish execution scope, decisions and command support

**Owns:** this plan; new `docs/plans/10-execution-inventory.md`; proposed decisions;
affected guidance/governance; `justfile`; `xtask/src/architecture_acceptance.rs`;
benchmark declarations. Read the corresponding protected-scope rules before editing.

1. Record starting source/lockfiles/overlay and preserve unrelated work. Refresh changed
   evidence only. Repair environment freshness through the applicable recipe without
   running integration at startup; a native extension build is not qualification.
2. Create one inventory covering every N package, R finding, accepted F disposition,
   deletion L01–L18 and acceptance A01–A16. Each row names source, consumers, generator,
   replacement, status and command/source-qualified evidence. Avoid a second status ledger.
3. Allocate D12's decisions and reconcile active guidance/crate restrictions. Keep old
   receipts historical; no silent attribution to new code or fabricated acceptance verdict.
4. Classify tests by behavior and register exact unit filters. Do not treat a whole
   library as unit-only. Python keeps explicit test categories. Compile integration
   targets if useful, but do not execute them before N18.
5. Extend `architecture-acceptance` to archive this plan, source including untracked
   inputs, dependency/overlay provenance, deletion inventory and A01–A16. Replace
   hard-coded Plan 09 receipt/copy labels while preserving exact-source and continuation
   semantics. Do not execute it yet.
6. Add `bench-consolidation` using the existing benchmark crate and N18's fixtures.
   Add deterministic structural count/growth checks as units; runtime benchmarks wait.

**Delete/update:** stale active-plan pointers and crate-count/import restrictions that
contradict the target. Historical plans/receipts remain documentation.

**Exit:** complete scope/ownership/oracle inventory, concrete decision records and
classified commands; no integration campaign started.

### N01 — Correct the demonstrated arithmetic and transfer defects

**Owns:** `pse-compiler/src/passes/p4/predicates/scalar.rs`, `python/pse/_transfer.py`
and isolated regressions. References R8/R10, review probes P1/P2.

1. Replace the unchecked integer remainder guard with checked arithmetic. Specify zero
   divisor, `MIN / -1`, overflow and nonintegral division behavior; preserve exact integer
   versus permitted real semantics. No panic/wrap/lossy result may claim exactness.
2. Let N11 reuse this kernel rather than introduce another evaluator. Keep fold eligibility
   independent of runtime evaluation support.
3. Replace struct/list-only Python traversal with native field enumeration and explicit
   dictionary-value/extension-storage handling. Use exact field comparisons inside the
   retained/storage-only/lost-metadata/mismatch classifier, retaining nested paths.
4. Cover map keys/items, union arms, list views, dictionaries, registered/unregistered
   extensions, names/types/nullability and the struct control. Unknown inspection must
   not silently produce a retained observation.

**Delete:** the unchecked `%` guard and superseded container dispatch.

**Unit exit:** regression tests call the real arithmetic helper and transfer classifier,
including P1/P2 cases; no store or complete compiler pipeline is needed.

### N02 — Resolve contracts and admit sealed handles

**Owns:** `pse-schema::{builder,compiled_contract,field_contract,model}`,
`pse-relations::columnar`, and generated expected-contract inputs.

1. Implement D01's descriptor/reference graph and metadata projections. Resolve enum
   domains, extension definitions/versions/parameters, keys, references and policies;
   detect missing/ambiguous definitions and bound cyclic comparison.
2. Replace runtime compiled-declaration strings with shared descriptors and registry-owned
   handle tables. Bind generated expected contracts once per actual owner; no global cache.
3. Carry sealed handles plus actual native batch/ownership evidence through construction,
   borrow, clone, concat, projection and import. Changed-schema projections require their
   own declared contract, not a copied parent handle. Keep evidence levels distinct.
4. Update generated adapters and all foreign-binding sites. A digest match still takes
   the exact admission path; a locally admitted handle takes the cheap path.
5. Separate documentation from execution identity through declared metadata roles while
   retaining exact full fields for storage/transfer observations.

**Delete:** batch declaration strings, repeated full-text borrow comparisons and old
generator constants once all adapters bind descriptors.

**Unit exit:** equivalent foreign registries admit exactly; copied IDs/digests with
changed enum/nullability/extension/check/reference fail. Cycles terminate. Foreign or
fabricated handles cannot bypass admission. Repeated local borrow reconstructs no text.

### N03 — Compile native local predicates and relational obligations

**Owns:** `pse-relations/src/validate` and new prepared-contract modules; schema
violation/obligation declarations; current catalog `session/capture/values.rs`,
`delta/predicates.rs` and `delta/nested_check.rs` consumers.

1. Lift existing native Delta field predicates into a non-Delta compiler accepting the
   resolved contract and actual preparation context. Compile once to native expressions;
   key prepared reuse by complete contract/implementation/relevant settings ownership.
2. Cover every currently declared field family: enum, range/finite, quantity/rational,
   tagged alternatives, collection, nested structure, dictionary and fixed-size IDs.
   Prefer built-ins; bind one reusable UDF/kernel per remaining semantic primitive.
3. Derive validity and structured violations together: stable rule, relation/key, nested
   occurrence path and observed value. Preserve visibility masks, cancellation, bounded
   reporting and typed errors. Arrow structural validation precedes semantic evaluation.
4. Extract key/reference/quantity/artifact obligations as native templates bound to actual
   relation identities. Preserve null/multiplicity/absence/four-valued rule semantics.
   Share requirement inputs and bounded stage barriers rather than one giant global plan.
5. Rewire local batch/column admission and session capture to native arrays. Expose common
   lowering hooks to N14; Delta serialization/builders stay in catalog. No per-batch full
   query planning and no second native-only validator with independent semantics.
6. Trace and replace/delete `validate_bundle`; remove unused alternate whole-model checks.

**Delete:** Cell-based local validation, duplicate semantic predicates, unused row-cell
bundles and redundant capture walks. N14 closes remaining Delta-specific duplication.

**Unit exit:** field-family matrix including hidden invalid values under null parents,
active-arm failures, dictionary domains, duplicate/null keys and absent references.
Local and storage predicate translations agree on violations/paths without durable writes.

### N04 — Replace Cell and consolidate value, identity and generator machinery

**Owns:** schema `model/cell.rs`, builder/self-description/compiled-contract/codegen;
relations cells/columnar; IDs frame/contract/canon; every literal/default consumer.

1. Implement one field-directed lossless native literal codec and scalar/column adapters.
   Rewire defaults, checked-value functions, registry construction and renderers. Preserve
   NaN payloads in literals separately from canonical hash normalization.
2. Build declared registry self-description directly with native Arrow builders in schema;
   do not depend on generated relations or introduce another universal dynamic row type.
3. Share canonical frame encoding, semantic metadata selection and native type traversal
   in IDs. Replace mirrored layout variants with checked descriptors; retain RowConverter
   and the chosen equivalence policy. Version any deliberate format change once.
4. Share generated typed builder/view primitives and borrowed accessors. Avoid a mandatory
   ScalarValue per cell, universal erased records, or a Serde round trip for bulk operations.
5. Use one field traversal with separate Rust/Python/JSON Schema/Markdown policies and
   native JSON serialization. Preserve typed authoring APIs, enums/newtypes and imports.
6. Rewire all Cell-based registry/hash/literal/default/self-description/fixture callers.
   Regenerate through generators and pure recipes before N17, never edit generated output.
7. Default to native columns. Use serde_arrow for a concrete row interchange only if a
   unit prototype proves IDs, nested metadata, null-parent, borrowing and simpler machinery.
   Otherwise remove it if unused. No speculative codec or proc-macro crate is required.

**Delete:** production Cell, conversion APIs, duplicated literal/framing implementations,
compiled-declaration text and obsolete generator/renderer walkers. Fix generators to
delete obsolete generated files. Preserve unrelated native types with similar names.

**Unit exit:** all-bit float literal round trips, canonical equivalence/metadata cases,
borrowed typed views, exact self-description, JSON escaping and Python projections.
Every active caller uses the native replacement; no old reader remains.

### N05 — Repurpose diagnostics and preserve native causes

**Owns:** `pse-diagnostics`, schema `catalog/enums_platform.rs`, catalog failure/error,
rules errmap and common classification/rendering callers.

1. Remove the empty diagnostic crate's relations/numerics/structural dependency edges.
   Declare classes and detailed codes at the leaf; generate/project registry and language
   vocabularies from it. Do not force useful detailed codes to become coarse classes.
2. Implement one borrowed native classifier with separate owned/shared attachment.
   Preserve concrete domain causes, Context/External/Shared/Collection structure and
   meaningful ordering/multiplicity of failures.
3. Replace repeated common classification/string maps. Keep legitimate domain error
   variants/context. Engine and catalog retain different error responsibilities.
4. Provide the engine-facing error boundary without a catalog dependency or reverse
   schema/ID edge. Do not simulate cloning with stringification or Cow.

**Delete:** duplicate common classifiers/vocabularies and unused high-level dependencies.

**Unit exit:** borrowed/owned/shared paths agree, causes remain downcastable, collection
handling is explicit, registry/Python projections share one vocabulary, dependency DAG holds.

### N06 — Extract engine services, production assembly and testkit foundation

**Owns:** new `pse-engine`/`pse-testkit`; generic catalog session/provider/cache mechanics;
runtime assembly; compiler context; manifests and crate-governance declarations.

1. Establish D04's acyclic boundaries. Move generic binding/admission/preparation/output,
   observations, function registration, policy interfaces and planner composition into
   engine. Keep Delta opening, publication selection, leases and snapshots in catalog.
2. Split the bound engine session from catalog composition. Native providers/streams
   retain actual owners/guards supplied by catalog; no Delta signature or name-only handle
   leaks into engine. Preserve caller-installed functions/rules/extensions.
3. Narrow AlgorithmContext and migrate rules/compiler/domain/backend execution callers.
   Keep legitimate durable orchestration in catalog-facing code. Do not require a storage
   crate just to obtain a stream type, native execution context or generic error.
4. Move generic bounded cache/flight/accounting mechanics as useful, leaving Delta-specific
   keys and freshness adapters in catalog. Preserve deployment-wide pool/cache sharing.
5. Implement the effective settings resolver and its semantic/operational/inspection
   projections. Preserve implementation generations, actual native builders and limit
   re-admission; remove conflicting independently maintained construction paths.
6. Create reusable production factories and dev-only testkit foundation. Keep features
   separated so foundation units need no storage/solver fixture or cyclic dev dependency.
7. Update all initial imports/exports/manifests directly; no compatibility re-exports.
8. Implement the authorized execution-assurance extension: native phase/operator/store
   instrumentation, explicit invocation terminal states, async/blocking context propagation,
   bounded local capture and direct native plan/metric inspection. Keep freshness and
   validation independent of observation; missing evidence is inconclusive. Preserve
   semantic and durable functional oracles. See the
   [assurance guide](../dev/native-execution-assurance.md).

**Implemented:** `pse-engine` now owns generic sessions, providers, caches, configuration,
errors and the production resource constructor. Catalog supplies typed durable selections
through opaque source witnesses, actual retained owners and explicit extension planners.
Runtime and dev-only `pse-testkit` use the engine constructor. All active callers use
`EngineFactory`/`EngineSession`; the old catalog modules, runtime reserve module and
shared duplicate test factory were deleted. No compatibility re-exports remain.

**Tested:** `just dev-native-engine`, 17 passed, 0 failed; `just dev-native-boundaries`,
9 passed, 0 failed. Both use the pinned toolchain, default nextest profile and explicit
Arrow force-validation, baseline zero. **Interface-checked:** workspace all-target
Clippy with force-validation and `-D warnings`, native family/DAG/deletion checks and
format/ADR checks. The [N06 receipt](../design_review/evidence/native-engine-implementation-2026-09-18.json)
and [inventory](10-execution-inventory.md) retain precise scope and commands. These do
not establish N18 integrated acceptance or close N07/N16's broader consolidation.

**Delete:** moved generic catalog modules, duplicate setup and obsolete dependency edges.

**Unit exit:** engine has no catalog/Delta edge; production has no testkit normal/build
edge; construction shares one runtime/pool/cache owner; native custom assemblies survive;
algorithm-context tests need no Delta root. Moving files alone is not completion.

### N07 — Consolidate operation families, providers and metrics

**Owns:** engine logical/physical/planner families; compiler/backend extension shells;
current session contract/cache/round/candidate/materialized consumers; Delta wrappers.

1. Classify every custom node into D05's families or replace it with ordinary native
   operators. Bind actual implementation identity, ports, consumption, effects, reset,
   cardinality and failure semantics. Domain algorithms remain typed implementations.
2. Share reconstruction/display/planner dispatch/stream mechanics where meanings match.
   Child-expression transfer has correct arity. Projection, limit and empty outputs
   cannot erase a required validation branch, lease or durable effect contract.
3. Consolidate immutable memory/stream sources with `MemorySourceConfig`, `StreamingTable`,
   `PartitionStream` and `RecordBatchStreamAdapter` as appropriate. Candidate sources
   expose no unproved uniqueness; selected sources advertise only established properties.
4. Preserve multi-reader immutable epochs, replacement exclusion and prepared-stratum
   reset. Use native transport without pretending WorkTable is the same lifetime contract.
5. Share native metrics and modern `child_stats_requests`/`statistics_from_inputs`
   delegation on transparent wrappers. Preserve existing cache/resident instrumentation;
   add useful plan/execution/row/error/cancellation counters without false exact estimates.
6. Specify shared-producer and command cancellation/settlement. Planning and EXPLAIN are
   effect-free; catalog supplies durable reconciliation, not the generic execution shell.

**Delete:** superseded node/planner/provider shells and stream boilerplate. Retained
specialized methods must have a declared semantic reason, not an old API obligation.

**Unit exit:** per-family reconstruction/reset/property matrix, no planning effects,
one shared producer execution, epoch lifetime, requirement retention and native metrics.
Use fake commands/streams for units; durable fault journeys wait for N18.

### N08 — Delegate complete native function capabilities

**Owns:** engine adapters moved from catalog scalar/aggregate/field-transfer/physical-field
modules and their registry/function-binding consumers.

1. Audit all required/provided methods of actual scalar/aggregate/window/higher-order
   adapters. Each hook is delegated, adapted with a field postcondition, or deliberately
   conservative with a concrete reason. Generate mechanical delegation where appropriate.
2. Restore compatible min/max `is_descending`, `value_from_stats`, `set_monotonicity`,
   beneficial ordering and simplification. Cover grouped/sliding/window/reversal,
   state fields, defaults and null behavior for every relevant adapter.
3. Share coercion, nested field/element/concat and return-field machinery. Rewrap native
   rewritten functions and re-establish extension/quantity/enum fields after rewriting.
4. Preserve scalar/array fast paths, volatility and native semantics. Ordinary Rust uses
   shared ID/math kernels directly; UDFs adapt those kernels to native execution.
5. Bind adapter identity to actual implementation and field policy, never name alone.

**Delete:** incomplete superseded wrappers and common handwritten delegation copies.
Use native built-ins directly where the wrapper has no remaining semantic purpose.

**Unit exit:** native-versus-adapted behavior/fields, ordered aggregate and statistics
substitution, empty/all-null groups, simplification/reversal and scalar/array paths.

### N09 — Share scoped traversal and precise dependency/reuse logic

**Owns:** engine traversal/preparation/config/function binding; catalog artifact consumption,
cache admission and source adapters; rule SQL parsing/binding helpers.

1. Share bounded traversal mechanics with separate contexts for rewriting, semantic
   evidence and observation. Keep correlation, recursive worktable, region, effect ancestry,
   invocation/epoch and actual ownership in relevant memo keys.
2. Implement per-family semantic demand: values plus requirements, row presence,
   multiplicity, keys, meaningful order and optional absence. Expose hidden cache producers
   to evidence while preserving optimizer leaves. Unknown extensions remain whole-input.
3. Separate syntax reuse from bound plan reuse. Syntax keys include SQL/dialect/parser
   options; preparation includes actual provider/function/rule/contract/settings owners.
   Cancellation or failed population cannot expose partial entries.
4. Centralize explicit settings classification/read-back. Unknown extensions remain semantic;
   operational ceilings remain enforced/reported and re-admitted on cache hits. Retain
   exact source/version freshness and maintenance invalidation adapters in catalog.
5. Bound shared-producer memoization and deep/recursive walks with cancellation and resources.

**Delete:** duplicated scope-blind walkers, repeated parsing at reusable syntax boundaries,
unsafe registry-only keys and parallel settings classification tables.

**Unit exit:** requirement-only/negative fact changes invalidate; scopes cannot collide;
changed native provider/UDF prevents stale reuse; shared traversal stays bounded; failed
population and changed budgets preserve admission. No durable incremental campaign yet.

### N10 — Keep compiler correspondence and provenance relational

**Owns:** compiler P7 inventory/evidence/paths and consumers; P8 axes/contexts/connections;
P9 selection/binding; template path resolution; P6 shared capture/requirements.

1. Inventory decoded scans by semantic operation and consumer. Replace matching/grouping/
   existence/difference with native joins, aggregates, anti-joins, projections and UNNEST.
   Do not call every `.filter` a join or rewrite a genuine bounded graph algorithm as SQL.
2. Carry stable relation/source/support keys alongside values, including negative evidence.
   Remove unzip-and-position-search correspondence. Explicitly define null equality,
   optional absence, multiplicity, deterministic tie/order and duplicate behavior.
3. Use native qualified Columns/LogicalPlanBuilder and generated field references in a
   small shared construction module. Avoid another alias grammar or parallel plan AST.
4. Share input capture/materialization at necessary algorithm/requirement boundaries.
   Preserve intentional reuse that bounds planning growth and repeated scans; do not
   replace it with a large expression tree or prescribe one query for a whole pass.
5. Start with P7 correspondence/provenance, then P8 domain/connection assembly, P9 selection
   and templates. Reuse P6's native joins; consolidate its capture/barrier assembly.
6. Expose admitted Arrow batches plus bounded indexes only to specialized remaining
   algorithms. Record why each retained row/index materialization is necessary.

**Delete:** replaced inventories/row joins/source-position recovery, redundant alias helpers,
duplicate captures and provenance scans; preserve meaningful graph/numerical indexes.

**Unit exit:** in-memory relation/operator cases cover reorder, duplicates, missing/null
matches and exact support keys. Count captures/producers/decoded rows structurally.
Compile pass targets; complete P0–P10/Delta journeys wait for N18.

### N11 — Share graph views and bind operator capabilities

**Owns:** mathir topo/walk/canonicalize/fold, schema math operator declarations,
compiler P4/P8/P10 graph consumers, quantity arithmetic and native kernel bindings.

1. Declare argument, payload-reference, guard and binding edges with explicit traversal
   purposes. Derive one indexed view from admitted relations; retain stable row/node keys.
   Use binding overlays instead of cloning the entire graph for each traversal.
2. Consolidate reachability, postorder/cycle witness and bounded traversal mechanics;
   preserve stable ordering and region-sensitive canonicalization. Cover payload-only
   references with a concrete P8 witness before concluding which old walker was wrong.
3. Bind evaluate/fold/lower/differentiate/quantity capabilities to implementations, not
   descriptive strings alone. Keep declared non-foldability, including Pow, meaningful.
4. Reuse checked N01 integer kernels and shared unit conversion arithmetic. Native
   built-ins implement matching semantics; explicit kernels/UDFs implement missing ones.
   Preserve finite/domain guards, overflow, signed zero and intended precision.
5. Evaluate petgraph only against this bounded replacement: choose it if it removes
   machinery while meeting traversal/witness semantics. Otherwise finish the shared view;
   no second persisted graph or runtime selector is permitted.

**Delete:** whole-graph binding clones, duplicated same-purpose walkers/evaluators and
parallel operator capability dispatch that the shared binding replaces.

**Unit exit:** payload-only dependency/cycle cases, ordered traversal, excluded unsafe
branch, exact arithmetic extrema, fold eligibility and differential unit-conversion bits.
Do not route the only arithmetic oracle through the production implementation.

### N12 — Preserve shared numerical and derivative computation

**Owns:** `pse-numerics/src/{scalar_math,differentiate,expressions}.rs`, numerical
program construction and existing backend callback consumers.

1. Lower admitted math graphs into an indexed value/derivative DAG, retaining node/region
   identity, operator bindings and sparse derivative structure. Share repeated values and
   derivatives instead of recursively cloning Expr subtrees.
2. Emit native projection stages/reusable physical expressions referencing shared column
   slots. Restrict sharing by guard/evaluation region and volatility. Keep branch-local
   invalid arithmetic unevaluated outside its selected branch.
3. Inspect logical optimization and physical lowering for renewed inlining. If a native
   rewrite expands shared expressions, use an explicit shared stage/materialization in
   the native program; do not ship the expanded path as a fallback.
4. Prepare once for actual schema/function/settings owners. Bind changing variables as
   Arrow batches and evaluate prepared stages in callbacks. Preserve Jacobian ordering,
   sparse bindings, cancellation and error propagation.
5. Reuse existing independently callable solve/evaluation components; do not add a new
   product solve workflow to demonstrate this refactor.

**Delete:** recursive value/gradient tree duplication, independent preparation of identical
   subexpressions and any superseded callback evaluator after all callers are rewired.

**Unit exit:** repeated-diamond graph growth is proportional to actual admitted DAG/stages
   rather than path expansion; guarded failures and volatile cases retain semantics;
   derivatives match an independent analytic/numerical oracle. Solver journeys wait.

Use increasing repeated-diamond depths (for example 8, 16, 32 and 64), shared multiple
outputs and a guarded invalid branch. Count distinct value/derivative nodes, emitted
expression nodes and prepared stages, separating unavoidable output/Jacobian size from
duplicate expansion. These structural units do not require a benchmark or solver run.

### N13 — Consolidate native memory ownership and accounting

**Owns:** IDs resource/owned-buffer/validation-extent modules, runtime reserve/assembly,
engine export/materialization/cache paths, FFI ownership and memory-aware kernels.

1. Replace reservation trait consumers with native MemoryConsumer/MemoryReservation.
   Use a minimal shared native reservation owner for allocations retained by multiple
   buffers/readers/FFI exports. Production and tests use native pools, including small limits.
2. Inventory actual allocation identity, shared/sliced buffers and foreign ownership.
   Fallibly reserve before owned allocation; account new copies and deduplicate shared
   backing storage. Do not use a nested slice estimate as retained capacity.
3. Prototype an allocation-aware Arrow-claim transfer using pre-reserved native capacity.
   Prove atomic/no-gap/no-double-charge behavior, shared ownership and final release.
   Enable the complete feature envelope only for safe adopted paths. Otherwise implement
   that boundary with the native owner; record the selected mechanism, not a deferred TODO.
4. Never re-claim a shared allocation against an unrelated invocation pool. Keep deployment
   budget ownership distinct from attribution; imported/exported owners retain accounting
   until their actual final release even after the producing task is cancelled.
5. Remove Cell-decoding forecasts and redundant scratch reservations as consumers vanish.
   Keep justified retained/visible/copy estimates separately named and documented. Do not
   claim native accounting equals process RSS or intercepts every foreign allocation.
6. Rewire caches, canonicalization, generated construction and Python export callers;
   remove old constructors/re-exports rather than wrapping the old trait with native names.

**Delete:** MemoryReserver/custom Reservation implementations, PoolReserver, fixed custom
   test budget and obsolete decode forecasts. Retain only native-backed ownership adapters.

**Unit exit:** failed admission, shared columns/dictionaries/slices, repeated/concurrent
   claims, copy/export, cancellation and final-owner release. Demonstrate one budget and
   no missed/double charge for the represented allocations; no blanket RSS guarantee.

### N14 — Consolidate Delta contracts, operations and retention

**Owns:** catalog Delta contract/predicates/nested-check/admission/DML/receipt/recovery/
maintenance modules; schema Delta mappings; cache/read-lease integration.

1. Implement D10's operation context while preserving per-builder semantics and actual
   caller session/runtime. Share commit metadata, resources, operation reporting and
   attempt handling; native write/update/delete/merge/optimize/vacuum remain the engine.
2. Centralize execution↔storage mapping and native casts from the resolved contract.
   Preserve UInt64/decimal, fixed binary, dictionary/list-child and extension restoration;
   explicitly refuse unsupported lossless mappings rather than silently erase meaning.
3. Use N03's shared predicate/obligation lowering for local/durable admission. Generate
   native CHECK/invariant declarations and the nested-predicate adapter where Delta SQL
   cannot express it. Invoke public operations; never call private validation helpers.
4. Replace unbounded commit reads, including `dml/execution.rs::write_count`, with one
   bounded exact-version action reader or native operation metrics when sufficient.
   Reuse it for receipts/recovery; preserve committed-but-observation-failed outcomes.
5. Consolidate exact request/transaction/receipt settlement and publication selection.
   Preserve cancellation ambiguity and maintenance/checksum outcome separation. A native
   SetTransaction action never substitutes for exact replay reconciliation at this pin.
6. Derive retention from ages, protected selections, readers, CDF and unresolved attempts.
   Keep automatic cleanup off ordinary commits. Configure native keep-versions/Full-Lite
   vacuum/log cleanup, maintenance fences and cache invalidation from one effective policy.
7. Preserve the selected overlay and existing file/snapshot/resident/CDF cache ownership.
   Any necessary upstream change uses the reproducible patch/source procedure and refreshed
   provenance; do not edit vendor output or rebuild Delta transactions locally.

**Delete:** independent Delta semantic validators, duplicate builder policy assembly,
   unbounded action readers, repeated mapping/retention decisions and stale operation APIs.
   Publication/attempt/lease semantics remain explicit target responsibilities.

**Unit exit:** execution/storage schema and predicate mapping; bounded reader edge cases
   using fake stores; policy builders and interruption-state transitions; shared retention
   protection matrix. Actual commit/reopen/vacuum/fault campaigns wait for N18.

### N15 — Unify Python settings, reports and native transfer boundaries

**Owns:** `pse-py/src/inspection/{settings,cache_settings,handles,cache_report,errors,stream}`,
Python inspection/codec/transfer code, schema Python generators and native stub generation.

1. Generate thin settings projection and constructor validation from the effective policy
   declaration. Keep conversion in the boundary crate; do not attach PyO3 to core types
   merely to avoid writing an adapter. `get_all` does not replace construction validation.
2. Replace positional resource/report tuples with named immutable records and exact stubs.
   Normalize duration representation, units/ranges and overflow behavior in Rust/Python.
   Keep semantic configuration distinct from operational read-back.
3. Use one authoritative ID text codec/admission rule; generated Python APIs call or
   faithfully derive that rule rather than independently implementing hex normalization.
4. Integrate N01's complete transfer classifier with exact full field contracts and N14's
   storage restoration. Preserve useful typed authoring classes and the four transfer states.
5. Rewire Arrow C-stream/export ownership to N13's native reservation owner and structured
   diagnostic causes to N05. Regenerate Python contracts and compiled API stubs through
   generators; update all user-facing consumers and examples.

**Delete:** independent settings property/validation lists, positional report unpacking,
   duplicate ID/duration conversion and old exported constructors/imports. No Python alias
   compatibility layer for replaced APIs.

**Unit exit:** constructor ranges, names/stubs, ID text, duration overflow, native field
   transfer and isolated C-stream owner release. Fresh-publication Rust/Python inspection
   remains N18 work, even if an old extension can already import successfully.

### N16 — Finish shared fixtures, unused code and documentation

**Owns:** test support across crates/tests/benchmarks, `pse-testkit`, manifests,
rule-source ownership, active guidance/docs/examples and all residual consumers.

1. Move repeated session/runtime/root/fault-store/assertion setup into testkit modules
   that call the production factory. Keep pure Arrow fixtures independent of storage and
   solver features. Reuse shared settings/resources without accidentally sharing state
   between tests. Retain independent scientific expected results.
2. Finish every source/caller/generator/import/export change in N01–N15. Search production,
   test, benchmark, example and Python surfaces; a path is not deleted when its last
   production caller disappears but its alternate implementation remains available.
3. Inspect rule SQL ownership. Keep readable authored SQL as the default; where repetition
   truly comes from a shared declaration, generate it through one real generator. Do not
   infer a missing generator from formatting or cache bound SQL plans by registry alone.
4. Review Cargo metadata, imports and actual reachability. Remove unused direct/transitive
   dependency declarations and dead entrypoints. Retain meaningful independently exercised
   solver/numerical kernels; label their reachability honestly. No new simulator caller is
   required just to keep or qualify those kernels.
5. Complete all fixture/generator updates, docs/book indexes, examples, decision references,
   agent direction and explicit specialized-code inventory. Remove stale restriction text
   and predecessor APIs from active guidance; historical evidence is not rewritten.
6. Prepare all integration/fault/performance cases now, compiling their targets if useful.
   Their execution remains blocked until N17. Do not leave test implementation for after
   the deletion barrier and call the code scope complete.

**Delete:** duplicate fixture frameworks, unused dependencies, abandoned callable surfaces,
   superseded SQL assembly and remaining old caller paths. Preserve useful algorithms and
   historical documents, not predecessor runtime objects.

**Exit:** each remaining bespoke kernel/adapter has an owner, functional purpose, native
   entrypoint and reason that built-ins alone do not replace it. Every old active path in
   the deletion ledger is removed or identified as a selected target mechanism with evidence.

### N17 — Close the complete implementation and deletion inventory

**Owns:** N00 execution inventory, deletion checker, acceptance harness and plan status.

1. Require N00–N16 code, all callers/declarations/generated output, fixture changes and
   L01–L18 deletions complete. No missing mechanism may be recategorized as qualification
   merely to cross this barrier. No target mechanism is complete only because it compiles.
2. Check every R/F/review-section obligation against implemented source and consumers.
   Classify generated repetition, useful specialized code and historical references
   separately from executable legacy. A textual search alone is not semantic closure.
3. Verify dependency boundaries and native feature/source provenance. Run compile-only,
   pure generation, lint and selected unit checks as needed; keep integration unexecuted.
4. Record unresolved behavioral proof as N18 tests, while allowing no unresolved target
   implementation/deletion decision. Resolve the bounded choices table before closure.
5. Record a dated barrier receipt with exact revision plus dirty-source hashes, status of
   all ledger rows, executed unit commands/modes/failure counts and deferred N18 commands.
   Extend the acceptance runner to refuse an incomplete inventory. No human confirmation
   prompt or arbitrary timeout serves as the barrier.

**Exit:** **Implemented**, with source evidence, for the entire target and its deletions.
   Behavioral acceptance remains unclaimed. N18 is now permitted to start.

### N18 — Qualify the complete target and record the outcome

**Owns:** existing architecture campaign extended for Plan 10, A01–A16, final
measurements, acceptance review, execution inventory and this plan's outcome.

1. Run `just architecture-acceptance <new-evidence-directory>` after N17, using the
   extended runner and existing checked-in profiles/timeouts. It owns the required Rust,
   Python, solver, feature, generator, quality and documentation commands; do not launch
   a duplicate full suite beside it.
2. Execute the targeted cross-boundary/fault cases in A01–A16 and the measurement matrix
   below. Preserve current process-model assertions, including quantity, kernel outputs,
   selected publication members and independent Rust/Python reopen of fresh stores.
3. Separate cold construction/planning/optimization/preparation from execution/callback
   cost, cache hits and durable IO. Record counts, allocations/pool peaks and RSS separately.
   A native type name or fewer lines is not measured improvement.
4. Fix observed regressions within scope. If a fix reopens an implementation/deletion
   obligation, close that inventory row again before more integration. Use focused
   regression checks and resume remaining gates; do not repeat passed broad campaigns
   without an affected guarantee. Never raise timeout/retry allowances just for green.
5. Preserve exact source boundaries for every receipt, including failure and continuation.
   An aggregate acceptance argument lists test identities/coverage and unaffected evidence;
   it is not falsely described as one untouched full-suite run on the final revision.
6. Independently assess A01–A16 and G1–G7 against the corrected implemented design. Finish
   decision/design reconciliation without relabeling an older review. Update plan/inventory
   and record what was built, a mistake corrected, and deliberate deviations.

**Exit:** every scoped target/deletion/acceptance obligation closed, zero failures against
   zero baseline for required final checks, measurements and limitations recorded. No new
   numerical IDAES parity, remote coordination or universal speedup claim by implication.

## Traceability

### Every independent finding

| Review finding | Selected realization | Packages | Acceptance |
|---|---|---|---|
| R1 — exact resolved equality | Sealed admitted handles, complete graph comparison and metadata roles | N02, N04 | A01, A03 |
| R2 — duplicate value validation | Shared native field/obligation compiler; delete Cell validators | N03, N04, N14 | A02, A03, A11 |
| R3 — extension dependencies | Family-specific transfer and scoped semantic evidence | N07, N09 | A05, A07 |
| R4 — lost native adapter capabilities | Provided-method conformance and full field-aware delegation | N08 | A06 |
| R5 — claims/accounting assumptions | Native reservations, allocation-aware transfer/lifetime boundary | N13 | A10 |
| R6 — decoded correspondence | Native compiler/template plans and stable support keys | N10 | A08 |
| R7 — numerical graph expansion | Region-aware shared value/derivative stages | N11, N12 | A09, A15 |
| R8 — arithmetic/edge roles | Guard fix, distinct operator capabilities and typed graph views | N01, N11, N12 | A04, A09 |
| R9 — Delta obligations/retention | Shared lowering, operation context, bounded reads and effective retention | N03, N14 | A11, A12 |
| R10 — Python nested transfer | Native nested enumeration plus exact extension-aware observations | N01, N15 | A04, A13 |
| R11 — semantic reuse keys | Scope-aware owners, syntax/bound-plan separation and explicit settings | N06, N09 | A07, A13 |
| R12 — unnecessary platforms | Two focused crates, diagnostic leaf, shared generators/factories/testkit | N04–N08, N16 | A03, A14, A16 |

### Every disposition of the initial review

| Initial finding | Implemented direction required by this plan | Packages |
|---|---|---|
| F1 — Cell/literals | Native columns and one lossless codec; preserve distinct semantic hash policy | N03, N04 |
| F2 — declaration strings | Exact complete admission then handles; reject fingerprint-only shortcut | N02, N04 |
| F3 — memory | Native reservations plus tested ownership bridge; no naive try-grow/claim transfer | N13 |
| F4 — framing/layout/order | Shared frame kernel and checked native descriptors; retain suitable RowConverter | N04 |
| F5 — extension triples/metrics | Explicit operation families; preserve and complete existing metrics | N07 |
| F6 — function wrappers | Full native hook delegation, rewrapping and field postconditions | N08 |
| F7 — tables | Native memory/stream adapters with truthful properties and epoch owner | N07 |
| F8 — traversal/settings/caches | Scoped evidence, correct child arity, effective settings and bounded reuse | N06, N09 |
| F9 — relational passes | Native correspondence/provenance; specialized indexed work remains explicit | N10 |
| F10 — graphs | Shared typed edge views, cycle witnesses and no whole-graph binding clones | N11 |
| F11 — operators | Separate capabilities, shared exact kernels, guarded native numerical lowering | N01, N11, N12 |
| F12 — Delta | Cohesive module, public builders, shared validation and protected retention | N14 |
| F13 — diagnostics | Leaf class/code declaration, borrowed classification and preserved causes | N05 |
| F14 — renderers | Shared traversal, native serialization and useful generated typed APIs | N04, N15 |
| F15 — fixtures | Dev-only testkit using production factories | N06, N16 |
| F16 — Python | Correct nested transfer, named reports, settings/duration/ID conversion | N01, N15 |
| F17 — reachability/dependencies/SQL | Remove proved dead paths; retain useful kernels; readable SQL and correctly keyed syntax reuse | N09, N16 |

### Supporting recommendations and review journeys

| Review scope | Explicit coverage |
|---|---|
| §§2–3 authority/lifecycle/evidence levels | D01–D04, D09–D11; N02/N03/N06/N13/N14; A01/A02/A10–A14 |
| §4.1 shared compiler | N03/N14, including visible occurrence masks and structured violations |
| §4.2 handles/generation/values | N02/N04, including transitive definitions and separate literal/hash equivalence |
| §4.3 ownership/crates | N05/N06/N16, including actual AlgorithmContext and EngineSession split |
| §4.4 execution/providers/adapters | N07/N08; no WorkTable substitution or uniform effect semantics |
| §§4.5–4.6 relational/graph/numerics | N10–N12; stable support keys, region-aware sparse derivative sharing |
| §§4.7–4.8 memory/Delta | N13/N14; accounting versus RSS, durable observation failure, lease-aware cleanup |
| §4.9 reuse/settings/diagnostics/Python | N05/N06/N09/N15/N16 |
| §5 ordinary nested-constraint extension | A02: one declaration reaches local checks, Delta and generated interfaces |
| §5 changed enum with copied digest | A01: complete foreign admission rejects it |
| §5 nested Arrow–Delta–Python transfer | A11/A13: exact round trip or explicit unsupported lossless mapping |
| §5 interrupted publication | A12: exact recovery and protected maintenance |
| §5 payload-only graph reference | A09: correct closure, cycle witness and guarded evaluation |
| §§6–7 gates/findings | Independent closure in N18; no composite score substitutes for an unresolved gate |
| §§8–10 alternatives/verification/open choices | Selected defaults and bounded choices below; A01–A16 and measurements |
| §11 dependency order/deletions | N00–N18 and L01–L18; N17 before integration |

## Deletion and consolidation ledger

Paths here identify responsibilities, not permission to delete a file containing useful
unreplaced code. The execution inventory enumerates exact symbols/callers at cutover.
Every row needs replacement plus deletion evidence; a renamed wrapper is not closure.

| ID | Remove / consolidate | Target owner / replacement | Closes in |
|---|---|---|---|
| L01 | Unchecked P4 division remainder and incomplete Python child dispatch | Checked arithmetic and native transfer traversal | N01 |
| L02 | Compiled-declaration strings, repeated text equality, generated declaration constants | Schema resolved handles and exact one-time admission | N02/N04 |
| L03 | `Cell`, cells/conversion APIs and all active literal/default/self-description uses | Arrow columns and shared lossless literal codec | N04 |
| L04 | Duplicate local field validators, capture PSE walkers and unused bundle validator | Prepared native predicate/obligation compiler | N03/N14 |
| L05 | Mirrored canonical layout and repeated frame/metadata policy code | Native type descriptor and shared IDs kernel | N04 |
| L06 | Repeated generator structural walkers and handwritten JSON escaping | Shared traversal and native serialization | N04 |
| L07 | Generic catalog execution services, old imports/re-exports and duplicate factories | Engine plus explicit catalog composition | N06/N16 |
| L08 | Duplicate extension/planner/physical/provider/stream shells | Native operators and explicit shared families | N07 |
| L09 | Incomplete/superseded field-preserving function wrappers | Conformance-driven native adapters | N08 |
| L10 | Scope-blind walkers, unsafe reuse keys and repeated settings classification | Scoped traversal and owned semantic keys | N09 |
| L11 | Decoded relational joins, source-position scans and redundant captures | Native correspondence/support plans | N10 |
| L12 | Binding graph clones, same-purpose traversal copies and duplicate operator dispatch | Shared typed graph views and capability bindings | N11 |
| L13 | Recursive Expr/gradient expansion and duplicate prepared subexpressions | Staged native numerical program | N12 |
| L14 | MemoryReserver/custom reservations/PoolReserver/custom fixed budget and dead forecasts | Native reservations/pools plus narrow lifetime adapters | N13 |
| L15 | Repeated Delta semantic predicates, mapping/builder/retention policies and unbounded reads | Shared contracts, operation context and exact bounded reader | N14 |
| L16 | Duplicate diagnostic vocabularies/classifiers and unused leaf dependencies | Repurposed diagnostic leaf preserving causes | N05 |
| L17 | Repeated Python settings/ID/duration conversion, positional reports and obsolete API aliases | Generated thin boundaries and named reports | N15 |
| L18 | Repeated fixture frameworks, proved unused dependencies/entrypoints and superseded SQL assembly | Testkit, actual production factories and owned rule source | N16 |

Retain explicitly: native provider/plan/cache APIs; publication/attempt/reader semantics;
canonical semantic kernels; quantity/graph/numerical algorithms with distinctive purpose;
native-backed lifetime owners; typed generated surfaces with consumers. No generic
“legacy exception” row is permitted. The final specialized-code inventory explains each
retained implementation and its native integration point.

## Verification

**Evidence vocabulary:** target code is **Proposed** until present, **Implemented**
only with source/caller/deletion evidence, **Interface-checked** for verified native
APIs, **Tested** only for named executed cases/conditions, and **Measured** only for
recorded observations. No formal proof or performance gain is assumed by this plan.

### Command discipline before and after the barrier

| Stage | Commands / behavior | Limits |
|---|---|---|
| Before N17 | `just check-library <package>`, `just check-package <package>`, `just check-test <package> <target>` | Compile only. Creating/rebuilding an extension is not permission to run inspection fixtures. |
| Before N17 | `just unit-package <package> '<reviewed-filter>'` | Recipe supplies `--features pse-relations/force-validate`; explicitly include the intended package and unit namespace. Inspect selected cases; no all-lib shortcut. |
| Before N17 | `just py-unit <unit-selection>` | No compiler/publication fixture, storage journey or solver work through a fixture. New categories remain explicit. |
| Before N17 | `just codegen-contracts`, format/lint/type checks, `just family-check`, `just docs` | Pure generation/static checks only. Full codegen/check recipes that execute physical package fixtures wait for N18. |
| N18 only | `just architecture-acceptance <new-directory>` | Extended runner includes final Rust tests, doctests, linked solver tests, editable extension, Python/quality, generated equality, features, docs and engineering inspection. |
| N18 only | `just bench-consolidation` and existing `just bench-cache`, owned by the campaign | Measure current target with recorded hardware/settings/source and cold/warm conditions. No implied repeated full benchmark suites. |

N00 introduces focused unit namespaces such as `consolidation_unit::` and records
the exact implemented filters. Example **Proposed**, not an existing test receipt:

```bash
just unit-package pse-schema 'package(pse-schema) and test(consolidation_unit::)'
```

Use the existing pinned toolchain/profile/cache; do not force fresh build directories,
disable caching or serialize all builds by default. Do not run `cargo clean` as routine
verification. Re-run tests after meaningful changes/failures, not merely to accumulate
green outputs. The final campaign may resume at remaining checks after scoped repairs;
preserve exact source and coverage rather than imply skipped gates ran.

### Acceptance obligations

All rows are **Proposed verification**. Units are implemented with their package and
run before N17; the listed integrated journeys execute only in N18. “Every declared
family” is an inventory generated from actual declarations, not a handwritten subset.

| ID | Claim / required oracle | Owner and final evidence |
|---|---|---|
| A01 | Complete foreign contract admission rejects copied IDs/digests with altered enum/extension/nullability/key/reference/check/policy; equivalent independent owners admit; cyclic references terminate; local handle checks avoid text rebuilding. | N02; adversarial units plus generated/imported adapter boundary cases. |
| A02 | One new nested constraint reaches local admission, native rule output, durable enforcement and publication obligations with consistent decisions/paths. Include hidden children, active alternatives, dictionaries, ranges, quantities, keys and absent references. Arrow safety precedes semantics. | N03/N14; generated field-family matrix and final cross-boundary journey. |
| A03 | No production Cell or declaration-text authority; lossless literals preserve float bits; canonical equivalence and metadata roles are deliberate; self-description and typed/generated fields are exact. No incompatible extra files survive regeneration. | N02/N04; native codec/hash/generation units and final `codegen-check`/consumer checks. |
| A04 | Real P4 arithmetic helper handles integer extrema/zero/nonintegral division without panic or false exactness; actual Python transfer classifier detects map-child changes and preserves its four states. | N01; direct regression cases, broader nested type matrix and N15 consumer integration. |
| A05 | Node families preserve effects/requirements/lease lifetime under rewrite, projection, limit, empty results, reconstruction, reset and cancellation. Epoch replacement respects active readers; shared producer executes once; statistics/metrics stay truthful. | N07; fake-stream/command units and final native pipeline lifecycle cases. |
| A06 | Adapters retain applicable native scalar/array, coercion, field, ordering, simplification, group/window/reversal/statistics/monotonicity hooks and actual identity. | N08; hook conformance matrix, native versus adapted physical plans/results and metrics. |
| A07 | Requirements, absence, membership, multiplicity and source keys remain dependencies. Correlated/recursive/hidden-producer scopes cannot collide. Changed provider/UDF/settings/epoch invalidates; resource-policy changes still re-admit; unknown extensions remain conservative. | N09; adversarial scope units and final clean-versus-reused native computation/Plan 09 cache obligations. |
| A08 | Existing compiler/template outcomes retain typed relations, quantities, chosen bindings, diagnostics and stable positive/negative provenance under reorder/duplicate/missing input. Pure correspondence remains native and does not reconstruct rows only to rejoin them. | N10; operator units, existing complete source-expression/equation compilation cases and final fresh heater/mixer publications. |
| A09 | Graph payload/binding/guard edges retain closure/order/cycle witnesses. Shared value/derivative lowering grows with actual DAG/stages rather than path count; excluded invalid branches stay unevaluated; Jacobians/order agree with an independent oracle. No planning in callbacks. | N11/N12; diamond/guard/payload units, existing numerical/backend tests and linked solver cases. |
| A10 | Fallible native admission and owner lifetime remain correct for shared/sliced/dictionary/foreign buffers, copies, concurrent claims, cancellation and final release. No second reservation universe or claim-induced owner reassignment. | N13; small-pool/owner units and final FFI/cache/storage flows; separately record accounted memory and RSS limits. |
| A11 | Declared execution↔Delta mapping preserves values and full restored field contracts across write/scan/reopen; unsupported lossless mappings fail explicitly. Local/durable obligations agree; builders use the actual session/functions/resources. | N03/N14; mapping units and fresh Delta round trips with exact native/Python inspection. |
| A12 | Faults before/after member commit, publication commit, metrics observation and cancellation yield distinguishable outcomes. Exact retry never duplicates effects; conflicting requests cannot share receipts. Readers/CDF/attempt history survive maintenance and caches observe fences. | N14; policy/state units then fault-injected Delta publication/recovery/vacuum/log-cleanup campaign. |
| A13 | Python settings constructor checks, ID/duration codecs, named resource reports, stubs and nested transfer agree with native declarations; error causes and C-stream ownership survive real publication export/reopen. | N15; units plus final editable-extension unit/component suite and independent Rust/Python fresh-store inspection. |
| A14 | Engine has no catalog/Delta dependency; diagnostics is a leaf; actual algorithm services are storage-independent; testkit is dev-only and fixtures call production construction. Shared causes/codes are preserved. | N05/N06/N16; metadata/import/DAG checks, constructor/classification units and fixture isolation. |
| A15 | Native integration produces compact/bounded preparation and eliminates intended duplicate work; measured claims distinguish planning, execution, IO, memory and code-generation cost. No aggregate speedup claimed from one microcase. | N10–N14/N18; measurement matrix and exact library feature/source provenance. |
| A16 | All review recommendations, L01–L18 deletions, callers/generators/tests/docs and required decisions are closed; no replacement remains an optional runtime path or unfinished TODO. Required final checks have zero failures against zero baseline. | N16–N18; source inventory, final campaign, independent A/G assessment and outcome. |

### Measurements and structural checks

| Concern | Deterministic/unit criterion before N18 | Measurements in N18 |
|---|---|---|
| Contract admission | Foreign comparison exact; admitted repeated borrow performs no complete text/descriptor reconstruction | Registry preparation and repeated borrow/batch admission time/allocations, equivalent/foreign cases |
| Native validation | All field families covered; no per-cell Cell/ScalarValue conversion or full planning per batch | Scalar/nested batches across sizes/null rates; compile versus evaluate cost, rows/s and allocations |
| Generated interfaces | One traversal; typed borrowed views and exact fields; generated diff contains no stale files | Generator/runtime code size separately; incremental/fresh compile conditions and admission cost |
| Function/provider adapters | Complete method matrix; plans retain native capabilities and truthful properties | Ordered min/max/statistics cases, nested expressions, plan operators/counters and actual results |
| Dependency/reuse | Scoped memo visits shared producer once per distinct context; negative facts retained | Clean versus warm/reused, changed requirements/providers, prepared count/hit/miss/eviction and invalidation |
| Compiler correspondence | No source-position recovery scans or row shuttling solely for a relational match | P7/P8/P9/templates at multiple sizes, input captures/scans/decoded rows, planning versus execution |
| Graph/numerical preparation | Shared DAG/stage nodes bounded by unique nodes/edges and derivative sparsity within regions; no recursive path expansion | Repeated diamonds and guarded graphs, optimizer/preparation size/time, callback allocation/time, Jacobian agreement |
| Resources | One native reservation owner; preallocation refusal and final release tested | Accounted peaks, retained/copy estimates and RSS reported separately; slice/FFI/cache concurrency |
| Delta | One operation context/reader/retention decision; bounded action reader | Open/replay/write/observation/maintenance counts and latency, cold/warm caches, protected-history cases |
| Extension locality | A nested contract/new operation uses shared declarations and mechanisms | Count independently authored policy changes, not generated lines; document genuine remaining bespoke steps |

Do not invent percentage speedup targets without a comparable baseline. Structural
regression criteria are mandatory; timings explain actual cost. Use existing trustworthy
receipts for historical comparisons only when source, hardware and settings are disclosed.
Do not retain an executable predecessor or run a legacy qualification campaign solely to
manufacture an old/new comparison. Keep expected mathematical/semantic results independent.

### Gate closure

N18 assesses G1 authority with A01/A03; G2 semantic fidelity with A02/A04/A11/A13;
G3 validity with A02/A04/A09; G4 hidden behavior with A05/A07; G5 recovery with A12;
G6 transformation/reuse with A05–A10; and G7 truthful capability evidence with A06/A15/A16.
Assess each independently as pass/fail/unresolved/not-applicable. An unresolved required
claim is not offset by a faster benchmark or a lower source count.

Report every required command, mode, failure count and zero baseline. Record warnings
and capability limits explicitly. Broad engineering/numerical acceptance is never inferred
from a static family check, generated equality, a successful import or a unit-only run.

## Open items

### Bounded implementation choices with selected defaults

No item below defers recommended scope indefinitely or requires a new permission prompt.
Resolve each in its package and record the result before N17. A change to the target
decision gets a short explicit rationale and updated affected oracles, not a hidden fallback.

| Choice | Selected default / bounded decision | Closure |
|---|---|---|
| Contract representation | Native fields plus normalized resolved reference graph and sealed owner-bound handles. Include transitive domains/extensions and exact comparison. | N02 implementation and A01; no fingerprint-only alternative. |
| Local native gaps | Use native predicates, explicit visibility masks and one UDF/kernel per unsupported semantic primitive. | N03 field-family matrix; no second full validator. |
| Row codec | Native array primitives and generated typed views. A serde_arrow prototype is only for an actual row interchange, never a reason to delay Cell deletion. | N04 adopt for that use or remove unused dependency; no dual codec. |
| Canonical ordering/preimage | Retain RowConverter and declared equivalence; share framing. Version deliberate format changes without historical reader retention. | N04 exact unit oracles and current consumer regeneration. |
| Graph library | Shared typed adjacency/binding view over current graph. Choose petgraph only if it simplifies this implementation while preserving witnesses/order/regions. | N11 one selected graph basis, A09. |
| Numerical CSE | Explicit shared stages and region-aware DAG. Do not assume native optimizer CSE prevents input-tree expansion or alias inlining. | N12 structural units and final preparation measurements. |
| Arrow claims | Native reservation owner is mandatory. Enable claim bridge only with proved pre-reserved transfer; use the native owner at boundaries the API cannot transfer safely. | N13 complete owner protocol either way; no legacy memory trait. |
| Delta extraction | Cohesive catalog module; no new Delta crate without a concrete newly discovered independent consumer/build reason. | N14 complete interfaces/callers, not an optional future extraction task. |
| Dormant solver surface | Keep independently meaningful tested kernels; delete abandoned alternatives. Add no new simulator workflow. | N16 reachability classification and existing N18 kernel/solver checks. |
| SQL ownership | Readable authored SQL by default; generate only from an actual shared declaration when it reduces independent decisions. | N16 one source per rule, N09 correctly keyed syntax reuse. |
| Publication/retention | Preserve required semantics through native builders and one effective protection policy. No weaker guarantee merely to delete coordination code. | N14 implementation plus A12 fault qualification. |

### Evidence and restart protocol

After each package, update its inventory rows and the package table with exact source/
consumer/deletion state and named unit receipts. Keep unexecuted integrated cases marked
`not_run`. If interrupted, record the last completed package, active files, unresolved
compile/unit issues and next dependent task. Do not replace this plan with a new “remaining
scope” plan or make the next session rediscover the same review.

Mark this plan `done` only after N18 and all required A/L obligations close. Failure or
unresolved behavior remains open even when the implementation inventory is complete.

## Outcome (implementation through N09 and N11)

### What was built

**Implemented and unit-tested:** N00–N09 and N11. Foundation work provides native contracts,
values, canonical framing and shared diagnostics. N06 extracts the engine, production
resource assembly and dev-only testkit, centralizes effective settings, preserves actual
native implementations and durable owners, and adds bounded execution assurance.
N07 replaces operation/planner/transport duplication, protects required work before native
optimization, and shares cancellation/completion and native metrics. N08 delegates complete
native function hooks and preserves fields through scalar, aggregate and statistics paths.
The [N07–N08 contract matrix](10-native-contracts-n07-n08.md) records the implementations,
retained specialization and deletions.
N09 unifies scoped traversal, semantic source demand, syntax reuse and actual-owner
preparation witnesses. N11 shares typed graph views, exact arithmetic and executable
operator capabilities, including payload/guard/binding provenance. The
[N09/N11 implementation contracts](10-native-contracts-n09-n11.md) record the replacements,
deletions and remaining boundaries.
The [execution inventory](10-execution-inventory.md) is the current completion ledger.
N10, N12 and N13 now use stable-key native correspondence, shared numerical stages and native allocation owners; their [slice record](10-native-data-execution.md) details retained algorithms and verification. N14–N18 remain open; this plan is not done.

### A mistake made and corrected

The first generated graph initializer exhausted the ordinary test-thread stack; small
constructors and iteration corrected it without raising limits. N06 also exposed an
incorrect family-name assumption: independently released `datafusion-tracing` is not an
Apache DataFusion family version. One explicit metadata classification now serves family
and blueprint-pin checks while all actual native dependencies remain checked. N07's
cloned-command unit exposed cache reconstruction assigning a new completion to unchanged
work; actual identity now survives that reconstruction. N08's ordered-reversal fixture
checks the returned native UDAF, and its statistics fixture checks metadata after substitution.

### Deviations from the plan, deliberate

The earlier N06 slice was expanded with native execution assurance; the subsequent
N07–N08 slice implements operation families and complete native adapters. No domain value, ownership,
cancellation or durable correctness oracle was replaced by a span-close assertion.
N16 still owns the broader fixture consolidation; N17 still blocks N18's integration
and performance campaign. Existing data and predecessor APIs are not compatibility goals.


### N07–N08 verification — 2026-09-19

**Tested:** `just dev-native-contracts`, default nextest profile, isolated `--lib`
selection with `pse-relations/force-validate`: **34 passed, 0 failed**, baseline zero.
This includes fake command settlement, requirement retention under zero limit,
cloned versus distinct commands, EXPLAIN laziness, native metrics, owner lifetime,
epoch reset, cache cancellation/spill, scalar/nested fields, min/max statistics,
ordered array aggregation/reversal/merge, named struct and typed concat.

**Interface-checked:** `just check-native-contracts` (workspace/all targets),
`just check-native-contracts-solver` (Ipopt feature/all targets; no solver execution),
`just lint-native-contracts` (changed consumers/all targets, `-D warnings`) and
`just fmt-check`: exit 0, zero failures against a zero baseline. The inventory records
conditions and the existing upstream Cargo future-compatibility notice.

No integration, compiler journey, durable storage journey, solver execution or
performance campaign ran. N17 remains open and N18 remains deferred.


### N09/N11 verification — 2026-09-19

**Implemented:** N09, N11, L10 and L12. Shared iterative native and typed-graph
traversals replace predecessor walkers; actual semantic owners govern reuse. SQL syntax
retention is bounded and separate from rebinding. Order-sensitive dependencies retain
exact selected versions because CDF/endpoint bag comparison cannot establish ordering.

The [execution inventory](10-execution-inventory.md#n09n11-completion--2026-09-19)
records the final isolated unit and static commands, zero-failure baseline, and the
existing untracked generated-file check failure. No integration or performance
qualification ran; A07/A09 final behavioral acceptance remains N18 work.


## N10, N12 and N13 implementation checkpoint — 2026-09-19

**Implemented:** native stable-key compiler correspondence and qualified capture;
borrowed indexed path traversal; shared value/derivative native numerical stages;
native DataFusion memory pools/reservations with allocation-aware Arrow/cache/FFI
ownership. The [slice execution record](10-native-data-execution.md) documents
replacement owners, retained finite algorithms, deletion boundaries and verification.

This checkpoint does not seal N17 or run N18. Complete source-expression compilation,
Delta publication/reopen, Python extension, linked solver and performance qualification
remain deferred. N14–N16 retain their original scope.


## N14/N15 implementation checkpoint — 2026-09-19

**Implemented / Tested (development units):** the
[Delta and Python slice record](10-native-contracts-n14-n15.md) records the complete
N14/N15 source changes, native APIs, deletions and command-qualified evidence.
N11 was not reopened. N14 shares declared mappings, caller-bound operations, bounded
commit observations and effective retention. N15 generates native settings/reports,
shares identity codecs and preserves structured diagnostics and Arrow allocation owners.

N16–N18 remain open. All integration, publication/reopen/vacuum/fault campaigns, solver
execution and performance acceptance remain deferred until N17 closes. The existing
untracked generated `validation_findings.rs` and pre-existing skill-script Ruff failures
remain explicit repository gate failures; no baseline or exclusion was introduced to
hide them. No git staging, commit or push was performed.

## N16/N17 source closure — 2026-09-19

**Implemented / Tested (development scope):** the [execution inventory](10-execution-inventory.md)
and [closure map](10-consolidation-closure.md) supersede earlier remaining-scope checkpoints.
All source/deletion rows are complete; the exact-source seal, verified development logs
and [binary-qualified case index](10-acceptance-cases.toml) govern N18 dispatch.
13 tooling units, 19 engine/fixture units and 9 static governance tests passed with
force-validate, default nextest profile, zero failures against baseline zero. Strict
all-target Clippy and pure generated-output equality passed. These checks establish
implementation readiness, not A01–A16/G1–G7 acceptance. N18 remains open.

### N18 execution checkpoint

Campaign `build/plan10/acceptance-2026-09-19-01` passed formatting, family/boundary
checks and full regeneration, then failed workspace compilation when incremental
cache writes exhausted the filesystem. No functional suite or benchmark ran. The
[execution inventory](10-execution-inventory.md) records exact evidence and the linked
continuation command. Disk recovery, remaining gates, measurements, independent review
and the final Outcome remain required; this plan is not done.

### Campaign 02 regression checkpoint

The [execution inventory](10-execution-inventory.md) now supersedes the initial N17
closure: the CI/force-validate run produced 944 passes, 92 failures, one timeout and
38 unexecuted cases after deliberate interruption. N16/N17 are reopened for grouped
repairs, with the failing property-test reproducer preserved. Final acceptance,
measurements, independent review and Outcome remain outstanding.
