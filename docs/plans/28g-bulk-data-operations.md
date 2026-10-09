---
title: Bulk data operations and sufficient protected crossings
status: in-progress
date: 2026-10-07
adrs: [ADR-0164]
review_sources: [docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md, docs/design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md, docs/design_review/reviews/design_review_parallel-execution-architecture_2026-10-08.md]
scenario_sources: [docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md#4-revealing-scenarios]
---

# 28g: Bulk data operations and sufficient protected crossings

## Responsibility and affected foundations

This companion owns the efficiency corrections to physical data units and operation contracts.
It develops PE02/PE03 and comparable source/analysis/authoring variants for [28a](28a-canonical-substrate-and-revisions.md),
[28c](28c-durable-execution-and-studies.md) and [28d](28d-connected-results-and-analysis.md).
Those documents retain revision, attempt, terminal, query and lineage semantics. The
[coordinator](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension) owns
coverage/dispositions; [28e](28e-rebuild-retirement-and-qualification.md) owns qualification.

The inspected product baseline is `5260a3e9a3cecd69b6358ab86d4e917ae2508b29`. Current source
has exact codecs, bounded self-contained IPC, scalar indexes, guarded effects, closed
descriptors, selected grouped acquisition, escaped-buffer leases and Arrow C streams.
They are suitable foundations. PE02 is scalar-index count imposing six/eight-row scientific
blocks, not a per-cell RPC. PE03 is a full-payload success echo and two protected crossings
per block. Existing append already bulk-inserts cell arrays.

Source inspection also found physical-document publication/reopening per chunk, whole edited
bundle byte copying, separate analysis node/edge pages, and sequential recovery metadata
reads. Their repeated work is visible; the largest permissible effect/lifetime unit needs
the focused decisions below. A shallow `Arc`/`Bytes`/Python dictionary clone is not by itself
a duplicated payload. Query syntax and bounded returned pages do not establish examined work.

## One bounded physical admission policy

Separate scientific rows/coordinates, IPC blocks, index pages and committed effects. Give
each a useful physical unit under one authoritative admission policy, rather than letting
an incidental scalar count choose every unit. Derive runtime and generated bounds from the
owning declarations, never independently maintained magic constants in each consumer.

Retain the existing initial limits: a self-contained scientific IPC block is at most
512 KiB, execution metadata uses the existing 128 KiB allowance, and the selected transport
profile initially caps gRPC messages at 4 MiB. The prospective native WS wire budget and
completed-response bounds are owned by [30a](30a-native-websocket-rpc-and-operation-lifetimes.md#bounded-realization-and-completed-reads);
protobuf accounting is not a proof of native-frame extent. T/D consumers migrate with
that adapter, retaining IPC/metadata declarations and exact effect units rather than
introducing another transport policy. Existing encoded-bound receipts remain gRPC evidence.
Apply the metadata allowance to each append's derived
index/descriptor envelope. Account for complete encoded request overhead and decoded/live
extent independently. A caller may request a lower admitted extent; exceeding the generated
server ceiling requires a deliberate interpretation/profile change, not a local bypass.

The writer chooses a useful contiguous row prefix that fits actual payload, encoded index
metadata, complete wire overhead and resource admission; exact maximality is not required.
Prepare row-metadata extents once, select a sufficiently large candidate, then encode/admit
through the actual Arrow/codec owners with bounded deterministic splitting retries. Reuse
prepared metadata rather than repeatedly rebuilding discarded candidates. Reserve builders
and scratch before allocation. Split an overlarge candidate deterministically and
refuse a single row that cannot fit. Empty completion is explicit. Small progress observations
may still flush promptly under the existing latency policy; batching never waits indefinitely
for a byte target or blocks terminal drain.

T0 establishes actual encoded metadata admission, including IDs, tags, bits and variable
strings, and a separate generated defensive cardinality ceiling. Derive that ceiling from
the positive minimum valid record extent or another justified finite schema bound; it bounds
pathological cardinality and replay work. Do not choose ordinary batches by dividing the
allowance by the maximum permitted per-cell string size, which would recreate tiny blocks
for compact identities. The 128 KiB append-index allowance is a proposed reuse of the existing
completion/descriptor metadata policy, not a claim that it already governs those indexes.
T1 uses the defensive ceiling for server checks and exact replay queries; the current
64-cell guard and `LIMIT 65` must change together. The server checks actual admitted fields
and bounds, not a client assertion of its own size. Do not substitute an unbounded comparison
or trust a byte cap to bound unlimited zero-sized records. The same declaration supplies
runtime policy, native guards, count ceilings and boundary controls.

Keep authoritative IPC rows, exact bits/nulls/signed zero, derived-index correspondence,
contiguous ordinals and frozen result membership. Payload and all its indexes become visible
atomically under the attempt fence. Closing ingestion still precedes terminal reconciliation;
large native work and ingestion stay outside the terminal transaction. Changing these physical
limits must follow version-first interpretation admission and controlled artifact regeneration
where persisted interpretation changes. No legacy reader or second result writer is introduced.

## Sufficient operation crossings

### Append acknowledgment

The append operation receives the complete expected immutable batch and index metadata.
The server performs current fence/ingestion checks and exact recorded-request comparison on
replay, then returns a compact acknowledgment identifying the admitted effect and immutable
descriptor. It does not echo IPC bytes on normal success. Existing request/result identities
and framed digests may supply sufficient fields; define them once through the generator.

The client verifies that the acknowledgment names this exact result set, ordinal, request and
coverage. A matching digest supplies identity, not scientific correctness. Rows/indexes retain
their independent admission. An identical acknowledged retry resolves the same effect;
changed bytes or metadata at that effect identity refuse. Lost acknowledgment settles through
the existing operation owner before retrying, with exact readback only where uncertainty
requires it. No new retry wrapper or scientific re-solve follows a storage acknowledgment loss.

### Protected reads

Use one protected bounded operation to obtain an admitted block's metadata and payload.
Connected selection already supplies block identities/extents; consume that plan instead of
performing another metadata RPC for the same block. Group compatible exact blocks only under
one combined payload/metadata/reservation allowance and the same immutable protected selection.

Metadata-first planning remains necessary where extent admission needs it. Reserve bounded
wire buffers before acquisition and decoded extent before Arrow decoding. The fused operation
rechecks membership, interpretation and current protection; it cannot turn a cached lease into
fresh authority. Preserve ordinal order and payload/index comparison before typed publication.
Rows before successful statement completion remain provisional. Cancellation closes/drains
unread work while escaped Arrow arrays retain their own allocation leases.

These contracts cover source ingress, result append and analysis acquisition through common
grouping/admission mechanics. Their effect receipts remain distinct: source revision publication,
result attempt ingestion, analysis activation and study claims are different transitions.
A generic transport success response cannot replace any of their completion guarantees.

## Broader consumer migrations and open choices

| Area | Target and decision needed before changing effects |
|---|---|
| Solve/fit/dynamic scientific tables | `result_projection` and `result_blocks` share the new physical admission policy. Scalar indexes no longer force an IPC stream/transaction per 64 cells. Preserve schema order, finite/nonfinite domains and complete sample coverage. |
| Reopened results and progress | `canonical_results`, connected reads, durable progress and Rust/Python wrappers consume sufficient read/ack contracts. Retain the distinction between live observations and sealed durable history, and their current completeness meanings. |
| Physical source documents | `Operations::sources` groups logical IDs, manifests and exact source payloads through existing selected acquisition. Preserve manifest version, package/path/chunk identity, lengths/digests and current protection. Publication should stage bounded groups under one complete package effect and publish the final revision coherently, rather than committing a head transition per chunk when no consumer needs it. T0 must settle current stage/edit cardinality and interruption recovery before extending that operation. |
| Edited authored bundles | `OwnedDocumentSet::edit` reuses unchanged parser owners but currently copies each document's bytes. Carry immutable byte ownership into the existing loader so changed bytes alone allocate anew. Preserve exact edit conflicts, spans, full required package hydration, cancellation and leases; this does not assume all scientific checks become incremental. |
| Analysis graphs | Existing append accepts nodes and edges, but endpoints must exist and activation requires full membership. Group compatible node/edge pages where that ordering holds. Keep separate stages where effect dependencies require them; do not create an alternate graph publisher. |
| Worker recovery metadata | Determine whether point/attempt/study/run reads can be acquired as one coherent operation under existing identities. Changing claim/predecessor/fence facts require fresh authoritative reads. Never merge distinct occurrences merely because their source is equal. |
| Relations/engine/Arrow export | Reuse checked batches, selective inputs, streaming and multiplicity-aware gather reservations. Replace collection/materialization only for an actual consumer that does not need it. Existing Python Arrow C streams remain the preferred result boundary. |

T0 also assesses exact SurrealDB 3.3.0 plans for active, historical, supplier, empty and skewed
selections. The streaming planner folds eligible constants, so optional OR alone is not a
scan diagnosis. Inspect examined work, ordering and maintenance costs before adding compound
indexes, decomposing predicates or selecting target-driven acquisition. Generate changes at
the registry/codegen owner and preserve complete negative inventories and revision intervals.

Follow a real analytical demand before proposing DataFusion provider/pushdown integration.
If that consumer benefits, preserve schema/null/order, filter/limit/projection semantics,
protected selection, late errors and allocation ownership. Otherwise improve existing native
selection/Arrow streaming. The [release-backed evidence](../design_review/evidence/production-execution-efficiency-2026-10-07/surreal/README.md)
and library skills inform these choices; Context7 is used for additional API documentation.

## Remaining native access-path decision

The enhancement review's [storage investigation](../design_review/reviews/design_review_plan-28-remaining-design-enhancements_2026-10-07.md#library-fit-and-remaining-investigations)
adds T6 to existing T0/T3/T5. A small result, bounded transfer or one RPC does not establish
bounded examined work. Existing [SurrealDB 3.3.0 evidence](../design_review/evidence/production-execution-efficiency-2026-10-07/surreal/README.md)
supplies hypotheses about optional predicates, compound indexes, ordered selection and correlated
supplier reads, not measured current-query scaling. This is a bounded decision package, not a
database/backend redesign or automatic adoption of every SDK feature.

Select actual bound operation families: selected-source supplier/name/membership acquisition
in `canonical_selection`, exact run/attempt/output/range and block acquisition in
`canonical_results`, and selected node/edge/result acquisition in `canonical_analyses`.
For each material choice, inspect predicates, projections, ordering, generated indexes and how
exact IDs/revision protection reach the query. Compare selective, empty and skewed selections
against representative retained extent; distinguish necessary closure work from a repeated or
unrelated scan. Use version-scoped EXPLAIN where source cannot settle the access path; ANALYZE
executes the query and belongs only in an admitted disposable/controlled case. No production
mutation, plan-output golden, latency gate or broad query census is implied.

Choose the simplest fitting access path: retain the bounded current operation when adequate;
otherwise use exact bindings/explicit projection, a justified compound index or composed native
selection at the existing owner. Include write/delete/retirement maintenance and index storage,
not only read benefit. A changed declaration is generated at its existing schema owner with
`just codegen`; no hand-edited SurrealQL or parallel migration authority. Preserve complete
negative/membership premises, ordering, byte/resource admission and protected visibility.

Streaming is conditional on an actual consumed benefit. `stream_items` rows are provisional;
statement/transaction completion remains required and item-count channels are not byte bounds.
A proposed long stream must account for reservation, snapshot/protection, cancellation, final
failure and draining unread data/escaped Arrow buffers. Default to the current bounded indexed
result path if it already supplies the required operation; no broad streaming migration or new
retry wrapper is scheduled merely because the SDK exposes it. Scientific kernels and native
iteration remain outside database transactions.

| Package | Decision and delivered scope | Acceptance and subsequent work | Status |
|---|---|---|---|
| T6 — Selected access paths | Source-backed assessment of bound source/result/analysis families; resolve only consequential query/streaming questions with current release documentation or bounded plan observations. Record adopted correction versus retained design, supported selection/extent, consumer and reopen trigger. | Selective/empty/skewed exact selection, interpretation/order, protected retirement/expiry, byte admission and final completion. Implement confirmed repeated-work corrections with their actual callers and generated declarations; delete displaced paths. T5/D1/D2 consume that scope; E4 measures any gain. | Investigation complete; result cursor correction and protected paging controls pass; retained choices and triggers recorded; E3/E4 pending. |

T6 must record a reasoned outcome before T5 closure. A retained design needs an observable trigger
such as a newly supported predicate/extent or measured disproportionate acquisition; it is not an
unowned indefinite follow-up. New scientific meaning or a material rule change returns to its
decision owner before dependent implementation. The other confirmed corrections need not wait
for unrelated query questions.

## Parallel protected-operation composition

T7 supports Parallel F01/S01/S03/S06 through
[A4's canonical decision](28a-canonical-substrate-and-revisions.md#parallel-canonical-protection-and-contention).
It owns bounded physical crossings, not guard safety or another retry authority. Existing
product stage/ack convergence and exact append replay must be reused where they already meet
the consumed contract.

For operations A4 identifies as unnecessarily repeated, compose metadata/payload selection or
equivalent reachability publication under one valid protected selection and useful byte-bounded
unit. Preserve every selector and its completed dependency receipt. Keep source activation,
product/root admission, result append/close/terminal, analysis activation and study claim as
their actual separate effect/recovery identities; one RPC is not permission to merge them.
Bulk payload work remains outside short terminal/guarded decisions. A grouping change that
widens the transaction or prolongs a conflicting protection must justify the complete-operation
tradeoff rather than claiming fewer crossings alone.

Do not echo large immutable content solely to establish acknowledgment. Normal success can
use the existing exact compact acknowledgment; uncertain delivery requires the actual owner's
settlement/readback. Changed replay refuses, and no storage acknowledgment loss restarts a
scientific solve. Provisional rows remain provisional until successful final statement completion.
Decoder and escaped Arrow ownership retain their current bounds.

| Package | Inputs and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| T7 — Parallel protected crossings | A4's selected atomic predicates and operation identities; B6 lifetime where consumed. Adopt useful bounded grouping/coalescing only for demonstrated equivalent operations. | Migrate canonical source/product/result/analysis consumers selected by A4; remove displaced helpers/round trips after controls. Race read/publication with protection release, expiry and retirement; test exact/changed/lost acknowledgments, empty/partial/skewed selection, byte admission and final-statement failure. A reasoned retained path needs a concrete scope and reopen trigger. | A4 selects local staging-RPC pacing; existing compact acknowledgments and protected grouping retained; composed serving controls pass, E3/E4 pending. |

T5's consumer reconciliation includes T7 and D1/D2 where their protected acquisition changes.
Existing T6 access-path evidence is retained; no second query census, streaming replacement or
mandatory new index is introduced by this extension. E4 measures whole-operation and store
waiting effects separately from candidate counts or RPC-count mechanisms.

## Work packages and dependencies

| Package | Inputs and delivered behavior | Migration, deletion and focused acceptance | Status |
|---|---|---|---|
| T0 — Settle physical units and variants | Coordinator coverage and existing codecs/stage/read owners. Set metadata/count extent, largest legal effect groups, actual caller demand and query/library applicability. | Record adopted versus justified retained units and the evidence selecting each. Query-plan observations are bounded to decision-changing predicates; no wholesale profiling prerequisite for PE02/PE03. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| T1 — Payload and index admission | T0's result bound plus A's generated codec/interpretation route. Implement one byte/extent-based scientific writer and derived server/replay bounds. | Migrate solve, fit, trajectory and all indexed result producers. Test narrow/wide/empty/single-overlarge rows, boundary splits, exact coverage/indexes, fence/close races and identical/changed replay. Delete the 64-cell-driven framing path and its obsolete assumptions. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| T2 — Compact exact append acknowledgment | Working current append ownership and settled sufficient request/descriptor identity; independent of increasing block size. | Change generator and actual Rust callers together. Test successful first/repeated effects, mismatched descriptor, changed request and committed lost-ack recovery. Remove full payload success echo/readback comparisons only after the replacements establish their guarantees. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| T3 — Composed protected acquisition | A3 protection plus exact result descriptors and reservation planning; independent of T1/T2 except changed stored-bound compatibility. | Migrate connected and direct block readers, grouped physical source reopening, and applicable recovery metadata consumers. Test retirement/expiry/cancellation, selected ordering, missing/changed blocks and final statement failure. Delete replaced singleton metadata/payload chains. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| T4 — Source and analysis effect grouping | T0's source-stage and node/edge decisions; working shared admission/grouping slices. | Migrate source package publication, unchanged-byte authoring reuse and applicable analysis publication. Test interrupted stages, exact final revision visibility, changed replay, endpoint ordering and activation completeness. Delete redundant revisions/copies/loops only where no required consumer remains. | Implemented; focused controls passed; enclosing E3/E4 acceptance remains. |
| T5 — Data consumer closure | T1–T4, completed T6 decision, working adopted corrections and T7/A4/B6/D1/D2 protected-operation migration. Reconcile coordinator coverage and every confirmed applicable caller. | All confirmed variants consume the shared target; justified distinct effects remain explicit. Compose complete source→solve→retained result→selected analysis/Arrow journeys in E3, then measure in E4. | Prior T1–T6 source reconciliation complete; parallel consumer source reconciliation complete; assembled E3/E4 pending. |

T1/T2/T3 can proceed independently once their own contracts are available. Acknowledgment
correctness and read correctness remain separate obligations even if larger blocks reduce
both call counts. Root owns shared schema/generators, policy declarations and integration.
Regenerate changed declarations through `just codegen`; never edit generated codecs/functions.

## Verification and evidence limits

**Interface-checked:** source inspection establishes the reviewed framing/echo/two-read
mechanisms and additional source-chunk/copy leads. Selected acquisition, checked Arrow
ownership and streaming already provide reusable foundations. No query timing or preferred
index access path was newly measured during plan authoring.

**Proposed acceptance:** compile affected packages and run narrow actual mechanism units
through `just check-package`, `just unit-package` and appropriate native recipes, with explicit
force-validation. Include null/signed-zero/large integral IDs and independent exact original
rows/indexes. Unit tests cover splitting, identity and ownership without unrelated solvers.
Server/recovery/stream journeys belong to E3 after the complete functional extension, not an
integration run for every package. Use the existing canonical recipes for their selected scope.

E4 observes total source acquisition, append/read and analysis/export operations, with narrow
and wide rows, trajectory/sample growth, selective/empty/skewed requests, cancellation and
reopen. Separate setup/schema installation and teardown from publication/query work. Report
wire bytes/crossings or construction counters only when they answer the relevant hypothesis;
no new telemetry framework or fixed RPC quota is required. Exact original transport checks
do not impose bit-identical independently solved scientific outcomes.

## Checkpoint

A4 attributed the observed conflict to same-owner staging and selected pacing of the
single bounded staging RPC, retaining native global guards and exact stage/readback identity.
T7 therefore retains existing compact append acknowledgment, grouped protected result/source
acquisition and analysis activation; no identity merge, longer transaction or second retry
owner is introduced. Actual protected multipage read-to-analysis handoff, exact-selection
retirement races, resumed cleanup and analysis completeness controls pass. A newly observed
conflict attributed to a distinct owner, or growth in a complete protected acquisition,
reopens that concrete path. T5/D1/D2 enclosing consumer qualification remains E3.
The earlier T0–T6 implementation and access-path controls below keep their evidence limits.

T6's populated current-v2 SurrealDB 3.3 investigations retain bounded source/numeric/analysis
paths and their complete projections. A proposed supplier compound index did not improve the
chosen correlated path, and globally forcing numeric/start indexes caused counterexamples.
The adopted block-page correction puts an explicit resume lower bound before the immutable
upper bound: the selected 1000-block fixture emits 10 index candidates instead of 1000 for
the same late resumed block, and after-last emits zero. Its actual protected paging control
passes. Scalar/dense pages use the resume bound first only inside the selected interval, preserving
the stronger selected range for preceding or disjoint cursors. The 24 equivalence cases
and actual protected scalar/dense ingestion/paging controls pass; selective late-page
fixtures emit 10 rather than 1000 candidates. Terminal and disjoint counterexamples keep
their bounded original projections. Candidate
counts are not disk-fetch, latency or scientific qualification claims; E3/E4 own composed
behavior and measurements. Reopen retained source/numeric choices when a supported selective
journey demonstrates examined/returned growth that a premise-preserving alternative improves.

T0–T4 functional source and focused actual-server controls are implemented. T5 reconciles
confirmed consumers with the shared publication/acquisition owners; enclosing assembled
consumer qualification remains at E3. Registry-derived limits now govern Rust, generated SurrealQL and supervisor
policy under substrate interpretation v2. Native CBOR metadata accounting is distinct from
gRPC's actual protobuf transport. A defensive cardinality ceiling follows the minimum valid
metadata object; it does not choose physical blocks. Scalar publication frames admitted payload
and metadata once per bounded row window, reserves live scratch, and no longer splits at64cells.

Append acknowledgments carry compact exact request/batch identity while server replay compares
full content. Protected result acquisition returns descriptor and payload together. Source
publication groups admitted stages into one final revision; reopening groups selected sources
and unchanged authored bytes share their existing allocation/parser owners. Analysis pages group
ready edges with nodes while retaining endpoint and activation requirements. Study recovery uses
an initial consistent metadata snapshot and retains fresh effect/fence validation.

The retained page/cursor loops supply complete bounded acquisition and renewal, so their64-entry
page sizes are not removed indiscriminately. Current connected consumers need Rust protected
streams, not a speculative TableProvider. Units and actual native controls cover large IDs,
null/signed-zero, wide rows, ownership, replay and grouping; Focused server controls exercise exact append/replay, protected acquisition, physical staging
and graph activation. E3 supplies the assembled caller and recovery acceptance. No timing or query-index claim is made.
