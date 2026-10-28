---
title: "Plan57f: Execution lifecycle and observation"
status: in-progress
date: 2026-10-27
doc_role: plan
doc_owner: docs/plans/57-library-leverage-and-correctness.md
doc_topics: [current-work, libraries, design, diagnostics, validation]
review_sources:
  - docs/library_leverage/reviews/execution-lifecycle-and-observation.md
  - docs/library_leverage/reviews/native-solver-integration.md
  - docs/library_leverage/reviews/declarations-registry-and-python-boundary.md
---

# Plan57f: Execution lifecycle and observation

This companion supplies common attribution, stop/drain, renewal, lifecycle, native-thread and
synchronous Python-wait mechanisms for CBO-013–018, while preserving the neutral single-flight
and I/O/admission owners (CBO-011). [Plan57e](57e-canonical-client-and-requests.md) owns all
CBO-010 retry/script work and the subscription disposition. The
[coordinator](57-library-leverage-and-correctness.md) owns the combined dispositions. This is
plan-document work; every replacement below remains proposed and pending implementation authority.

## Foundation assessment and governing distinctions

The complete G2 reassessment and supporting accounts at product source `4f52fd239`, plan
baseline `93946f572`, additional patterns revision 4, are the starting evidence. The selected
standard is Core 3.4, Efficient Architecture 1.0, test assessment 1.2 and the pse-arrow binding.
Process-simulator 1.5 constrains resource and truthful outcome behavior; no numerical algorithm
or solver capability claim follows from consolidating these mechanisms. Blueprint §18.8,
§20.6, §21 and §23.1 remain the contract owners.

Current source already preserves the difficult invariants in several places:
`CancelSource::join_native` signals then keeps polling the same future; `jobs.rs` supervises
the join independently of a dropped caller; staged sessions pass per-request observation into
a long-lived native thread; flights retain a producer after waiters leave; and
`durable.rs::Heartbeat` is joined after its owned renewal work. The foundation problem is
repeating policy-free mechanics around these owners, not an absence of cancellation libraries.
For example, window-team lanes omit dispatcher/span propagation while Python pull captures
only tracing. Unifying attribution fixes that drift, provided it never grants authority.

Four distinctions govern the target:

1. Attribution explains work; authority permits it or keeps a resource alive. A captured span
   must never smuggle a population lease, connection activity, compute scope or I/O permit.
2. Stop requests and deadlines signal intent. Only actual completion/join permits release.
   An abandon-safe waiting future and a completion-owned native future require different races.
3. A lifecycle boundary has a shared vocabulary, but each operational population belongs to its
   performing owner. A staged-session join is not a native-job join counter event.
4. Notification can wake an observer; canonical claims, durable fences and authenticated kernel
   facts retain authority. Observation never grants release.

This assessment applies AP-01–AP-06 and DP-19/DP-20 to those consumed semantics. Under AP-07 and
H21/H22/H24/H26/H27, a small owner can remove carrier-selection drift and repeated race logic
without multiplying tasks, subscriptions, recorders or retained state. No new generic executor,
global subscriber, cancellation service or numerical pool is needed. No quantitative benefit
is claimed; polling cadence and actual resource lifetimes remain deliberate owner policies.

## Target contracts and composition

### Attribution context at runtime boundaries

`pse-runtime::ExecutionContext` captures an optional runtime handle, optional hub observation,
the exact `invocation::TransferredContext`, tracing dispatcher/span and W3C parent carrier.
`capture()` never starts a recorder. `enter()` owns a fixed nesting order and composite guard;
the dedicated-native variant also enters the existing native-execution thread scope.
`instrument(future)` composes invocation/observation scoping and tracing
`Instrument::instrument`/`WithSubscriber::with_subscriber` without installing a subscriber.
Nested invocation recording still starts through `invocation::start_scope`.

