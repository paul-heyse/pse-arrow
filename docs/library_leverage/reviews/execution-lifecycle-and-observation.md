# Execution lifecycle, cancellation and observation

**Group G2 of the [comparative review](README.md), 2026-10-27.**
Opportunities: [CBO-010](../codebase/persistence-and-transport.md#cbo-010) (retry policy part),
[CBO-011](../codebase/persistence-and-transport.md#cbo-011),
[CBO-013](../codebase/execution-lifecycle-and-observation.md#cbo-013) to
[CBO-018](../codebase/execution-lifecycle-and-observation.md#cbo-018) (CBO-016 in-process side only).

| Item | Value |
| --- | --- |
| Reassessment | 2026-10-27 at source `4f52fd239`; consumes the complete original review and updated inputs at `b29fcf802`, base patterns 0.2 and additional patterns revision 3 (refined to 4 here). All named dispositions were reconsidered with representative current source, consumers and inherited evidence; unchanged evidence retains its original conditions. |
| Original source inspected | `main` at `3e9a2525b`, clean for the inspected paths (dirty files at session start are scripts/docs unrelated to these crates). The audit baseline was `dbc4c9436`; the one material change in scope is `ee44d218d`, which rewrote `pse_operations::transport::operation_scope` as a pinned future that selects the enclosing minimum deadline on first poll and drops a completed or cancelled child inside that clock. It strengthens, and does not change, the original-clock premise of CBO-010. |
| Accounts consumed | Codebase account `c2f8b485a` (with its 2026-10-27 correction bullets); library assessment `34f9f07ee`; additional patterns revision 2 (`LOP-X001`–`LOP-X010`); base patterns edition 0.2. |
| Realizations | Installed: tokio 1.53.2, tokio-util 0.7.19, tracing 0.1.44, tracing-subscriber 0.3.23, tracing-opentelemetry 0.34.0, opentelemetry/SDK 0.33.1, futures-util 0.3.34, event-listener (via `pse_diagnostics::Cancellation`), object_store 0.13.2, quick_cache 0.6.24, rustix 1.1.5, pyo3 0.29.2, pyo3-async-runtimes 0.29.0; host kernel 7.0.0, CPython 3.14.8. Not in the workspace, assessed from their accounts: backon 1.6.0, tryhard 0.5.2, moka 0.12.16, async_singleflight 0.6.2, metrics 0.24.6, signal-hook-registry 1.4.8. |
| Standard | Core 3.4 with the efficient-architecture heuristics; process-simulator profile 1.5 (not central here; no numerical semantics change); pse-arrow binding. Contracts: §18.8, §21 (durable claims and leases), §23.1. |
| Evidence | Every target below is *Proposed*. Enabling capabilities are *Interface-checked* where stated. No probe was run; no quantitative claim is made. |

## Group summary

**The joint decision.** All eight opportunities describe one lifecycle: work is admitted, may wait,
runs (often on a dedicated OS thread), can be asked to stop by a caller, a deadline, a lost lease or
a Python signal, keeps its resources until it actually exits, and reports each boundary to several
observers. The codebase already has the right *semantics* everywhere I looked — completion
ownership, stop-first ordering, joins that witness thread-local destruction, observation that works
without a subscriber. What it lacks is a small number of *local owners* for the repeated mechanics,
so each site re-derives its own carrier set, stop race, renewal loop, retry loop and sink calls.

No new library is recommended for this group. The libraries that could have replaced the mechanics
either cancel by drop where the code needs completion ownership
([LOP-X005](../assessment-patterns.md#lop-x005)), need a tracing subscriber where observation must
work without one, or reduce to a thin shell once the repository's classification, pacing and clock
rules are reproduced. The coherent target is six local owners built on capabilities the workspace
already uses (tokio `select!`/`interval`, tokio-util `CancellationToken`, tracing `Instrument`,
the `pse_diagnostics` cancellation and wake types):

1. **`ExecutionContext`** — one captured *attribution* context with one ordered `enter()` and
   relationship-aware derivations (CBO-014). Authority carriers stay explicit.
2. **A typed stop value** — `StopReason::{Cancelled, Deadline}` over the existing
   cancellation-plus-deadline pairs, with a stop-first *race* for abandon-safe waits and a
   *drain* (the generalized `CancelSource::join_native`) for completion-owned waits (CBO-015).
3. **`LeaseRenewal`** — one renewal owner for durable runs and study admission (CBO-015).
4. **Owner-typed lifecycle events** — grown from the existing `NativeAttempt`, with a closed
   boundary vocabulary shared by the native supervisor, staged sessions and the Python pull (CBO-013).
5. **`NativeThread`** — one spawn-and-join owner for dedicated native threads, consuming 1, 2
   and 4 (CBO-018).
6. **Conflict retry policy** in `pse-operations` and an **interruptible wait helper** in
   `pse-py` (CBO-010, CBO-017).

CBO-011 (single flights, I/O gate, load admission) and the in-process side of CBO-016 are best
*retained*; the only clear-win change found there is deleting an unconsumed public wrapper.

**Reinforcing choices.** `ExecutionContext` is what `NativeThread`, the flight producer spawn and
the Python pull thread enter; the typed stop value is what `NativeThread`'s join wait, the
admission waits and the Python helper's cancel path consume; the lifecycle events are what the
drain and join points emit. Building them in that order lets each later owner delete a block of
per-site code rather than wrap it.

**Competing options that were rejected for the whole group.** Making tracing the backbone (span
extensions as carriers, a layer as the fan-out) would unify CBO-013 and CBO-014 in one move, but it
contradicts a structural property: `pse-diagnostics` has no `tracing` dependency, the hub and
operational counts must work and start before any subscriber or exporter exists, and spans can be
filtered or sampled ([tracing](../libraries/tracing/README.md); §23.1 "Sampling off does not
disable owner metrics", "Library code does not install a global subscriber"). That would be a
regression under the no-compromise rule, so it is not a live alternative.

**Interactions with other groups.**
- **G3** owns the SurrealQL prelude half of CBO-010 and the store-side half of CBO-016. The retry
  owner proposed here wraps whatever request builder G3 selects; nothing here depends on G3's
  choice. If G3 adopts LIVE SELECT or changefeeds, the notification should arrive as a *hint* into
  the same "re-check the authority" wait shape described under CBO-016, with polling kept as the
  fallback.
- **G1** owns the `observe-primary` subprocess that `wait_study_inventory` spawns each observation
  interval (CBO-020) and process pidfds (CBO-004). If G1 adopts `tokio::io::unix::AsyncFd` for
  process pidfds (tokio `net` feature), the optional thread-pidfd join for CBO-018 becomes cheaper
  because the reactor machinery is shared.
- **G4/G5/G6** own authority carriers that `ExecutionContext` must *not* absorb:
  `pse_math::construction::scoped`, `pse_backend_native::execution::compute_scoped`, the prepaid
  leases moved into flight producers, and Symbolica's thread-local abort binding (CBO-021). The FFI
  stop flag read by C callbacks (G6) is the reason the two cancellation types stay.
- **G8** owns the diagnostic code vocabulary (`pse_diagnostics::vocabulary`, CBO-045). The closed
  lifecycle-boundary vocabulary proposed under CBO-013 should live next to it or in
  `pse-diagnostics`, decided with G8.

**Consequence-based priority** (kept separate from prerequisite order):
1. CBO-014 — divergent carrier subsets already lose attribution silently on real paths (window-team
   lanes run without the caller's tracing span or dispatch).
2. CBO-015 — the renewal cadence has drifted from the admitted lease policy at one site, and each
   hand-written stop race is another place to miss a wake.
3. CBO-013 — inspection consumes boundary names by string (`"native.joined"`); a closed vocabulary
   protects that consumer and keeps metric populations at their owners.
4. CBO-010 — five pacing forms and duplicated predicates and caps; no defect found, but the next
   stricter rule will not reach every loop.
5. CBO-017 — consolidation of six wait loops with deliberately different stop policies.
6. CBO-018 — consolidation only; the cost it names is small.
7. CBO-011 — retain; delete one unconsumed wrapper.
8. CBO-016 (in-process) — retain.

**Prerequisite order:** CBO-014's context value and CBO-015's stop value first; then CBO-013's
lifecycle events; then CBO-018 and CBO-017, which consume all three. CBO-010 and the CBO-015 lease
owner are independent of the rest.

<a id="cbo-014"></a>

## CBO-014 — Execution context re-established by hand at each thread or task boundary

**Responsibility and target intent.** Work that moves to another task, OS thread or long-lived
worker must keep the caller's *attribution*: observation operation (hub), invocation recorder,
tracing dispatcher and current span, OTel parent carrier, and the native-execution thread scope
(§23.1: "Explicit propagation follows async tasks, native threads, shared producers/waiters and
actual managed/native-child execution"; "Bounded W3C parent context crosses explicit
host/task/thread boundaries"). That is promised. What is incidental is that each site picks its
own subset and order.

Observed at `3e9a2525b`:
- `math/jobs.rs` (native job thread): runtime handle, invocation, observation, native execution
  scope, dispatcher, span — six carriers in that order.
- `math/staged.rs` (`Request` work closure on the session thread): invocation, observation,
  execution scope, dispatcher, span; the context travels *inside each request message*.
- `workflow/shooting/window_team.rs` (lane threads): runtime handle; observation and invocation for
  preparation; observation, invocation and execution scope per request. **No tracing dispatcher or
  span is entered on either path.**
- `pse-py/src/workflow.rs::blocking_native_call` (pull thread): dispatcher and a telemetry span
  only; no observation or invocation context.
- `pse-columnar/src/flight.rs` (shared producer task): observation scope only; callers add
  `telemetry::ProducerContext` (a *linked* span) and `PopulationScope` themselves.
- Async wrappers: `invocation::inherit`, `observation::inherit`/`child`/`scope`,
  `telemetry::inherit`, `workflow::inherit_runtime_activity`, `canonical::inherit_activity`.

The same capture sits beside carriers of a different kind. `ConnectionActivity` keeps a canonical
client's physical lifetime open until its activities drain (`transport_lifetime.rs`).
`PopulationScope` is "explicit propagation into a sanctioned dependent loader. Concurrent children
refuse" (`jobs.rs`). The task-local `OPERATION_DEADLINE` and the `ACTIVE_IO` lease chain in
`stores/admission.rs` are also carriers. These are **authority carriers**: propagating them
extends a lifetime or grants admission. Losing an attribution carrier loses explanation; wrongly
propagating an authority carrier changes behaviour.

**Credible complete options.**
- *Retain current.* Correct where each site chose well; the cost is the divergence above and a
  missed carrier on every new boundary.
- *Tracing span extensions as the single carrier* (`tracing/span-carried-context`). Ruled out: a
  span carries nothing without an enabled subscriber, and the recorder must be the exact `Arc` with
  no span ([tracing](../libraries/tracing/README.md)).
- *An executor feature.* tokio task-locals are not inherited by spawned tasks or threads, and the
  unstable task hooks carry no user data (`tokio/task-local-context`). No library propagates into
  OS threads or foreign callbacks; the library assessment searched this and found none.
- *A spawn helper only* (`spawn_native(name, stack, work)`). Partial: it misses the staged request
  path, where context travels in a message to a long-lived thread, and the async wrappers.
- *A local `ExecutionContext` value* covering attribution carriers only, with authority carriers
  explicit — recommended.

**Comparative argument.** The decisive distinction is attribution versus authority. A single
"capture everything" value would be simpler to write and wrong: it would hand
`ConnectionActivity` or `PopulationScope` to children that must not inherit them, and would let a
shared producer inherit the first waiter's lease chain. Keeping authority explicit preserves the
owners' rules (DP-01, DP-20) while removing the part of the dance that has no decision in it.
Within attribution, one value with one `enter()` fixes the order once, makes the window-team gap
impossible to reintroduce, and puts the "foreign thread" knowledge (runtime handle, execution
scope) in one place (DP-16, AP-06). It composes with the existing pieces rather than replacing
them: `invocation::TransferredContext` already captures and enters the recorder half and becomes a
field; `telemetry::inherit` is exactly `in_current_span().with_current_subscriber()` and becomes
the async half. Normal use changes from five or six capture lines plus five or six guards to one
capture and one guard. Growth (a new carrier such as the OTel `Carrier`) is one field, not an edit
at every site.

**Revealing case.** *Shared producers.* `flight.rs` creates a `SharedProduction` observation
child, and `preparation.rs` starts the producer with `ProducerContext::run`, which makes a
**linked** span rather than a child of the first waiter. A naive `ExecutionContext::capture()` at
the spawn would parent shared work under whichever caller arrived first and misattribute it to
that caller's invocation. The context value therefore needs relationship-aware derivations, not
just capture/enter: `nested(kind)` (same operation, child observation, child span),
`shared_producer(kind)` (new observation child with `Relationship::SharedProduction`, linked span,
no caller recorder), and plain `enter()` for "continue this operation on another thread". With
those, the case strengthens the design rather than refuting it: the relationship decision moves
from ad hoc carrier choice to a named derivation.

A second case: *window-team lanes gain a span.* Entering the caller's dispatcher and span on lane
threads changes behaviour: tracing events emitted inside lane work become children of the job span
and, in a Python host with a scoped dispatcher, reach that dispatcher instead of the thread's
default. This is the intended attribution, but it is a behaviour change and could raise telemetry
volume. If the omission was deliberate, the derivation can say so explicitly
(`enter_without_trace()`), which is still better than an unexplained subset.

**Disposition: Recommend.** Bounded target:
- **Mechanism.** A `pse-runtime` type `ExecutionContext { runtime: Option<Handle>, observation:
  Option<ObservationContext>, invocation: TransferredContext, dispatch: tracing::Dispatch, span:
  tracing::Span, parent: telemetry::Carrier }` with:
  - `capture()`;
  - `enter(&self) -> Entered` (one composite guard in a fixed order; on dedicated native threads it
    also enters `native_execution_scope()`);
  - `instrument(fut)` for tasks, composing `TransferredContext::scope`,
    `pse_diagnostics::observation::scope`, `tracing::Instrument::instrument` and
    `WithSubscriber::with_subscriber`;
  - `nested(kind)` and `shared_producer(kind)` derivations.
- **Kept explicit.** `ConnectionActivity`, `PopulationScope`, `OPERATION_DEADLINE`, `ACTIVE_IO`,
  cancellation values, construction and compute scopes, and library-held thread-locals.
- **Retained rules.** The recorder stays the exact `Arc`; nested invocations still start through
  `invocation::start_scope`; `capture()` never creates a recorder; "unavailable" stays distinct
  from "not requested".
- **Consumers.** `jobs.rs`, `staged.rs` (the `Request` carries one `ExecutionContext` instead of an
  observation field plus loose dispatch and span), `window_team.rs`, `workflow/run.rs`,
  `pse-py/src/workflow.rs`, runtime callers supplying flight producer futures, `NativeThread`
  (CBO-018). The neutral `pse-columnar` flight module must not import `pse-runtime`.
- **Machinery that disappears.** Per-site capture/enter blocks; `telemetry::inherit`,
  `observation::inherit` and `invocation::inherit` as separate public wrappers, which become
  `ExecutionContext::capture().instrument(..)`; `RequestObservation`'s loose dispatch and span.
- **Added.** One small type and its unit tests: order, the no-subscriber case, the
  shared-producer link and the no-recorder case.
- **Transition.** Site by site; each site's existing tests on observation and recorder attribution
  keep their assertions.

**Rule impacts.** None. A refactor within §23.1's propagation contract; no ADR or formal review.

**Evidence.** Target *Proposed*. Carrier sets *Interface-checked* by reading `jobs.rs` (native job
spawn), `staged.rs` (`Request`, `RequestObservation`, request work closure),
`window_team.rs` (lane spawn and request loop), `pse-py/src/workflow.rs::blocking_native_call`,
`invocation.rs`, `observation.rs::{scope, inherit, child, native_execution_scope}`,
`telemetry.rs::{inherit, ProducerContext}`, `transport_lifetime.rs` and `jobs.rs::PopulationScope`.

<a id="cbo-015"></a>

## CBO-015 — Cancellation, deadline and lease renewal composed separately at each site

**Responsibility and target intent.** A stop request must reach async waiters, CPU loops and C
callbacks. A stop never releases work: "One supervisor retains inherited cancellation, original
deadlines, native destruction/join and charges through drain" (§18.8). Cancellation and deadline
stay distinct outcomes. Separately, a durable claim's lease must be renewed while its work runs,
and losing it must stop that work. All of that is promised. The two cancellation types are a
consequence of the FFI requirement, not an accident.

At `3e9a2525b` there are 19 `sleep_until` stop or deadline races in production code (18 in
`pse-runtime`). They fall into three shapes the codebase already distinguishes, though no type
names them:
- **Wake-source loops.** Cancellation and deadline are only wakes, and the loop re-checks:
  `jobs.rs` population admission, `staged.rs` admission, `initialization/dependencies.rs`.
  These are unbiased `select!`s whose arms are `{}`.
- **Abandon-safe races with stop first.** `select! { biased; cancel => Cancelled, sleep_until =>
  Timeout, work => … }`: compiler admission, pool retry waits and flight waits in
  `preparation.rs` and `modeling/preparation.rs`.
- **Signal-and-drain.** Signal the stop, then keep awaiting the same work: the native join in
  `jobs.rs` (inline, with a deadline arm and `source_owns_deadline`), and
  `CancelSource::join_native`, which is used by `staged.rs` and `workflow/run.rs` and has no
  deadline arm.

A `scope_stopped(&ExecutionScope) -> ProviderError` helper already exists in `math.rs`. It is
compiled only under `feature = "solver-petsc"`, and `staged.rs` has an inline copy.

Lease renewal is written twice:
- `durable.rs::Heartbeat` spawns a task with `interval(policy.heartbeat)`,
  `MissedTickBehavior::Skip`, a oneshot stop, a failure `Mutex` and a cancel-on-loss call.
  `stop()` waits for an in-flight renewal.
- `study.rs::admit_study_candidate` renews inline in a `select!` loop every `lease / 3`, ignoring
  the admitted `policy.heartbeat`. `work` is not polled while the renewal call awaits.

The defaults (lease 30 s, heartbeat 10 s) make the two cadences equal only by coincidence.

**Credible complete options.**
- *Retain current.*
- *Fold the deadline into a token* (a timer task cancels a `CancellationToken`). Ruled out: it
  merges `Cancelled` with the timeout outcome and loses `source_owns_deadline`
  ([tokio-util](../libraries/tokio/tokio-util.md)).
- *`timeout_at` / `run_until_cancelled`.* Both let ready work win over a ready stop and drop the
  work; ruled out for the stop-first and drain shapes ([LOP-X005](../assessment-patterns.md#lop-x005)).
  Acceptable only where neither ordering nor ownership matters, and no such site was found.
- *Replace the native `Cancellation` with tokio-util's token.* Ruled out: C callbacks need a
  runtime-free atomic, and the token exposes none.
- *A local typed stop value plus two combinators, and one lease owner* — recommended.

**Comparative argument.** A typed `StopReason` answers the tokio-util account's objection to
folding deadlines into tokens. The deadline joins cancellation as a *reason*, not as a cancellation,
so outcomes stay distinct and no timer task is created. The value already exists three times as a
pair: `CancelSource { token, deadline }`, `pse_kernels::ExecutionScope { control, deadline }`, and
loose `(Cancellation, at)` pairs in `jobs.rs`. Giving each pair one `stopped()` future, through a
small trait in `pse-runtime` so `pse-kernels` stays runtime-free, removes the repeated
`match deadline { Some(at) => sleep_until(at), None => pending() }` blocks. Each site keeps its own
`select! { biased; … }` with its own closed and wake arms, because ordering among closed, wake and
stop sources is site policy. Generalizing `join_native` to accept a stop value with a deadline makes
the `jobs.rs` native join use the same drain owner as sessions and runs. Its `on_stop(reason)`
callback lets the site decide whether to signal the native flag (the `source_owns_deadline` case).

For leases, one owner removes a duplicated loop and an observed cadence drift (DP-01: the admitted
policy is the single authority for heartbeat). It also replaces a failure `Mutex` with a
`JoinHandle` that returns the failure. Making the study renewal concurrent is a behaviour change.
Durable runs already renew concurrently with work that writes under the same fence
(`append_result_block`), so the store tolerates it. The inline pause looks incidental; whether it is
intended is one of the [open questions](README.md#open-questions).

**Revealing case.** *A simultaneous stop and ready result in an admission wait.* Under `biased`
ordering the stop wins and no permit is taken — the current contract. A helper that returned
`work` when both were ready would admit cancelled work. The recommendation keeps `select!` at each
site and only supplies the stop future, so the bias remains visible where the policy lives.
*Lease lost mid-renewal:* the current `Heartbeat::stop` waits for an in-flight renewal, and an
`AbortOnDropHandle` route would abandon the RPC midway. The owner must use a cooperative stop
(`CancellationToken`) and await the task, not abort it.

**Disposition: Recommend.** Bounded target:
- **Stop.** A `pse-runtime` trait `StopSignal { fn check(&self) -> Result<(), StopReason>; async
  fn stopped(&self) -> StopReason }` implemented for `CancelSource`, `ExecutionScope` and a
  `(Cancellation, Option<Instant>)` adapter. `StopReason` maps to the existing typed errors
  (`MathRuntimeError::Cancelled`, `admission_timeout()`, `ProviderError::{Cancelled, Deadline}`).
  It replaces `scope_stopped` (no longer feature-gated) and the inline copies.
- **Drain.** `drain_on_stop(stop, work, on_stop: FnOnce(StopReason) -> Drain) -> (T, Option<StopReason>)`,
  the generalized `join_native`, used by the native job join, the staged session, `workflow/run.rs`
  and `NativeThread` (CBO-018).
- **Lease.** `LeaseRenewal::spawn(store, fence, policy, on_loss) -> LeaseRenewal`, using
  `tokio::time::interval(policy.heartbeat)` with `MissedTickBehavior::Skip`, a child
  `CancellationToken` for stop and a `JoinHandle<Option<Arc<WorkflowError>>>`, plus
  `stop().await` and `scoped(work)` for the study form.
- **Retained rules.** A ready stop wins; cancellation never releases work; the deadline is the
  minimum over enclosing clocks (`with_deadline`, `child_with_deadline`, and `operation_scope`'s
  new first-poll minimum); the FFI flag stays readable without a runtime; renewal stop waits for an
  in-flight renewal.
- **Intended behaviour changes.** Study admission renews concurrently, at `policy.heartbeat`.
  Generalizing `join_native` also gives the staged-session and `workflow/run.rs` drains a deadline
  arm they do not have today; that is a behaviour change to accept explicitly, or the helper takes
  the deadline as optional so those drains keep their current form. *(Revised after the omission and preservation challenge.)*
- **Machinery that disappears.** About 15 deadline-arm blocks, the duplicate `scope_stopped`, the
  inline native-join race in `jobs.rs`, `Heartbeat`'s failure mutex and oneshot, and the study
  renewal loop.
- **Added.** One trait with three impls, one drain function, one lease type.
- **Not adopted.** Library combinators beyond those already in use.

**Rule impacts.** None. The work stays within §18.8 and the §21 lease policy contract. The
cadence change is a bug-fix-class alignment with the admitted policy; no ADR.

**Evidence.** Target *Proposed*. *Interface-checked*:
- the three wait shapes in `jobs.rs`, `staged.rs`, `preparation.rs`,
  `modeling/preparation.rs`, `initialization/dependencies.rs`, `artifacts.rs`, `math.rs`;
- `cancel.rs::join_native`, `pse_diagnostics::Cancellation`, `pse_kernels::ExecutionScope`;
- `durable.rs::Heartbeat` and `LeasePolicy::default`, `study.rs::admit_study_candidate`;
- tokio `Timeout` poll order and tokio-util `run_until_cancelled` bias, as recorded in the library
  accounts.

<a id="cbo-013"></a>

## CBO-013 — One lifecycle event fanned out to several hand-fed observation sinks

**Responsibility and target intent.** Native lifecycle boundaries (attempt, admission wait,
admitted, dispatched, stop requested, deadline expired, joined) must reach these observers:
- the observation hub, which works with no subscriber;
- the operational counts, which start before any exporter, saturate visibly and are read by passive
  checkpoints and OTel callbacks;
- tracing (spans and diagnostic events), which is optional and may be sampled;
- the invocation recorder, which is bounded at 64 intervals and 128-byte names.

§23.1 promises that "Native operational … populations come from their performing owners" and that
"Sampling off does not disable owner metrics". The four sinks are therefore deliberately
independent. What is incidental is that each boundary spells its sink calls and names by hand.

**Premise corrections (evidence at `3e9a2525b`).**
- The capture layer (`tooling/capture/layer.rs`) is *not* hand-fed. It is a `tracing_subscriber::Layer`
  deriving its lifecycle packets from span and event callbacks. There are four hand-fed sinks, not
  five.
- "About 81 call sites keep `stage`, `phase` and the operational metrics in step" overstates the
  footprint. `invocation::stage` has 9 production sites. `observation::phase` has 97, but nearly all
  are single-sink phase guards. The genuinely multi-sink boundaries are concentrated in:
  - the native supervisor in `jobs.rs` (`NativeAttempt` counts, hub phase and booleans, a tracing
    span, `pse.telemetry.diagnostic` events, the recorder stage);
  - the staged session request path in `staged.rs`;
  - the Python pull, through `telemetry::lifecycle` in `pse-py`.
- Boundary names are a de facto vocabulary. `tooling/inspection.rs` matches the hub boolean
  `"native.joined"` by string. `"native.cancel.requested"` is emitted from `jobs.rs`, `staged.rs`
  and `pse-py`.
- `operational_metrics::NativeAttempt` is already half a lifecycle owner. It has `admitted()`,
  `dispatched()`, `cancelled()`, `expired()`, an `OwnerInterval` histogram and supervisor
  accounting. Joined is counted separately (`native_joined()`) inside the `spawn_blocking` closure.

**Credible complete options.**
- *Retain current.*
- *Tracing as backbone*: one layer feeds the hub, recorder and counts. Ruled out:
  - nothing is observed without a subscriber;
  - filters remove spans;
  - `pse-diagnostics` has no tracing dependency (structural);
  - counts must start before an exporter.

  This would be a regression ([tracing](../libraries/tracing/README.md)).
- *`metrics` facade* or *OTel synchronous instruments*. Ruled out: no recorder or provider means lost
  counts, there is no saturation flag, and the second facade has no in-process read
  ([metrics](../libraries/metrics/README.md), [opentelemetry](../libraries/opentelemetry/README.md)).
- *A local, owner-typed lifecycle event* — recommended.
- *Declaration-driven instrument installation* (`MetricDeclaration` drives the observable builders).
  A small optional local table. The callbacks differ by source, so the gain is limited.

**Comparative argument.** The repository already reached the right shape in `NativeAttempt`. Moving
the hub booleans, the phase guards, the tracing events and the joined count into the same owner
makes each boundary one call, and keeps each sink's absence semantics inside the owner (hub
`Option`, recorder `Option`, always-on atomics, no-op tracing). A closed boundary enum gives
inspection a typed key rather than a string. The decisive constraint is *population ownership*: an
operational count belongs to the owner whose population is declared. The native supervisor's
`native_cancel_requests` is defined as "User-cancellation branch selections while join was still
pending" in that route. So the event type must be per owner (`NativeJobLifecycle`,
`NativeSessionLifecycle`, `PythonPullLifecycle`), sharing the *vocabulary* and the hub and tracing
emission but not the counters.

**Revealing case.** *A staged session cancel.* Today it emits the tracing code
`native.cancel.requested` but does not increment `NativeEvent::CancelRequested`. A single shared
lifecycle type that always counted would silently change the declared population of an exported
metric, which query descriptors and evidence documents cite. Owner-typed events with a shared
vocabulary preserve that. *A boundary with no recorder interval:* most boundaries have none, and the
recorder's 64-interval cap must not be spent on fine boundaries. The owner method decides which
boundaries are recorder intervals (today only `native.worker_execution`), rather than every event
becoming one.

**Disposition: Recommend.** Bounded target:
- **Vocabulary.** A closed `LifecycleBoundary` enum (admission wait, admitted, dispatched, stop
  requested, deadline expired, drain settled, joined) whose `as_str()` is the existing hub, tracing
  and inspection name. Its home is decided with G8: `pse-diagnostics` or the vocabulary module.
- **Owners.** `NativeAttempt` becomes `NativeJobLifecycle` with one method per boundary, each
  emitting hub, counts and tracing (and the recorder where declared). The staged session and the
  Python pull get their own lifecycle types emitting hub and tracing only. `telemetry::lifecycle`
  becomes their tracing emission.
- **Consumers.** `jobs.rs` supervisor, `staged.rs` request and session paths, `pse-py`
  `blocking_native_call`, `tooling/inspection.rs` (matches the enum's name),
  `NativeThread`/`drain_on_stop` emission points (CBO-015, CBO-018).
- **Retained.** All four sinks remain independent; observation without a subscriber; counts before an
  exporter; "unavailable" distinct from zero; recorder bounds; populations at their owners; the
  capture layer as a tracing consumer.
- **Disappears.** Paired `observation::current().map(|c| c.boolean(..))` and
  `tracing::info!(code = ..)` lines at each boundary; `native_joined()` as a free function; string
  literals duplicated across three crates.
- **Optional.** Drive `install_runtime_metrics`'s name, description and unit from
  `MetricDeclaration` in a small table. Low value.

**Rule impacts.** None. Metric names, populations and hub field names are unchanged, so no
generated query descriptors move. If the vocabulary enum lands in `pse-diagnostics` and generated
consumers read it, `just codegen` applies under the usual route.

**Evidence.** Target *Proposed*. *Interface-checked*:
- `tooling/capture/layer.rs`, `tooling/operational_metrics.rs` (`NativeAttempt`, `NativeEvent`,
  snapshot), `jobs.rs` boundary emissions;
- `staged.rs` cancel emission, `tooling/inspection.rs` string match, `telemetry.rs::lifecycle`;
- `invocation.rs::stage` (9 call sites, counted with `rg`);
- the §23.1 text on owner populations and sampling.

<a id="cbo-018"></a>

## CBO-018 — Two executor threads per native job to wait for a join

**Responsibility and target intent.** Native work runs on a named thread with an explicit stack.
Completion, and the release of CPU, slot, compiler exclusion and storage, are signalled only after
the thread has exited and its thread-local and foreign destructors have run (§18.8: "closing the
session joins its thread, which witnesses native and thread-local destruction"). A panic maps to a
typed error. That is all promised. Using a blocking-pool thread to perform the join is incidental.

**Premise corrections.**
- Only two production sites await a join through `spawn_blocking`: the per-job join in `jobs.rs`
  and the per-session join in `staged.rs`. `window_team.rs` and `surrogate.rs` join synchronously in
  `Drop` on the thread that owns them, which costs no executor thread.
- The staged join occupies its blocking thread for the *whole session lifetime*, including while
  the lane is parked idle in the service pool. That is a larger footprint than "per job", though
  still bounded by the lane population.
- `shutdown_native_lanes` uses the job-permit population as its drain witness (slots are dropped
  only after join), so a separate task tracker would duplicate it.

**Credible complete options.**
- *Retain current.*
- *Thread pidfd readiness* (`linux-pidfd/thread-exit-readiness`). Inside the new thread,
  `rustix::process::pidfd_open(gettid(), PidfdFlags::from_bits_retain(PIDFD_THREAD))` with the fd
  handed back. Outside, `tokio::io::unix::AsyncFd::readable().await`, then a non-blocking
  `JoinHandle::join()`. Needs tokio `net` declared (present in the unified graph) and kernel 6.9 or
  later (host is 7.0, an accepted floor).
- *A dedicated runtime with `thread_stack_size`*: runs async tasks, not a `!Send` native owner
  that must be constructed, used and destroyed on one thread. Ruled out.
- *A completion message sent from inside the closure*: it fires before thread-local destructors.
  Ruled out by the preservation constraint.
- *A local `NativeThread` owner*, keeping `spawn_blocking(join)` internally — recommended.

**Comparative argument.** The repeated responsibility is larger than the join. Each site names
the thread, sets the stack, enters context (CBO-014), maps panics, counts joins (CBO-013) and races
the join against stop and deadline (CBO-015). One owner holds all of that. Whether the join wait
uses a blocking thread or a pidfd then becomes an internal detail that can change without touching
consumers. The pidfd route's benefit is one parked thread per running job or retained lane. No
measurement shows that this matters at the workstation's population, and tokio's blocking pool is
far from its limit. Its cost is fd hand-off code, a new reactor registration and a new tokio
feature. Kernel ordering supports it: in `kernel/exit.c` (v6.12) `do_exit` calls `exit_mm()`, whose
`exit_mm_release` performs the clear-child-tid futex wake, before `exit_notify()`, which notifies
`PIDFD_THREAD` waiters for sub-threads. So a later `pthread_join` should not block. Thread-local
destructors run in user space before the exit syscall. Still, the benefit is not demonstrated, so
the swap is not recommended now.

**Revealing case.** *A cancelled caller drops its future while the native thread runs on.* Today the
supervisor task owns the join (the `Caller` guard requests a stop on drop, and the spawned
supervisor keeps joining). A `NativeThread` whose join future detached the thread when dropped
would release slot and storage while native code runs, violating DP-20 ("An opaque kernel still
running after cancellation retains its resource charge"). The owner's join must stay owned by the
supervisor task, exactly as now. This is LOP-X005 again, and it is why the owner is not a
"fire and forget" spawn helper.

**Disposition: Recommend (consolidation); pidfd swap deferred.**
- **Mechanism.** `NativeThread::spawn(name, stack, ctx: ExecutionContext, on_joined, work) ->
  NativeJoin`, where `on_joined` is the owner's join observer (a small trait or closure), not a
  shared lifecycle type. `NativeJoin::drain(stop, on_stop)` uses CBO-015's `drain_on_stop`, waits
  through `spawn_blocking(join)`, calls the owner's observer and maps panics to
  `MathRuntimeError::Panic`. *(Revised after the omission and preservation challenge.)* A shared `&NativeJobLifecycle` would make staged-session
  joins count in `native_joined()`, which today only the native job (`math/jobs.rs`) feeds.
- **Consumers.** The `jobs.rs` native job and the `staged.rs` session thread. Synchronous
  join-on-drop sites stay as they are.
- **Retained.** Join witnesses TLS and foreign destruction; resources are released after join; the
  explicit stack and name; staged-session joins are not counted as native jobs joined.
- **Conditional branch.** If a later measurement shows blocking-pool pressure (for example a much
  larger retained-lane population), swap the internal wait to thread pidfd readiness. The host floor
  makes a fallback unnecessary, and the probe below would confirm the non-blocking join.
- **Machinery.** Removes two inline spawn/join blocks; adds one small type.

**Rule impacts.** None.

**Evidence.** Target *Proposed*.
- *Interface-checked*: `jobs.rs` and `staged.rs` join sites, `window_team.rs` and `surrogate.rs`
  `Drop` joins, `staged.rs::shutdown_native_lanes`.
- *Interface-checked* (ordering only): kernel v6.12 `kernel/exit.c` order of `exit_mm` and
  `exit_notify` and the `PIDFD_THREAD` notify; the body of `exit_mm_release` was not re-read.

<a id="cbo-017"></a>

## CBO-017 — Signal-aware blocking waits at the Python boundary written per call

**Responsibility and target intent.** A synchronous Python call stays interruptible while native
work runs detached. On a signal the site applies its own stop policy, then raises the Python signal
exception with the diagnostic as `__cause__` where one exists, and never attaches Python from
worker callbacks. §21 promises: "Python interruption drains the original operation and preserves
any actual acknowledgement identity on the raised exception."

The sites differ deliberately, and a helper must preserve that:
- `blocking_native_call`: cancel, then join, then raise with cause; panic after drain is reported.
- `blocking_on_with_interrupted_value`: cancel, keep polling the retained future to completion,
  then attach recovery keys (`analysis_key`, `artifact_intent`, …) and raise. It also checks once
  more after the final poll, so an acknowledged result still reports its keys.
- Capture join (`diagnostics.rs`): `request_stop`, which stops recording and not science, then
  continues waiting; 20 ms interval.
- `NativeLaunchedCommand.wait`: cancel the command, which returns its status, and raise at once.
- `task_dump`: no cancel ("an interrupted observer leaves its issued ticket with the actual
  owner"); raise at once.
- `next_batch`: pre-check only, so a pending signal does not issue a request.

Intervals are 100 ms or 20 ms.

**Credible complete options.**
- *Retain current.*
- *pyo3-async-runtimes.* Drop-based cancellation, which cannot "cancel then drain then raise with
  cause", and it fails inside a running loop. Ruled out for synchronous calls; correct for the
  existing `wait_async` path.
- *signal-hook.* An extension does not own the process signal table. Ruled out.
- *CPython `signal.set_wakeup_fd`* inside one helper (`cpython/signal-wakeup-fd`).
- *A local helper with one interval and an explicit per-site policy* — recommended.

**Comparative argument.** PyO3 offers no wakeable `check_signals`. Context7 `/pyo3/pyo3` (FAQ and
guide) recommends calling it regularly, so some loop is unavoidable. A helper parameterized by a
detached "wait up to Δ for completion" closure and a site policy, `OnSignal::{Drain(stop),
Return(stop)}`, removes the repeated loop, interval and ordering. Cause attachment, recovery keys
and panic reporting stay at the site (DP-21: structured diagnostics stay with their owner).

The wakeup fd would remove interval latency only for main-thread waits, and only if every
completion source (mpsc, tokio future, child command, capture session) were adapted to an fd.
It is also a single process-global slot that asyncio and IPython install, and it must be saved,
restored and forwarded around each wait. That is real complexity and a new failure mode for
embedding hosts, bought for sub-interval latency on Ctrl-C, which no consumer has asked for.

**Revealing case.** *A user interrupts an analysis whose result was acknowledged during the final
poll.* A helper that raised as soon as it saw the signal, before checking for a ready value, would
lose the recovery keys that §21 promises. The helper's contract therefore puts "check completion
first, then signal" in each iteration and returns `(Option<T>, Option<PyErr>)` to the site, which
reproduces `blocking_on_with_interrupted_value`'s current ordering. *Interrupting a capture join:*
mapping every signal to `cancel` would stop the measured command rather than the recorder. The
site's `OnSignal` value keeps `request_stop`.

**Disposition: Recommend (local helper); wakeup fd not recommended.**
- **Mechanism.** `pse-py` `interrupt::wait(py, interval, poll: impl FnMut(Duration) -> Option<T>,
  on_signal: OnSignal) -> (Option<T>, Option<PyErr>)`, with an optional pre-check, a single declared
  interval, and lifecycle emission (`stop requested`, `drain settled`) through CBO-013's vocabulary.
- **Consumers.** The six sites listed above.
- **Retained.** GIL released while waiting; per-site stop action; cause and recovery-key
  attachment; panic-after-drain reporting; no attach from worker callbacks.
- **Behaviour change.** One interval for all sites. The implementing plan chooses the value; making the
  20 ms sites slower or the 100 ms sites faster is a latency trade with no correctness effect.
- **Conditional branch.** Reconsider the wakeup fd only if a host-specific latency requirement
  appears. Even then, restrict it to main-thread waits with save and restore, and probe it under an
  asyncio loop and IPython first.

**Rule impacts.** None. This sits inside the Python boundary's existing behaviour and is not a
change to the boundary contract.

**Evidence.** Target *Proposed*. *Interface-checked*: the 10 `check_signals` sites in
`pse-py/src/workflow.rs`, `diagnostics.rs` and `inspection/stream.rs`; Context7 `/pyo3/pyo3` FAQ
"Ctrl-C doesn't do anything while my Rust code is executing"; library accounts for the wakeup fd,
signal-hook and pyo3-async-runtimes.

<a id="cbo-010"></a>

## CBO-010 — Conflict retry and guarded-transaction preludes assembled per site (retry policy part)

*The SurrealQL prelude part belongs to G3.*

**Responsibility and target intent.** Retry definite transaction conflicts and capture-busy refusals
within the original request clock. Do not retry after cancellation. Rebuild the complete decision on
each attempt, and attribute exhaustion as `ConflictExhausted { operation, attempts }`.

Where the outcome is uncertain (a lost acknowledgment or commit), settle it from the immutable
receipt before any further effect, and never blindly replay. §21: "Unknown acknowledgments reconcile
immutable operation receipts before another effect". `CanonicalError::retryable()` says such an
observation "never grants replay of an uncertain mutation". All of that is promised.

**Footprint at `3e9a2525b` (larger than the account recorded).** There are ten loops in
`pse-operations`, using five pacing forms:
- `10ms << min(a,4)` plus 0–63 ms uuid jitter: `transport::protected_query`, `canonical.rs`
  problem-turn variant, `activate_source_stage`, `admit_product`, `retry_stage_mutation`.
- The same without jitter: `begin_stage_inner`.
- `yield_now`: `cleanup_stage_query` and two loops in `canonical_retention.rs`.
- `1 << a` plus 0–15 ms jitter: `initialize`.
- A fixed 20 ms contention wait: `acquire_source_stage`.

Caps: `GUARDED_ATTEMPTS` and `MUTATION_CONFLICT_ATTEMPTS` (both 32), two separate `RETRIES = 8`, and
a literal 8. The capture-busy class is uncounted, bounded only by the clock. The `conflict()`
predicate is duplicated in `canonical_staging.rs` and `canonical_retention.rs` beside
`CanonicalError::is_conflict()`.

At least three loops interleave **settlement** between attempts:
- `activate_source_stage` reads the operation receipt and may return it, refuse on
  `OperationReused`, or settle;
- `admit_product` checks `product_acknowledged`;
- `acquire_source_stage` re-reads the source-edit receipt.

**Credible complete options.**
- *Retain current.*
- *backon 1.6* `Retryable` + `when` + `adjust` + `notify`, wrapped in the existing `within_clock`
  ([backon](../libraries/backon/README.md)). Clock and drop semantics fit. Reproducing the uncounted
  busy class and the repository's pacing requires `adjust` with private counters, which leaves the
  builder as a shell. Settlement must hide inside the factory.
- *tryhard 0.5.2.* Busy retries would consume the budget, and `on_retry` is `tokio::spawn`ed
  (detached work after cancellation). Weaker than backon.
- *A local retry owner with an attempt classification that the step returns* — recommended.

**Comparative argument.** The value of a retry library is its loop. Here the loop is the smallest
part: classification needs the typed error *and* sometimes a durable readback; pacing is
repository policy; the clock is the repository's task-local. A local owner whose step returns
`Attempt::{Done(result), Conflict(error), Busy(error)}` lets settlement-interleaving sites decide
classification after their readback, where backon would force that into the factory. The owner can
hold the policies as named constants (`GUARDED`, `RETENTION_ROOT`, `INITIALIZATION`,
`STAGE_CONTENTION`) instead of five inline formulas, and performs `exhaust_conflict` and
`operation_failed` once. It changes no SurrealQL, so it composes with any G3 outcome. Adding backon
would add a dependency whose contribution after configuration is about fifteen lines, with
pacing semantics (proportional jitter) that differ from current behaviour. That is not a clear win.

**Revealing case.** *A lost acknowledgment on a source activation that actually committed.* A
generic retry that classified `IncompleteResponse` or a conflict as "retry" without the receipt
readback would replay a committed mutation. That violates "never blindly rerun" and could mint a
second acknowledgment path. The step-returns-classification design keeps the readback before the
decision, which is the case that rules a combinator-first design out for these loops. The framing is
recorded as [LOP-X012](../assessment-patterns.md#lop-x012).

**Disposition: Recommend (local owner); backon and tryhard not adopted.**
- **Mechanism.** `pse_operations::transport::retry_conflicts(policy: RetryPolicy, operation,
  step: impl FnMut(u32) -> Fut<Output = Attempt<T>>) -> Result<T, CanonicalError>`, always run inside
  `within_clock` or the caller's `operation_scope`. `RetryPolicy { attempts, pacing:
  Pacing::{Exponential { base, cap_shift, jitter }, Yield, Fixed(d)} }`.
- **Predicates.** One `is_conflict` and one `is_capture_busy`; the duplicate `conflict()` helpers
  are deleted.
- **Retained.** Every current cap and pacing as named policies; the uncounted busy class; the
  original clock; no detached retry; per-attempt rebuild; settlement before the next effect; typed
  exhaustion.
- **Open question for the maintainer** (listed in the [open questions](README.md#open-questions)). Is `begin_stage_inner`'s missing jitter intentional? If
  not, aligning it to the guarded policy is a clear-win minor change. Otherwise keep a named
  no-jitter policy.
- **Machinery.** Removes ten loop bodies and five formulas; adds one function and one policy enum.

**Rule impacts.** None. This is a refactor within the retry contract. It is independent of G3's
prelude decision.

**Evidence.** Target *Proposed*. *Interface-checked*: every loop listed (`transport.rs`,
`canonical.rs`, `canonical_staging.rs`, `canonical_retention.rs`), `CanonicalError::{is_conflict,
is_capture_busy, exhaust_conflict, retryable}`, `operation_scope` at `ee44d218d`, and the backon and
tryhard accounts.

<a id="cbo-011"></a>

## CBO-011 — Single-flight, I/O gates and load admission beside caching libraries

**Responsibility and target intent.** Deduplicate concurrent loads under a finite key and waiter
limit. A native producer keeps its leases until it actually exits, even after every waiter leaves;
the last waiter requests cancellation and joins the producer's return. Errors are typed and shared,
and a retiring flight is a typed refusal. One shared I/O gate covers every store, with nested
wrappers borrowing the enclosing permit and permits released between list items. Load admission
pairs a semaphore permit with a pool reservation. The DataFusion pool is the single budget. All of
this is promised (§18.8 and DP-20).

**Options and argument.** The library accounts assessed every candidate and each is ruled out on
completion ownership or gate semantics:
- moka `try_get_with`, quick_cache placeholder guards and async_singleflight leaders run the producer
  in the caller and drop it with the caller;
- `LimitStore` has a private gate, no re-entrancy, and releases early for file payloads;
- `TaskTracker` supplies no flight semantics.

I re-read `flight.rs`, `stores/admission.rs` and `cache_service/load.rs`. The composition is already
library-led where it can be (`tokio::spawn` for producer survival, `Shared` over a oneshot for
fan-out, a `Cancellation` for the stop, `select! { biased; stop … }` for the waiter). The remaining
local code is the semantic core. *Retain current* is the strongest complete option.

**Premise correction.** `pse-engine/src/cache_service/flight.rs` (a DataFusion-error wrapper over
`pse_columnar::flight::Flights`) has **no consumer** in the workspace. It is declared `pub mod
flight` and nothing instantiates it (`rg` over `crates/`). The production flights are the five
`MathService` tables and `workflow/physical_cache.rs`'s admission flights, all using the neutral type
directly. The "cache service" consumer named in the opportunity therefore does not exist. Deleting
the wrapper is a clear-win minor change (DP-16: a bespoke layer with no current requirement).

**Revealing case.** *The last waiter leaves while a native load is mid-factorization.* A drop-based
library flight would drop the producer, releasing its prepaid leases while the native thread runs.
That is exactly the regression DP-20 forbids. `load_owned_until` instead requests cancellation and
awaits the producer's return before the caller releases its own resources.

**Interaction.** Runtime callers capture relationship-aware `ExecutionContext` and instrument
their supplied producer future. The neutral flight owner retains its shared-production
observation scope and completion ownership. It cannot import the proposed runtime type:
`pse-runtime` already depends on `pse-columnar`. Any extra propagation hook must stay generic
and carry no runtime policy. `PopulationScope` and prepaid authority remain explicit.

**Disposition: Retain current**, plus deletion of the unconsumed `pse-engine` flight wrapper.

**Rule impacts.** None.

**Evidence.** *Interface-checked*: `pse-columnar/src/flight.rs`, `pse-engine/src/cache_service/{flight,load}.rs`,
`pse-engine/src/stores/admission.rs`, consumers in `pse-runtime/src/math.rs`, `math/preparation.rs`,
`math/artifacts.rs`, `workflow/physical_cache.rs`, `workflow/modeling/canonical.rs`; the
negative-consumer search covered `crates/**/*.rs`.

<a id="cbo-016"></a>

## CBO-016 — Waiting by polling for state owned elsewhere (in-process side)

*The store-side half (LIVE SELECT, changefeeds) belongs to G3; the `observe-primary` subprocess
belongs to G1 (CBO-020).*

**Responsibility and target intent.** Waiters observe state whose authority is a canonical claim or
row, or a primary receiver in another process. Correctness never depends on notification delivery;
observation grants no release authority. Polling with leases is the documented contract.

**What is in-process.** Classifying the listed sites at `3e9a2525b`:
- **Worker discovery** (`worker.rs`: 500 ms poll, 30 s recovery) waits for attempts that are usually
  submitted by *another process*, because `pse-worker` is separate from the submitting host.
- **`wait_study_inventory`** (100 ms) waits on canonical inventory written by workers.
- **`acquire_source_stage`'s 20 ms wait** waits on another writer's stage.

All three are cross-process. The sleep-poll loops cited in `pse-operations` are nearly all inside
`#[cfg(test)]` modules: the protection-release retries in `canonical_result_retention.rs` and
`canonical_lifecycle.rs`, `canonical_artifacts.rs` and `canonical_execution.rs`. Production
`forget_run_results` refuses rather than waits. The in-process change-notification capability is
already present and used where producer and waiter share a process:
- `pse_diagnostics::WakeSignal` (register-then-check, used by the native exclusion gates and
  retention idle waits);
- tokio `Notify` (worker publication handoff, `PopulationLane::drain`, `shutdown_native_lanes`);
- `TaskTracker` (`ResultReadDrain` release tasks).

**Options and argument.** Adding an in-process hint, such as a `WakeSignal` published on local
submission, helps only when submitter and worker share a process. That is not the primary topology,
so it would add a second wake path for little effect. A protection-release signal would serve only
tests. The test helpers could instead await the release through a test-support hook on the existing
`TaskTracker`, without new production machinery. The genuine improvements are cross-process and
belong to G3 (store notification as a hint) and G1 (owner-change observation without spawning an
interpreter).

**Revealing case.** *A study waiter in the same Runtime as its worker.* An in-process hint would cut
latency there, but the authoritative inventory read must still run after the wake, and the
cross-process case still needs the poll. The hint adds a path without removing one.

**Disposition: Retain current** for the in-process side.
- **Optional minor.** In `wait_study_inventory`, race the 100 ms sleep with `cancel.cancelled()` so
  cancellation is recorded promptly rather than at the next tick. This is a one-line change with no
  semantic effect.
- **Conditional on G3.** If G3 adopts a store subscription, it should enter these waits as a hint
  that triggers the same authoritative re-read, with the current poll as fallback. No G2 machinery is
  needed beyond `select!`.

**Rule impacts.** None.

**Evidence.** *Interface-checked*: `worker.rs` discovery loop, `study_execution.rs::wait_study_inventory`,
`canonical_staging.rs::acquire_source_stage`, the test-module placement of the `pse-operations`
sleep loops, and `WakeSignal`, `Notify` and `TaskTracker` users.
