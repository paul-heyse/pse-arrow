# Execution lifecycle, concurrency and observation

This page covers `pse-runtime` sessions, native jobs, durable workers and studies, cancellation,
observation and telemetry, and the Linux process facts the host and runtime read. Contracts are owned by
[§18](../../authoritative_design/sections/numerical-execution.md#section-18),
[§21](../../authoritative_design/sections/workflows-and-results.md#section-21) and
[§23](../../authoritative_design/sections/operations-and-validation.md#section-23). The baseline is on
the [entry page](README.md).

**System picture.** The runtime owns a process-wide `SharedRuntime`: a DataFusion `FairSpillPool`, a
shared resource budget, job and CPU semaphores, and a math service that runs native jobs on dedicated OS
threads. Those threads are joined, together with thread-local foreign destructors, before ownership is
released. Consumers are operators, review and capture tooling, durable workers and studies.

Already strong, and not re-opened here:
- `ExecutionContext` captures and enters attribution state once, and `NativeThread` with its custody
  carrier owns native thread spawn and join for jobs and sessions.
- `pse_diagnostics::stop::{stop_first, drain_on_stop}`, `CancelSource::join_native` and
  `LeaseRenewal::spawn_with` are shared cancellation and renewal owners.
- `pse_diagnostics::lifecycle::LifecycleBoundary` is the shared vocabulary for discrete native
  boundaries.
- `tokio_util::CancellationToken`, `Semaphore`, `Notify`, `FuturesUnordered`, `tokio_metrics`, the
  OpenTelemetry `BatchSpanProcessor`, `rustix` pidfd calls, and the host's systemd job and property
  signals (`JobRemoved`, `PropertiesChanged`) are used idiomatically.
- `pse_diagnostics::supervised_child::SupervisedChild` is the one child-process owner for compilers,
  capture launches and host preparation.

Legitimately custom: the paired CPU and memory permit protocol under which no native owner waits for
memory, joining before release, claim fences and leases on the canonical store, the closed
host-operation catalogue, and the same-host, no-daemon diagnostics model.

**Coverage on this page.** Read closely: `execution_context`, `native_thread`, `cancel`, the `durable`
lease and streamer, the `study_execution` frontier and closing sets, the `worker` dispatch loop, the
`math/staged` and `math/initialization` schedulers, the hub's `start`, `update_at` and first part of
`OperationReader::snapshot`, telemetry loss and registration, and in `pse-host` the manager, admission,
foreground and process-fact parsers. Only skimmed: `workflow/run`, `workflow/staged`, `completion`,
`study_diagnostics`, `reproducer`, `horizon`, `strategies`, `observation_detail` collection start,
`tooling/inspection*` comparison statements, `capture/cpu.rs`, `manager/subgroup`, and the bodies of
`pse-host/tests`. `execute_lane` and the publication handoff were not read.

<a id="cbo-079"></a>

## CBO-079 — Operation custody carriers and per-layer re-admission

- **Operation and consumers.** Every request, spawn and task crossing needs the current operation's
  attribution, activity authority, clock and response custody. Consumers are modeling preparation, the
  canonical transport, workers and math lanes.
- **Current mechanism.** Attribution is consolidated in `ExecutionContext`, composed with
  `inherit_runtime_activity`. Authority travels separately. `pse-operations` has eight non-test task-local
  carriers (`OPERATION_DEADLINE`, `RESPONSE_OWNER`, `SELECTION_CUSTODY` and others), each with a hand
  `pin_project` adapter that re-enters it per poll. `pse-math` adds thread-locals
  (`construction::OWNER`, `library::CURRENT_ABORT`, `typed_receipts::SESSION`). Preparation layers each
  repeat `admit_activity`, `check_service_bindings` and `preparation_driver` (about 380, 154 and 48
  matches), and about 45 `too_many_arguments` escapes carry services through signatures.
- **Why it matters.** A missed carrier silently loses a charge, attribution or custody on one path
  (Proposed). The repeated preamble also hides which layer is the real gate.
- **Patterns.** LOP-X002 (primary), LOP-08, LOP-09.
- **Evidence.** `canonical_transport.rs`, `transport_lifetime.rs`, `request.rs`, `workflow/mod.rs`,
  `execution_context.rs`, `math/preparation.rs`, `modeling/cases.rs`, and `pse-math` `construction.rs`,
  `library.rs`, `typed_receipts.rs` were read. Counts are `rg` shape indicators.
- **Footprint.** Inspected: those owners and representative call sites. Unread: remaining composition and
  raw `tokio::spawn` sites.
- **Preservation constraints.** Activity is authority and attribution is not (§20.6): copied attribution
  alone cannot mint continuation. `without_request` strips attribution deliberately for shared producers.
  Nested expiry reports the original clock, and a held activity never outlives drain. No guard crosses an
  await. A no-owner math scope stays a pure use.
- **Counterevidence and uncertainty.** Distinct types stop authority being confused with attribution.
  Nested `admit` returns the existing continuation, so the cost is source burden. `check_service_bindings`
  at public entries is a real trust boundary, and Tokio has no task-local inheritance. A small typed
  admission context or one spawn helper may need no library.
- **Status.** Current mechanism.
- **Research questions.** Could one proof-of-admission value, obtained at the public edge, replace the
  repeated preamble? Can a single spawn point absorb capture and reject raw spawns? Do `tracing` span
  extensions or `TaskTracker` serve foreign callbacks, given the hub must work without a subscriber?

<a id="cbo-084"></a>

## CBO-084 — Record families and metric units are free strings across crates

- **Operation and consumers.** Native owners publish `CoherentRecord`s and `Metric` observations. Review
  analysis, inspection statements, frozen ingress and the Arrow detail views interpret them, and operators
  and evidence documents depend on the answers.
- **Current mechanism.** A family is a free string such as `"evaluator_demand_batch.v1"`, with fields
  built from string and unit tuples (`pse-math::assembly_observation`). Interpretation re-spells it:
  `family_question` in `review_analysis/questions.rs` is a string `match`, and `inspection/performance.rs`
  and `inspection.rs::native_detail_explanations` read the same records. Family literals recur across
  crates, and `.config/preparation-cases.json` lists `detail_families` again. About 547 `Metric::*(`
  constructions use free strings, and `pse-backend-native/src/observation.rs::metric_unit` is a string
  match defaulting to `"native_unspecified"`. Evidence references are hand-formatted JSON-pointer strings
  resolved only by tests.
- **Why it matters.** A renamed field or unit degrades to "unavailable", which the system treats as honest
  missing evidence, so a typo looks legitimate and no compiler error appears. Adding a field is a
  cross-crate edit (Proposed).
- **Patterns.** LOP-02, LOP-X007, LOP-06.
- **Evidence.** `questions.rs` (`family_question`, `evaluation_cost`), `observation/producer.rs`,
  `metric_unit`, `ipopt/sequence.rs::terminal_timing`, and literal counts by `rg`.
- **Footprint.** Inspected: interpreter and producer types. Located only: producer call sites in
  `pse-backend-native` and `assembly_observation`.
- **Preservation constraints.** Self-describing records let frozen readers work without producer code.
  Unavailable stays distinct from zero. Hot-path prepared facts do not allocate, and old versions stay
  readable. Producers cannot depend on `pse-runtime`.
- **Counterevidence and uncertainty.** Native-name metrics are deliberately open, "unspecified" is an
  honest label, and sparse foreign families may not fit a struct. The proposal concerns the declaration,
  not the wire form. A declaration crate would sit below `pse-math`; `pse-diagnostics` already does.
- **Status.** Current mechanism.
- **Research questions.** Could a small derive, a declarative macro or the `pse-schema` registry, which
  already owns diagnostic views, be the single authority? Does registry codegen reach below `pse-math`?

<a id="cbo-016"></a>

## CBO-016 — Waiting by polling for state owned elsewhere

- **Operation and consumers.** Waits on state someone else owns: workers discovering claimable attempts,
  study waiters and observers, cancellation reaching a running attempt, and host waits on child exit,
  unit state, launch locks and capacity.
- **Current mechanism.** Runtime waits are timers: `worker.rs` `settings.poll` (default 500 ms) plus a
  recovery cadence, `wait_study_inventory` at 100 ms, `StudyWaitControls.poll_seconds` and
  `study_observation.rs`. A running attempt learns of cancellation when lease renewal is refused. The
  client refuses LIVE by design (`UNSUPPORTED_LIVE_RESPONSE`; §20.6 infers no live subscription lifetime),
  and the schema has neither LIVE nor `CHANGEFEED`. Study readiness is already a hand-built durable
  notification stream (`advance_study_readiness` reads rows after a `notify_after` cursor). In
  `pse-host`, `foreground` loops `try_wait` at 5 ms, installed-service start and stop poll at 50 ms beside
  job signals used elsewhere, flock retry at 5 ms recurs in four modules, and `Coordinator::admit` waits at
  50 ms. An observer poll's store-side cost is [CBO-071](persistence-and-transport.md#cbo-071).
- **Why it matters.** Latency has an interval floor and idle waiters load the store (Proposed). The host
  loops have event mechanisms nearby.
- **Patterns.** LOP-12, LOP-08, LOP-X005.
- **Evidence.** `worker.rs`, `study.rs` `wait`, `study_execution.rs`, `study_observation.rs`,
  `canonical_studies.rs` readiness methods, host `foreground`, `admission`, `manager`.
- **Footprint.** About ten production sites; `pse-operations` sleeps were not classified.
- **Preservation constraints.** Polling and leases stay the authority; a notification is a wake hint
  before the same guarded read, and a missed one loses nothing. Bounded requests, no daemon. A blocking
  flock cannot be cancelled, so deadline-bounded nonblocking retry has a reason.
- **Counterevidence and uncertainty.** Polling is simple, restart-safe and truthful. Idle cost is
  unmeasured, long solves may not care, and admission across arbitrary processes has no shared notifier.
- **Status.** Current promise (polling); wake notification Proposed.
- **Research questions.** Does LIVE or `CHANGEFEED` deliver under backpressure within the bounded profile?
  Would an in-process `Notify` cover the common case? Does the readiness cursor map onto a native change
  feed?

<a id="cbo-083"></a>

## CBO-083 — Owner-affine worker teams implemented separately

- **Operation and consumers.** Run non-`Send` native evaluators in parallel under admitted memory and CPU:
  fit experiments, shooting windows, the surrogate, and native sessions.
- **Current mechanism.** Hand-built forms share one shape: a named OS thread constructing its owners
  locally, a ready channel, a request channel, `catch_unwind`, drop-joins and ordered reduction.
  `ExperimentTeam` (`fitting/experiments.rs`) and `WindowTeam` (`shooting/window_team.rs`) use stride
  assignment; `math/staged.rs` sessions and `math/surrogate.rs::actor` are further actors, and the
  surrogate has two stacked command loops (`pse-egobox`, then a `JoinedPool` loop in
  `surrogate/library.rs`). `NativeThread` is shared only in part. Foreign-memory coverage is written twice
  and differs: `foreign.rs::Coverage` consults the parent for `Linear` storage "to preserve strict
  unknown-bound refusal", while `WindowCoverage` returns `Ok(())`.
- **Why it matters.** Panic classification, trial invalidation and observation transport change in several
  places. The coverage difference may be a semantic gap or justified; it is a question for review, not a
  claimed defect.
- **Patterns.** LOP-09, LOP-08, LOP-X002.
- **Evidence.** `experiments.rs` and `foreign.rs` in full, `window_team.rs` to `evaluate`, the surrogate
  actor, `staged.rs`.
- **Footprint.** Inspected as listed. Suspected, unread: `fitting/profile.rs::dispatch` and dynamics
  worker owners.
- **Preservation constraints.** Workers die on their owning thread before join, and thread-local
  destruction precedes charge release. A started experiment is never replayed, reduction order is
  independent of completion, and admission precedes construction. Native trial and terminal failures stay
  distinct.
- **Counterevidence and uncertainty.** Egobox strategies are `Sync` but not `Send`, so the surrogate
  layering has a stated reason. Retained residents and inline width-one mode are real differences, and a
  shared abstraction could cost more than the copies.
- **Status.** Current mechanism.
- **Research questions.** Does any scoped pool support per-thread non-`Send` state with panic capture and
  drain on failed start? Must the two surrogate layers be two threads?

<a id="cbo-081"></a>

## CBO-081 — Completion-owned scheduling loops written per site

- **Operation and consumers.** Issue lanes, stop on a known failure, and drain every issued future before
  return. Consumers are local studies, durable workers and the ready-block executors.
- **Current mechanism.** `study_inner` and the closing sets in `study_execution.rs`, the `worker.rs`
  dispatch loop, `initialize_ready` (`math/initialization/ready.rs`) and `blocks_step`
  (`math/solves/blocks.rs`) each use `FuturesUnordered`, explicit lane vectors and `select!` over cancel
  and completion. The two math loops repeat reservation, deferral, commit and stop-then-drain, and differ
  where no intent is visible: iteration-reservation failure breaks with `defer`, and `blocks_step` records
  an error when nothing runs. There is no `JoinSet` or `TaskTracker` in non-test `pse-runtime`.
- **Why it matters.** Drain obligations are restated per loop, so one loop could drop work (Proposed).
- **Patterns.** LOP-08, LOP-09, LOP-X005.
- **Evidence.** `study_inner`, `worker.rs` loop, `initialize_ready`, `blocks_step`, `dependencies.rs`.
- **Footprint.** Inspected those regions. Suspected, unread: `execute_lane`, `close_lane`,
  `study_operations` lanes and `workflow` schedulers.
- **Preservation constraints.** Known failure stops dispatch and every issued future is polled to
  completion; ready stop outranks ready candidate. Reporting stays in canonical order, with fairness
  (`available / slots`), the last-iteration rule and stage atomicity. `PrepaidLaneFuture` frames stay sized.
- **Counterevidence and uncertainty.** The loops borrow from the caller, which excludes `JoinSet`, and
  structured-concurrency crates tend to cancel by drop. Result types and rebind steps differ, and Kahn's
  algorithm is about 30 lines. The study-loop half is the report's lowest-confidence candidate.
- **Status.** Current mechanism. Weak.
- **Research questions.** Does any non-`'static` task scope drain on drop and define simultaneous
  readiness? Is a local drain group over `FuturesUnordered` enough?

<a id="cbo-080"></a>

## CBO-080 — Cancellation has three owners split by a dependency ceiling

- **Operation and consumers.** Cooperative stop for scientific callbacks, async work, columnar streams,
  compiler and structural analysis, and supervision.
- **Current mechanism.** `pse_diagnostics::Cancellation` is a flag plus `event_listener::Event`, with
  async, blocking and waker waits and a `Deref` to `Arc<AtomicBool>`. `pse_columnar::CancellationToken`
  wraps the `tokio_util` token with a child tree and `DropGuard`, and `pse_runtime::CancelSource` adds a
  deadline. `ExecutionScope` carries both a `Cancellation` and its `AtomicBool`, the compiler polls the
  flag before Salsa's `cancellation_token`, structural analysis takes `&AtomicBool`, rules take the
  columnar token, and three `Cancelled` errors are mapped. About 63 production signatures take
  `&AtomicBool`-shaped parameters, and at least ten files name both kinds. Residue in otherwise shared
  composition: the inline `select!` in `workflow/staged.rs::bounded`, and `math.rs::scope_stopped` gated on
  `solver-petsc`.
- **Why it matters.** Only one branch has trees and only the other has a blocking wait, so a stopped parent
  cannot naturally stop a numeric child without a bridge (Proposed).
- **Patterns.** LOP-09, LOP-X005, LOP-X002.
- **Evidence.** `cancellation.rs` in full, `pse-columnar/src/resource.rs`, `pse-runtime/src/cancel.rs` and
  `stop.rs`, `ExecutionScope` and the compiler `checkpoint`; bridges counted by file, not each read.
- **Footprint.** Owners inspected; the cross-crate bridge bodies not.
- **Preservation constraints.** A stop request never releases work. No runtime in scientific code, and the
  wasm study module builds. Register before check. Deadlines are never renewed (§14.4).
- **Counterevidence and uncertainty.** `dependency-ceilings.semantic` forbids `tokio` in semantic roots,
  `event_listener` fits there, and a tokio-util token has no blocking wait. Consolidation probably adds a
  tree to the runtime-free owner, which is new custom code. Bridge burden may be small.
- **Status.** Current mechanism.
- **Research questions.** Is there a runtime-free hierarchical token? What do the bridges actually do?

<a id="cbo-082"></a>

## CBO-082 — Paired resource admission wait recurs

- **Operation and consumers.** Wait for an atomic pair of job and CPU permits, sometimes with memory, with
  release notification, cancellation, abandonment and closure. Consumers are compute resumption and phase
  entry.
- **Current mechanism.** `staged::ComputeOwner::resume`, `jobs::quiescent_phase_owned_construction` and
  `jobs::reserve_active` each register `released()`, `enable()` the notified futures, try, then `select!`
  on the right release. `jobs` and `cpu` are separate semaphores tried as a complete pair so a queued large
  request cannot block smaller work. `quiescent` retries by shrinking memory, and `open_session` and
  `open_session_queued` repeat a `try_acquire_owned` error mapping. `pse_engine::resources::ReleaseNotifyingSemaphore` supplies the notification.
- **Why it matters.** Delicate concurrency is written more than once (Proposed).
- **Patterns.** LOP-08, LOP-09, LOP-X005.
- **Evidence.** The named functions were read; `strategy/admission.rs` was not assessed as a wait loop.
- **Footprint.** Three wait sites inspected; others suspected.
- **Preservation constraints.** Register before inspecting. No partial hold across a wait. The original
  clock is retained, closed admission keeps its errors, and dispatched work is never cancelled by drop.
- **Counterevidence and uncertainty.** §18.8 is specific, and an off-the-shelf semaphore will not express
  paired try-only admission without a starvation policy. The right owner may be one local primitive in
  `pse-engine`.
- **Status.** Current mechanism.
- **Research questions.** Does any multi-resource acquisition primitive keep try-only pairing and release
  notification? Is a local `pse-engine` primitive smaller than three copies?

<a id="cbo-013"></a>

## CBO-013 — One interval spelled per sink; metric declaration beside installation

- **Operation and consumers.** Native owners report phases and facts; operators, capture tooling and
  evidence documents consume them. Metrics reach OpenTelemetry through 18 instruments and generated
  passive consumers.
- **Current mechanism.** Discrete boundaries are owned: `LifecycleBoundary`, `NativeJobLifecycle::observe`
  and `NativeSessionLifecycle::record` feed hub and tracing from one call. Intervals are not. The
  `math/jobs.rs` worker names one interval three times on consecutive lines: `observation::phase`,
  `info_span!` and `invocation::stage`. `grouped.rs` pairs a span with a hub phase, and there are about 71
  non-test `phase(` sites. For metrics, `tooling/metric_declarations.rs` declares each instrument, but
  `telemetry.rs` hand-registers a matching builder and callback (the `kind` string does not choose the
  builder), and each dimension enum has hand `ALL`, `as_str` and `count(sample)` beside a snapshot field and
  static atomics. Each callback takes its own `snapshot()`.
- **Why it matters.** A new boundary or metric member takes four or five coordinated edits, and declared
  `kind` and `attributes` can drift from what the callback emits (Proposed).
- **Patterns.** LOP-09, LOP-02, LOP-03.
- **Evidence.** `jobs.rs`, `execution_lifecycle.rs`, `metric_declarations.rs`, `operational_metrics.rs`,
  `telemetry.rs` registration, `grouped.rs`; the test `metric_dimensions_are_exact_finite_producer_domains`.
- **Footprint.** Inspected: those files. Suspected: whether span and phase overlap elsewhere.
- **Preservation constraints.** The hub works without a tracing subscriber, since `pse-diagnostics` has no
  `tracing` dependency. Populations stay distinct (sessions must not move job counters). Counts begin
  before any exporter, unavailable stays distinct from zero, and dimensions carry no identity.
- **Counterevidence and uncertainty.** Spans can be filtered, so they cannot be the only record. A hub fed
  by a `Layer` would sit behind the dispatcher. Eighteen instruments change slowly, and a macro could hide
  plain code. The remedy is an interval owner, not an event bus.
- **Status.** Current mechanism.
- **Research questions.** Could `MetricDeclaration` drive installation, with `strum` and `enum-map`, which
  the workspace already has? Does the current OpenTelemetry SDK offer batch callbacks?

<a id="cbo-085"></a>

## CBO-085 — Span-loss accounting scrapes SDK internal diagnostics

- **Operation and consumers.** Telemetry users must know whether trace export lost spans.
- **Current mechanism.** `ObservedBatchProcessor` (`tooling/telemetry.rs`) wraps `BatchSpanProcessor` and,
  on each `on_end` and at shutdown, enters a private `tracing::Dispatch` so `SdkLossLayer` can catch the
  SDK events `SpanDroppingStarted` and `SpansDropped`. It reads `dropped_span_count` and `max_queue_size`
  and matches capacity back to the owned processor. Target, event and field names are SDK internals, held
  by unit tests that imitate them.
- **Why it matters.** A dependency bump can turn loss accounting into "no loss observed" without a failure,
  and the dispatcher swap adds work on the span path (Proposed).
- **Patterns.** LOP-10, LOP-11.
- **Evidence.** `sdk_loss_event`, `LossFields`, `SdkLossLayer`, `ObservedBatchProcessor`, read for this
  mechanism.
- **Footprint.** Traced here. Metric and log paths are not handled the same way (`SdkLossSignal`).
- **Preservation constraints.** Loss stays unknown, not zero, when unobserved. No private field values are
  rendered, and no global subscriber is installed.
- **Counterevidence and uncertainty.** SDK diagnostics carry no processor identity, so the code may be the
  best available at this pin. An exporter wrapper counts export failures but not queue drops, and an own
  processor reimplements the queue.
- **Status.** Current mechanism.
- **Research questions.** Do later `opentelemetry_sdk` versions publish dropped counts through a typed
  interface or self-metric? Is a small upstream seam plausible?

<a id="cbo-019"></a>

## CBO-019 — Process facts: incomplete adoption of the owner that exists

- **Operation and consumers.** Placement admission, owner and manager identity, listener ownership and
  preparation read `/proc` and cgroup facts.
- **Current mechanism.** `pse-diagnostics/src/process_facts.rs` wraps `procfs-core` `Stat` and `Status`.
  Hand parsers remain in `pse-host` (`manager::{cgroup_mount, process_group, singleton_pid}`,
  `admission::meminfo`, `activity::unified_group`), `canonical_files::ProcessIdentity::current`, `xtask`
  `producer_preparation::process`, the `pse-worker` cgroup walk, and `native_service::owns_listener`
  (socket tables and the fd table). The installed `procfs-core` has `Meminfo`, `MountInfos`,
  `ProcessCGroups` and `Status.nspid`. Two smaller bypasses share the shape: a second partial model beside
  `cargo_metadata`, and `git rev-parse` shell-outs beside gix.
- **Why it matters.** The most safety-relevant identity checks use position parsing beside an established
  parser, so format knowledge and its tests live in several places (Proposed).
- **Patterns.** LOP-03, LOP-09.
- **Evidence.** The named functions were read; `procfs-core` source was searched.
- **Footprint.** Inspected: host and the owner. Further `/proc` readers elsewhere are located by search only.
- **Preservation constraints.** Fail closed on ambiguity: two matching rows refuse. Malformed and
  non-UTF-8 rows stay failures. Observer namespace stays distinct, inherited and absent `cpu.max` stay
  distinct, and the walk continues to the root.
- **Counterevidence and uncertainty.** Hand parsers are short and may be stricter than a library, `xtask`
  lacks the dependency, and the value is uniformity, not a known bug.
- **Status.** Current mechanism.
- **Research questions.** Do the `procfs-core` mount and cgroup parsers keep ambiguity refusal? For
  `owns_listener`, does `sock_diag` or inherited sockets remove the table walk? Identity policy is
  [CBO-004](cross-cutting-mechanisms.md#cbo-004).

<a id="cbo-164"></a>

## CBO-164 — CPU team ownership spans Rayon, OpenMP, MKL and FERAL with no single owner

- **Operation and consumers.** Carrying an admitted thread width into native work, so that a lane's solver, factorization and linear algebra use the team admission granted and no more. §18.8 owns "actual transitive teams" and admitted widths. Consumers are POUNCE and FERAL solves, implicit-block factors, MKL-backed solvers and SCIP.
- **Current mechanism.** Separate carriers per supplier. `pse-backend-native/src/pounce.rs` builds a scoped Rayon pool through `ThreadPoolBuilder::build_scoped` when `threads != 1`, and an `Admission` thread-local carries the width. `pse-math/src/implicit.rs` derives `Par::rayon(threads)` from the admitted width and the enclosing pool. `pse-backend-native/src/mkl.rs` saves and restores `omp_set_num_threads`, and `scip.rs` refuses when nested OpenMP defaults are not serial. [CBO-083](#cbo-083) covers owner-affine actors and [CBO-030](native-solver-integration.md#cbo-030) process locks; neither covers pool lifetime, nested width or a shared team owner.
- **Why it matters.** A new supplier or nested route decides width independently, and a pool constructed per scoped call may be the wrong lifetime for repeated solves (*Proposed*; cost not *Measured*).
- **Patterns.** LOP-07, LOP-09, LOP-X018.
- **Evidence.** The sites named above, read at those sites only. Added from the independent challenge; no pass in this audit examined the responsibility as a whole.
- **Footprint.** Inspected as named. `joined_pool.rs` is a further owner. Other OpenMP and Rayon consumers are suspected but unread.
- **Preservation constraints.** Admitted widths are enforced; four distinct kinds of concurrency stay explicit (Plan 28); one BLAS and OpenMP per process; reproducibility of scalar sums; no oversubscription of the 16-lane premise.
- **Counterevidence and uncertainty.** Each supplier exposes a different control, so a shared owner may reduce to a width value and a convention. Pool construction may be negligible beside the solve.
- **Status.** Current mechanism.
- **Research questions.** Can one owner and pool lifetime carry admitted width across these suppliers? Does Rayon's scoped pool have a retained form that fits per-attempt admission?

## Examined and not retained

- **Hub windowed aggregation and its size model.** The 62-slot ring with partial coverage is lossy under
  contention by design (`try_lock`, a lost counter) and no aggregator library has that contract. Its
  prepay arithmetic belongs to [CBO-001](cross-cutting-mechanisms.md#cbo-001). One small lead stays local:
  `Diagnostics::publish` splices serialized operations by searching for the bytes `"operations":[]`, and a
  parallel struct of raw values would state it.

## Retired opportunities

<a id="cbo-014"></a>

- **CBO-014 — Execution context re-established by hand at each thread or task boundary.** Retired
  2026-10-30: `443ca3ecb` introduced `ExecutionContext`, `NativeThread::spawn` takes it, and non-test
  `enter_optional` occurs only inside `execution_context.rs` and the observation primitive. Remaining
  authority carriers are deliberate. Successor: [CBO-079](execution-lifecycle-and-observation.md#cbo-079).
  Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-014).

<a id="cbo-015"></a>

- **CBO-015 — Cancellation, deadline and lease renewal composed separately at each site.** Retired
  2026-10-30: `LeaseRenewal::spawn_with` is shared by durable runs and study admission, `stop_first`,
  `drain_on_stop` and `join_native` are shared, and `Heartbeat` is gone. The residue is the inline
  `workflow/staged.rs::bounded` copy and the `scope_stopped` feature gate. Successor:
  [CBO-080](execution-lifecycle-and-observation.md#cbo-080). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-015).

<a id="cbo-017"></a>

- **CBO-017 — Signal-aware blocking waits at the Python boundary written per call.** Retired 2026-10-30:
  `600d4df30` deleted `pse-py`, and no `check_signals` remains; signals use `tokio::signal` in
  `pse-host/src/foreground.rs`, `cli/control.rs` and `pse-runtime/src/bin/support/signals.rs`. Successor:
  none. Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-017).

<a id="cbo-018"></a>

- **CBO-018 — Two executor threads per native job to wait for a join.** Retired 2026-10-30: `NativeThread`
  and `NativeJoin` in `443ca3ecb` own spawn, join and accounting for jobs and sessions. The one blocked
  thread per owner is the custody contract, since join must precede release. Successor:
  [CBO-083](execution-lifecycle-and-observation.md#cbo-083). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-018).

<a id="cbo-020"></a>

- **CBO-020 — Primary-receiver supervision split between the Rust runtime and a Python script.** Retired
  2026-10-30: `600d4df30` deleted `scripts/surreal_server.py`, and `NativeCanonicalService` in
  `pse-operations` owns the service through the host manager. Successor:
  [CBO-065](developer-tooling.md#cbo-065) for the live residue. Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-020).
