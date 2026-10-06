---
title: Connected result queries and analysis
status: in-progress
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md, docs/design_review/reviews/design_review_plan-28-completion_2026-10-06.md]
scenario_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#representative-journeys]
---

# 28d: Connected result queries and analysis

## Responsibility and chosen boundary

This companion of [Plan 28](28-surrealdb-unified-substrate.md) owns exact connected selection,
scientific result layouts, Rust/Python query consumers, Arrow exports and derived graph
analysis. [28a](28a-canonical-substrate-and-revisions.md) owns identity/schema/codecs and
protected reads. [28c](28c-durable-execution-and-studies.md) owns attempt staging and terminal
admission. This document supplies their minimal canonical reader early, then develops the
connected interface. Shared decisions and finding dispositions belong to the coordinator.

The maintainer selected **native queries plus Arrow** on 2026-10-05. Retire
`Runtime.query(sql, result=..., publication=...)`, `TableReader::query` and the DataFusion
SQL convenience bridge. SurrealQL is not a drop-in SQL implementation. Update call sites,
examples and tests to native problem/revision/run/output selectors and documented native
query operations. DataFusion is retained only for a necessary remaining capability with an
actual consumer; it is not retained to keep the old query API alive.

Current consumers include the Python workflow boundary, runtime result/table interfaces,
catalog inspection and Arrow streaming. Preserve exact physical interpretation and the
typed Python contract without `typing.Any`. Change the public boundary through R0 before
production adoption. Semantic declarations remain at the registry owner; codegen emits
the required codecs, closed Python documents and boundary types.

## Connected selection and query meaning

The primary read selects a problem, immutable revision, run/attempt and output/coordinate
coverage, then follows canonical relationships to definitions, interpretation and results.
An explicit run identity is stable across process restart. Define `latest terminal` separately
from `latest usable` under a named policy; use C1's recorded sequence and terminal eligibility.
A later failed run cannot silently return an earlier result while claiming to be that failed
run. Read exactly C's closed admitted result-set descriptor, so late or abandoned staging
cannot enter the output through a broad result-set scan. Multiple revisions and attempts
are visible explicitly rather than flattened into a
single mutable problem document.

Native Surreal queries/functions provide structural selection, graph traversal, filtering,
projection and justified aggregation over canonical records. Domain query operations return
typed selections. An advanced read-only native query interface may expose database-native
records with explicit schema/interpretation; it must not imply arbitrary query values are
scientifically admitted result sets. Scientific result access uses exact codecs and recorded
coverage. Query execution does not grant mutation or terminal-admission authority.

Persist the resolved input selection and the interpretation of any query/function used to
derive an execution input or analysis. A query string over the current head cannot reproduce
an earlier run by itself. Source problem/revision/run identities and method/configuration
versions accompany derived outputs. Pre-simulation and post-simulation graph selections use
the same substrate without erasing these distinct provenances.

Streaming uses the qualified gRPC SDK route with bounded queues. Rows before statement-end
success are provisional. An Arrow stream surfaces final failure to its consumer, and any
durable derived analysis seals only after successful completion. Cancellation closes/drains
the stream; it cannot assume every server execution phase stops immediately. Export writes
use an explicit incomplete/complete lifecycle so an interrupted file is not announced as a
successful export. Multi-page reads pin their immutable selection through A3; they do not
mix current heads from different pages.

## Scientific result organization

Use native tables for operation-shaped scientific facts and graph relations for identity,
membership and provenance. Store ordinary scalar/output observations as rows selected by
run, result set, output and coordinate. Store long homogeneous trajectories as immutable,
bounded blocks keyed by run/result set, coordinate partition and sequence/time range, with
shape/order/coverage metadata. This avoids a record per dense array element while retaining
indexed discovery of requested ranges. Block size is an implementation parameter exercised
under E3's longer-trajectory journey, not a new global bundle size.

Use the same exact cell codec for rows and blocks. Ordered coordinates, units, basis, state,
quality/partiality and validity conditions remain available with the selected values. Missing
scientific values, database `NONE`/`NULL` and tagged raw diagnostic values are different meanings.
An exact f64 bit payload is authoritative; a mechanically derived finite numeric projection
supports native range/group operations where valid. A nonfinite or absent value does not
silently become a numeric zero. Preserve finite scientific fields' nonfinite refusal; only
explicit raw diagnostic domains can retain nonfinite payloads. Numeric projections never
replace the exact payload on export. Full-range u64 fields and selector bounds use A's exact
integral decimal codec, not an unchecked u64-to-i64 cast or floating comparison.

