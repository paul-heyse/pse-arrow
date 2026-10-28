---
title: "Plan 58e: Retained data and delivery"
status: draft
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [storage, design]
---

# Plan 58e: Retained data and delivery

## Purpose and basis

**Proposed:** deliver O08 and the data portion of O09 from the
[principal functional review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md#O08),
with O15 retained as a conditional interoperability inquiry. A Rust operator selects exact
retained results, composes native relations and exports useful data without Python objects,
mathematical initialization or provider preparation. The
[coordinator](58-rust-owned-system.md) owns cross-package decisions and assembled acceptance;
this document owns E01–E04. This draft schedules work and authorizes no production changes.

The inspected source baseline is `0659343f123fefa2371ac7d1659e3af7f75f9949`, on 2026-10-28.
Blueprint §20.2, §20.4, §20.6 and §23 own publication, protected result lifetime and operational
meaning. Core 3.4, its efficient-architecture companion and Process Simulator 1.5 guide the
foundation assessment. Selection/delivery preserves physical metadata and partiality; it
does not establish a new scientific result or relax original admission.

The [hard-cutover review](../plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md#I02),
published at `c3b7f136d`, supplies I01/I02/I10/I12/I14. The coordinator owns adoption status;
this companion owns protected payload interpretation/delivery, while D owns canonical execution
lineage, result references and schema applicability. No additional store or payload-format migration
is selected by moving durable workflow meaning into SurrealDB.

## Existing foundations and target

`crates/pse-runtime/src/workflow/result_snapshot.rs` already provides full and selected protected snapshots,
`session_state()` and `table_provider()`. `SnapshotProvider` advertises `Inexact` filtering,
prunes while retaining DataFusion residual predicates, and withholds an early limit when
filters could yield false positives. `workflow/cohort_provider.rs::CohortProvider` advertises
`Unsupported` arbitrary filter pushdown and checks the selected deployment pool. Preserve
these different contracts rather than promising uniform exact pushdown.

The extraction is substantive: `ResultBlockDecode` retains `SourceServices`, accesses pool,
registry/admission and native state through preparation, and cohort state also captures that
service bundle. `SourceServices` in `workflow/services.rs` contains preparation, canonical
and publication capabilities. Passing the whole bundle under a narrower name would leave
O08 incomplete. Extract the actual read/selection/decoding/session/custody dependencies and
move existing providers onto them; no mandatory new crate or universal data service follows.

Compact [control](58d-canonical-control-and-connectivity.md#d01) needs connection/auth/bounds;
retained data adds protected selection, receiving validation, decoding and delivery custody.
Scientific execution separately adds model/math/native services. Direct Rust consumers use
typed requests, snapshots and Arrow batches without Python serialization. Authoritative
completed payloads remain sealed Arrow IPC. Parquet is an explicitly selected export
derivative; no database backend or authoritative payload migration is presumed.

Persisted definitions/revisions, attempts, realized points and useful calculation observations
must identify these same result references. A later comparison of repeated studies reads their
original canonical lineage and protected payloads rather than rebuilding a separate execution
history. Native task graphs and every runtime event are not analytical requirements. Keep
scientific calculation/progress observation and diagnostics; build observations and derivative
build-specific queries/alerts deliberately retire through B/F.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [E01](#e01) | planned | Result selection and decode owners | Retained reads construct from actual data capabilities without SourceServices/math. |
| [E02](#e02) | planned | Native result and cohort relation owners | Composable protected relations preserve configured state, pool and residual semantics. |
| [E03](#e03) | planned | Payload allocation and export owners | Native selected delivery/export preserves custody through early close and partial failure. |
| [E04](#e04) | planned | Data consumer and integration owners | Consumers migrate, conditional transports are dispositioned, and H owns assembled evidence. |

<a id="e01"></a>
## E01 — Native retained-data construction

Consume [D01's narrow connection boundary](58d-canonical-control-and-connectivity.md#d01)
where a canonical reader needs it, plus [T01's decision route](58-rust-owned-system.md#t01)
and [T02's operation inventory/contracts](58-rust-owned-system.md#t02).
Reuse `canonical_results::ResultRead`, capture grants and original
manifest descriptors. Supply immutable registry/schema interpretation, receiving admission,
configured native state, deployment pool and activity/cancellation owners directly to the
retained-data operations. Do not obtain them by constructing preparation/math services.
Share the existing `ProtectedResultDecode`, `ResultBlockDecode` and IPC receiving logic
between ordinary terminal reads, selected readers and captured cohorts.

A Rust client reopens a terminal attempt, requests one relation/owner/field/range selection,
and receives exact selection authority before payload discovery. Full snapshots retain their
complete frozen inventory; selected snapshots freeze the complete selected universe, not an
unread tail. A stale grant, reclaimed original, incompatible schema or unavailable captured
member remains an explicit outcome. Live workbench pages cannot substitute for a frozen cohort.
Delete replaced data-only Runtime/SourceServices constructors and migrate their consumers.

Targeted controls construct/select/decode with mathematical initialization unavailable,
compare selected versus independent full-read results, preserve original capture attribution,
and reject invalid schema/grant/issuer without broad source preparation. Keep explicit
force-validation on every Arrow-consuming correctness closure.

Use D's established schema/store capability rather than rebuilding its whole applicability proof
for every read. External payloads, grants, captured member identity and concurrent reclamation
still require their real admission checks; Rust typing does not prove remote bytes are valid.
Consume [D05](58d-canonical-control-and-connectivity.md#d05)'s interpretation/version boundary for open
readers across migration and refuse unsupported interpretation explicitly rather than silently rebuilding
mathematical services.

<a id="e02"></a>
## E02 — Protected native relational composition

Use E01's data context for `ResultSnapshot`, `SelectedResultSnapshot` and frozen cohort
providers. Keep their original configured `SessionState`, functions and shared deployment
memory pool available to native queries; a fresh default `SessionContext` must not silently
replace that environment. Retain protected original manifests through execution and returned
buffers. Use DataFusion `TableProvider` and `DataFrame::execute_stream` for cross-attempt
joins, windows, aggregates and comparisons rather than client-side hydrate/filter loops.

Preserve projection, inexact snapshot pruning plus residual filters, and cohort unsupported
pushdown. Do not apply limit before an inexact residual can reject rows. Selective decoding
must preserve physical quantity/unit meaning, exact member attribution and unavailable/empty/
pending distinctions. Wrong-pool queries refuse. Public composition may expose the adopted
Arrow/DataFusion types inside this boundary without exposing mutable scientific execution.

Targeted controls join independently selected attempts, aggregate physically typed outputs,
exercise a false-positive pruning predicate with limit, reject another deployment pool and
retain cohort partiality after early scan termination. Compare results to small independently
specified expected rows, not only a second use of the same pruning implementation. Retire
replaced Python-only query assembly once retained consumer paths use this native composition.

Add a repeated-study comparison that selects recorded attempts and realized-point/source lineage
from canonical state, then joins their protected outputs. Worker replacement must not create a
different history interpretation or require importing the deleted Python projection machinery.

<a id="e03"></a>
## E03 — Delivery custody and useful exports

Build selected batch/IPC export on E01/E02 and existing `workflow/result_files.rs` and
`pse-columnar/src/ipc_file.rs` admission. Bound predecode input, output and queued work with
the selected deployment policy; retain backpressure and cancellation/drain. Selection grants,
response attachment, decoder working memory, retained arrays and transport flush have
different lifetimes. Reader close stops further demand while returned data retains its
necessary original authority/allocation; it must not prematurely acknowledge physical drain.

Reuse `pse-columnar/src/owned_buffer.rs::attach_bytes`: existing `LeasedBytes` uses
`Bytes::from_owner` to hold a prepaid `AllocationLease` through clones and slices. Do not add
a second owner/reservation wrapper for the same bytes. A small view can retain a large backing
allocation; an explicit useful copy may release it, with copied-buffer admission and fidelity
checked. Preserve exact-response custody across partial attachment and delivery cancellation.

Native export writes an exact selected IPC artifact, publishing completion only after the
chosen file contract finishes. A failed/partial export is not a completed result attachment.
Add Parquet only for a selected export consumer, documenting retained/lost metadata and its
derivative status; the canonical source remains IPC. Targeted controls cover early reader
close with retained arrays, sliced Bytes leases, malformed/oversized IPC before decode,
partial attachment, cancellation during delivery and failed export completion. Remove
replaced Python materialization/stream holders and their mechanism-specific fixtures together.

Keep publication semantics specific: immutable payload creation retains its no-replace/completion
contract, selected executable descriptors may replace atomically, and canonical scientific commits
have transaction/acknowledgement semantics. Reuse private staging/filesystem primitives where they
fit; do not consolidate these into a universal generation receipt or preserve Python mirror hashing.

<a id="e04"></a>
## E04 — Consumer cutover, conditional interoperability and H handoff

Migrate retained Rust inspection/export callers onto E01–E03; coordinate retired Python
`TableStream`/MCP retained-session machinery with its frontend owner. A CLI is optional,
PyO3 remains deferred and MCP depends on the coordinator's selected subset. Native data
acceptance cannot depend on any of those adapters.

Migrate each actual consumer with its owner, configuration, generator/manifest entries and targeted
tests; delete retired outputs and assertions for the removed mechanism in that same cutover.
No temporary Python stream bridge is needed to make an unfinished development unit appear complete.

For O15, a named immutable-file download need first selects ordinary completed IPC-file HTTP
delivery. Investigate Arrow Flight only when an actual interoperable batch-stream consumer
requires it: 59.3 `FlightDataEncoderBuilder`, `FlightRecordBatchStream` and `DoGet` are
candidates, with dictionary fidelity, oversized values, queues, authentication and original
snapshot custody to settle. Approximate encoder sizing is not a hard message/memory bound;
`pse-columnar::flight` is existing keyed single-flight coordination, not Arrow Flight.
Signed remote URLs require an actual immutable remote-artifact consumer plus authorization
and retention covering the download interval. `object_store::Signer::signed_url` grants no
execution/read-retention authority. With no trigger, record non-selection and finish the core.

**Proposed verification:** [H](58h-qualification-and-publication.md#h01) receives native
selection→composition→export with Python absent,
early close/retained-output, concurrent reclaim, partial file attachment and delivery-cancel
journeys. E packets own targeted compile/unit controls; H owns integrated qualification and
scope-end static checks after all functional work. The baseline is zero failures. No tests,
builds or measurements ran for this draft; library source inspection is Interface-checked
evidence only, and quantitative performance remains unmeasured.

## Release-specific mechanisms

Use [DataFusion overview](../libraries/datafusion.md) for discovery and
[55.1 TableProvider](https://docs.rs/datafusion/55.1.0/datafusion/catalog/trait.TableProvider.html)
and [execute_stream](https://docs.rs/datafusion/55.1.0/datafusion/dataframe/struct.DataFrame.html#method.execute_stream)
for the selected contract. Overview 55.2.0 differs from pin 55.1.0; Arrow overview 60.0.0
differs from pin 59.3.0, and object_store overview 0.14.2 differs from pin 0.13.2.
Use [Bytes 1.12.1 from_owner](https://docs.rs/bytes/1.12.1/bytes/struct.Bytes.html#method.from_owner)
and [object_store 0.13.2 Signer](https://docs.rs/object_store/0.13.2/object_store/signer/trait.Signer.html)
as precise custody/conditional-signing references. The principal review appendix preserves
additional Flight and transport sources; newer overview APIs do not silently select upgrades.