Context travels in staged `Request` messages, not only at thread creation. A reused lane receives
the actual request's context on each request, preventing attribution from the previous request
from persisting. Consumers include `math/jobs.rs`, `math/staged.rs`, shooting `window_team.rs`,
`workflow/run.rs`, `pse-py/src/workflow.rs` and runtime-owned producer futures in preparation,
artifacts and physical caches. Window-team tracing becomes visible to the caller's scoped
dispatcher; that is an intentional correction with tests for attribution and absence semantics.

Derivations are explicit: continuation on another thread uses the same context; `nested(kind)`
creates child observation/span relationships; `shared_producer(kind)` uses SharedProduction
observation and a linked span without inheriting the first waiter's recorder. Unavailable,
unrequested and zero stay distinct. The neutral `pse-columnar::flight` module does not import
`pse-runtime` (which already depends on columnar). Runtime callers instrument their supplied
producer future with the relationship-aware context; a necessary neutral hook remains generic.

`ConnectionActivity`, `PopulationScope`, `OPERATION_DEADLINE`, `ACTIVE_IO`, cancellation,
construction/compute scopes and library TLS stay explicit at their current owners. Transport
typed request options from Plan57e carry response and lifetime authority rather than being
folded into attribution. This also prevents a shared producer borrowing a first waiter's
prepaid or connection lifetime by accident.

### Typed stop, races and completion-owned drains

`StopReason::{Cancelled, Deadline}` and a small `StopSignal` interface expose a synchronous
check and async stopped wake, with adapters for `CancelSource`, kernel `ExecutionScope` and
the current cancellation/deadline pair. Keep flat FFI cancellation readable without a runtime;
map reasons to each owner's existing typed error instead of returning a generic cancellation,
including the native job's existing `source_owns_deadline` distinction.
The clock is the minimum of enclosing absolute clocks, including Plan57e's operation scope,
never a fresh duration per wait or retry.

Abandon-safe admission/readiness waits use an explicit biased stop-first race: if stop and
ready result are simultaneously available, stop wins. Wake-source loops register before
checking and recheck the owner after wake. Completion-owned work uses
`drain_on_stop(stop, retained_work, on_stop)`: pin the original work once; signal the native
leaf and create the owner's drain guard on stop; keep awaiting the same future; return its
outcome and optional reason. Never implement drain using timeout/drop of the retained future.
The guard stays alive until completion, including panic and lease-loss cases.

Deadline is an explicit optional input to the adapter. Existing staged-session and
`workflow/run.rs` cancellation-only drains remain cancellation-only in this consolidation;
adding a deadline arm there would be a distinct behavior choice and is not selected. Existing
deadline-aware admission/kernel paths retain their current arms. This resolves the review's
potential behavior widening while preserving a reusable mechanism.

`LeaseRenewal::spawn(store, fence, policy, on_loss)` owns a child stop token and
`JoinHandle<Option<Arc<WorkflowError>>>`. `stop().await` joins any renewal already in flight;
`scoped(work)` gives study admission the same lifetime discipline. Use admitted
`policy.heartbeat` and `MissedTickBehavior::Skip`, preserving each owner's first-renewal
timing explicitly rather than inheriting interval's immediate first tick accidentally.
The durable path preserves its current immediate first tick; study admission deliberately moves
from `lease / 3` to concurrent renewal at its admitted heartbeat, with the first renewal after one
heartbeat from admission rather than an immediate tick. The lease is already acquired at admission;
policy requires heartbeat shorter than lease. Test nondefault policies where heartbeat and
lease/3 differ before replacing the inline loop. Loss requests stop and reports the actual
renewal failure; it cannot imply scientific completion or release a fenced owner.

### Owner-typed lifecycle and dedicated native threads

Place closed `LifecycleBoundary` vocabulary in `pse-diagnostics`, available to runtime, Python
and inspection without introducing a runtime dependency into neutral code. Its existing names
cover admission wait, admitted, dispatched, stop requested, deadline expired, drain settled and
joined. Coordinate vocabulary/diagnostic integration with Plan57h; no generated consumer is
edited directly. `NativeJobLifecycle`, grown from `NativeAttempt`, owns job counters and its
existing recorder interval. `NativeSessionLifecycle` and `PythonPullLifecycle` share vocabulary
and hub/tracing emission but do not increment job counters. The join observer is supplied by
the owner; there is no universal lifecycle object that counts every join.

