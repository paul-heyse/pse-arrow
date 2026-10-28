# Study execution and database boundary

## Scoped baseline

Inspected 2026-10-28 at `64a217de2`, including current uncommitted operations
`canonical_transport.rs`, `canonical_transport_tests.rs` and `transport_lifetime.rs`.
This coordinator-integrated account incorporates the read-only storage researcher's
diagnostic contribution. It is a scoped supplement to the historical whole-system
account, not a whole-tree refresh. Core 3.4, heuristics 1.0, process-simulator 1.5
and the current binding apply; test assessment 1.2 applies to assurance questions.
The separate test account retains its own scope and baseline.

Inspection covered Python/runtime composition, workbench access, study creation/outcomes/
finalization/cohorts, attribution, request/body/outbound custody, exact settlement,
physical files and selected database integration. The [research basis](../evidence/study-execution-and-database-boundary.md)
retains capability sources and limits. No tests, builds, services, fault injection or
measurements ran. Numerical algorithms, whole-tree scientific permission and every
bootstrap participant were not re-audited. Plans retain implementation status.

<a id="da-046"></a>

## DA-046 — Metadata access requires the scientific runtime composition root

**Principles:** AP-01/02/06/07; DP-10/16/17; H3/H6/H17/H27.

`pse-py/src/workflow.rs::NativeRuntime::new` unconditionally invokes
`inspection::runtime::acquire(settings)`, validates the imported native artifact,
and constructs `Runtime::from_shared_local` before deferred canonical connectivity.
`inspection/runtime.rs::acquire` constructs SharedRuntime, obtains the scientific
session factory, binds a BoundModel, and establishes the executor/Python async bridge.

The native workbench entry routes through this owner and `recording_operation`.
The performing `pse-runtime/src/workflow/study_workbench.rs::study_workbench_page`
needs admitted activity, canonical issuing-store/cursor identity, bounded metadata
response storage and `select_study_workbench`. It does not formulate or solve a model
or hydrate scientific result payloads.

A cold metadata browser therefore acquires scientific construction and loaded-artifact
failure dependencies before its compact query can begin. Warm callers inherit the
retained scientific-budget compatibility restriction. Diagnosing or changing a catalog
consumer requires understanding unrelated model/session admission. This is source-
established coupling, not a measured startup or memory claim.

Counterevidence: the shared root is reused, not rebuilt per page; the operations query
already provides compact bounded projection; scientific execution legitimately consumes
the larger root. `initialize_database` also demonstrates a distinct administrative
entry. Construction alone is not evidence of solver invocation or numerical workspace
allocation. Related DA-021 concerns runtime-carrying domain values; this finding's
primary cause is the public metadata operation's construction boundary.

**Evidence:** Implemented construction/query path; Interface-checked consumed capabilities.
No alternative client was implemented or exercised.

<a id="da-047"></a>

## DA-047 — Poll and child-destruction attribution have different contracts

**Principles:** AP-02/05/06; DP-08/21; H1/H4/H18.

`ExecutionContext::instrument_with` attaches its OpenTelemetry carrier inside a poll
closure. `pse-diagnostics::observation::Scoped` installs observation only during poll;
its child has ordinary field destruction. Native tracing combinators differ too:
Instrumented enters its span on poll and pinned destruction, while WithDispatch
installs the subscriber on poll without corresponding custom destruction scoping.

By contrast, `canonical_client/owned.rs::ObservedBody` explicitly installs dispatch,
span and OTel context around both polling and inner-body destruction. Current dirty
operations inheritance wrappers distinguish unpolled, polled-cancellation and completed
destruction for activity/response authority; that is not an attribution correction.

A child may execute under captured attribution but emit ambient cleanup diagnostics
under the dropping caller's observation/subscriber/OTel context. Authors must understand
the wrapper composition to add such a diagnostic safely. This is a source-established
observational lifecycle inconsistency, not an established scientific permission,
premature-release or physical-drain failure.

Counterevidence and uncertainty: synchronous `ExecutionContext::enter` restores carriers
lexically; request-body destruction is explicitly scoped; Tokio invocation locals have
different semantics from the custom observation wrapper. Existing controls were inspected,
not rerun. No misattributed event was reproduced. Whether a particular async boundary
intends polling-only attribution remains a review contract question; no universal wrapper
is selected. [Exact tracing source](https://github.com/tokio-rs/tracing/blob/tracing-0.1.44/tracing/src/instrument.rs).

**Evidence:** Implemented differing scopes; Interface-checked composition.

## Scoped corrections to inherited premises

These corrections supersede the named current sites without closing other historical
manifestations or replacing their primary finding homes:

- **DA-033:** research dataset/selection creation and project settlement now use
  `canonical_settlement::settle_exact`, distinguishing matching receipt, reused identity,
  proven absence and unresolved readback while retaining both causes. The earlier decisive
  divergence at these sites is superseded. Different effect/fence lifetimes do not prove
  a universal envelope is needed.
- **DA-035:** study creation cursor-resumes bounded bulk point/run insertion from
  `installed.next_ordinal`. Outcomes are compact row/byte-bounded projections; finalization
  coalesces pages before files; cohort aggregation uses one lazy CohortAnalysisState
  provider. Former per-point creation, unconditional small files, eager reader union and
  whole-creation clock premises cannot be reused as current facts.
- **DA-034:** append queues and study finalization use inline payloads below
  RESULT_INLINE_BYTES. Larger sealed files retain integrity, sync, instance exclusion,
  intent and attachment obligations. This refresh neither dismisses them nor renews
  historical timing ratios.
- **DA-042:** telemetry selects explicit owner-stage fields and exact canonical request
  metadata; ObservedBatchProcessor/SdkLossLayer consume native loss diagnostics. Older
  prefix-export and compiled-away-loss premises are superseded. Export does not prove
  downstream ingestion.

## Integrated database responsibility remains an open solution inquiry

The actual patched SurrealDB 3.3.4 build selects MX 1.0 SSI, synchronous AOL/fsync,
snapshots, positive bounds and native reader/version/GC/queue/poison diagnostics.
Transactions, persistence and engine reclamation are already delegated strengths.
Readiness acquisition, pure Rust policy and hint application already share native
database scope. Exact receipts and authoritative rereads preserve uncertain effects.

MVCC pins do not intrinsically cover exported readers, host process identity, native
join or uncertain external effects. Bytes backing ownership does not establish flush
of copies. Existing outbound Buf/flush-fence/driver custody is relevant counterevidence
against treating provider drop as sufficient drain.

KV 1.0 is eligible but not the selected backend. Direct engine APIs are not automatically
exposed through the database or guest module. No violation follows merely from leaving
a capability unused. The solution inquiry is which complete state/payload transitions
can share a database transaction and remove cross-resource coordination, while preserving
external execution and bootstrap facts that still have a distinct purpose.
