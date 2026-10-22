---
title: Protected result batches and native selection
status: in-progress
date: 2026-10-21
adrs: []
doc_role: plan
doc_owner: docs/plans/49-library-native-remediation.md
doc_topics: [storage, libraries, design, validation]
---

# Protected result batches and native selection

Execution state and pause handoff: [the coordinator](49-library-native-remediation.md#pause-handoff--requested-closeout-2026-10-21).

This companion develops R4 and F06 from the
[integrated review](../design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md)
§4.2 and its [recommendation audit](../design_review/reviews/audits/design_review_in-use-library-remediation_2026-10-21_audit.md).
[Plan49](49-library-native-remediation.md) owns mutable package status, source coverage,
rule decisions and finding dispositions. The package explanations below own local target,
consumer integration and acceptance, without duplicating that ledger.

**Proposed:** canonical guarded appends retain exact metadata per admitted IPC record
batch; DataFusion plans selection and conservatively prunes whole batches. Remove the
per-scalar-row SurrealDB locator and its index writes after every reader/writer migrates.
Keep the protected file, complete relation admission, exact coordinates, scientific bits
and one decoder. A narrower projected decoder, Parquet format transition and literal-set
pruning are separate bounded inquiries, not assumed benefits of this target.

RC04's selection-contract decision is recorded at GOV01. GOV02 routes any metadata,
commit or interpretation change before dependent implementation. The selected MX 1.0
store target is developed in [49a](49a-surrealmx-canonical-store.md); this companion consumes
its guarded append, lifecycle, protected-read and fenced migration products. It does not
redefine those authorities or make scientific qualification follow from database success.

## Baseline and focused foundation assessment

Blueprint §19.2 owns result meaning; §20.1/§20.2 own exact publication; §20.4 owns protection
and retirement; §21 owns public Arrow access. Core 3.4, efficiency heuristics and Process
Simulator 1.5 govern the assessment. Read complete scientific rows and original coordinate
meaning as constraints on physical optimization, rather than treating per-row locators as
a required representation.

Current source has a reusable protected boundary:

- `workflow/result_projection.rs` maps the four scalar relations to owner/partition keys,
  builds locators and implements `scalar_predicate` with Arrow kernels.
- `workflow/result_snapshot.rs` freezes inventories, retains one `ResultRead`, decodes with
  `FileReader<ProtectedFile>`, verifies full schema/rows and admits `FieldCheckedBatch`.
- `workflow/connected_results.rs` supplies typed readers, full and selected snapshots,
  original-coordinate selection/review-pack reconstruction, live/protected export paths.
- `pse-operations/src/canonical_execution.rs` and `canonical_results.rs` enforce bounded
  append/replay and exact set/manifest membership; `canonical_retention*` and generated
  retention functions remove the corresponding metadata under protection.
- `pse-schema/src/catalog/substrate.rs` declares block/output/row metadata and indexes;
  `pse-codegen/src/codegen/surreal_execution.rs` and `surreal_retention.rs` emit writers,
  replay checks and bounded collection. Supervision/maintenance also consumes those tables.

The old path writes one `canonical_result_rows` record and three index entries per scalar
row. Existing block rows already describe batch coverage, but scalar coordinate statistics
are absent. The provider reports filters unsupported. Selection verifies each stored
locator against the original row and applies handwritten kernels. Replacing those writes
is a structural reduction; the effect on total latency and 16-lane contention is unmeasured.

The foundation assessment keeps full protected decode/admission. The existing decoder
checks the complete relation schema; `FileReaderBuilder::with_projection` cannot silently
return a narrower batch as a fully admitted relation. Pruning a whole IPC record batch can
avoid its decode; sub-batch range selection alone cannot. Retain exact permission and
integrity checks even where statistics imply an empty answer.

### Selected native mechanisms and source scope

| Mechanism | Realization and limit |
|---|---|
| Arrow `lexsort_to_indices` and `take` | Sort new scalar batches by their declared relation key before assigning authoritative positions; preserve source values/bits. Existing imported coordinates are not reassigned. |
| DataFusion 55.1 `PrunableStatistics::new(Vec<Arc<Statistics>>, schema)` | Present exact admitted block statistics as containers. Unknown fields/statistics conservatively retain the batch. |
| `PruningPredicateBuilder::new().with_file_schema(...).try_build(...)` | Build pruning from the physical expression compiled by the actual session. `try_build` consumes `Arc<dyn PhysicalExpr>`, not the public logical `Expr` directly. |
| `TableProviderFilterPushDown::Inexact` | Expose filters while leaving final residual filtering in the engine; an early limit cannot underfill the requested filtered result. |
| Arrow IPC `FileReader::set_index` | Visit surviving record batches inside the existing decoder and its protected file/ownership scope. |
| DataFusion `PruningStatistics::contained` | Optional literal guarantees only if a cheap existing exact owner summary can supply them; standard `PrunableStatistics` returns unknown. |

The integrated review points to exact installed sources for DataFusion 55.1.0
`datafusion-common/src/pruning.rs`, `datafusion-pruning/src/pruning_predicate.rs` and Arrow
IPC 59.3.0 `src/reader.rs`. These were inspected for construction, exact-statistic extraction
and batch positioning. Upstream references are the
[55.1.0 pruning builder](https://docs.rs/datafusion-pruning/55.1.0/datafusion_pruning/struct.PruningPredicateBuilder.html),
[55.1.0 statistics adapter](https://docs.rs/datafusion-common/55.1.0/datafusion_common/pruning/struct.PrunableStatistics.html)
and [59.3.0 IPC reader](https://docs.rs/arrow-ipc/59.3.0/arrow_ipc/reader/struct.FileReader.html).
Web rendering of these API pages was unavailable during authoring; the exact installed
release sources, rather than a latest-version example, supplied the interface evidence.

## Target dataflow and retained meaning

### Exact batch metadata is a product of admission

The registry owns which relation keys and output fields support selection. Generate those
declarations for producer, operation and reader use instead of repeating field lists.
Four scalar sort/key shapes must be represented:

| Relation | New-batch order | Selection identity/partition |
|---|---|---|
| Solve variables | `(run_id, step, symbol_id)` | Symbol owner and optional step. |
| Solve constraints | `(run_id, step, row_id)` | Constraint-row owner and optional step. |
| Fit parameters | `(run_id, parameter_id)` | Parameter owner, no invented step. |
| Fit observations | `(run_id, experiment_id, observation_id)` | Observation owner and experiment partition. |

Dense trajectories retain their symbol-major path and sample/coordinate semantics. No
universal `symbol_id` layout replaces these meanings. Sorting takes place before logical
row positions are published. Preserved payloads, snapshots and review packs continue to
name their original `(batch, offset)` and half-open relation ranges. For existing data,
compute statistics without rewriting/reordering its immutable payload; min/max remains
sound even when old rows are unsorted. This avoids a new per-row permutation structure.

For each admitted record batch, derive exact row/null counts and supported key min/max
from the actual admitted batch. Attach these to its existing block metadata and bind them
to relation/schema, payload digest, file/batch identity and interpretation. Finite value
statistics, where used, must state exceptional-value handling and remain conservative.
A replay equality check proves metadata is unchanged, not that the first supplied values
were true. Only the controlled admission producer creates trusted exact statistics.

A changed payload, schema, interpretation or generation invalidates the statistics premise.
Schema migration distinguishes old and new metadata explicitly; it must never label missing
old statistics exact. Legacy preserved batches are either freshly reconciled against their
protected bytes before cutover or retained conservatively with unknown statistics. This is
one target reader over mixed statistical knowledge, not a legacy locator production path.

Guarded append commits block metadata and the exact closed payload descriptor together,
under the existing generation/fence and replay request identity. SurrealDB determines
which batches exist and are visible; DataFusion cannot extend that universe. Live-sample
files keep their acknowledged sealed-file-per-batch shape. Ordinary packed IPC can contain
several record batches; pruning granularity remains a record batch rather than a whole file.

### Canonical predicates run under the actual engine

Lower `output_results(field, minimum, maximum, missing)` once to DataFusion expressions
through the registry-owned field declaration. Keep all existing modes:

| Request | Required selected rows |
|---|---|
| All three options absent | `TRUE`: retain null, NaN and both infinities as well as finite values. |
| `missing = true` | `IS NULL`; combining this with a bound remains invalid. |
| `missing = false` | Non-null finite rows only. |
| Either numeric bound | Finite rows meeting inclusive lower/upper comparisons; absent `missing` does not disable the finite requirement. |
| Nonfinite/reversed bounds or unsupported field | Explicit refusal under the existing input contract. |

Use a conditional finite guard (`value > -inf AND value < +inf`) only in modes that require
finite values. An unconditional guard would regress the unfiltered path. Signed zeros
compare numerically equal while returned arrays retain original bits. NaN payloads,
infinities, subnormals and missingness retain their scientific diagnostic meaning.

Plan filters through the same DataFusion session, optimizer, functions and resource owners
used by the runtime. Compile the logical predicate to the physical expression required by
`PruningPredicateBuilder`; build conservative container pruning against exact block facts.
Statistics failure or unsupported pruning means keep the batch and evaluate the residual,
not an empty result or a less complete selection.

Report pushdown `Inexact`; preserve predicate columns even if absent from requested output.
Residual filtering occurs before satisfying `LIMIT`. A false-positive first batch cannot
consume the whole output allowance and hide matching rows later. Apply final output
projection after surviving full-schema decode/admission. SQL ordering/selection must retain
original row coordinates; do not substitute a newly filtered row number as the stored offset.

### Protected reading, cancellation and retained allocations

Use one `ProtectedResultDecode` and charged decoder turn. `FileReader::set_index` skips
unselected batches, then the existing file integrity/schema/extent and field-admission
checks establish the surviving batch. Apply requested row ranges, predicate and output
projection while preserving original coordinate mappings. Predicate truth is not a new
payload authority. Pruning does not bypass authorization, ticket freshness or frozen
manifest membership.

Full snapshots, bounded selected pages, native SQL registration and exported Arrow streams
share this behavior. Keep actual metadata-response and retained-buffer grants through
transport work and unpolled readers, including cleanup on cancellation. A full-file integrity
read can remain necessary even when only one batch is decoded. State that residual work
when explaining an expected read benefit; neither key pruning nor IPC seeking promises
zero I/O for the skipped batch.

New batches pay one sort/statistics pass before publication and remove row-shaped guarded
metadata writes. Reads pay block metadata/planning and decode only surviving batches.
High overlap or unordered numeric values may prune little; min/max cannot reject every
absent owner ID. Batch size affects selectivity, publication acknowledgement and memory.
Tune it only inside existing payload/latency bounds with representative evidence.

### Native file caches need actual source premises

F06 is separate from protected IPC pruning. `NativeCacheService::unbound_config()` safely
sets native file cache capacities to zero; current `bind_state*` helpers have no reached
caller in the inspected production paths. `ProtectedFile` IPC decode bypasses those native
file-source caches, while preparation/analysis cache-service consumers remain active.

RES05 first establishes any reached DataFusion file source, its source-generation owner
and operation lifetime. Bind its bounded scan state through `bind_state_with_generation`
using an immutable identity or actual lifecycle generation, including same path after
content change. Stable `ObjectStore` Arc or registration identity alone is insufficient.
Keep unqualified stores unbound. If no real native-file consumer remains, remove only the
unused file-cache adapter branch and its tests/configuration; retain the active cache
service. This is a bounded evidence-driven choice within F06, not blanket cache removal.

## Execution packages

| Package | Required input | Delivered behavior and replacement |
|---|---|---|
| RES01 | GOV01 RC04 decision, GOV02 route and STO03 shared generator ownership | Registry selection keys/fields and exact admitted batch statistics bound to payload identity; replace scalar metadata construction. |
| RES02 | RES01 block contract and current protected decoder | Inexact DataFusion pruning/filtering with complete protected admission; replace unsupported pushdown and handwritten predicate production path. |
| RES03 | RES01/02 plus all scalar/dense/portable consumer contracts | Migrate every reader/writer, remove locator relation/index/replay/collection machinery after replacement controls pass. |
| RES04 | Superseded by maintainer fresh-data decision | No historical batches/coordinates require backfill; new result behavior remains RES01–RES03/RES06. |
| RES05 | Reached native-file-source inspection and source-generation owner | Bind generation-correct native file caches or remove only unused adapters; preserve active preparation/analysis service. |
| RES06 | Functional result/store integration and inquiry outcomes | Selected native/Python/SQL/export/retirement acceptance; feed QUAL01–QUAL05 without a separate full campaign. |

### RES01 — Produce exact statistics with the batch

Extend registry declarations for all four key shapes, output fields and batch facts. Sort
new scalar rows before authoritative positions, admit them and compute bound metadata from
that exact product. Existing dense sample ordering stays separate. Migrate append request
encoding, operation bounds and replay equality through shared generator ownership with
STO03. Do not hand-edit generated contracts or copy field lists into templates.

Controls cover each relation's key/partition, optional step/experiment, null counts,
empty batches and exceptional values. A changed digest/schema/interpretation rejects or
invalidates facts. Corrupt supplied min/max must not acquire `Exact` through unchanged replay.
Compare statistics against independently chosen input rows, rather than an encode/decode
round trip alone. Remove replaced scalar metadata builders when all producers have moved.

### RES02 — Prune whole batches and preserve residual semantics

Implement the native physical pruning route inside the existing provider. Keep the original
predicate behavior as a differential oracle while building the replacement; the integrated
review's standalone 63/63 receipt did not exercise every production mode and is not new
product acceptance. Remove the old production kernels after the replacement controls pass.

Controls cover all-None selection, missing-only, finite-only, inclusive/reversed bounds,
±0, ±infinity, signed/payload NaNs and subnormals. Compare exact selected bits and original
positions. Exercise output projection excluding predicate columns, unknown statistics,
unsupported filters and a limit with false-positive early batches. A pruned whole batch
must avoid native decode; a surviving subrange may still decode its whole batch. Corrupt
protected file/schema and expired permission must still fail through the original boundary.

### RES03 — Complete consumer migration and delete locators

Move full/selected snapshots, scalar output readers, typed Rust access, DataFusion SQL,
Python tables/streams, export/live/review selections and portable review-pack reconstruction
together. Dense trajectories continue their existing path with shared protected admission.
Migrate durable append, batch append/replay, restore/import, retention/retirement, operator
inspection and generated maintenance callers, including script references to locator tables.
[49g](49g-contracts-maintenance-and-tooling.md) owns shared maintenance caller edits; this
package supplies their replacement result contract.

Once replacement tests pass and all callers move, delete `canonical_result_rows`, its key/
selection/batch indexes, per-row encoder/decoder and locator verification, append arguments,
immutable-locator replay scans, locator collection loops, scalar predicate refusal branch,
row-locator-only extent limits and fixtures. Keep real encoded metadata, file and admitted
result-size bounds; deleting a locator cap does not make output unbounded. An expression-only
partial change that retains locator writes does not complete RES03 or R4.

### RES04 — Historical backfill superseded

On 2026-10-21 the maintainer explicitly selected regeneration of all existing data.
There is no retained-data conversion/backfill or STO07 preservation boundary. RES01–RES03
remain functional result redesign work, independently scheduled after or alongside the
fresh store. Tests of new result coordinates/protection remain required; no legacy
payload compatibility fallback is introduced. This is a scope supersession, not a pass.

### RES05 — Settle the reached native cache integration

Inspect actual file registration/scan consumers before editing the service. For a reached
source, wire bounded state at that source's planning lifetime and prove equal-path unequal-
generation reads cannot reuse stale listing/statistics/metadata. Verify immutable-generation
reuse and finite cache admission under concurrent requests. If no reached consumer exists,
remove the unused adapter only and validate surviving preparation/analysis operations.
Do not claim a hit-rate benefit from object-store identity or from protected IPC reads.

### RES06 — Integrate acceptance and resolve bounded inquiries

Run the relevant assembled Rust/Python/native SQL, result export, live-to-terminal,
cancellation, retirement and protected reread journeys after functional scope, coordinated
with [49h](49h-qualification-and-design-publication.md). Measure complete append-plus-select
work at equal payload sizes, batches, durability, predicate selectivity and lane count.
Report structural locator-write removal separately from measured latency/memory/throughput.

Three inquiries remain distinct: a projected decoder needs a sound narrower admission/trust
contract; `contained` needs cheap exact existing owner summaries; Parquet page/bloom/
`ParquetAccessPlan` needs a justified format transition including conservative IEEE pruning
and live-file publication. Record an evidence-based adopt/non-adopt disposition and trigger
at the coordinator. None blocks the selected IPC locator removal unless implementation
reveals it is required for supported behavior. Do not introduce a replacement membership
index or parallel decoder merely to obtain an apparent free win.

### Inquiry dispositions during execution

**Interface-checked, 2026-10-21:** I02–I05 and I16 have source-based no-additional-adoption
dispositions at the coordinator. The production-source census found no DataFusion listing,
Arrow or Parquet file-source consumer of the unused adapter. Protected results use their
own permission-bound IPC reader cache; the reached Parquet authoring importer uses a
direct Parquet reader. The unused adapter is removed; surviving preparation/analysis
cache acceptance remains part of RES05. This establishes neither cache hits nor a speedup.

The pinned [DataFusion55.1 pruning contract](https://github.com/apache/datafusion/blob/55.1.0/datafusion/common/src/pruning.rs)
requires complete literal containment; existing min/max/null/row facts do not supply an
exact multi-value set. The [Arrow59.3 IPC reader](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-ipc/src/reader.rs)
projects by omitting unselected fields, so footer schema equality alone cannot admit
their omitted batch contents. Keep unknown containment, full protected admission and
engine residual/projection. Parquet's actual [access-plan mechanism](https://github.com/apache/datafusion/blob/55.1.0/datafusion/datasource-parquet/src/access_plan.rs)
does not itself justify replacing the current protected format. Revisit only the concrete
consumer and evidence triggers recorded in I03–I05; no format or projection experiment
was run for these dispositions.

For I16, Arrow59.3 [buffer ownership](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-buffer/src/bytes.rs)
and [pool reservations](https://github.com/apache/arrow-rs/blob/59.3.0/arrow-buffer/src/pool.rs)
track allocated backing with infallible reservations, while DataFusion's
[Arrow adapter](https://github.com/apache/datafusion/blob/55.1.0/datafusion/execution/src/memory_pool/arrow.rs)
uses growing reservations. These cannot replace fallible pre-allocation admission or
imported lease custody; adding them over current charges would duplicate accounting.
Existing generated HeapUsage and shared result-fact owners remain selected. The inquiry
does not establish a universal Arrow overhead cost or reject future native ownership
integration; a concrete compatible lifecycle and an actual comparison must justify it.

## Verification and handoff

**Proposed verification:** implementing packages use focused `just check-package` and
`just unit-package` routes for affected operations/runtime/engine/codegen owners, with
explicit force-validation for Arrow correctness closures. Regenerate declaration changes
with `just codegen`. Scope-end integration/hygiene/manual checks belong to QUAL01–QUAL05;
package boundaries do not trigger repeated static or assembled campaigns.

**Interface-checked/source-derived basis:** current producer/decoder/consumer sources,
review/audit and exact installed DataFusion/Arrow interfaces identified above. Historical
standalone probes retain their original scope. **Not run during authoring:** tests,
builds, probes, code generation, data conversion or product qualification. No numerical
or complete-system benefit is established by this document.

The coordinator retains RC04, R4/F06 and dependent rule/finding decisions. Fresh STO07 activation does not consume RES03/04. Result redesign activation still
requires RES03's working replacement and its own targeted/assembled evidence.
On closure, enduring result-selection and metadata meaning moves to blueprint §19.2/§20.1/
§20.2/§21 through GOV02. The coordinator records final acceptance and limits and retires
eligible material under ADR-0096.
