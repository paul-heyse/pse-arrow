---
title: Native caching and completion of the DataFusion and Delta hard pivot
status: done
date: 2026-09-17
adrs: [ADR-0046, ADR-0050, ADR-0065, ADR-0066, ADR-0068, ADR-0069, ADR-0070]
phase: 1
---

# Native caching and completion of the DataFusion and Delta hard pivot

**Complete — 2026-09-18.** C00–C12, including the carried Plan 08 E00–E10
scope and legacy deletions, are implemented and qualified for the existing local
process-model architecture. The [acceptance review](../design_review/reviews/design_review_native-cache-pivot-acceptance_2026-09-17.md)
assesses Q01–Q14, K01–K11 and G1–G7 independently. The
[execution inventory](09-execution-inventory.md) and
[measurements](09-cache-measurements.md) preserve commands, source boundaries,
failures repaired during qualification, and the limits of the claims.

**Tested, baseline zero:** the exact union of the initial Rust campaign and affected
continuations covers **1,002/1,002 ordinary tests** (993 original, nine added, none
removed). This is aggregate coverage across recorded source boundaries, not a claim
that one untouched final full-suite run passed. Final affected checks include 268
schema/relation/rule/compiler cases and 253 provider/source cases. The last kernel
case passes in **309.658 seconds** under the unchanged **360-second** CI deadline,
with all 390 outputs, exact unit assertions and Delta publication/reopen retained.
The [Rust ledger](../design_review/evidence/native-cache-rust-qualification-2026-09-18.json)
identifies every accepted test and archived failure.

**Tested and Measured:** four complete heater/mixer publications and independent
Rust/Python inspection, 72 final Python unit/component tests, 12 linked solver
cases, 84 feature checks, and the 84-record native cache matrix have passing receipts.
The final extension independently reopens all four engineering publications.
Formatting, strict Clippy in both modes, generated equality, Python quality,
environment checks, rustdoc and book checks pass. Earlier source boundaries for
solver/features/doctests/benchmarks and Rust engineering publications remain explicit;
none is silently relabeled as a rerun on the final source.

No timeout, retry allowance, runtime budget, scientific assertion or legacy pathway
was added to obtain a pass. No new simulator function, numerical IDAES parity,
remote destructive coordination or universal speedup is claimed. Proposed ADRs
remain proposed; this completion record does not change their status or the blueprint.