The hub, saturating operational atomics, optional tracing and bounded invocation recorder
remain independent sinks. Sampling or absent subscribers do not remove hub state or counts;
"unavailable" remains distinguishable from zero. The recorder retains its 64 intervals and
128-byte name bounds and records only owner-declared intervals, currently native worker
execution, not each fine boundary. Capture's tracing Layer stays a consumer; it is not another
hand-fed sink. Optional MetricDeclaration-driven installation is not required for this target.

`NativeThread::spawn(name, stack, context, on_joined, work) -> NativeJoin` consolidates the
jobs and staged-session dedicated-thread paths. It preserves thread names/stacks, panic mapping,
join witness of TLS/foreign destruction and the owner's population-specific callback.
The supervisor retains the join when the caller future drops. The initial implementation
continues `spawn_blocking(join)` and consumes the drain primitive; thread-pidfd readiness is
deferred. A readable thread pidfd alone would not justify releasing storage before actual join.
Synchronous join-on-drop in window teams/surrogates and shutdown of retained lanes remain with
their current owners; they are not automatically converted to this async join owner.

### Synchronous Python waits

One `pse-py` helper takes `py`, an explicit current interval, a detached completion poll closure,
and an explicit signal policy. Preserve 100 ms and 20 ms at their present sites; interval
normalization is not part of this plan. It checks completion before signals in each iteration,
retains the first Python signal, and returns both value and signal to the site. Optional
precheck and final-check policy preserve the actual boundary ordering. The GIL is released
while waiting; worker callbacks never attach Python. Context7's PyO3 FAQ confirms periodic
`Python::check_signals`; no wakeable signal-check primitive is supplied by that API.

| Consumer | Signal action and completion contract |
| --- | --- |
| `blocking_native_call` | Cancel, join the original pull, then raise with cause; report panic after drain. |
| `blocking_on_with_interrupted_value` | Cancel, finish the retained future, perform the final signal check, attach actual acknowledgement/recovery keys and cause, then raise. |
| Capture join in diagnostics | `request_stop` stops recording, then waits; it never cancels science. |
| `NativeLaunchedCommand.wait` | Ask command cancellation for its status and return the signal according to the current immediate-return contract. |
| `task_dump` | Return signal without cancelling the issued observer ticket; the actual owner retains it. |
| `next_batch` | Precheck only; a pending signal issues no new request. |

Cause, `analysis_key`/`artifact_intent` and other recovery attachment stay at their owners.
Completion during the last poll must not lose acknowledged identity. The helper must express
these existing policies rather than force all six through drain or all six through immediate
return. Existing `wait_async` continues its pyo3-async-runtimes path. signal-hook would take
process signal-table authority that the extension does not own; process-global
`signal.set_wakeup_fd` adds host/asyncio/IPython coordination without a required latency benefit.

## Packages and dependencies

Only the table below owns package progress. The coordinator owns aggregate opportunity state.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| F01 | in-progress | Plan57f | Relationship-aware attribution used at the enumerated runtime boundaries. |
| F02 | in-progress | Plan57f | Typed stop/drain and joined renewal mechanisms replace reviewed repetitions. |
| F03 | in-progress | Plan57f | Owner-typed lifecycle vocabulary and all four independent sinks preserve populations. |
| F04 | in-progress | Plan57f | Jobs/session native thread owners migrate without weakening join custody. |
| F05 | in-progress | Plan57f | Synchronous Python policies share mechanics and preserve acknowledgement identity. |
| F06 | in-progress | Plan57f | Neutral flight behavior is retained; unused wrapper is deleted and handoff complete. |

<a id="f01"></a>

### F01 — Relationship-aware execution attribution