Indexes cover problem/revision/run lookup, admitted result-set selection, output coordinates
and block ranges. Query plans and projections must be inspected for representative shapes;
an indexed single-hop graph optimization is not a guarantee for every multi-hop request.
Broad aggregation can have large live state even with a small output. Use narrow native
projection and streaming for supported operations, and bounded Arrow/native analysis for an
operation the database cannot execute with suitable resources. Do not rebuild a durable
second analytical store or hydrate the whole problem for one requested output.

## Graph analysis and integrated tools

Provide two concrete composed analysis journeys: pre-simulation incidence/dependency traversal
to inspect a selected problem, and post-simulation upstream/downstream traversal connecting
selected outputs and available sensitivity evidence to their source declarations. Topology,
semantic compilation dependencies, equation incidence, execution ordering and sensitivity
evidence are distinct relation meanings. Topological reachability is not a quantitative
sensitivity result. Analysis requiring a numerical or graph algorithm uses an existing library
over an operation-shaped selected graph when Surreal's native operations do not supply it.

Persist derived analysis under its method/configuration and exact source selection. It can
be queried later alongside the run without becoming new authored problem authority. Changing
source data invalidates current applicability; old immutable analysis retains its recorded
meaning. New graph algorithms can add an analysis family and method binding without changing
run sealing or native solver ownership. New physical concepts can still require compiler work.

Expose the supported domain operations through the repository's actual Rust/Python consumers.
Native functions, schema introspection and an optional domain API can reuse those contracts.
Add MCP/search/live capabilities only for a concrete consumer: CRUD cannot bypass admission,
search ranking cannot establish physical eligibility, and live notifications are hints followed
by authoritative reads. If LIVE is adopted, resubscription and explicit registration cleanup
belong to the adapter. Experimental WASM scientific execution and generic tooling installations
are not required for Plan 28 completion.

## Remaining completion, read and analysis slices

