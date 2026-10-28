---
title: "Plan57e: Canonical client and guarded requests"
status: in-progress
date: 2026-10-27
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, libraries, storage, design, validation]
review_sources:
  - docs/design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md
  - docs/library_leverage/reviews/canonical-store-transport.md
  - docs/library_leverage/reviews/execution-lifecycle-and-observation.md
  - docs/library_leverage/reviews/result-data-and-prepaid-accounting.md
---

# Plan57e: Canonical client and guarded requests

This companion develops CBO-009 and the whole CBO-010: both conflict retry and guarded-script
composition. The [coordinator](57-library-leverage-and-correctness.md) owns selection,
cross-series disposition and completion. This document owns the packages below; the [authorized execution checkpoint](#authorized-df-execution--2026-10-28)
records the maintainer-selected implementation scope. SurrealDB remains
canonical (U3). Conditional upstream work is included (U1), without authorizing contact.
Recorded point replay (U2) does not grant transport-level replay of application operations.

## Database-boundary review amendment

The [boundary review](../design_review/reviews/design_review_study-execution-and-database-boundary_2026-10-28.md)
changes the combined planned target. E's direct Rust gRPC client is an implemented foundation
with outstanding assembled acceptance, not the mandatory public Python ingress. K02 owns that
selection under operator-confirmed review RC02; L owns durable transition/actor composition.
Earlier G3 option B and RC09 statements describe the original E selection. They do not constrain
K's approved investigation or imply renewed qualification. Keep original clock, receipts,
serialized admission, predecode bounds and actual descendant/flush custody wherever consumed.

### E05 — Reconcile transport mechanisms with the selected boundary

Use the research companion and exact Hyper 1.11.1/Tonic 0.14.6/Prost 0.14.4/protocol 0.13.1
sources. Compare retaining owned transport with Tonic decoder/executor/connector composition
and Hyper-util connection coalescing. A Decoder sees a complete framed message after encoded
buffering; it can remove duplicate framing only if earlier allocation admission remains sound.
Supplier stubs hardcode ProstCodec, so codec injection requires explicit client/generator ownership.
Channel reconnection does not restore application capabilities; Bytes consumption does not prove
copied-buffer flush. TaskTracker close does not seal admission. Preserve connection-bound worker
sessions or explicitly revise the affected contract through the decision route.

Output a retain/replace decision for each complete obligation, the actual consumers and deletions,
and targeted malformed/oversize, early-response/backpressure, restoration and two-driver drain
cases. K02 may remove public obligations; it cannot retroactively discharge existing worker ones.
Use current focused custody tests, including the dirty inline-child corrections, without claiming
that they prove actual managed execution. No extra transport or compatibility shim is selected.

CBO-016's blanket production-LIVE refusal below is superseded for investigation by L03. Polling
remains the implemented path until that package decides and qualifies a bounded alternative.
L03 owns notification/catch-up semantics; E supplies any selected transport slice. CBO-046/048
and upstream proposal content retain their existing limits. No upstream contact is authorized.


## Foundation assessment and baseline

The consumed comparison is the complete 2026-10-27 reassessment at product source `4f52fd239`,
published before plan baseline `93946f572`, including supporting accounts and additional-pattern
revision 4. Its [G3](../library_leverage/reviews/canonical-store-transport.md) and
[G2 retry analysis](../library_leverage/reviews/execution-lifecycle-and-observation.md#cbo-010)
are proposals. They do not supersede blueprint §20/§20.6 or ADR-0214's current realization.
The focused assessment uses Core 3.4, Efficient Architecture 1.0 and the pse-arrow binding;
process-simulator 1.5 remains applicable to scientific consumers without adding a numerical
method change here.

Source reading confirms two useful foundations. `transport.rs::operation_scope` preserves the
enclosing minimum absolute deadline on first poll and destroys its child inside that clock.
`transport_lifetime.rs` already distinguishes caller admission, continuation activity and physical
shutdown, including readers-first close and two independently settling driver closures. These
are contracts to carry forward, not redundant wrappers to delete by resemblance. Existing
`canonical_transport_tests.rs`, `canonical_lifetime_peer_tests.rs`, `canonical_grpc_peer.rs` and
the lifetime unit tests provide consumed-behavior evidence that must survive ownership movement.

The unnecessary coupling is the home of repository policy inside patched SDK internals,
followed by a second repository lifetime tier. A narrow client within `pse-operations` can own
both without another crate. The principal cost is explicit ownership of the SDK's actually used
result semantics. The approximately 355 query builders are a review census, not an execution
completion count: implementation must cover the union of operations and `xtask` consumers.
Changing the client and envelope together removes a second call-site migration. It does not
justify widening the wire allowance, weakening typed errors or collapsing activity ownership.
This applies AP-01/AP-02/AP-05, DP-15/DP-17/DP-20, and H18/H21/H23/H24/H27 to the full request.

## Resulting responsibility and physical path

The chosen target is G3 option B: `SurrealDbServiceClient<OwnedHttp2>` from
`surrealdb-protocol =0.13.1` with `rpc`, and `surrealdb-types =3.3.4` with `proto`, inside
`pse-operations`. These versions match the inspected server realization; a protocol upgrade is
not bundled with this migration. The generated tonic client provides capability/session RPCs
and server-streaming `query` over `GrpcService<tonic::body::Body>`. Supplier protobuf types,
value conversions and `SurrealValue` remain library-owned. The repository owns the bounded
HTTP/2 connection, request policy, script envelope and result assembly. The
[protocol account](../library_leverage/libraries/surrealdb/protocol.md) records exact source
paths and conversion rules; the four SDK rules below are acceptance requirements.

The request path is: caller scope and its original clock → admitted request and typed options
→ newly built guarded envelope → capability/session readiness → owned HTTP/2 transport
→ cumulative wire preflight before prost decode → ordered statement assembly and typed error
decode → domain result extraction/delivery. The server request timeout is a ceiling inside
`RequestContext`, not a replacement clock. Application and control admission remain distinct.
Request context carries typed response ownership/limits; it is no longer an opaque `Arc<dyn
Debug>` flowing through engine-API internals. The accounting companion owns how the prepaid
allowance is granted; this client owns enforcement at the body boundary and lifetime retention.

One connection owner serializes Open/Closing/Closed admission and caller mint/close with activity
acquisition. `TaskTracker::token/close/wait` may unify completion tracking after admission, but
`close()` does not prevent another token or task. The state machine must seal new admission
separately, retain already admitted continuation rights and keep close rounds retryable. Final
reader release is a prerequisite before either physical driver is closed. Ordinary and activation
drivers settle independently even if one stalls or fails; each error and acknowledged progress
survives cancellation of the original close waiter. Closing is not Closed after an uncertain
driver outcome. Two clients per store retain their purpose even if their tracking mechanism shares
an implementation. Context7 returned contradictory summaries of `close`; the installed
tokio-util 0.7.19 source `task/task_tracker.rs` explicitly settles this distinction.

Session restoration joins the failed generation, attaches the replacement and reapplies only
acknowledged setup entries in order, without coalescing. Only a new, undispatched operation may
use it. No query/mutation/transaction is automatically replayed. Missing `QueryEnd`, unknown
frames, invalid counts, missing trailers and interrupted delivery fail closed with truthful
dispatch uncertainty. The immutable operation receipt settles an uncertain mutation before any
further effect. Wire limits apply cumulatively across frames before protobuf allocation/decode;
W3C trace injection and the typed `surreal-error-bin` cause chain remain at the transport owner.

### Result contract and guarded script

The façade offers the surface actually used: query/bind, `take<T: SurrealValue>`, statement count,
error extraction/statistics and the union of control/signin/select needs discovered in `xtask`.
It is not a general replacement SDK. Preserve these SDK behaviors explicitly:

- Trim submitted query endings consistently, including source-echoing parse diagnostics.
- Compact unframed statement indices and skip Kill results while respecting their original index.
- Preserve Option/Vec/Value cardinality, missing-index behavior, removal of taken error slots and
  conversion-error wrapping. `[]` and `[NONE]` become `None`; multiple values do not become one.
- Rebuild typed errors trailer-first, with the bounded 8 KiB error envelope, status fallback,
  kind-specific codes, details and cause chain. Preserve substantive-error precedence over
  NotExecuted/Cancelled. For several substantive errors select the lowest statement index,
  making the SDK's incidental hash iteration deterministic; never select a cancellation merely
  because it has the lower index.

The inherited [G3-P1 observation](../design_review/evidence/library-leverage-surrealdb-direct-client-2026-10-27/README.md)
matched 37/37 result observations on one in-memory fixture run per mode after these rules were
reproduced. Its 527-line direct client omitted the bounded lifetime tier. It is evidence of result
feasibility, not bounded transport, workload or assembled qualification.

The script type requires an explicit envelope: `ProtectedRead { recheck: Pin | PinOrTicket }`,
`ProtectedAdmission`, `ProjectAuthority { issuer }`, `StageGuard`, or `DeadlineOnly`, with the
actual guarded authorities as typed inputs. It binds the deadline/RPC timeout and authority fields,
emits BEGIN/body/postlude/COMMIT, and returns a typed extraction handle for its result slot.
There is no default guard and no framing inside raw SurrealQL bodies. ProtectedAdmission retains
the deliberate absence of the protected-read postlude. Dynamic selection heads remain readable
body text. The body and envelope are rebuilt inside each retry attempt, preserving one atomic
decision in one RPC. Stored-function guard consolidation is deferred; it would change the catalog
digest and require fresh activation. Implicit transactions and multi-RPC interactive transactions
are not selected: neither removes useful work, and the latter widens transaction lifetimes.

### Retry is an operation decision

One local `retry_conflicts(policy, operation, step)` owns pacing and typed exhaustion. The step
returns `Attempt::{Done(result), Conflict(error), Busy(error)}` only after any required receipt
readback. It runs inside the original `within_clock`/`operation_scope`; it launches no detached
retry task. Central predicates use `CanonicalError::is_conflict/is_capture_busy`; unknown or
uncertain errors do not acquire replay permission from a retryable label.

Named policies preserve the current guarded exponential+jitter, no-jitter begin-stage,
initialization exponential+jitter, retention yield and stage-contention fixed 20 ms behavior.
Preserve caps of 32 or 8 where currently selected, and the busy class bounded by clock without
consuming the conflict count. This plan resolves the review's missing-jitter question by keeping
the current begin-stage policy; aligning it needs evidence of a concrete contention problem.
`activate_source_stage`, `admit_product` and `acquire_source_stage` keep settlement/readback
inside their step before another effect. backon/tryhard add little after reproducing this
classification, counters, pacing and settlement; tryhard's detached retry callback is additionally
unsuitable. They are not dependencies of this plan.

## Alternatives, rule dependencies and evolution

Retaining the patched SDK is the current qualified realization until cutover. A small transport
patch still leaves restoration and request context coupled to its engine; an engine-SPI hybrid
depends on an explicitly unstable supplier interface. Neither is a second production path in
the target. A new crate is unnecessary because `xtask` already depends on `pse-operations`.

The direct client's `Error::from_wire` is public but `#[doc(hidden)]`. Record a pin hold with that
exact rationale, removal trigger and affected conversion tests. U1 schedules a reviewable upstream
proposal for a public protobuf-error conversion and gRPC conformance-runner coverage. Optional
public `from_engine` remains secondary. No upstream contact is authorized by plan creation and
no contribution is a prerequisite for local cutover. Reopen SDK transport injection only when
the supplier actually ships transport/context/close-and-join/restoration seams that meet the
consumed contract. A newer protocol or server is a trigger to recheck frames and vectors, not to
upgrade automatically.

E01 classifies the exact impact on ADR-0214 realization wording and blueprint §3.3, §20/§20.6,
overview and scope text naming the remote SDK (coordinator RC09). A refactor preserving the
accepted contract follows the ordinary route; dependency removal alone does not require an ADR.
If the target changes a governed contract, obtain explicit operator disposition and complete its
ADR/formal review route before dependent implementation. This draft accepts no such rule change.
Enduring architecture wording still moves through its amendment route. CBO-046's generator placeholder choice belongs to Plan57h;
codec path migration here supplies its input. U3 keeps development knowledge/evidence consumers
on the canonical store, so those consumers must be ported rather than silently retired.

## Packages and readiness

The following table owns local package progress. Anchors identify delivered capabilities rather
than whole-document barriers. Once authorized, use targeted compile/unit routes with explicit
force-validation; full integration and static checks belong to the coordinator's scope-end
qualification.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| E01 | in-progress | Plan57e | Consumed result, close and request interfaces have discriminating contract cases. |
| E02 | in-progress | Plan57e | Protocol client owns bounded transport with serialized admission and completion custody. |
| E03 | in-progress | Plan57e | All guarded decisions and retries migrate once with original settlement semantics. |
| E04 | in-progress | Plan57e | Supplier patch coupling is removed and assembled qualification is handed off. |
| E05 | open | Plan57e | Complete transport alternatives and affected contracts reconciled with K02 |

<a id="e01"></a>

### E01 — Pin the consumed result and close contracts

**Prerequisite:** current source and test owners reconciled. Classify RC09's contract impact;
contract-preservation investigation and baseline cases do not wait for a hypothetical rule change.
Inventory query/control consumers including `xtask`, the interactive conflict
fixture, telemetry filters, SurrealKit build and generated codec producers. Pin the four result
rules using supplied frames/known outputs and substantive-error precedence. Current SDK hash
iteration leaves ties incidental; baseline-preservation controls must not require the new
lowest-index rule to pass on unchanged code. Specify that deliberate target rule as an E02
acceptance case. Preserve owner-level transport/lifetime tests, including refused close, caller-mint race,
cancelled close waiter, failed reader prerequisite, both driver error orders and retry progress.
These tests are the acceptance contract before removing SDK implementation tests. Deliver the
typed request/script/result interface and consumer inventory to E02/E03 and Plan57h/57i.
**Targeted acceptance:** result-cardinality/error cases and lifetime state/race controls under
the current implementation, with deliberate malformed/missing frames reaching the boundary.
No production client is deleted at this preparatory boundary.

<a id="e02"></a>

### E02 — Move bounded transport and connection completion

**Prerequisite:** E01's contract cases; the accounting companion's unchanged response allowance
interface, and Plan57f's attribution/stop interfaces where consumed, not its entire completion.
Build the protocol/types façade and move owned HTTP/2, handshake, session restoration, wire
preflight and request context into one operations owner. Establish serialized sealing separately
from tracked completion and retain readers-first/two-driver independent settlement.
**Targeted acceptance:** scripted peer loss before/after dispatch, oversize/malformed frames,
unknown frame, missing QueryEnd/trailer, restoration interruption and non-coalesced setup,
original deadline through queued admission/decode/delivery, caller/continuation/close races,
two-driver stall/failure/retry and cancellation of a close waiter. Preserve 16-lane capacity tests.
For several substantive errors, the new client selects the lowest original statement index
deterministically while retaining substantive-error precedence; that correction becomes required
here, not a false passing prerequisite against the original E01 realization.
Delete moved internal branches and duplicate lifetime machinery only when no consumer remains;
do not retain a second production transport for comparison. E01's contract tests migrate intact.

<a id="e03"></a>

### E03 — Port guarded decisions and retry as one consumer migration

**Prerequisite:** E02's client surface and E01's result contract; exact guard-variant behavior
mapped at the actual sites. Port the roughly 355 builders once, including `xtask` signin/control
needs. Introduce the explicit envelope and typed result handle and the local attempt-classifying
retry owner together. Move all ten reviewed retry loops, preserving pacing/caps/readbacks.
**Targeted acceptance:** every envelope frame parses under canonical-maintenance; admission
has no accidental read postlude, pin/ticket variants refuse correctly, original clock spans all
attempts, busy does not spend conflict count, final conflict attributes operation/attempts, and
lost acknowledgement settles the original receipt without another mutation. Fake clocks and
controlled jitter exercise pacing; existing conflict fixtures retain actual server conflict
typing. Add a single-call conflict case for the production client. Preserve controlled commit-order
scientific race fixtures through a test-only generated-protocol held-transaction helper with
explicit clocks and owned driver join; no SDK or production interactive-transaction API remains.
A single buffered RPC cannot acknowledge intermediate state and then select its commit order.
Keep the owner-level conflict, exhaustion, retention, seal and capture assertions. Remove three authority wrappers, repeated bind sets,
postludes, slot arithmetic, ten inline loops and duplicate conflict predicates when replaced.

<a id="e04"></a>

### E04 — Remove supplier patch coupling and hand off qualification

**Prerequisite:** E03's complete consumer port, equivalent contract coverage and approved
realization wording. Change codec generator paths to `surrealdb_types`, coordinate Plan57h's
generator edits and `just codegen`, port telemetry target filters and SurrealKit's source recipe
to released SDK 3.3.4 for tooling only. Remove production vendored SDK/engine-API dependencies,
their deltas/reconstruction notes and `grpc-lifecycle-test-support`, preserving server patches
and unrelated vendor inputs. Build SurrealKit at qualification to settle the inherited static
claim; do not treat its build as canonical-client qualification. Produce conditional upstream
proposal content without sending it. Handoff all affected assembled journeys to Plan57j.

## Retained policy and observable reopen triggers

CBO-016 retains cross-process polling and authoritative claim/inventory rereads. Production
LIVE is refused. Measured discovery latency or idle query load material at 16 lanes across
worker processes reopens a separately admitted subscription class: independent lifetime,
per-notification allowance, kill-on-drop, re-registration and immediate authoritative reread,
polling backstop, capability-gated resume only. The inspected server has at-most-once,
non-resumable delivery; protocol cursor fields do not establish resumption. A short request-class
ADR and §3.3 consumer qualification would precede adoption. Changefeeds still poll and add
cursor/retention/write obligations, so they are not adopted without that evidence.

CBO-046 retains registry-owned constants projected as catalog literals and the existing exact
grammar parse at catalog test and maintenance activation. `DEFINE PARAM` would add runtime
state and shadowing without a demonstrated need. Reopen on an actual parameter planning or
escaping requirement; Plan57h may improve checked substitution without changing this meaning.

CBO-048 retains authenticated readiness bound to InvocationID, listener/unit identity and
health, plus offline physical backup and external-payload verification. `/ready` heartbeat
freshness is not a replacement readiness authority; logical export is not a backup. Reopen on
a different deployment/consumer readiness need or actual backup capability requirement.
Plan57g owns host supervision retention; E04 only removes incidental SDK build coupling.

## Verification and checkpoint

**Proposed:** all E packages; no implementation or transport qualification was run while
authoring. **Interface-checked:** current owners and cited source interfaces; inherited G3-P1
37/37 remains limited to its recorded result observations. Compile/unit checks during execution
should use `just check-package pse-operations`, the relevant targeted `just unit-package`
filters and `xtask` checks after its port, all through supported environment/validation routes.

Local acceptance proves result, framing, retry and connection obligations under controlled
faults. Plan57j owns actual canonical Rust/Python/worker/maintenance journeys, installation and
generated-consumer checks, sixteen-lane bounded transport/lifetime controls, scope-end hygiene
and publication. Final closeout requires all consumers on the single path, no replaced vendor
mechanism, preserved owner tests, approved authority wording and those assembled results.
No latency improvement is claimed; any future measurement must answer a named workload decision.

## Authorized D–F execution — 2026-10-28

The maintainer authorized implementation of D01, E01–E04 and F01–F06 through the
conversational execution approach. Work starts from clean `6d13fc986`. The existing
package owners retain status and acceptance; 57j owns selected assembled qualification.
Full powerset/no-default validation remains deferred to series end. No upstream contact,
data reset or formal ADR status change is authorized.

### Conditional upstream proposal content — not submitted

**Proposed:** expose a supported conversion from the protocol protobuf error to
`surrealdb_types::Error`, retaining kind-specific codes, typed details and recursive
causes. The present 3.3.4 client must use the public but doc-hidden `from_wire`
constructor; a supported fallible protobuf conversion would remove that coupling.
Unknown kinds/details and malformed causes must preserve typed refusal rather than
become a retryable conflict. Validate query-frame and unary trailer errors, bounded
8 KiB envelopes, status fallback, NotExecuted/Cancelled and nested causes.

**Proposed:** extend the protocol conformance runner with a gRPC driver and the
consumed session/query/result-cardinality cases. Preserve the separate local
fault-injection suite for cumulative predecode budgets, uncertain dispatch,
original clocks and physical drain; conformance examples cannot establish these
lifetime contracts. Neither proposal was sent, and no local adoption depends on
an unreleased supplier change. Revisit the dependency hold only after an inspected
compatible release and the affected conversion/transport controls pass.

### Test-realization amendment — 2026-10-28

Current fixture inventory includes controlled interactive transactions for source-retention,
collector/seal, protected-read/writer and capture ordering. The original single-call-only fixture
proposal cannot preserve those intermediate acknowledgements and chosen commit orders. The
coordinator selected a test-only protocol helper using the same bounded owned HTTP/2 mechanics,
original contexts and actual driver settlement, plus a new production single-call conflict
control. This changes test realization, not production transaction eligibility or scientific
assertions. No vendor SDK or public production begin/commit route is retained.

### Current implementation checkpoint — typed request custody

The direct protocol client and operations/tooling consumers are implemented in the working tree;
focused acceptance is still being completed. The replaced SDK and engine-API vendor trees
and all workspace dependency edges have been deleted; the standalone tooling SDK declaration
lives in Cargo metadata rather than an unused workspace dependency. Guarded scripts own framing,
result extraction and sealed authority bindings. Existing single bounded native primitives retain
their request shape. Test-only held transactions retain the original controlled commit/rollback
ordering without adding a production interactive API.

Integration exposed request followers outside the initial operations census. Runtime terminal,
completion, captured and artifact reads, study captures and physical decoders also need explicit
custody. `RetainedRequest` carries only closed, already issued operational guards;
`ResponseAllocation` composes actual `MemoryReservation` and `AllocationLease` byte owners.
Separate clearable request cells and independent driver holds retain both through physical drain,
including caller abandonment, without treating a guard or an opaque debug object as a byte grant.
The runtime helper nests these two existing owners; it acquires neither authority nor capacity.

SDK type reexports also appeared in generated codec identity inputs. The generator retains the
established logical codec tokens for `CanonicalSchemaV1` and qualifies physical Rust paths only
after hashing, including macro bodies. Regeneration preserves the canonical, evidence and knowledge
schema digests; it does not require a data reset or catalog activation. Full code generation and
current consumer compilation are still being completed. Earlier client failures retain their
original scope; selected actual-server journeys remain required after the consumer port.

Independent implementation review found that Hyper's supplied executor spawns socket and
request-body tasks independently of the returned connection dispatcher. Dropping that dispatcher
alone cannot establish physical drain. The client is being corrected to own the complete executor
task group from before handshake, seal admission atomically, and join all actual tasks before
publishing drain or releasing bootstrap custody. Outbound body/chunk custody must also survive
backpressure until its actual destruction. Earlier inbound-response and focused contract passes
retain their bounded scope; they do not discharge this newly identified physical-closure obligation.

The same review requires refusal of a mixed Batched-to-Single response rather than truncation,
connection retirement for an unexpected LIVE response, and sealed original deadline/RPC-timeout
bindings. Metadata profile selection preserves the already composed lifecycle instead of creating
a later clock or duplicating owner cells. These repairs require their current discriminating
controls and real canonical journeys before E02/E03 can close. Native parser coverage of every
envelope is separately selected through canonical-maintenance.

The physical repair is implemented and the current focused client/parser suite passes. Four
revealing witnesses cover actual task/destructor and socket custody, copied/chained outbound
storage and early-response admission under the original deadline. The first early-response
fixture used an upload smaller than Hyper's default window; explicit small receive windows and
positive outstanding-upload assertions established actual backpressure before the passing retry.
Copied source buffers do not establish socket drain: the typed outbound Buf records the I/O flush
requirement while the codec consumes it, and pending custody releases only after that successful
flush or full native task join. Known response acknowledgements remain distinct from physical
settlement. Actual canonical application/worker and Python journeys still remain.

Consumer reconciliation also found the runtime exporter selecting the removed SDK span target.
The client now owns its exact request-span metadata contract; the runtime exporter must consume
that contract while still refusing private events and lookalike spans. That final observation
consumer correction precedes assembled trace qualification.


The client-owned request span predicate now drives the runtime exporter; its exact-parent and
private-event exclusion control passes in the linked runtime. The separate generated-wire
output-owner refusal and logical-codec-identity controls also pass. SDK trees, their two complete
deltas, obsolete reconstruction exceptions and license annotations have been retired together;
current developer routes point to the canonical owner. Historical review inputs remain in Git.
Actual canonical application/worker/xtask journeys and current Python acceptance remain pending.


Scope-end consumer preparation found three recovery-fixture constructors still passing the old
reservation carrier. They now consume the same charged reservation into the existing typed
allocation lease, preserving its original extent and sealed-file custody. Current module
preparation also exposed native RPC features reaching the workspace-unified WASI value closure.
RPC activation now lives on the native operations dependency edge; the actual production WASI
module builds again, without changing workspace unification or module capabilities. Native
operators and selected application journeys require the current post-correction build.


The first assembled operations selection was interrupted after exposing nine substantive failures:
the scientific-cell fixture had omitted its explicit canonical policy and emitted no schema;
a scripted source-stage peer still supplied its old single-result layout; maintenance expected
unrelated tables' events; four native study fixtures omitted the current population digest;
and two controls exhausted their original clocks. The declarations/peer/fixtures now preserve the
original assertions and current contracts. The TCP owner now selects NODELAY before handshake,
matching ordinary RPC transport expectations rather than widening a deadline. The old owned
adapter had also omitted this option; no differential latency or general speedup is claimed.
Current focused execution of these corrections precedes the renewed assembled selection.