**Prerequisite:** actual carrier and relationship consumers reconciled at implementation start.
Deliver context capture/enter/instrument plus request-message carriage, with tests for nesting,
exact recorder Arc/no new recorder, no subscriber, absent observation, W3C parent, linked shared
producer and reused-lane attribution. Migrate jobs, staged requests, window teams, workflow and
Python pull boundaries. Delete replaced public inherit wrappers and loose RequestObservation
dispatch/span fields only after every caller moves; keep narrower owner scopes where their
authority differs. Runtime-producer instrumentation supplies Plan57b and other native consumers
without waiting for F04/F05. **Targeted acceptance:** controlled threads/tasks show the right
operation/recorder/dispatcher, no trace leakage after guard drop and no first-waiter producer
parenting or authority inheritance.

<a id="f02"></a>

### F02 — Stop/drain and owned lease renewal

**Prerequisite:** existing stop/error/clock contracts mapped; independent of F01 implementation
except where attribution is used by spawned renewal work. Implement typed adapters, abandon-safe
race and retained-future drain, then migrate reviewed deadline blocks and `scope_stopped` copies.
Consolidate Heartbeat and study renewal, preserving explicit cadence/first-tick semantics and
joining in-flight renewals on stop. Remove replaced races, failure mutex/oneshot and inline
renewal loop once behavior is exercised. Make the study inventory sleep cancellation-aware
through the abandon-safe stop race, preserving its canonical poll cadence and authoritative
inventory reread. This improves stop responsiveness without introducing an in-process producer
for a cross-process fact. **Targeted acceptance:** simultaneous stop/readiness,
early/late enclosing deadline, no-runtime FFI flag, cancellation during native work, stop while
renewal is in flight, cancellation during inventory sleep, renewal failure during export and
original fence retention. Study admission
must renew while its admitted work is blocked and stop renewing before release. The delivered
stop/drain contract is usable by Plan57e without whole-companion completion.

<a id="f03"></a>

### F03 — Lifecycle vocabulary with population ownership

**Prerequisite:** F01's attribution contract and agreed vocabulary location with Plan57h;
F02's stop reasons for corresponding boundaries. Replace repeated hub boolean/tracing pairs
with owner methods, move job join counts into the job lifecycle observer, and type inspection's
`native.joined` key. Keep staged/Python owners separate. Delete replaced literals/free
`native_joined` and duplicate sink emission only when callers move. **Targeted acceptance:** no
subscriber/exporter still gives exact hub and count changes, job cancellation while pending join
counts once, staged/Python joins do not increment job populations, absent recorder stays absent,
and repeated boundaries do not spend extra recorder intervals or hide saturation.

<a id="f04"></a>

### F04 — Dedicated native spawn and join owner

**Prerequisite:** F01 context, F02 retained-work drain and F03 owner-specific callbacks, each
available as working targeted-tested contracts. Move jobs and staged-session spawn/join blocks
to NativeThread. Delete their superseded inline implementations and implementation-specific
tests, retaining owner-level assertions for stacks/names/panic/resource-release/populations.
**Targeted acceptance:** caller-drop still joins, stop still retains leases/output buffers,
thread-local and foreign destructors finish before on_joined and release, panic maps after drain,
and staged-session join never enters the native-job population. No thread-pidfd or AsyncFd
dependency is required to finish this package.

<a id="f05"></a>

### F05 — Explicit synchronous Python signal policies

**Prerequisite:** F01/F03 attribution and lifecycle interfaces; F02 drain semantics where used.
Implement the helper with each caller's current interval, precheck/final-check and stop policy;
migrate all six categories in the table. Delete superseded polling loops, preserving their
owner contract cases and all structured cause/recovery attachment logic. **Targeted acceptance:**
pending precheck issues no request, final-poll signal retains an acknowledged key/intent, capture
interruption stops recording alone, task-dump ticket survives interruption, command return keeps
its current status semantics, detached callback never attaches Python, and panic reports only
after join. Deliver this working wait capability before Plan57i's public native-boundary migration;
do not make that consumer wait for unrelated F06 work.

<a id="f06"></a>

### F06 — Retained flights and assembled handoff

