---
title: "Plan 58e: Retained data and delivery"
status: done
date: 2026-10-28
adrs: []
review_sources: [docs/functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md, docs/plan_improvement_reviews/plan58-rust-hard-cutover-2026-10-28.md, docs/functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md]
doc_role: plan
doc_owner: docs/plans/58-rust-owned-system.md
doc_topics: [storage, design]
---

# Plan 58e: Retained data and delivery

## Purpose and basis

**Implemented (selected functional scope):** deliver O08 and the data portion of O09 from the
[principal functional review](../functional_reviews/rust-first-system-functional-opportunities-2026-10-28.md#O08),
with O15 retained as a conditional interoperability inquiry. A Rust operator selects exact
retained results, composes native relations and exports useful data without Python objects,
mathematical initialization or provider preparation. The
[coordinator](58-rust-owned-system.md) owns cross-package decisions and assembled acceptance;
this document owns E01–E04. The initial draft scheduled work; the authorized implementation
and its acceptance limits are recorded in the [Outcome](#outcome-recorded-after-implementation).

The original inspected source baseline was `0659343f123fefa2371ac7d1659e3af7f75f9949`, on 2026-10-28.
The 2026-10-29 amendment incorporates the [database-boundary review](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md)
at `a6a144bce`. Its source-qualified O05/O08 inform repeated analysis and the protected payload
boundary; the coordinator owns those dispositions separately from the original O01–O15.
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
typed requests, snapshots and Arrow batches without Python serialization. Preserve the actual protected
IPC hybrid: `result_projection.rs` keeps admitted encoded blocks up to the declared
`RESULT_INLINE_BYTES` threshold inline and uses `reserve_result_file`/`seal_file` for larger bodies.
Canonical manifests, grants and accepted lineage refer to those same blocks. Parquet is an explicitly
selected export derivative; no database backend or authoritative payload migration is presumed.
The threshold remains registry-owned rather than copied into another configuration authority.

Persisted definitions/revisions, attempts, realized points and useful calculation observations
must identify these same result references. A later comparison of repeated studies reads their
original canonical lineage and protected payloads rather than rebuilding a separate execution
history. Native task graphs and every runtime event are not analytical requirements. Keep
scientific calculation/progress observation and diagnostics; build observations and derivative
build-specific queries/alerts deliberately retire through B/F.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [E01](#e01) | done | Result selection and decode owners | Math-free retained reads use independent data capabilities and original protected decode/admission. |
| [E02](#e02) | done | Native result and cohort relation owners | Native protected relations preserve configured state/pool, residual-before-limit, exact selected universe and partiality. |
| [E03](#e03) | done | Payload allocation and export owners | Selected native IPC/export and retained-array/Bytes custody reuse existing owners; assembled delivery-failure journeys remain with H. |
| [E04](#e04) | done | Data consumer and integration owners | Native consumers migrated and Python holders retired; conditional transports remain unselected; H owns assembled evidence. |

> The table and final Outcome record current selected completion. Dated checkpoints below
> are historical: their open work, removed commands and narrower passing selections retain
> their original conditions. They do not override the final Outcome or qualify H.

### Native data prerequisite checkpoint — 2026-10-29

The authorized A01–A07 execution consumes this E01/E02 prerequisite slice.
`RetainedDataServices::new` now takes an already opened canonical store and bound
engine model, creating an independent logical read caller without constructing mathematical
services. Protected readers, snapshots, progress interpretation and frozen cohorts retain
that capability. Completion decoding and permission interpretation share the existing
store/pool implementation; obsolete SourceServices read-operation ownership is removed.
`StudyCohortRelation` exposes the original configured state and protected provider for native
relational composition.

The selected native study test closes all scientific callers and native lanes, constructs
only a new engine model/read capability, and reads the original root result. The selected
admission control preserves configured session extensions, pool identity and close/continuation
semantics. [A's selected evidence](58a-native-product-and-scientific-workflows.md#native-data-and-public-caller-evidence)
records these checks. This establishes the native construction slice; full E01/E02 acceptance
and assembled H qualification remain open.

Independent source review found that retiring Python stored-result/cohort/stream wrappers
still captured their full Python runtime owner. The cohort/observer projection is now
removed by A07/G03; stored-result/stream consumers retain their retirement obligations. A07/E04 own their actual consumer retirement;
the native extraction does not establish that those wrappers are math-free. Protected join,
residual-limit, wrong-pool and payload-custody coverage remains necessary at the relevant
native consumer before E02 closes.

The subsequent native repeated-analysis test now exercises protected cross-attempt joins,
typed aggregates, original unavailable/empty classifications and wrong-pool refusal after
all scientific callers and native lanes close. It preserves original captured attempts and
repeats comparison under corrected engineering context without rebuilding science. A owns
that selected passing evidence and G the deleted Python aggregate projection. Residual-limit,
sealed-payload/reclamation and full E01/E02 acceptance remain open; these are not implied by
the narrower passing consumer.

The native passive study observer now retains `RetainedDataServices` too. It can open and
reconnect through an independently constructed read caller after all mathematical services
close; attaching local progress still retains no run cancellation lease. Runtime convenience
composition delegates to that same implementation. The three result-pruning controls now
construct private `NativeCanonicalService` fixtures and math-free data capabilities, removing
their old Python fixture-registry/global-state dependency. Their explicit engine assembly
selects existing Contract observation so DataFusion task propagation does not depend on
mathematical startup side effects.

**Tested:** [A's native read/observation checkpoint](58a-native-product-and-scientific-workflows.md#native-read-and-observation-follow-up)
records four passing controls, zero failures. The formerly unexercised residual-before-limit,
complete selected universe after stream consumption, configured-state/pool, native predicate
reuse and protected-pruning controls now have selected acceptance. Frozen page/detail/aggregate
exports and passive reconnect run after scientific shutdown. Broader sealed-payload concurrent
reclamation, cancellation/delivery custody and assembled E/H qualification remain open.


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
between ordinary terminal reads, selected readers and captured cohorts. Decode the existing bounded
inline IPC body or protected sealed file through the same retained interpretation contract; do not
assume every payload is external, or expand numeric arrays into scalar database records to simplify
metadata selection. D decides which exact output is applicable; E admits and retains the actual bytes.

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

Consume A05's repeated-analysis caller from
[database-boundary O05](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O05):
select accepted attempts by authored revision, physical/effective context, realized point and original
scientific qualification from D's canonical history, then join only the demanded protected numeric
relations here. Compare yesterday's and today's studies and a later corrected context while preserving
the earlier frozen cohort. Domain effective time, recorded revision and optional future storage-version
time are distinct; MX MVCC/snapshots do not supply that analytical history automatically.

Worker replacement must not create a different history interpretation or require importing deleted
Python projection machinery. Retire duplicated execution-history indices, repeated client hydration and
summary reconstruction only where the canonical query plus protected native relation covers their
retained meaning. Expected comparison rows must be independently specified, including unavailable,
empty and scientifically unusable outputs rather than treating every recorded attempt as usable.

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

For external bodies, construct/seal immutable bytes before canonical acceptance of their reference.
A stale attempt must not attach a result after a new fence; a sealed body followed by failed or lost
canonical acknowledgement retains the original operation identity for reconciliation. Reclaim an
unaccepted body only when its ownership and publication uncertainty permit it, never merely because a
receipt was not observed. Inline bodies still require original admission and canonical acceptance.
Targeted controls distinguish file completion, accepted reference, acknowledged publication and actual
reader/transport release, including a retained reader while the publishing caller closes.

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

For [database-boundary O08](../functional_reviews/surrealdb-state-execution-boundaries-2026-10-29.md#O08),
a bucket/file-value extension needs a named artifact consumer and a demonstrated reduction in existing
catalogue/publication machinery. Compare it with the current IPC hybrid and require an explicit
record-versus-file/object visibility, protection, uncertain publication and reclamation contract.
Bucket writes do not automatically commit atomically with canonical records; do not select them
merely because the database exposes a file API. Retain the current payload path with a documented
non-selection when that obligation adds more machinery than it removes. Any chosen migration is
separately authorized and joins D05's schema/fence boundary; no existing data reset is implied.

For original-review O15, a named immutable-file download need first selects ordinary completed IPC-file HTTP
delivery. Investigate Arrow Flight only when an actual interoperable batch-stream consumer
requires it: 59.3 `FlightDataEncoderBuilder`, `FlightRecordBatchStream` and `DoGet` are
candidates, with dictionary fidelity, oversized values, queues, authentication and original
snapshot custody to settle. Approximate encoder sizing is not a hard message/memory bound;
`pse-columnar::flight` is existing keyed single-flight coordination, not Arrow Flight.
Signed remote URLs require an actual immutable remote-artifact consumer plus authorization
and retention covering the download interval. `object_store::Signer::signed_url` grants no
execution/read-retention authority. With no trigger, record non-selection and finish the core.

**Historical draft verification proposal:** [H](58h-qualification-and-publication.md#h01) receives native
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

## Native protected-read consumer continuation — 2026-10-29

Five existing protected-result controls now construct private native canonical services instead
of relying on the Python fixture registry or configured development store. Four read-only callers
construct math-free retained-data services. The compiler-pressure control keeps its actual shared
pool and compiler leases; it does not substitute a simulated allocation. Exact range/schema,
empty demand, decoded arrays surviving reader close/reclamation, output indexes, cancellation and
IPC export assertions remain and pass. Owning fixture teardown retains original panic and
independent cleanup diagnostics.

Existing public repeated-analysis callers already freeze cohorts and export selected pages, exact
IPC detail and independently checked analysis through the native owner; no new transport or export
framework is selected. Bucket/file values, Arrow Flight and signed URLs remain unselected without
a concrete consumer trigger. Remaining E/G work removes the last Python stream/materialization
consumers and reconciles their declarations; H still owns assembled delivery-cancel, partial
attachment and deployment qualification.

## Outcome (recorded after implementation)

### What was built

**Implemented · 2026-10-29:** E01–E04 complete the selected native retained-data scope.
`RetainedDataServices` constructs independently of preparation/math; protected snapshots,
cohorts, passive observation, repeated comparison and selected IPC export use the existing
canonical selection/decode/custody owners. Configured DataFusion state, pool identity,
residual-before-limit, exact selected universe and unavailable/empty distinctions remain.
The last Python stored-result/stream/materialization consumers and their declarations retire
with G in `600d4df30`; no Python holder or replacement transport is required for these readers.

**Tested:** zero-failure baseline. Five protected-result controls pass within the earlier
ten-case covariance/research/protected-selection run. Four read-only callers construct
math-free capabilities on private native services; the compiler-pressure control retains its
real shared pool/compiler leases. Selected/full schema and range checks, empty demand,
returned arrays surviving reader close/reclamation, output indexes, cancellation and IPC
export retain their original assertions. Earlier native repeated-analysis/pruning controls
exercise joins, configured state/pool, false-positive residuals before limit and frozen cohort
partiality after scientific shutdown. The native-tools run passes 723 tests, zero failures
(`7152b88a-c932-41f1-84be-83a9b7687485`), including lower-level columnar/Bytes custody controls;
that aggregate is not 723 independent retained-data journeys. The coordinator's
[verification matrix](58-rust-owned-system.md#remaining-bg-outcome--2026-10-29) records commands,
modes and the selected scope; no additional tests ran for this documentation closeout.

### Mistake made and corrected

**Implemented:** the first data-only extraction still depended on mathematical startup's
DataFusion observation setup. Private read fixtures now select the existing Contract
observation explicitly and assert the original configured session/pool instead of relying on
that startup side effect. Native read construction and scientific execution remain separate.

### Deliberate deviations and remaining limits

**Implemented:** reuse the admitted inline-IPC/sealed-file hybrid and existing Bytes ownership;
no payload migration, bucket/file-value authority, Arrow Flight, signed URL service or Parquet
export framework was selected without a concrete consumer. H retains assembled delivery-cancel,
partial-attachment, concurrent-reclamation and deployment/recovery campaigns. These selected
controls establish neither a full scientific/reference/performance campaign nor completion of
all Plan58; historical receipts and raw data retain their original authority.