Use the [capability investigation](../design_review/evidence/plan-28-surrealdb-capabilities-2026-10-06/README.md)
within D's existing exact selector and Arrow boundary. Finding status stays at the
[coordinator](28-surrealdb-unified-substrate.md#finding-dispositions); assembled state stays in E.

| Slice and prerequisite | Delivered boundary and consumers | Meaningful targeted completion |
|---|---|---|
| D2 completed selection/export, after A1 completion control | Exact run/attempt/result-set/output/coordinate selection yields scientifically interpreted rows or a failure. Awaited bounded queries may remain; incremental gRPC rows are provisional until successful statement completion. `canonical_results.rs`, runtime connected results and Rust/Python inspection/export share the rule. | A late statement error, cancellation or missing terminal completion must prevent a staged prefix from becoming a completed export/eligible artifact. Also cover empty successful selection and normal multi-page selection. Use the actual chosen SDK adapter, not a hypothetical transport defect. Delete any alternate convenience route that bypasses exact completion. |
| D2/D3 composed protection, after A3/C2 lifecycle correction | A multipage reader retains exact source/result protection; an analysis admits its own retained root before releasing source protection. Copies decoded into Arrow carry their own DataFusion memory reservation after DB protection ends. | Interleave retirement/reclamation with an actual multipage read and with analysis root admission, then reopen the admitted analysis. Require complete exact rows or explicit refusal, no torn selection or released source gap, and escaped decoded-buffer accounting. Separate existing race tests are not this composed control. |
| D3 native graph representation | Typed regular edges retain method, sequence, kind/provenance and occurrence identity. Direct ID/projection and bounded native traversal support actual dependency/incidence/lineage consumers; algorithms and physical sensitivity retain their existing owners. | Inspect one actual traversal predicate and index path. If INLINE edge fields improve its locality without changing meaning, generate them through the registry and verify equivalent selection/lineage; otherwise retain regular relations with the decision explained. LIGHTWEIGHT endpoint-pair edges cannot replace metadata-bearing edges. Any timing gain belongs to E4. |

The graph representation decision is bounded by actual consumed query shapes; it is not a
prerequisite for unrelated result/recovery controls or a mandate to add speculative graph
features. SurrealDB recursion/shortest path does not establish numerical sensitivity,
centrality/community analysis or full Cypher semantics. Preserve method and derivative axis
lineage. Surrealist is optional human diagnosis, and neither GraphQL/MCP nor a restored SQL
convenience route is necessary for these consumers.

Fresh linked Python controls need E2's actual installed producer association and version-first
readmission. Exercise unsupported-version refusal before malformed-current-shape decoding,
exact wire ID comparison and eligible persisted replay. Current source fixes are not fresh
execution evidence. D's earlier positive controls remain local; E3 owns end-to-end retention,
completion, Python and restart composition.

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| D1 — Canonical result layout and minimal reader | R0, implemented A2 codec/index slice and C1 envelope. Declare row/block layouts once; read admitted terminal results by exact identities with coverage/interpretation. | Supply C2's actual retention reader and Rust result consumers. Remove matching Delta table-member reads as each consumer moves. | Implemented; targeted native controls passed |
| D2 — Native query and Arrow boundary | D1, A3 protected reads and C2 actual terminal runs. Implement selectors, native operations, gRPC provisional-row handling and Arrow export. | Migrate Python native bindings, workflow query calls, catalog inspection, examples and tests. Delete SQL convenience APIs and publication-handle dependencies. | Native and focused linked scientific consumers passed; eligible deployment-receipt control pending |
| D3 — Connected analyses and extension seam | D2 and B2 semantic dependency descriptions. Implement the two analysis journeys with exact source/method lineage and eligible library algorithms. | Move existing applicable graph/result inspection consumers; expose reusable typed operations, not a second semantic API. | Native journeys, bounded visitor and focused linked analysis consumer passed |

D1 is an early working slice, not a claim that D2/D3 are complete. Root coordinates result
declarations and Python boundary generation. C owns sealing; D cannot implement a shortcut
that publishes raw database rows as a successful scientific run.

## Verification

**Proposed acceptance:** touched-package compile checks and targeted force-validating units
exercise each mechanism. Python transport units use the recipe-owned linked environment when
required. [28e](28e-rebuild-retirement-and-qualification.md) owns assembled testing once all
functional work is implemented.

Use explicit expected coordinates and scientific values for S04/S05/S07. After a process
restart, select output Y for problem X at two revisions and two attempts; verify the exact
chosen run, unit/basis/state and coverage. Distinguish latest terminal failure from latest
usable success. Exercise signed zero, finite-field nonfinite refusal, declared diagnostic
payloads, integer boundaries and domain missing
values through both native reads and Arrow reconstruction. Compare with independently authored
fixtures, not only round trips through the same codec.

Cancel a large streamed selection, force statement-end failure after rows, and interrupt an
export. No successful derived artifact may contain the provisional prefix. Run a multi-page
selection while retention attempts deletion; its protected identities remain stable. For
block ranges, verify ordering, gaps and partially covered slices without fetching all blocks.

The graph-analysis controls distinguish incidence/topology from sensitivity and verify method
lineage for stored analyses. Representative native query plans must support the required
indexed lookup/projection; do not claim generic graph or aggregation scaling. Performance
measurements belong to E4 and do not substitute for selection correctness.

## Checkpoint and next step

D1/D2 have working exact protected native reads and local Arrow export. Dense trajectories
retain original scientific IPC blocks with output-group indexes; sparse scalar indexes retain
exact bits, missingness and original row coordinates. Decoder preflight is bounded against
the actual generated schema before allocation. The real nested diagnostic schemas exposed
an overly small transport-metadata limit, which was repaired without bypassing scientific
field validation.

Native result selectors and Arrow streams replace publication handles and SQL convenience;
registry reflection uses its declared Arrow relations directly. Targeted native controls pass
for exact historical selection, output-group selection, interrupted export and result-buffer
protection. The minimal reader's fixture teardown now refuses live returned owners and drains
automatic release tasks before removing its isolated database.
The copied Arrow boundary separately retains decoded allocation ownership. Its focused
control passed with exact original rows, reader-gated retirement, reclamation and database
teardown while a returned array remains live, and reservation release after its final drop.
Database protection belongs to the renewable reader; decoded arrays need no subsequent
storage access or indefinite database pin.

D3's actual pre-simulation incidence/dependency and post-result sensitivity/provenance journeys
pass their native control. It retains the reported derivative in its original scientific row,
records exact method/configuration/source/attempt lineage, reopens the same graph, and blocks
result retirement until analysis withdrawal. Source edits produce a new analysis identity while
the old graph retains its recorded revision. Reachability remains distinct from quantitative
sensitivity. A bounded contribution visitor replaces the pre-analysis temporary full
dependency inventory. Its targeted control preserves the original projections and exercises
early stopping and cancellation; the actual analysis consumer rerun also passed.

Focused linked Python controls now exercise canonical reopening and both analyses,
registry/cache reflection, an unusable study predecessor and repeated retained study
occurrences. The retained study boundary now propagates its original scientific `RunId`
separately from the opaque canonical lookup keys. Authored solve/warm-start and completion
projection consumers also passed.

The remaining selected scientific consumers exposed a shared preclaim failure whose original
diagnostic was masked, and an assertion that expected incidental sample-major ordering from
dense storage. C repaired the lost diagnostic. Its preserved cause identified a physical
source schema larger than the result-block limit even for an empty table: canonical revision
inputs had incorrectly used the scientific-result codec's byte bound. Their explicit source
purpose now uses A's existing object bound with the same strict borrowed preflight, while
scientific results retain their smaller bound. Actual required source schemas and nonempty
authored rows passed the focused source codec controls; oversized source payloads remain
refused by the result reader. The trajectory consumer now checks exact rows
in the declared stored `(symbol_id, sample)` order and compares its export with those retained
rows; independent analytic values remain unchanged. The refreshed linked extension passed
the process, dynamics/transient fit and three SCIP cases with their original numerical and
resource-limit assertions. The missing-key diagnostic assertions also exposed the original
registration cause directly, instead of accepting absent persistence.

The public Python deployment-receipt control is prepared but waits for the actual eligible
current producer capture and matching imported extension. It checks actual deployment
admission, an unchanged independently recreated revision, original outputs and reopening
after clearing the preparation cache. Distinguishing reconstruction from fresh semantic
admission remains the existing strict native controls' responsibility; the public boundary
does not expose a replay-origin signal.
The K4 study measurement adapter now reopens exact per-point native results and uses the shared
isolated-fixture teardown barrier; the parent benchmark compilation passed and E4 measurements
remain unrun.
Assembled testing waits for all companion functional scope.

## Outcome (recorded after implementation)

### What was built

**Implemented:** exact native run/attempt/manifest, latest-problem and output selectors;
bounded original Arrow IPC result blocks, scalar/dense selection indexes, terminal progress,
atomic local export, and persisted scientific analysis graphs. Rust and Python consumers now
use canonical identities and native queries. Public SQL convenience, publication handles and
their replaced fixtures are removed. Scientific result meaning and original row coordinates
remain separate from graph reachability and storage lookup keys.

**Tested (2026-10-06):** the targeted native result-block and connected-analysis selection
passed 12 controls through `just unit-native-package`, with explicit canonical/native solver
features and force-validation. The named
`canonical_decoded_arrow_survives_reader_drop_and_result_reclamation` control passed against
the selected isolated server: exact signed-zero/null/finite values remained usable after
reader drop, storage reclamation and Runtime drop, then released their final allocation.
Focused linked Python scientific repairs passed five selected process, dynamic/fitting and
limited-result controls; `just py-unit-native -n 0 python/pse/tests/test_generated_contracts.py`
passed 27 contract controls. The baseline was zero failures. The actual eligible Python
deployment receipt association/reopening control and assembled E3 qualification remain pending.

### A mistake made and corrected

Canonical source IPC inherited a result-block limit too small for the actual authored
declaration schema. Source and result purposes now use their own existing finite limits,
with shared strict framing/preflight; result limits were not broadened. Reopening also
preserves the scientific RunId separately from opaque canonical lookup keys.

### Deviations from the plan, deliberate

Fully decoded copied Arrow arrays retain accounted memory without an indefinite database
pin. Read leases protect further storage access, while final array drop releases allocation.
Analysis methods retain original incidence and reported sensitivity provenance; reachability
is not a derivative or rank claim. Scaling and timing remain E4 obligations.
