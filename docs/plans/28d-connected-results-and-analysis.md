---
title: Connected result queries and analysis
status: draft
date: 2026-10-05
adrs: []
review_sources: [docs/design_review/reviews/design_review_surrealdb-unified-simulation-substrate_2026-10-05.md]
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

## Work packages

| Package | Prerequisite and delivered behavior | Consumer migration and deletion | Status |
|---|---|---|---|
| D1 — Canonical result layout and minimal reader | R0, implemented A2 codec/index slice and C1 envelope. Declare row/block layouts once; read admitted terminal results by exact identities with coverage/interpretation. | Supply C2's actual retention reader and Rust result consumers. Remove matching Delta table-member reads as each consumer moves. | Scheduled |
| D2 — Native query and Arrow boundary | D1, A3 protected reads and C2 actual terminal runs. Implement selectors, native operations, gRPC provisional-row handling and Arrow export. | Migrate Python native bindings, workflow query calls, catalog inspection, examples and tests. Delete SQL convenience APIs and publication-handle dependencies. | Scheduled |
| D3 — Connected analyses and extension seam | D2 and B2 semantic dependency descriptions. Implement the two analysis journeys with exact source/method lineage and eligible library algorithms. | Move existing applicable graph/result inspection consumers; expose reusable typed operations, not a second semantic API. | Scheduled |

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

No D package is implemented by plan authoring. D1 follows R0, A2 and C1 and is the early
reader needed by C2. This document owns D progress/local evidence; the coordinator owns
US01/US02/US03/US04 finding dispositions. No optional integration is a hidden prerequisite.

## Outcome (recorded after implementation)

### What was built

Pending implementation and named evidence.

### A mistake made and corrected

Pending execution.

### Deviations from the plan, deliberate

Pending execution; consequential decision changes follow the decision route.