**Prerequisite:** F01's runtime-producer adoption and a fresh consumer search for the wrapper.
Delete unconsumed `pse-engine/src/cache_service/flight.rs` and its module export if it remains
unconsumed. Retain `pse-columnar::flight::Flights`, shared futures/oneshot fan-out, typed retiring
refusal, finite key/waiter limits and last-waiter signal-and-join. Retain the shared reentrant I/O
gate, release-between-list-items and semaphore+pool-reservation admission. **Targeted acceptance:**
last waiter leaving mid-production keeps prepaid resources until producer return; nested gate
borrows enclosing permit; multiple stores share the gate; list iteration does not hoard it.
Keep these owner tests even when deleting the unused wrapper's implementation tests. Hand off
native, Python, canonical attribution and cancellation journeys to Plan57j.

## Retained choices and inquiries

CBO-016's in-process premise is corrected: worker discovery, study inventory and stage contention
are cross-process waits. Existing WakeSignal/Notify/TaskTracker mechanisms already serve true
local producers. No extra submission hint or test-only production signal is adopted. F02's
cancellation-aware study sleep records cancellation promptly while preserving authoritative
inventory; it does not change receiver-observation heartbeat or canonical poll cadence.
Plan57g supplies primary kernel observation; Plan57e owns any future subscription class.
Final inventory reread after drained/changed-owner remains mandatory.

Reopen thread-pidfd readiness only on measured blocking-pool pressure from retained native lanes;
then validate exit versus TLS destruction and actual nonblocking join before replacing the wait.
Reopen a wakeup fd only for a concrete host-specific signal-latency requirement, with main-thread
save/restore/forwarding tested under asyncio and IPython. Reopen any polling interval or first
renewal policy on its own demand/evidence, not because a helper makes uniformity convenient.
No formal rule change is claimed for the selected refactors; a discovered boundary/metric
population change returns to the coordinator before dependent adoption.

## Verification and checkpoint

**Proposed:** F01–F06. **Interface-checked:** the cited current owners and Context7 PyO3 guidance;
no compile, native-provider or Python journey ran during plan authoring. During execution use
`just check-package`/targeted `just unit-package` for pse-diagnostics, pse-runtime and affected
engine/operations consumers with explicit force-validation. The Python wait package uses the
targeted installed correctness-extension tests through the repository's Python route; its build
is an execution obligation, not an authoring prerequisite.

Local cases establish race ordering, propagation, sink populations and resource custody.
Plan57j owns assembled native job/session/window-team, Python interruption/recovery, canonical
lease-loss, flight/retention and host receiver interaction journeys after functional scope ends,
plus hygiene and publication. Passing local helpers cannot establish foreign-thread completion,
real Python signal handling or canonical operation settlement in the assembled system.

## Authorized D–F execution — 2026-10-28

The maintainer authorized implementation of D01, E01–E04 and F01–F06 through the
conversational execution approach. Work starts from clean `6d13fc986`. The existing
package owners retain status and acceptance; 57j owns selected assembled qualification.
Full powerset/no-default validation remains deferred to series end. No upstream contact,
data reset or formal ADR status change is authorized.

### Current implementation checkpoint — ownership and interpreter controls

The runtime attribution, neutral stop/drain, owner-specific lifecycle populations, native join
owner and unused flight-wrapper deletion are integrated. Focused Rust controls have exercised
these mechanisms; linked scientific and assembled canonical/Python acceptance is still pending.
The six interpreter policies now share mechanics while preserving their original intervals,
stop actions, acknowledgement attachment and completion order.

The extension is a Python-linked `cdylib` without a Rust library test harness. Its pure polling
policy therefore has a standalone test target using the exact production policy module; its four
ordering/drain controls have passed. Actual interpreter behavior remains separately exercised by
the correctness-extension tests. An isolated test-only C caller arms CPython's pending signal flag
and calls the real native `next_batch` without intervening Python bytecode, so the planned precheck
control can establish that no reader is consumed. Its dispatch smoke control passed; the actual
stream control awaits the rebuilt extension. No production testing hook was introduced.