This plan integrates the [caching review](../design_review/reviews/design_review_datafusion-delta-caching_2026-09-17.md),
additional opportunities and corrections established during planning, and **every
remaining E00–E10 obligation** in [Plan 08's current checkpoint](08-schema-first-native-data-pivot.md#current-remaining-scope--2026-09-17).
It is the active successor execution sequence, not a second implementation track.
Plan 08 remains the source of completed implementation receipts and A01–A08/Q01–Q14
acceptance definitions. The crosswalk below assigns all unfinished work here.

**Execution order:** correct identity and declare cache/dependency contracts; implement
accounted native cache services and Delta library seams; complete prepared execution,
Delta opening, residency and exact reuse; finish all callers, deletions and fixtures;
**only then run integration and performance qualification**. Use compilation,
generation and isolated unit tests while implementing. A compiler/storage/solver
journey remains integration even when located under `--lib`.

**Hard pivot:** retain implemented target foundations and useful domain algorithms;
delete replaced execution paths and declarations with their consumers. No migration,
old-data preservation, compatibility decoder, dual authority or transition period.
Keep current high-level process-model outcomes and independent scientific assertions.
No new simulator workflow, sweep operator, solver family, distributed execution or
remote deployment is required. The dependency and cache contracts will support those
future workloads without implementing them here.

## Context

### Current implementation and immediate restart

**Implemented, with scoped evidence in Plan 08:** exact recursive Arrow contracts,
complete tagged values, coherent math/numerical rows, native invariant plans and 130
SQL inference declarations, common provider admission, native compiler/source edits,
exact Delta publication, typed member-attempt/dependency receipts, conservative
retained-generation reuse, bounded CDF, reader leases and native maintenance. The
predecessor driver/stage/memo, closed rule algebra, custom stores and Python snapshot
handles are deleted. Native insert defaults and direct generated Arrow transfers
are implemented. These are foundations to extend, not tasks to repeat.

**Historical planning baseline (superseded by the active checkpoint below):** the review's principal gaps were: runtime entries
are copied into semantic settings; runtime file-cache limits remain implicit;
`Round::execute` recompiles evolving rules; Delta open paths construct fresh table
loads; no shared snapshot/resident service or consumed-column dependency contract
exists. Plan 08's `CacheStore` type-complexity finding and the by-value `retention`
argument to `prepare_maintenance` in the expanded lifecycle fixture remain to fix.
Earlier compilation and unit receipts do not certify the latest whole tree.

**Planning checks:** `just doctor` reports **1 environment-freshness failure, baseline
0**. `just metadata` and `just --list` succeed. Planning does not require a rebuilt
extension; environment repair remains C00 work. No compilation, test, runtime probe,
benchmark or integration campaign was run to create this document. `just plan`
created the new plan using the repository recipe.

### Library evidence and boundaries

**Interface-checked:** DataFusion **55.1.0**, Arrow/Parquet **59.3.0**, object_store
**0.13.2**, delta-rs **58f07cd62bfbce3649a7e1c87c696288068ae184**, kernel
**8ba063f8f84fec222000f66d40d70911d7c79675**. Research used the local
[DataFusion skill](../../.codex/skills/datafusion/SKILL.md),
[Delta Lake skill](../../.codex/skills/deltalake/SKILL.md), their indexed API pages,
Cargo metadata and exact checked-out sources. No Context7 was used for these families.
The [planning evidence manifest](../design_review/evidence/native-caching-pivot-plan-2026-09-17.json)
records source locators, revisions, hashes and the limits of inspection.

Primary capability references used:

- `datafusion_execution::cache::{Cache, CacheKey, CacheValue}` and
  `cache_manager::{CacheManagerConfig, CachedFileMetadataEntry}`:
  [cache contracts](../../.codex/skills/datafusion/content/api/datafusion_execution.cache.md)
  and [manager injection](../../.codex/skills/datafusion/content/api/datafusion_execution.cache.cache_manager.md).
  Exact `default_cache.rs` and Parquet `metadata.rs` establish eviction and key behavior.
- `datafusion_physical_plan::execution_plan::{ExecutionPlan, reset_plan_states}`:
  [execution contract](../../.codex/skills/datafusion/content/api/datafusion_physical_plan.execution_plan.md)
  and the pinned source's traversal/default reset implementation.
- `deltalake_core::kernel::snapshot::Snapshot::{update, try_new_with_engine}`,
  `table::DeltaTable::update_incremental`, `TableProviderBuilder::with_snapshot` and
  `with_eager_snapshot`: [snapshot surface](../../.codex/skills/deltalake/content/api/deltalake_core.kernel.snapshot.md)
  plus exact snapshot/provider/builder sources. Delta operations are on `DeltaTable`.
- `buoyant_kernel::snapshot::{Snapshot, SnapshotBuilder, IncrementalReplay}`:
  [builder/CRC scope](../../.codex/skills/deltalake/content/api/buoyant_kernel.snapshot.builder.md)
  and [owned-heap/checksum methods](../../.codex/skills/deltalake/content/api/buoyant_kernel.snapshot.md).
  Cargo exposes this crate under the name `delta_kernel`.
- Native Parquet pushdown, predicate caching, page indexes, bloom filters and footer
  hints: [configuration catalog](../../.codex/skills/datafusion/content/catalogs/config-options.md).
  Load classes/checkpoint/commit behavior and CDF's private cache were checked in
  pinned delta-rs source, not inferred from a public builder name.

None of these inspections establishes a speedup. Interface availability and production
integration are separate obligations. Dependency modifications in C04 require a new
immutable source revision or reproducible local override and refreshed evidence.

## Decisions

### D01 — Distinct cache lifetimes with one durable authority

Delta logs/publications remain the data authority. Cache hits are disposable derived
reads and must pass current admission. `CacheFactory::cache_plan` performs no IO,
publication or eager materialization. No serialized Delta snapshot, persisted physical
plan or opaque object hash establishes reusable model state.

| Plane | Native mechanism / owner | Selected deployment |
|---|---|---|
| File metadata | Native `CacheManager`/`DefaultCache`, namespaced by storage binding | Required, declared/accounted; shared by ordinary scans and CDF after C04 |
| File statistics and listings | Native `TableScopedPath` caches | Declare/configure/account; enable for actual ListingTable consumers, not as a second Delta file inventory |
| Invocation results | Existing native cache extension, pool and SpillManager | Retain; correct round-epoch sharing and ownership as C05 requires |
| Prepared stratum | Native logical/physical plans and `reset_plan_states` | Required; one preparation per invocation/stratum, stable round sources |
| Delta snapshots | Native Snapshot/EagerSnapshot and seeded update | Required, bounded shared retention with explicit load capabilities |
| Decoded relations | Owned Arrow batches via a native provider and common cache service | Required for eligible exact selected scans, lazy and bounded |
| Durable semantic reuse | Typed native dependency queries and exact publications | Extend with consumed-column evidence; never route through an effectful `df.cache()` |
| Durable replay acceleration | Native checkpoints; qualified kernel checksum facilities | Checkpoints required; CRC integration implemented with explicit capability/outcome and measured default selection |
| Parquet predicate cache | Native decode pushdown and bounded reader cache | Explicit execution policy; qualify selective/wide scans and total concurrency cost |
| Object bytes | Native ObjectStore registry decorator seam | Declare/test routing now; remote cache implementation activates when a remote root is admitted |
| Inspection | Native metadata/table-function relation and execution metrics | Required; bounded, read-only, excluded from semantic reuse identity |

The shared deployment runtime owns one service instance; session factories and scoped
sessions carry its `Arc`. Rebuilding a factory for the same runtime must not create an
independent cache budget. `pse-catalog` owns contracts/services needed by native plans;
`pse-runtime` constructs and accounts them without a reverse dependency from catalog
into runtime. No new crate is needed for this ownership arrangement.

### D02 — Correct semantic identity before deploying caches (F1, F5)

Maintain separate, derived inventories for **semantic inputs**, **resource policy**
and complete diagnostic read-back. Exclude known runtime storage/cache/pool ceilings
and spill locations from result identity; retain them in effective policy enforcement,
resource reports and the full `information_schema.df_settings` consistency check.
Time zone, function/rule implementation identity and meaning-bearing policies remain
semantic. Unknown configuration extensions remain conservative dependencies; do not
classify every execution/planner option as harmless merely because it affects speed.

Apply this separation at all session construction, derived-scope capture, settings/
profile hashing, native dependency capture and member-attempt comparison sites.
Capacity/cache changes can affect admission or performance without proving different
values. Re-admit the current request under its actual budget even on a cache hit.
Budget-independent identity does not authorize recovery across unrelated opaque
implementation generations. Test those cases separately.

Separate **attempt/invocation identity** from a retained **implementation generation**.
Current `ArtifactPlan.operation_id` facts must not automatically invalidate every
projected dependency comparison between two input versions. Retain actual native
implementations under an owned generation; use declared, versioned identities only
for implementations with an established contract. Names alone cannot make unknown
UDFs, planners or extensions portable across processes.

### D03 — Additional findings and refinements to the caching review

These choices refine the review's mechanisms while retaining its intended outcomes.
The source implementations are recorded in the active checkpoint; behavioral acceptance remains separately tracked.

| ID | Additional opportunity or correction | Chosen consequence / evidence |
|---|---|---|
| N01 | Shared `FileMetadataCache` uses `Path`; the pinned validity check compares only size and last-modified, not store identity, ETag or object version | Namespace keys with actual registered store generation and canonical root. Equal paths/sizes/timestamps in two stores must not share a footer. C03/C04; exact cache-manager and Parquet metadata sources |
| N02 | `Cache::put` is infallible and DefaultCache evicts internally without a public eviction callback | Use native caches under explicitly reserved capacity envelopes, not an inaccurate reserve-on-put/release-on-remove wrapper. Show capacity reservation separately from retained bytes. C03; native Cache/DefaultCache contracts |
| N03 | `reset_plan_states` traverses and calls `reset_state`; its documented dynamic-filter/recursive-query limits are not a universal rejection validator | Enforce eligibility before reuse; make round sources conservative and cache completions epoch-specific. A mutable source with an initially empty batch must never optimize into permanently empty results. C05 |
| N04 | An in-process eviction hook alone cannot observe another process's maintenance; idle leased cache entries can prevent maintenance forever | Keep idle entries lease-free, acquire a fresh read lease on reuse, and validate a native maintenance/version fence before serving cached state. Fence every destructive maintenance attempt before deletion; invalidate even after interrupted maintenance. C06/C07/C10 |
| N05 | `SnapshotIdentity` and internal materialized-file validation are private at the pin; the outer Delta Snapshot exposes no complete public retained-heap measure | Retain the actual native snapshot behind an application lookup key; use native internal validation, not a copied identity algorithm. Add a narrow upstream-facing retained-size API with C04 before claiming snapshot byte accounting |
| N06 | Root/version/projection/view flag is insufficient for decoded results when a publication selects a revision or a provider applies row policy | Include exact member selection, domain/layout/physical schema, store generation and all eligibility-relevant semantics. Recheck requirements/authorization on every use. Integrate provider reads as well as explicit `df.cache()`; Python table reads do not automatically call it. C07 |
| N07 | A consumed-column label alone does not enforce use or remove whole-version/operation-generation invalidation | Generate projected algorithm argument views, inspect native expression dependencies and retain key/membership/multiplicity/absence obligations. Replace whole-version equality only for a proved projection; retain exact selections as provenance. C02/C08 |
| N08 | Duplicate concurrent misses, unbounded open fanout and eagerly cloned inspection entries can erase cache benefits | Coalesce in-flight loads by exact key, bound replay/read/decode concurrency, account inspection staging and expose aggregate metrics separately from entry listing. Share failures only for a live load, never as durable absence. C03/C06/C07/C09 |
| N09 | Kernel CRCs summarize file counts/sizes, metadata and transactions; they do not contain all active Add actions | Integrate CRC acceleration for eligible metadata/recovery paths; retain checkpoints and native replay for file enumeration. Reject the review's blanket O(1)-full-open expectation. C04/C06 |
| N10 | Checkpointing only the publication root leaves repeatedly updated member tables with unbounded tails | Apply declared native checkpoint policy to applicable control and member commits, retaining cleanup exclusively in admitted maintenance. Reconcile post-commit checkpoint failure without replaying data writes. C06 |
| N11 | A one-partition cache can be a downstream bottleneck, but arbitrary repartitioning may alter ordering or duplicate partition reads | Preserve/derive truthful partition and order properties; use native repartition operators where measured. Do not implement a custom batch partitioner or imply an ordering guarantee. C07/C12 |
| N12 | Resource options also occur outside `datafusion.runtime.*`; caller pool scopes already share the cache manager | Use one typed classification and service owner; retain caller scope quotas, reserve predicate-cache concurrency and prevent shared caches from bypassing per-operation result limits. C01/C03/C09 |

### D04 — Cache ownership, accounting and admission

Extend `ResourceBudget` with a typed cache declaration: metadata/statistics/listing
ceilings and TTL, snapshot/resident ceilings, bounded in-flight load count/bytes and
inspection capacity. Predicate cache size and open/replay concurrency have explicit
native settings and resource consequences. Zero is a supported cache-disabled mode;
a miss or capacity refusal executes the normal admitted native read. No second plan
or storage backend is introduced for cache-off qualification.

For native file caches, retain `DefaultCache` and its byte-LRU/TTL behavior within a
pool-reserved capacity envelope. Cache construction/limit increases reserve first;
configuration cannot raise the native limit beyond its reservation. Retained-value
sizes, map/key overhead allowance and reserved capacity are separate report fields.
Do not claim this accounts arbitrary allocator behavior or unowned Arc clones held
inside upstream code; identify their transient exposure and test peak RSS. A future
exact per-value approach requires ownership through native clones and eviction, not
just callbacks around `put`/`remove`.

Snapshot/resident service entries carry shared reservation owners. Eviction removes
the service reference; external readers keep their reservations until their last
buffer/snapshot owner drops. Track live pinned bytes separately from cache residency.
Use native `DefaultCache` with typed `CacheKey`/`CacheValue` entries for these bounded
stores where its contract fits; do not write another LRU/TTL implementation. Couple
the reservation owner to the provider/stream that receives a native snapshot so a
bare snapshot Arc cannot escape its accounting lifetime.
Deduplicate Arrow buffer accounting by allocation ownership where batches share
buffers. Incoming decode/replay and entry inspection need bounded transient budgets;
post-allocation size measurement alone is not reserve-before-allocate assurance.
Expose that upstream limit explicitly and bound fanout until a native allocation hook
exists. Cache pressure cannot consume the whole query working set: validate ceilings
against the deployment limit and an explicit working-memory reserve.

`list_entries()` clones its map/values and is O(entries). A `pse_cache_entries` native
table function must budget this work; SQL `LIMIT` outside the function is not a bound
on that clone. Supply a cheap aggregate metrics relation, bounded entry-detail
arguments, cancellation and an explicit resource error when enumeration cannot fit.
Inspection performs no cache population, table opens or LRU hits; it must not retain
values after producing its typed diagnostic batches.

### D05 — Shared snapshot/residency keys and maintenance

A lookup key includes canonical root, actual storage registration generation, exact
Delta version, table identity/contract and load capabilities; snapshots themselves
remain native. Record full load configuration for reproducibility, but distinguish
capability-affecting fields from replay concurrency/IO runtime ownership. Do not use
`DeltaTableConfig::PartialEq` as proof of identical IO-runtime ownership.

Type required capabilities at each open: **Metadata** (no file materialization),
**Query** (files/statistics), and **Maintenance** (operation-specific files/statistics).
`skip_stats` is eligible only when the actual command does not need statistics;
optimize is not automatically assigned a stats-free class. A metadata snapshot cannot
satisfy a query merely because root/version match. Use native update for forward
refresh on a private clone/Arc; never mutate a snapshot already bound into a plan.
Exact version lookups never accept a cached notion of latest.

On maintenance: acquire the existing exclusive local lease; commit a native Delta
maintenance fence on every affected root **before** destructive work; evict matching
snapshot/resident entries and pending insertions; then run native maintenance. Every
new cache consumer acquires a read lease and checks an uncached native LogStore latest-
version observation against its validation generation. A changed/unknown generation
causes invalidation and authoritative reload, not a hit. Conservative invalidation
on ordinary new commits is acceptable initially; narrowing it requires native proof.
The fence is coordination provenance in Delta, not a second table-state store. A crash
after the fence but before/during deletion must still prevent stale cache reuse.

This closes cross-process maintenance for the supported coordinated local envelope.
Unmanaged external deletion and remote reader exclusion remain explicitly unqualified;
no cache promises protection from them. Cache eviction is never file deletion.
Latest resolution and fresh lease/fence checks may require IO even on a warm open;
zero replay is a measured cache property, not zero IO or universal version availability.

Resident entries contain complete admitted selections for an explicit projection and
physical schema. Limits, residual filters, revision selection, deletion vectors and
row policies cannot disappear from a key. Prefer caching below expressions and above
the exact selected native scan; ordinary expressions remain native plans. Filtered or
partial streams are not stored as complete relations. Publish an entry only after
successful full completion; canceled/abandoned/failed fills never become hits. Preserve
caller CacheFactory behavior and expose the same resident service through the selected
provider so plain Rust/Python reads can benefit without hidden eager reads at open.

### D06 — Prepared fixed-point execution

The finite fixed-point node owns one prepared stratum and an invocation-local round
workspace. Compile candidate/query/delta variants, support plans, emptiness probes,
union/difference and merge plans once. Stable native round-source providers read
immutable batch snapshots for a named epoch; publish the next epoch only after all
current streams settle. A stream must never observe mixed rounds.

Round-source schema/partition/order properties are invariant; row counts, emptiness,
cardinality-derived constraints and other changing statistics are unknown/conservative
at preparation. Reset physical operator state with native `reset_plan_states` between
rounds. A dedicated derived execution profile enforces the documented restrictions:
no dynamic filters or nested native recursive query in reusable inner plans. It does
not disable ordinary Parquet decode-filter pushdown globally. Unsupported custom
operators must supply a valid reset/effect contract or be refused for this mode;
do not silently return to the deleted per-round planner path.

Round cache completions are keyed by invocation/stratum/round epoch and input identity;
changing a round buffer invalidates dependent completions even though the outer
invocation is unchanged. Immutable inputs may share across rounds only with a proof.
Reset metrics/cancellation/tasks and account both old and next buffers during turnover.
The selected physical plan is a fixed exact execution strategy, not an approximation
of domain meaning. Pin conservative planning choices initially; native repartition/
join tuning can change after measurements while preserving the semantic contract.

### D07 — Consumed-column dependencies and structural reuse

Extend the registry's algorithm input contract and generated reflection with a typed
consumption declaration: all columns, or a declared projection plus key/membership/
ordering obligations. Use field paths only where an actual nested-field projection is
provable; otherwise depend on the whole enclosing value. Generate argument views from
the same relation declaration. A function that receives an unrestricted full row has
not proved a narrower dependency because its signature says “structural.”

For native plans include columns used by filters, joins, grouping, sorting, windows,
row policy, requirements and support recovery, not just output projection. Record
row membership, cardinality/multiplicity and absence scopes even for `COUNT(*)` or
zero-column projections. Unknown extensions consume all inputs or require freshness.
Classify per actual consumer/output group, not by a global list of columns called
“parameter” or “value”; a numerical value may control equation selection or units.

Retain exact old/new member selections as provenance. For qualified projections,
compare consumed values using native null-safe grouped counts and bidirectional
set difference, with canonical typed fingerprints as a fast discriminator rather
than a replacement for declared semantics. Expired source/evidence history falls back
to an explicit fresh computation. CDF can reduce comparisons only when a complete
bounded interval is available; insert/delete and key/filter changes always participate.
Do not permit unchanged projected values to waive a current required validation.

Decouple data-version changes from retained implementation generation as D02 requires.
Carry proof dependencies into member-attempt receipts and `prepare_reuse`, remove the
unconditional whole-selection equality that would otherwise defeat projection reuse,
and preserve whole-input behavior for undeclared/opaque consumers. Qualify value-only
and structural edits against clean recomputation of existing native algorithms. The
sweep operator itself remains out of scope; no blanket promise that every parameter
change leaves every structural artifact unchanged is made.

### D08 — Native replay, CDF and storage seams

Use registry-declared Delta checkpoint intervals and native commit hooks for control
and applicable member tables. Keep `cleanup_expired_logs(false)` on ordinary commits.
Reader-retention decisions and destructive cleanup stay in explicit maintenance.
Checkpoint/CRC failures after data commit are settled against the actual committed
version; do not repeat an effectful child to “retry the cache.” Record acceleration
failure separately from commit outcome and retry acceleration at an admitted boundary.

C04 supplies the missing upstream surfaces: inject the shared, namespaced metadata
cache into CDF; expose truthful native snapshot retained-size information; and expose
kernel incremental CRC/checksum configuration through delta-rs's native construction/
post-commit path. Prefer an upstream immutable revision if available. Otherwise use a
minimal reproducible source override/patch with provenance in this repository until
an equivalent upstream revision exists. Never edit Cargo's cache or `external/`, pin
a moving branch, serialize private state or copy the Delta replay algorithm into PSE.
No upstream message/PR publication is part of this planning task.

CRC summaries accelerate eligible metadata/statistics/transaction work. Full scans
still need native active-file enumeration and data files. Missing/unsupported CRCs
use the ordinary Delta path; malformed evidence never invents empty state. Keep a
bounded incremental replay policy and explicit outcomes; measure before choosing
CRC-on defaults. Such native fallback is an acceleration miss, not legacy retention.

Declare the runtime ObjectStore decorator seam for log JSON, checkpoints and Parquet,
including store identity, mutable-object validation, conditional writes and limits.
A Parquet-only byte cache and dormant `delta-cache` feature are not selected. Do not
cache negative latest/log listings indefinitely or rewrite storage preconditions.
Local file IO already benefits from the OS page cache; build a remote byte-cache
backend only when a remote root and backend-specific correctness fixtures enter scope.

## Plan

### Dependency order and package status

**C00–C12 complete.** The table retains the dependency order used for implementation.
Terminal acceptance is supported by the scoped final receipts and independent review above.

| Package | Depends on | Deliverable | Status |
|---|---|---|---|
| C00 | — | Current compile checkpoint, decision/scoping records and complete carried-forward inventory | Complete |
| C01 | C00 | Semantic/resource/implementation identity separation | Complete |
| C02 | C01 | Generated cache-policy, checkpoint and consumption/dependency contracts | Complete |
| C03 | C01–C02 | One accounted native cache service, namespace boundary and inspection substrate | Complete |
| C04 | C02–C03 | Reproducible Delta CDF/accounting/CRC integration surfaces and refreshed pins/evidence | Complete |
| C05 | C01–C03 | Prepared fixed-point execution and round ownership | Complete |
| C06 | C02–C04 | Shared native Delta opener, incremental snapshots, checkpoint policy and maintenance fences | Complete |
| C07 | C03/C06 | Lazy resident selected relations and native scan integration | Complete |
| C08 | C02/C05–C07 | Column-aware exact dependencies and real durable reuse integration | Complete |
| C09 | C03–C08 | Native pushdown/resource/inspection/extension coverage | Complete |
| C10 | C01–C09 | All remaining Plan 08 code/caller/schema/recovery/deletion closure | Complete |
| C11 | C00–C10 | Target-only fixture/oracle inventory, generated/static closure; integration start barrier | Complete |
| C12 | C11 | Final current-function integration, measurements, quality and independent architecture gates | Complete |

### Active implementation checkpoint — 2026-09-17

**Implemented:** semantic/resource and owned implementation identities; generated cache and
complete dependency contracts; shared native envelopes/namespaces/single-flight/load admission;
reproducible Delta CDF/accounting/CRC source seams; prepared native strata and reset validation;
common exact/seeded snapshot opens, returned committed-state retention and checkpoint/fence
policies; lazy resident selections with exported-buffer ownership; enforced argument projection
and native CDF/endpoint multiset reuse; native inspection, scan policy and Python cache reports.
Both JSON receipt decoding and serialization now retain explicit staging reservations.
Superseded per-round planning, scattered fresh loads and the last production relational Cell
conversion are removed. Current functions and useful algorithm/FFI/parser boundaries remain.

The complete [execution inventory](09-execution-inventory.md) maps C00–C12 and every carried
E/Q obligation to sources, consumers, generators, deletions and named target oracles. It also
records the deliberate limits: unknown cold implementation identities refuse; current algorithms
retain Whole consumption unless access is enforced; upstream allocation/replay metrics are not
fabricated; remote reader exclusion and unmanaged deletion remain unqualified.

**C11 complete at its recorded source boundary:** all implementation, caller/declaration/deletion and fixture obligations
are actioned. Required feature compilation passes (53 depth-two combinations plus 31
no-default package checks). **Still open:** C12 integration and independent
Q01–Q14/G1–G7/K01–K11 acceptance.
Whole-workspace strict Clippy passes in both default and no-default modes. Pure
Rust/Python/docs generation equality, compiled API stubs and Python quality pass.
The campaign started with
`just architecture-acceptance build/plan09-acceptance-20260917`. Integration was
permitted after this complete source/static cut. Current continuations, repairs and
passing checks are distinguished in the execution inventory; the measured cache matrix
and selected defaults are recorded separately from terminal functional acceptance.

**Final C12 receipt — 2026-09-18:** source archive
`build/plan09-acceptance-20260918-25` records the final production source. The exact
coverage ledger closes every original residual and all nine added regressions.
Native support aggregation, visible-value decoding, construction-once validation,
materialized witness conditions, shared producer restoration and registry declaration
borrowing have affected regression coverage. The complete publication and cold-reader
receipts are in the [engineering ledger](../design_review/evidence/native-cache-engineering-2026-09-18.json).
All package and acceptance obligations are closed within the scope stated above.

**Historical C12 receipts (not the final status):** the first complete Rust campaign,
`just architecture-acceptance build/plan09-acceptance-20260917-03 --start-at test`,
ran default-profile force-validation with no retries and baseline zero: **963 tests,
826 passed, 136 failed, one 120-second timeout**, 1247.789 seconds. The receipt records
that source state; it does not certify subsequent repairs. Full-suite execution is
not being repeated after each fix. Targeted residual runs qualify native schema,
rule, provider, lifecycle and fixture repairs. The second broad Rust campaign stopped
at 976/982 cases (937 passed, 22 failed including one interruption, 17 timeouts and six
not run); those older-source failures do not certify the repaired implementation.
Those historical failures are repaired and covered in the final ledger. The full 84-record cache matrix is
complete and selects the conservative defaults in [the measurement receipt](09-cache-measurements.md).

Repairs include expanded-view source lineage for consumed-column evidence; nullable
recursive work-table fields with authority metadata kept at the relation boundary;
native round reset immediately after exhaustion; typed rule fixtures; native MERGE
target/source expression scope; publication diagnostics; native CDF metadata kept
separate from persisted field descriptors; full-range timestamp tick persistence;
and deferred provider write hooks behind execution requirements (including EXPLAIN).
Regenerated conformance fixtures reflect the current registry. Formula qualification
uses the published Perry endpoint table and explicitly records its separate NIST
measurement residual instead of widening the old mismatched tolerance.

**Historical Tested receipt:** the focused native-construction/Delta contract run passed 33 of 35 tests;
the later schema/checksum selection passed 22 of 23. The 64-test residual selection
then passed 52, failed 11 and timed out once at 120.009 seconds. The next 16-test
selection passed 12, failed four; all round/admission/typed-alternative repairs in
that selection passed. These are separate overlapping selections, not an aggregate
pass count. Subsequent source edits require their own qualification. Engine timings
are being split into composition, preparation and execution under the existing CI
profile; timeout configuration has not been increased.

**Interface-checked / Tested limitation:** the pinned kernel cannot bootstrap a CRC
from an ordinary delta-rs snapshot without an eligible native checksum seed.
`native_kernel_crc_seed_advances_through_delta_writes_and_corruption_falls_back` passes
with a real native committed transaction. Unsupported CRC remains a replay fallback;
defaults and full-file scan claims must respect this boundary.

**Tested:** the isolated catalog cache/config/round/receipt selection named in the inventory
passed **27 tests, 0 failed, baseline 0, 91 filtered**, force-validation. The compiler argument
projection unit passed **1 test, 0 failed, baseline 0, 28 filtered**. These predate the new
storage-only `delta_journeys` module, which must be excluded from subsequent unit-only commands.
**Interface-checked:** the Delta override verifies all 142 files; pure contract generation and
preceding whole-workspace compile passed. The
[implementation acceptance review](../design_review/reviews/design_review_native-cache-pivot-acceptance_2026-09-17.md)
now closes all seven gates for the stated scope. Exact commands and source
boundaries remain in the inventory and final receipts.

### C00 — Restore the current checkpoint and record changed contracts

**Surfaces:** Plan 08 checkpoint, `session/cache.rs`, catalog lifecycle fixture,
proposed ADRs, plan/oracle inventories, guidance and existing xtask acceptance tooling.

1. Fix the known cache type-complexity lint and the remaining maintenance argument
   reference; refresh the environment with `just py-sync` when code work resumes.
   Compile the latest targets and reconcile remaining compiler/rule/feature lint
   findings. Do not infer a clean workspace from the first catalog error disappearing.
2. Record decision changes before their implementation: resource accounting extension;
   semantic/dependency identity and generated argument contracts; checkpoint/fence
   commit behavior; the invocation/resident/durable reuse model. Follow the repository
   ADR route for hashing, metadata/Python/commit contracts, SHOULD or blueprint changes.
   ADR-0068/0069 are proposed, not accepted. Supersede accepted records when needed;
   no opportunistic blueprint edits or fabricated review acceptance.
3. Update execution guidance and current-function oracle inventory to the combined
   scope. Keep Plan 07 future simulator inventory excluded. Retain prior scoped receipts.
4. Rewrite the caching overview as a pinned, evidence-labeled capability map (F8),
   including D03 corrections. No second `CachingPolicy` authority or persistent cache
   subsystem survives in the implementation instructions.

**Exit:** exact restart fixes addressed; required decisions recorded; C01–C12 have a
single source/caller/deletion inventory. No integration campaign has begun.

### C01 — Separate semantic and resource identity

**Surfaces:** `pse-catalog/src/session/{config,snapshot_session,semantic_extent}.rs`,
`artifact/dependencies.rs`, `delta/attempt.rs`, engine profile/inspection/settings tests.

1. Derive complete read-back and semantic/resource views once from actual native state;
   route construction, clone/scope capture and information-schema checks consistently.
2. Remove resource-only fields from semantic hashes/receipts, with exact typed absence
   preserved. Keep unknown options conservative. Correct dependency `max_bytes` policy
   treatment without dropping its current admission requirement.
3. Separate invocation/attempt identity from owned implementation generation. Retain
   actual caller functions/rules/planners and safe refusal for unknown cold identities.
4. Replace obsolete whole-settings equality fixtures with exact semantic/resource
   assertions. Hard-pivot receipt versions where contracts change; no old receipt decoder.

**Isolated units:** equal semantics across spill/cache/pool differences; unequal time
zone/domain function/profile semantics; complete diagnostic read-back; scope capture;
resource denial still enforced on an otherwise eligible hit. Cross-process recovery
belongs to C12, and unrelated opaque generations must still refuse.

### C02 — Declare and generate the new contracts first

**Surfaces:** `pse-runtime` budget/report; schema `model/algorithm.rs`, relation table
policy, `catalog/publication.rs`, algorithm/reference reflection and generators;
Python settings and generated types; dependency comparison inputs.

1. Add the cache budget/resource fields, checkpoint policies, typed load requirements
   and consumption declarations from D04/D05/D07. One declaration owns each meaning;
   native policy builders and Rust/Python/reflection outputs are projections.
2. Replace dependent optional-field combinations in native dependency evidence with
   complete typed alternatives where appropriate. Distinguish provenance selection,
   implementation identity, whole-input dependency, consumed projection, absence and
   observation. Do not add an unvalidated nullable-column convention.
3. Generate projected argument access from relation/argument declarations; include
   consumption in algorithm/registry identity and expose it through native reflection.
4. Carry all residual Plan 08 E01/E02 field, integer, reference/quantity and collection
   audits into this schema cut. Remove orphan predecessor declarations; regenerate
   Rust/Python/docs/fixtures only through generators.

**Isolated units:** every dependency alternative and invalid combination; projection
paths/keys/absence; registry identity changes; budget overflow/zero/headroom; declared
checkpoint policy. No physical package/conformance journey is run during generation.

### C03 — Build the accounted shared native cache services

**Surfaces:** runtime budget/env/report/factory, catalog session factory/resources,
native CacheManager adapters, invocation cache, common provider storage binding.

1. Construct native cache instances from C02 policy; reserve capacity envelopes and
   validate actual limits/TTL using CacheManager read-back. Preserve caller cache
   factories and scoped pools; every factory for one deployment shares service ownership.
2. Add store/root namespace views over metadata caches without changing object paths
   passed to storage. Derived native session/runtime views share existing pool, disk,
   registry and functions; they replace only the namespaced cache view. Unknown provider
   namespaces use an uncached native read until bound, not unsafe shared metadata.
3. Supply bounded in-flight load coordination, cancellation, drop/error cleanup, entry
   size limits and aggregate metrics. Do not hold a map lock across IO or computation.
   Failed population leaves no reusable entry or permanent negative-cache fact.
4. Bound entry inspection; preserve distinct capacity, retained, pinned-reader,
   in-flight and query working-set accounting. Size cache policy for the deployment,
   with a declared cache-disabled mode and explicit query-memory headroom.
5. Complete Plan 08 dependency receipt decode/growth accounting work alongside these
   services. Native Arrow batch reservations alone do not account JSON metadata decode.

**Isolated units:** two stores with identical path/size/mtime remain distinct;
put/replace/oversize/eviction/TTL/resize/clear accounting; concurrent miss coalescing;
cancellation of one/all waiters; service reuse across factories/scopes; non-mutating
bounded inspection; reservations remain while a reader owns an evicted value.

### C04 — Close narrow Delta library integration gaps

**Surfaces:** workspace dependency declaration/lock and any source override, Delta
integration adapters, capability evidence and local skill build manifests.

1. Provide a CDF builder parameter for the actual shared namespaced native metadata
   cache/reader factory. Eliminate the unaccounted per-builder private cache in the
   selected PSE route. Keep CDF's native file selection and four-image semantics.
2. Expose retained native snapshot heap/materialized-file extents and ownership needed
   by C03/C06. Include kernel/log segment metadata and stats/action buffers; do not
   pretend that row count or serialized JSON length is native owned-memory size.
3. Expose bounded kernel CRC replay options and checksum creation through native
   delta-rs construction/post-commit hooks. Preserve actual commit-versus-acceleration
   outcomes; no private kernel reconstruction or PSE replay implementation.
4. Resolve the source deterministically: choose an upstream immutable revision with
   these interfaces, or commit a minimal reproducible source override/patch carrying
   upstream license/provenance. Refresh lockfiles, family metadata, skill indexes and
   capability-map pins together. No floating fork branch or external source-cache edits.
5. Compile interface probes and use isolated upstream/unit tests for the new seams.
   Full CDF/storage/checkpoint/recovery tests wait for C12. The dormant `delta-cache`
   Cargo feature is not a solution to these missing call paths.

**Exit:** callable, reproducible interfaces for shared CDF caching, honest snapshot
accounting and optional CRC acceleration. CRC is not advertised as eliminating file
replay. External contribution/publication is a separate action, not a blocking step.

### C05 — Prepare fixed-point strata once

**Surfaces:** `pse-rules/src/strata/{native,native_state,rounds,native_support,relational}`,
rule binding/delta/support helpers; catalog prepared execution/cache context.

1. Introduce one invocation-owned prepared stratum over typed native round sources.
   Prepare candidates, per-occurrence delta variants, support, emptiness and merge/
   difference plans together; retain real requirement/effect children and exact schemas.
2. Enforce D06 eligibility/properties before planning. Never provide first-round empty
   statistics as permanent facts. Implement source reset/epoch snapshot capture and
   require all previous round streams/tasks to settle before publishing next state.
3. Reset native plan state and round-dependent invocation caches. Preserve four-valued
   truth, stratification, multiplicity, additional support for existing facts, absence
   evidence and finite incomplete outcomes. Static inputs can share only when proved.
4. Add counters separating SQL binding, analysis, logical optimization, physical
   planning and execution per stratum/round. Delete the old per-round compile/rebind
   route once callers use the prepared program; keep no fallback planner implementation.
5. Finish remaining Plan 08 native function/field-transfer/support/default audits that
   intersect these plans; support units already passed are evidence, not work to redo.

**Isolated units:** empty-to-nonempty round source, join retained-side reset, round
cache invalidation, immutable sharing, epoch isolation/concurrent invocations,
cancellation/error release, profile rejection and stable properties. Multi-round
inference/domain journeys and performance assertions are C12 integration work.

### C06 — Share native Delta state and bound replay

**Surfaces:** catalog `delta/{provider,publication,write,publish,attempt,changes,maintenance}`,
load helpers, lease coordination and session-owned services.

1. Replace scattered fresh builders with one native opener carrying explicit target
   version, storage binding, load capabilities and cache service. Keep actual caller
   session/planner/runtime and `RequireSessionState` everywhere applicable.
2. Deduplicate publication control/member/input opens; metadata-only verification
   avoids eager file/stat materialization. Bound concurrent root/member replay under
   the deployment budget. Reuse actual snapshots through native provider builders.
3. Retain exact snapshots and use seeded native forward update on a private owner.
   Cache committed states returned by native operations where trustworthy; avoid
   redundant post-write replay while still verifying actual commit/attempt outcomes.
4. Enable declared checkpoints for control and applicable member commits. Integrate
   C04 CRC options for eligible paths, with cleanup disabled outside maintenance.
5. Implement D05's maintenance fence/invalidation under existing OS leases. Validate
   cache hits after acquiring read ownership, prevent late in-flight insertion after
   invalidation, and carry ownership into physical scans/streams. Cache entries themselves
   must not hold idle locks that block maintenance forever.
6. Complete remaining recovery/profile/history and control-transaction-index fixture
   coverage from Plan 08 E06/E09. Do not narrow missing-history refusal to make replay
   counters look good. Checkpoint and maintenance outcomes remain native typed effects.

**Isolated units:** load-capability matching, canonical root/store generation,
lookup/latest distinction, forward update selection and in-flight invalidation state
machine; typed checkpoint/fence policies. Actual Delta equality, concurrent checkpoint,
failed post-commit acceleration and two-process maintenance races wait for C12.

### C07 — Reuse decoded exact selections through native providers

**Surfaces:** native cache factory/planner, materialized/candidate/selected Delta
providers, common preparation, publication/TableReader and Python stream ownership.

1. Recognize eligible bounded pure selected scans without volatile expressions or
   hidden side effects. Preserve exact selection/layout/schema and row-policy meaning
   in eligibility/key derivation. Requirements and authorization still execute.
2. Lazily fill resident entries on execution and admit only complete successful results.
   Share one in-flight fill; choose native scan on capacity refusal. Support eligible
   provider reads as well as `df.cache()` so repeated Rust/Python table reads participate.
3. Return reservation-owned Arrow batches/streams and fresh read/fence ownership.
   Evict service references independently of live readers; charge live pinned bytes.
   A closed Python handle must not invalidate a separately owned exported stream.
4. Preserve native filter/limit residuals and truthful statistics/partition/order.
   Use native repartition only when C12 measurements justify it; add no handwritten
   round-robin cache dispatcher. Do not cache a filtered subset under a complete key.
5. Delete newly redundant eager materialization/read adapters with no semantic role;
   retain invocation spill as the native bounded execution mechanism, not durable memo.

**Isolated units:** projection/revision/schema/store isolation, no eager execution,
no population on partial/error streams, single-flight behavior, per-operation quotas,
external batch ownership after eviction and drop. Delta/Python cross-process journeys
remain deferred to C12.

### C08 — Make exact reuse sensitive to consumed values

**Surfaces:** generated algorithm arguments/native_dependencies, compiler algorithm
bindings, native expression dependency extraction, `artifact/dependencies.rs`,
`artifact::prepare_reuse`, Delta attempt/dependency readers and CDF normalization.

1. Apply C02 consumption contracts to actual existing algorithms, with restrictive
   projected argument access or whole-input fallback. Capture all plan-used columns,
   requirements, keys/multiplicity/absence and implementation-generation facts.
2. Produce native comparison plans over exact selected inputs, with fingerprints as
   prefilters and actual null-safe typed comparison where required. Keep physical
   version provenance distinct from the reuse equivalence obligation.
3. Replace unconditional whole-version/operation-instance equality on qualified reuse
   paths. Preserve complete input behavior for unknown algorithms/UDFs, source-changing
   policies, volatile observations and missing/expired evidence. Do not weaken retry
   identity or use a schema/plan hash as proof of equivalent computation.
4. Integrate complete bounded CDF as a change-locality accelerator; update pre/postimage,
   delete/insert, selection/filter/key and contract-change cases remain explicit.
5. Replace duplicate snapshot/context/memo invalidation remnants, if found, with this
   one typed relation/query path. Complete remaining Plan 08 source-edit/caller coverage.

**Isolated units:** undeclared-column access refused; value-only versus key/structural
change classification; NULL/multiplicity/zero-column/absence behavior; resource-only
settings do not alter result dependencies; unknown generation still refuses. Existing
algorithm structural-reuse versus clean-recompute oracles execute only in C12.

### C09 — Complete native scan policy, observability and extension coverage

**Surfaces:** execution settings/config classification, native provider stats/filter
hooks, typed metadata table functions, resource reports and existing inspection UI/API.

1. Declare native Parquet decode pushdown and predicate-cache ceilings; distinguish
   them from F2 dynamic-filter restrictions. Preserve row-group/page/bloom pruning
   and measure footer hint/reorder-filter options rather than inventing a scan engine.
   “Pushdown false” does not imply no pruning or unconditional whole-file decoding.
2. Account per-reader predicate-cache concurrency against request/deployment budgets.
   Capture actual read-back and provider behavior; a configuration entry alone is not
   evidence that the Delta/CDF reader used it.
3. Expose cache hit/miss/bypass/eviction, retained/pinned/capacity/in-flight bytes,
   replay/action/decode counts, planning-stage/round counts and post-commit acceleration
   outcomes. Reports distinguish metrics unavailable upstream from observed zero.
4. Use native registry/table functions for read-only inspection and provider lineage;
   do not make resource telemetry a semantic dependency that self-invalidates reuse.
5. Declare/test the ObjectStore decorator routing seam across log/checkpoint/data reads,
   using a small in-memory counting store for units. Keep mutable heads, listings,
   conditional writes and credential/store identity explicit; no remote service setup.
6. Complete Plan 08 ordinary-extension fixture mapping: tagged value/reference/invariant
   plus native expression/provider/parameterized relation through the same metadata,
   requirements/effects, persistence and inspection contracts without another roster.

**Isolated units:** settings/read-back, native cache injection, bounded non-mutating
inspection, semantic/resource separation, store routing and policy rejection. Actual
pushdown IO/row metrics and ordinary-extension journeys execute in C12.

### C10 — Complete every remaining Plan 08 implementation and deletion

**Surfaces:** the complete E00–E09 caller/schema/export/generator/fixture inventory;
compiler/numerics/domain boundaries, catalog and runtime, Rust/Python inspection/codecs.

1. Finish remaining field alternatives, collection masks/ordering/correspondence,
   integer/quantity/reference and generated-remnant audits. Completed schema cuts stay.
2. Finish native invariant diagnostics, all used function/field-transfer hooks and
   omitted-default/explicit-NULL/derived-value write coverage across Rust/SQL/Arrow.
   Remove unnecessary metadata restorers and relational Cell round-trips; retain useful
   generated storage codecs and actual parser/algorithm/FFI boundaries.
3. Audit all native hierarchy/factory/view/no-scan/parameterized/DML routes, actual
   caller state and truthful provider properties. Route product fixtures through common
   admission; retain intentional registry-free/upstream contract probes as such.
4. Finish exact numerical program/vector/Jacobian dimensions and identity, existing
   evaluator/Ipopt effects, cancellation/settlement and lifetimes. Keep explicit linked/
   unlinked capability behavior; no new numerical/simulator families.
5. Complete source/edit/rename before-image and identity-bound reference fixtures;
   compiler dependency/requirement composition; member/root recovery, interrupted and
   empty-attempt reclamation, control history/checkpoint and retained CDF cases.
6. Finish Python publication/logical descriptor/inspection/build/engineering consumers,
   exported Arrow ownership, cold-process fixtures, unsupported-codec refusal and
   read-only EXPLAIN. Refresh stubs and generated schemas through their generators.
7. Close deletion of remaining predecessor imports/declarations/exports/fixtures and
   the new superseded per-round/fresh-load/cache policy duplicates. Do not reintroduce
   Driver/store/memo/rule algebra. Delete unused dependencies/modules where justified;
   apply ADR requirements if removing a crate changes architecture.

**Exit:** all implementation and deletion obligations carried from Plan 08 are actioned;
all target integration fixtures exist and compile, ready for C12. Unit/static success
is explicitly not final scientific, storage, recovery or Python acceptance.

### C11 — Close the implementation inventory before integration

**Surfaces:** generated contracts/fixtures/stubs, docs/capability maps/decisions,
`08-current-function-oracles.md`, xtask architecture campaign, benches and test manifests.

1. Finish formatting, compilation, strict lint, required feature compilation and pure
   generated equality for the current target. Static checks are permitted throughout;
   avoid treating a recipe that executes journeys as a static check.
2. Assemble a receipt matrix linking each E obligation, F/N finding and Q obligation
   to concrete target fixture/test/benchmark names. Existing unexecuted fixtures are
   marked not_run, not passed. Preserve useful independent scientific/domain oracles.
3. Extend `architecture-acceptance` to cover this plan, with exact source/dependency/
   policy provenance, cache-mode and phase timings, IO/plan counters and resource data.
   Ensure no path silently raises test timeouts or substitutes a narrower campaign.
4. Add ordinary benchmarks for fixed-point rounds, Delta open/replay, decoded reopen,
   projection invalidation and selective scan behavior. Fixture creation and benchmark
   implementation land now; compiler/Delta/end-to-end benchmark execution waits for C12.
5. Review the complete source/caller/declaration/deletion inventory. **Do not start
   C12 until C00–C10 code and deletions plus this inventory are complete.** No repeated
   intermediate engine/conformance/Python campaigns are part of the sequence.

### C12 — Final integration, measurements and architecture acceptance

1. Run the target-only current-function campaign once after C11, including existing
   source/edit→math/compiler, numerical/linked-unlinked solver, publication/retry/CDF,
   exact/projection reuse, Rust/Python/Arrow cold ownership and maintenance workflows.
2. Execute cache-on/off and fresh-native recomputation comparisons using the same target
   architecture. Do not retain a predecessor planner/store to manufacture an oracle.
   Fix valid failures in the target, then rerun affected gates; repeat a full campaign
   only when intervening changes or unresolved coverage justify it.
3. Run the cost matrix below; distinguish preparation, execution, IO, decoding and
   resource retention from compilation. Choose numerical cache/scan/checkpoint defaults
   from these measurements and document limitations, including upstream transient heap.
4. Complete final family/generation/governance, features, Python quality/tests, native
   solver, docs/ADR/agent checks required by the current source. Baseline is zero;
   record exact commands, mode, counts, exclusions and source boundary.
5. Resolve Plan 08 Q01–Q14 and G1–G7 independently, plus the K obligations below.
   No cache optimization compensates for a validity/recovery/ownership failure.
   Close this plan only after all implementation, deletion and qualification evidence
   exists. Future simulator workflows remain explicitly unclaimed.

## Traceability and deletion ownership

### Caching review and additional findings

| Review finding | Disposition | Packages / acceptance |
|---|---|---|
| F1 semantic resource pollution | Implement first; also distinguish invocation from implementation identity | C01/C08; K01/K07 |
| F2 prepared rounds | Adopt with enforced reset eligibility, unknown changing stats and epoch cache isolation | C05; K04 |
| F3 snapshot/replay/checkpoint/load class | Adopt; cover members as well as control; private API and cross-process fence corrections apply | C04/C06; K05/K08/K10 |
| F4 resident relations | Adopt for exact admitted selections; provider reads and explicit cache share one service | C07; K06/K08 |
| F5 projection dependencies | Adopt with generated restricted arguments, membership and actual native comparisons | C02/C08; K07 |
| F6 cache budgets/observability | Adopt; native capacity reservation replaces naive callback accounting | C03/C09; K02/K03/K09 |
| F7 hidden persistence/second authority | Reject those overview tiers; enforce effect-free lazy caching and native durable publication | C00/C03/C07/C11; K11 |
| F8 overview evidence | Rewrite with pins, API/source evidence and corrections | C00/C04/C11; K11 |
| F9 CDF/CRC gaps | Required native CDF injection/CRC interface work; add snapshot extent seam; no universal O(1) claim | C04/C06; K05/K10 |
| F10 remote seam/predicate cache | Declare the general ObjectStore seam, implement explicit native pushdown/cache policy | C03/C09; K03/K09/K10 |
| F11 partitioning observation | Preserve truth; native repartition only on measured benefit | C07/C12; K06/K10 |
| N01–N02 namespace/accounting | Additional required correctness work | C03/C04; K02/K03 |
| N03 round reset/epochs | Additional required correctness work | C05; K04 |
| N04 cross-process invalidation | Additional required maintenance work | C06/C07/C10; K08 |
| N05 private snapshot/extent | Additional native interface work | C04/C06; K02/K05 |
| N06 selected residency/provider use | Additional selection and consumer coverage | C07; K06 |
| N07 projection enforcement/identity | Additional schema and native reuse integration | C01/C02/C08; K07 |
| N08 concurrency/inspection | Additional bounded sharing and diagnostics | C03/C06/C07/C09; K02/K09 |
| N09–N10 CRC scope/member checkpoints | Replay correctness and broader native checkpoint deployment | C04/C06; K05/K10 |
| N11–N12 properties/scoped budgets | Native partition and budget enforcement | C01/C03/C07/C09; K02/K06/K10 |

### All remaining Plan 08 scope carried forward

| Plan 08 step | Obligations retained / caching refinement | Owning packages |
|---|---|---|
| E00 / SP00 | Guidance, decisions, current-function oracle, acceptance command and deletion inventory; caching review evidence added | C00/C11/C12 |
| E01 / SP01–02 | Remaining field/alternative/collection/quantity/integer and generated-remnant audit; consumed-input/cache declarations use the same schema basis | C02/C10/C11 |
| E02 / SP03 | Exact selections, nested/composite references, absence/multiplicity and source support, owned streams; selections remain part of residency and reuse proof | C02/C05/C07/C08/C10/C12 |
| E03 / SP04–05 | Coherent math/numerical identity/dimensions, effect/lifetime/solver semantics and cost; caching never replaces numerical validity | C05/C07/C10/C12 |
| E04 / SP06/SP10 | Native invariants/rules/field transfer/defaults/derived values, remaining Cell audit; add prepared strata and projected arguments | C02/C05/C08/C10 |
| E05 / SP08 | Full native hierarchy, caller state/admission, effects/resources, truthful provider hooks and fixture routing; shared cache service lives here | C01/C03/C04/C06/C07/C09/C10 |
| E06 / SP07/SP09 | Complete artifacts, native member/root recovery, actual identity and cold refusal, control history; refine checkpoint/fence and post-commit outcomes | C01/C02/C06/C08/C10/C12 |
| E07 / SP10 | Source/edit/compiler caller/dependency audit and no old stage/memo replay; prepared execution and projected dependencies replace repeated work | C05/C08/C10/C12 |
| E08 / SP11 | Latest compile/lint fix, complete Rust/Python/codecs/inspection/deletion/ownership/generation closure | C00/C07/C10/C11/C12 |
| E09 / SP12 | Exact dependencies/freshness, bounded CDF, leases, maintenance, interrupted/empty attempts/history/resource gaps; integrate all cache validity edges | C01–C04/C06–C10/C12 |
| E10 / SP13 | Final current-function integration, extension proof, costs, quality/features/docs and independent Q/G closure | C11/C12 only after full implementation/deletions |

Nothing in this crosswalk reopens deleted legacy implementations. Original Plan 08
package exits, schema review traceability and independent high-level functional
assertions remain mandatory; caching modifies their implementation and proof, not
whether they must hold.

### Deletion ledger for this plan

| Replace/delete | Target / owner |
|---|---|
| Mixed semantic/runtime identity maps and duplicate capture/filtering | One classified native inventory; C01 |
| Unconditional whole-member/operation-instance reuse equality for qualified projections | Typed provenance plus consumption/implementation proof; C02/C08 |
| Per-round SQL binding/analyzer/optimizer/planner and candidate provider rebuild path | One native prepared stratum/epoch sources; C05 |
| Unscoped implicit cache defaults and repeated cache service construction | Declared, accounted, namespaced native service; C03 |
| Scattered fresh Delta loads and unconditional post-write reloads | Capability-aware shared native opener and committed-state handoff; C06 |
| Blanket `with_create_checkpoint(false)` on ordinary applicable commits | Declared native checkpoint policy; C06 |
| Private per-CDF metadata cache on the selected execution route | Shared cache injection through a reproducible Delta dependency surface; C04 |
| Proposed serialized-snapshot and effectful persisted `CacheFactory` tiers | Remove from guidance; use checkpoints and explicit Delta artifact publication; C00/C11 |
| Residual Plan 08 predecessor imports/declarations/generators/fixtures/exports | Delete or replace useful assertions with target fixtures; C10/C11 |

## Verification

The matrix below defines required proof, baseline **0**; current execution status and
named isolated-unit/static receipts are in the active checkpoint and execution inventory.
The cost matrix is **Measured** in [the measurement receipt](09-cache-measurements.md);
it does not establish complete functional qualification. Plan 08 receipts retain their
named earlier source scope; the original review does not certify this successor plan.

### Correctness and resource obligations

| ID | Required proof | Phase |
|---|---|---|
| K01 | Resource-only differences preserve semantic identity; semantic/function changes differ; retry still checks actual implementation identity and budget admission | Isolated units C01; actual retry C12 |
| K02 | Correct limits/read-back/accounting under TTL/eviction/resize, oversize, concurrent misses, cancellation and pinned readers; no doubled factory budget or unbounded inspection | Units C03/C07; resource pressure C12 |
| K03 | Same path/size/mtime across two stores cannot share metadata; actual Delta/CDF use the injected namespaced cache; mutable store reads honor validators | Units C03/C04; storage routes C12 |
| K04 | Empty→nonempty rounds, hash/cross-join reset, support/multiplicity, epoch isolation and failure cleanup; zero SQL/analyzer/optimizer/physical planning after stratum preparation under stable inputs | Units C05; existing inference journeys and counters C12 |
| K05 | Fresh versus retained/incrementally updated snapshots agree on selected rows and native Add-action contents; exact old/new versions and load classes do not alias; Metadata cannot satisfy Query | Units C06; Delta/checkpoint/CRC journeys C12 |
| K06 | Resident versus fresh selected scans agree including NULLs, duplicates, revision filters and schema/view modes; no partial-result population; stream ownership survives handle drop/eviction | Units C07; Rust/Python/Arrow journeys C12 |
| K07 | Proven value-only changes reuse eligible structural outputs; key/filter/policy/structural changes invalidate; unknown dependencies refuse; reused outputs agree with clean native recomputation | Units C02/C08; existing algorithms C12 |
| K08 | Same-process and two-process maintenance fencing defeats stale snapshot/resident hits, late fill insertion and interrupted cleanup; idle caches do not indefinitely pin files | Coordination units C06; actual Delta/process races C12 |
| K09 | Cache inspection is bounded/non-mutating; resource telemetry is not semantic identity; actual predicate-cache/pushdown/read metrics and unknown counters are truthful | Units C09; actual reader qualification C12 |
| K10 | Checkpoint and checksum errors after commit do not repeat data writes; missing CRC uses native replay; controls and member tails measured; caches off/on and partition choices compared | C12 |
| K11 | No hidden persistence, copied Delta replay engine, predecessor authority or unversioned evidence; ordinary extension uses existing provider/schema/effect boundaries | Static inventory C11; extension journey C12 |

### Measurements that choose defaults

Implement benchmark fixtures with the packages; execute these only in C12. Use ordinary
bench/test infrastructure and native metrics/counting stores. No wall-time threshold
is justified by the current evidence; record mechanism counts and absolute phase costs.

| Experiment | Dimensions / required output |
|---|---|
| `fixed_point_rounds` | Small/scaled existing strata, rule/head/round counts; one-time SQL/binding/analysis/optimization/physical-plan times and counts, each round's execution/support/merge cost, memory peaks |
| `delta_open` | Members M ∈ {1,8,32}, commits K ∈ {1,100,1000}; checkpoint interval, cold process, warm retained snapshots, incremental update, metadata/query load classes, concurrent opens; log/object requests, actions replayed and retained bytes |
| `resident_reopen` | Small/scaled nested/coherent members, repeated Rust/Python reads, complete/projected/revision-selected scans; decode counts, Arrow bytes, hit/bypass counts, pinned memory, cold versus warm costs |
| `dependency_projection` | Existing value-only and structural edits; comparison/CDF cost versus recompute, actual reused output groups, full independent output oracle |
| `delta_scan_pushdown` | Wide/selective and unselective scans; pushdown/cache cap/page indexes/bloom availability/footer hint, reader concurrency; bytes fetched, rows decoded, predicate memory, residual correctness |
| `cdf_and_crc` | Four image kinds, bounded/gapped ranges, shared CDF cache on/off; CRC present/absent/stale and bounded replay; metadata versus full-file enumeration costs kept separate |
| `cache_pressure` | Budget below/near/above working set; concurrent fills, cancellation, eviction with pinned readers, single/multiple native partitions; reserved/retained/peak RSS/spill bytes and correct refusal behavior |
| `maintenance_lifecycle` | Actual checkpoints/optimize/vacuum/log cleanup, protected old/CDF versions, interrupted/empty attempts, cross-process fences; root/member file/log growth and cold recovery |

Record toolchain, dependency revisions, machine, profile, dataset shape, native settings,
cache state and exact commands. Distinguish fresh process, fresh cache and OS page-cache
state; do not label an OS-warm file read “cold disk.” Do not flush host caches or run
`cargo clean` as part of a performance comparison. Compilation time is reported apart
from runtime. Native cache-disabled/recompute controls use the same target code.

### Final gates and command discipline

- During implementation: `just check-library`, `just check-package`, `just check`,
  pure generators, strict static checks and carefully selected `just unit-package`
  tests with explicit `pse-relations/force-validate`. Review what a selected test does.
- After C11: `just architecture-acceptance <new-directory>` with C12 additions,
  appropriate native solver and Python/cold-process checks, and the measurements above.
  Record baseline **0**, actual failure/skip counts and feature/profile per receipt.
- Complete Plan 08 **Q01–Q14**; independently reassess **G1 authority**, **G2 fidelity**,
  **G3 validity**, **G4 effects**, **G5 recovery**, **G6 transformation/reuse**, and
  **G7 truthful capability**. All are resolved for the in-scope combined implementation in the
  [acceptance review](../design_review/reviews/design_review_native-cache-pivot-acceptance_2026-09-17.md).
- Fix target failures; retain independent domain assertions, refuse unknown capability
  honestly, and never substitute increased timeouts, lint baselines or old code.

## Execution decisions and retained capability limits

| Item | Bounded resolution; not permission to omit target work |
|---|---|
| Delta integration revision | C04 selects an immutable upstream revision or reproducible minimal override; preserve one type universe and regenerate evidence. No waiting indefinitely for an external merge |
| Snapshot/transient allocation extent | C04 exposes native retained size; C03/C06 bound in-flight construction and document remaining upstream pre-allocation limitations before claiming resource coverage |
| Cache sizes, checkpoint interval and predicate-cache defaults | C02 declares all policies, C03 implements safe envelopes, C12 chooses recorded defaults from the cost matrix; no implicit process-size-independent 71 MiB budget |
| Prepared custom operator reset | C05 requires a truthful reset contract and stable properties. Unsupported profiles refuse specifically; no legacy per-round engine is retained |
| Proven consumer projections | C08 narrows only consumers with enforceable access/dependency evidence; all other current consumers retain exact whole-input behavior. This is conservative native execution, not an old compatibility path |
| Remote byte-cache backend | The registry/identity/conditional-IO seam is in scope now. Its implementation trigger is admission of a real remote root and backend-specific tests; distributed execution and remote destructive coordination remain unclaimed |
| Native cache repartition / CRC default enablement | Capability integration and truthful properties are required; select deployment settings from C12 results, not review speculation |
| Decision acceptance | Record required ADR/design changes via the sanctioned route. This draft does not accept ADR-0068/0069 or silently amend immutable governance |

## Outcome (recorded after implementation)

### What was built

**Implemented:** generated cache/consumption contracts and conservative semantic identity;
one accounted native cache service; reproducible Delta CDF/snapshot/CRC seams; prepared
strata; native snapshot/resident selection; projected CDF/endpoint reuse; maintenance
fences; receipt staging; typed inspection and Python ownership; target-only fixtures,
benchmarks and source-provenance campaign. Replaced paths and their final callers are
deleted. The inventory and independent acceptance review record final behavioral qualification.
**Tested/Measured:** see the final C12 receipt and exact ledgers above; baseline zero.

### A mistake made and corrected

Planning corrected the assumption that public `SnapshotIdentity` could be used as an
application key, that wrapping `put`/`remove` observes every native eviction, that
`reset_plan_states` itself enforces all reuse preconditions, and that CRC summaries
replace full active-file replay. It also made store namespaces, revision selections,
round epochs and cross-process maintenance explicit cache-validity obligations.

Implementation found and fixed an optimizer ambiguity when a CDF payload named
`value` contained its own `value` field: qualifying the input alias preserves the native
projection optimizer and exact bag comparison. An isolated NULL/duplicate regression
passes. CDF instrumentation was also corrected to bind each measurement to its actual
store, and feature compilation now uses the pinned solver interface instead of the
host's incompatible Ipopt.

Final qualification also exposed hidden null-parent bytes, repeated derivation heads,
reexecuted native construction constraints and a bypass of the existing registry
declaration cache. Repairs preserve visible-value refusal, complete provenance,
actual witness outcomes and exact external-declaration comparison. The last kernel
journey passes the original deadline; its failed runs remain in the ledger.

### Deviations from the plan, deliberate

The integrated plan deploys the review's useful cache planes directly, with native
capacity accounting and narrow Delta integration surfaces where public APIs fall short.
The remote byte-cache backend and new simulator workflows remain outside implementation;
their schema/storage seams are included. Integration and performance qualification
remain consolidated after all code and deletions, as directed by the maintainer.
