---
title: Instrumentation, causal scope and live inspection
status: review
date: 2026-10-14
baseline: a08f5fa091db1840fc95b46c4d4f849ce4c5de62 plus the inspected working tree
standard: Core 3.4; Process Simulator 1.5; pse-arrow binding
doc_role: evidence
doc_owner: docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
doc_retention: while-dependent
---

# Instrumentation, causal scope and live inspection

This is supporting source analysis for the [principal review](../../reviews/design_review_simulation-performance-diagnostics_2026-10-14.md). It supplies bounded findings and intended contracts, not an overall disposition or a second status owner. The coordinator owns publication, cross-inquiry reconciliation and global library comparison.

The functional target is to inspect a simulator operation several minutes after it began: identify current work and elapsed/cumulative cost, see what changed during the preceding minute, and distinguish observed convergence, repeated work, waiting and teardown. It applies before numerical execution as well as during steady solves, nested solves, dynamics, fitting, studies, managed workers and publication. An intermediate point is an observation, not a qualified result. Missing observations cannot establish a hang or lack of progress.

The user permanently eliminated environment/provenance proof campaigns. Diagnostic correlation does not require rebuilding, comparing producer bytes or proving equivalent historical execution conditions. Semantic fences, scientific checks and managed ownership/drain remain relevant because diagnostics must explain their actual effects without changing them.

## Evidence and boundaries

**Implemented, source-inspected:** the mechanisms below are present in the assigned baseline and working tree. No tests, builds, services, sampling sessions or measurements were run for this inquiry. The interrupted benchmark's Criterion log estimated approximately 622 seconds per cold iteration; this motivates the inquiry but neither independently measures a completed iteration nor attributes that interval to a particular producer.

The standards most directly engaged are AP-02/AP-04/AP-05/AP-06/AP-07, DP-04/DP-05/DP-10/DP-19/DP-20/DP-21 and PS-07/PS-08/PS-10/PS-12. H12/H14, H21/H22 and H26 guide shared-work, lifecycle and diagnostic-overhead assessment. Scientific formulation, solver eligibility and numerical accuracy are not re-reviewed here.

The negative claims are bounded to the inspected public APIs, launch paths and collectors: runtime workflow/run/progress/durable; math jobs/artifacts/preparation/native_child; engine assurance/observation/trace; backend callback/solve/observations and selected native adapters; benchmark Phases; supervisor worker/primary/observer launch paths; resource reporting and the activity observer. This is not an exhaustive inventory of every backend getter or third-party profiler capability.

## Existing responsibilities worth preserving

| Responsibility | Source owner and observed behavior |
|---|---|
| Engine operation and native-plan observation | `crates/pse-engine/src/session/assurance.rs::OperationObservation` distinguishes completed, failed, cancelled and abandoned. `TaskTracer` explicitly propagates span plus subscriber through DataFusion async and blocking tasks and reports `TaskPropagation::ForeignHook` when completeness is unproved. |
| Bounded engine diagnostics | `session/observation.rs::diagnostic_pool`, `CompletionGuard`, `ExecutionSnapshot` keep optional capture separately charged and retain terminal operator metrics without retaining a live execution plan. Drop records abandonment. |
| Native work and failure evidence | `crates/pse-backend-native/src/callback.rs::CallbackState` preserves counts, durations, rejected trials and typed causes. `solve.rs::Evidence` and `WorkEvidence` keep unavailable counters absent. String-keyed metrics do not decide qualification. |
| Local live solver progress | `solve.rs::Progress` and `workflow/run.rs::RunHandle::progress` provide bounded copied events and a dropped count. Cancellation does not release the actual native owner before join. |
| Durable ordinary progress | `workflow/durable.rs::Tap` and `Streamer` separate observed, queued, dropped, acknowledged and uncertain events. The actual tap never waits for canonical I/O or a full channel. Writer failure is recorded separately from scientific completion. |
| Terminal progress reads | `workflow/progress.rs::ProgressStream` reads immutable progress batches under existing closed-result protections. Its exact terminal semantics are sound for historical result inspection. |
| Resource observations | `pse-runtime/src/env.rs::SharedRuntime::report` separates pool reservations from whole-process RSS and reports cache aggregates. `scripts/activity.py` observes actual managed units and checkout processes, explicitly without claiming lock-owner knowledge. |
| Test-local structural capture | `pse-testkit/src/capture.rs::Capture` retains span parent/closed status and explicit sampled/truncated/interrupted coverage. This is a useful existing contract example, not a production live profiler. |

The architecture owner, blueprint §23.1, already distinguishes observability, derivation and scientific results. These distinctions remain preservation constraints, not reasons to limit the new diagnostic target to existing terminal APIs.

## IL-01 — Completion-owned observation cannot explain an unfinished operation

**Finding, source-established; high consequence for the requested target.** Important work precedes the public live handle, while benchmark observations become externally visible only after the whole sampling campaign completes.

Decisive path:

* `workflow/modeling/declared.rs::ModelingPackage::declared_execution_with_demand` awaits `self.prepare(...)` to obtain the authored model and fixture inputs.
* `workflow/modeling/engines.rs::prepare_analysis_attempt` awaits `prepare_solve_seed`; `modeling/cases.rs::finish_case` awaits `MathService::prepare_solve` before producing a runnable preparation.
* `benches/benches/native_process.rs::process` awaits declared execution and `prepare_analysis` before calling `prepared.start()`. Its `case_preparation_and_start` timer is recorded only after those calls return.
* The benchmark installs `tracing_subscriber::registry().with(compiler_phases.clone()).init()`, with no formatting/log output layer. `native_process/phases.rs::Phases::on_close` stores elapsed duration in an in-memory map. `process` writes the JSON only after `group.finish()` and retained-runtime cleanup.

This explains the absence of phase output during the long first cold iteration. An open construction span has not contributed a closed-span total; even completed phases stay private until sampling ends. Setting `RUST_LOG` alone does not install a consumer that prints them.

The gap is broader than a benchmark presentation issue: `RunHandle` cannot expose preparation that must finish before the handle is constructed. Production preparation contains expensive work without current phase boundaries, including `workspace.rs::ValueProducts::bind` → `CasePlan::presolve_domain_facts`, factorable construction and `MathService::discover_class` → `PreparedCase::prepare_affine_rows`.

**Counterexample:** a request spends minutes in affine preparation, or waits for memory before a native thread starts. An inspector sees no handle-native events, and the benchmark exposes no completed phase data. Identical silence can represent useful symbolic work, a resource wait, teardown, disabled capture or failure.

**Intended contract:** an operation has inspectable diagnostic identity and lifecycle before the first expensive effect; current phase, phase age, last observed transition and evidence coverage remain inspectable while unfinished. Preparation, admitted/queued/running work and actual drain are distinguishable. The execution owners emit actual boundaries; benchmark/reporting consumers display them without independently inferring phase from logs.

**Minimal comparison:** existing tracing spans and a subscriber that exposes current lifecycle are a plausible composition basis; adding only an end-of-run formatter cannot meet this contract. Adding an event for every arithmetic primitive is unnecessary. Whether the sink is in-process, file-backed or exported remains a design choice.

**Uncertainty:** the 622-second interval's internal dominant phase is not established by this source analysis. The missing live boundaries prevent that causal distinction; this finding does not substitute a guessed hotspot for evidence.

## IL-02 — Current progress retention cannot answer a late rolling-window question

**Finding, source-established; high consequence.** Local progress retains an initial prefix, while the public persisted progress reader requires a terminal attempt. Neither supplies a remote last-minute view of an active long run.

`pse-backend-native/src/solve.rs::Progress::push` states: "Allocate only up to the admitted bound; later events increment a dropped count." Its vector appends until `limit`, then drops subsequent events. `snapshot` clones that prefix and count. This is bounded and honest about omission, but is not rolling retention or a latest-value snapshot.

`pse-runtime/src/workflow/progress.rs::ProgressStream` states: "A terminal attempt's recorded event stream ... this reader never follows provisional stored rows." `Runtime::progress` calls `CanonicalStore::read_results`. In `pse-operations/src/canonical_results.rs::terminal_read_owner`, the attempt must be terminal, closed and have ingestion closed. `read_results` then requires its `closed_manifest` and admits a manifest ticket. Durable `Streamer` can append progress during execution, but these public reads do not expose that provisional history.

**Counterexample:** inspect a run after its in-memory event cap filled in the first minute. It is still producing iterations, but `RunHandle::progress` returns the same early events with a larger dropped count. A different process cannot use `Runtime::progress` until terminal sealing succeeds. A last-minute trend cannot be recovered from either API.

**Intended contract:** distinguish cumulative snapshots, recent history and historical result evidence. A live diagnostic view identifies its observation interval, missing ranges and latest observed state. A rolling window does not imply complete history; a terminal receipt does not imply live availability. Live diagnostic access must not weaken closed scientific result admission or expose provisional outputs as qualified results.

**Minimal comparison:** existing `ProgressTap`, bounded event DTOs and coverage counters provide useful inputs. Simply increasing the first-N cap remains unsuitable for arbitrary-duration runs. A bounded recent view plus cumulative observations is a candidate; the exact storage/access mechanism is not prescribed here.

## IL-03 — Causal context stops at runtime task, thread and process boundaries

**Finding, source-established in the inspected launch paths; high consequence.** The engine has explicit propagation, but lifecycle ownership propagation elsewhere is not trace propagation. Semantic attempt keys and managed process identities alone do not reconstruct a causal execution graph.

* `pse-engine/src/session/assurance.rs::TaskTracer::trace_future` captures `Span::current` and current `Dispatch`, then instruments the future. `trace_block` restores both around blocking work. A foreign hook explicitly limits the claim.
* `pse-runtime/src/workflow/mod.rs::inherit_runtime_activity` propagates runtime and canonical lifecycle activity, not spans/subscribers.
* `pse-runtime/src/math/jobs.rs::submit_source_timed` spawns an async supervisor. `MathService::admitted` creates a named OS thread, enters Tokio and installs compute/construction ownership, without restoring caller trace context in the inspected closure.
* `workflow/fitting/experiments.rs` creates owner-affine experiment threads and enters Tokio; the inspected thread setup likewise does not propagate caller tracing context.
* `pse-columnar/src/flight.rs::Flights::load_owned` spawns a completion-owned loader for a shared key. Its waiter ownership is explicit, but no diagnostic producer/waiter graph is exported.
* `math/native_child.rs::{Hello,Request,Response}` carries attempt/execution/request identity and remaining allowance, but no diagnostic parent links or progress protocol. The child execution uses `Progress::new(0)` and returns a final point/refusal. Startup, transfer, child work and reap are one parent deadline, not separate observable diagnostic phases.
* `scripts/surreal_server.py::{worker_environment,primary_environment,worker_scope_command,observer_scope_command}` carries selected state, worker slot, capabilities and managed placement. In those inspected paths, no diagnostic trace correlation is established between observer, resident primary and individual canonical requests.
* `pse-operations/src/canonical_transport.rs` propagates request deadline and response/effect ownership. Those scopes do not themselves expose request latency and server work under a distributed diagnostic context.

**Counterexample:** a fitting objective awaits several experiment lanes, one of which launches native work or blocks on a canonical request. A root-only trace can show a long caller interval but cannot identify which lane, request or child caused it. A global subscriber may capture named spans on those threads while losing their parent relationship; capture is not correlation.

**Intended contract:** every actual task/worker/child/request boundary has discoverable diagnostic relationships to its operation, including multiple callers waiting for one shared producer. The diagnostic graph distinguishes ownership, execution nesting, waiting and reuse; it does not add trace identity to mathematical cache keys or replace existing attempt fences. Unknown propagation remains explicit.

**Minimal comparison:** extend the existing tracing composition and compact correlation links before considering an independent workflow recorder or mandatory distributed tracing backend. Merely installing a global subscriber captures events but does not establish the missing relationships. Framework choice and provider library capabilities are the coordinator's separate inquiry.

## IL-04 — Available clocks and counters lack a joined cumulative/window interpretation

**Finding, source-established contract gap; medium-to-high consequence.** Existing observations are useful but cannot safely be summed or compared as one causal cost account without additional scope semantics.

`CallbackState::evaluate` times completed callback work, including rejected trials and nested operations, and stores per-demand maps. `CallbackState::finish` moves them to `SolveReport.metrics` as `callback.<demand>.calls/seconds`. There is no public live snapshot of those worker-local maps in the inspected path. `NativeCounters::Ipopt` explicitly says CPU time includes callbacks and overlaps wall/system time. `native_process/phases.rs::Phases` measures creation-to-close lifetimes, aggregates by static name and has no parent overlap/exclusive-time model. The resulting benchmark metadata correctly calls these spans "inclusive synchronous compiler spans."

`MathService::artifact` increments cache hit/miss counters around retained lookup and shared flights. A miss is not necessarily a unique construction: a caller may join an already-running producer. Cache reports expose active loads and aggregate hits/misses, not each waiter's latency or causal links. `math.rs::counted` separately tracks nested preparation counts, which are not durations or critical-path attribution.

**Counterexample:** two callers wait 30 seconds for the same preparation; an outer callback spends 20 seconds in an inner solve. Adding both waiter latencies and producer time triple-counts construction, while adding outer and inner callback time double-counts elapsed work. A delta across a retry boundary can subtract counters belonging to different attempts.

**Intended contract:** observations declare unit, clock, scope, owner and accumulation/reset semantics. Cumulative and window deltas remain scoped to the same producing operation/attempt. Exclusive time subtracts the union of relevant child intervals, with parallel overlap distinguished from total work. Shared producer execution is recorded once, while each waiter's latency remains observable. A diagnostic answer may report unavailable attribution rather than inventing additive totals.

**Minimal comparison:** retain native counters and boundary timers as supplied; compose a causal view from them rather than normalizing all algorithms into a fictitious universal work unit. Precise sampled CPU attribution is a separate observation from wall time and request latency.

## IL-05 — Native progress meaning is richer than a universal convergence indicator

**Finding, source-established semantic gap for the new live target; high consequence if presentation overclaims.** Existing method-specific facts should feed live inspection without turning callback points or normalized errors into scientific results.

`pse-backend-native/src/lib.rs` routes `foreign_nlp` to `uno/callback.rs`, shared by the active persistent Ipopt path. Its intermediate callback publishes iteration, restoration, normalized objective, native primal/dual error, barrier, step norm, regularization, step lengths, line-search trials and normalized stationarity when available. It captures a bounded current coordinate prefix before lossy telemetry. That snapshot remains worker-local and is attached to final native observations by the adapter; the public live event payload does not expose the complete point.

`solve/observations.rs::NativeSnapshot` explicitly says "not a resumable state" and records submitted-coordinate authority, partiality and phase. KINSOL's report currently sets `current: None`; its fixed-point iteration count includes attempted maps and is not an accepted-state count. `RawMapHistory` separately preserves bounded samples and latest/last-successful map observations, including trial failures. `dynamics/integrator.rs::step_observation` distinguishes current method configuration from the successful return; `NativeStepObservation::Diffsol` says current step is not necessarily the step just accepted.

**Counterexample:** objective falls while primal violation rises; restoration changes error meaning; repeated residual callbacks are trial evaluations within one outer iteration; a DAE getter reports the next proposed step rather than an accepted step. A generic "improving" flag or complete candidate display would misstate these observations.

**Intended contract:** live answers retain method phase, normalization/coordinate authority, trial versus accepted iteration, event/sample coverage and actual state availability. "Residual decreased over the observed window" is an observation, not convergence or physical feasibility. Repetition requires operation/attempt identity plus observed sequence, not equal counter names. Original-space assessment remains independent and is never triggered merely to populate a dashboard.

**Minimal comparison:** project existing typed native observations and backend-owned event fields. Do not force unavailable data into a lowest-common-denominator solver API or reconstruct native decisions from string-keyed metrics.

## IL-06 — Resource snapshots are not operation attribution

**Finding, source-established scope gap; medium consequence.** Pool accounting and process measurements are correctly distinguished today, but cannot identify last-minute resource cost of one concurrent operation.

`SharedRuntime::report` exposes aggregate reserved bytes, shared pool peak since the last observer reset, top current consumers, cache reports and process peak RSS. `reset_observation_peak` resets a shared pool observer, not an operation-private timeline. `peak.rs::process_peak_rss` reads whole-process lifetime `VmHWM`. The host activity observer reports actual managed units, memory and elapsed lifetime; it does not correlate memory/CPU/I/O changes to simulator phases or semantic operations.

**Counterexample:** two runs share one runtime; one resets the observation peak while another allocates. The process already had a larger RSS peak before either began. Neither snapshot can identify which current operation caused the reported high-water value. A resident primary's memory also cannot be attributed to its observer's individual case.

**Intended contract:** expose scope explicitly: operation-owned claims, shared retained capacity, process/cgroup observations and external contention remain different observations. Window deltas require repeated samples from a declared interval; high-water values are not interval allocation totals. Missing ownership attribution remains unknown. Diagnostic resource collection must not create new execution caps or alter existing admission policy.

## IL-07 — Diagnostic lifecycle and cardinality need a complete interruption story

**Finding, source-established boundary gaps with substantial positive controls.** Local completion behavior is stronger than end-to-end interrupted diagnostic availability.

Preserve these controls:

* `RunHandle` retains actual native ownership through join; last-handle drop requests cancellation, not immediate resource release.
* `Flights::load_owned` retains cancelled work until exit; a final waiter leaving requests cancellation without admitting a duplicate still-running producer.
* `durable.rs::Tap::observe` uses try-lock/try-reserve, bounds an event before cloning, and records losses. `Streamer::finish` closes the queue, joins its writer and distinguishes failed/unacknowledged publication from scientific outcome. `Drop` closes the queue, allowing bounded pending appends to finish under their owner.
* Engine `OperationObservation` and `CompletionGuard` record abandoned work. `ExecutionSnapshot.truncated` explicitly marks a curtailed snapshot.

Remaining gaps are concrete. Benchmark diagnostic state is written only at successful campaign end; a panic, signal or cancellation before that point loses the in-memory phase report. Durable recorded progress may remain inaccessible through the terminal reader when sealing fails. A long opaque callback has no completed timing delta until it returns. Engine `ExecutionTrace::record` silently declines additional observations after 1024 entries or collection-reservation refusal, so an empty/short collection alone does not establish complete observation coverage. `pse-testkit::Capture` already demonstrates explicit open-span/truncation coverage, but is not wired as a production lifecycle history.

**Counterexample:** cancel a long opaque native call and inspect while its destructors drain; kill a benchmark before sampling ends; fail progress publication while science succeeds; exceed the engine collection cap during a large study. The inspector needs to distinguish running, stopping, drained, capture curtailed and publication uncertain rather than infer them from silence or one result flag.

**Intended contract:** diagnostic prefixes and current/open state have their own lifetime and explicit coverage, independent of scientific sealing. Normal cancellation, panic/drop and hard process death have declared survivability limits. Optional writer failure is observable and does not change scientific qualification. High-rate observations are selectively detailed and bounded; total/omitted counters and latest state preserve usefulness when detailed history is curtailed.

**Minimal comparison:** existing nonblocking taps, separate diagnostic pool, terminal guards and typed coverage are strong building blocks. An always-durable per-primitive event log would add avoidable crossings and can perturb the computation it explains. A purely terminal collector is too weak for the target. Choice of local checkpointing, streaming export or attachment-based sampling should follow the actual failure and inspection needs.

## Live attachment is complementary, not retroactive history

An on-demand stack or sampling profile can answer where a process is working or waiting from attachment onward. It cannot reconstruct the preceding minute's objective/residual history, completed symbolic phases or evicted event details. One stack sample also cannot distinguish a persistent hotspot from a brief visit.

The contract should distinguish three answers: current observed state, history actually collected before inspection, and samples collected after attachment. Each names coverage and sampling interval. Collection from start is necessary for a retrospective window; it need not mean storing every callback or enabling expensive profiling continuously. Native symbols, permissions, library support and attachment overhead remain capability questions for the coordinator. No attachment tool was exercised here.

## Integration and decisive scenarios

The findings interact: an operation identity must exist before preparation (IL-01), cross actual execution boundaries and shared producers (IL-03), and bind the time/coverage semantics of live history (IL-02/IL-04/IL-07). Native interpretation (IL-05) and resource scope (IL-06) constrain what the resulting view may claim. One generic timestamped log does not by itself settle those contracts.

Revealing cases are a slow cold preparation before a handle exists; two callers sharing construction while one cancels; parallel fitting lanes with one slow child; an opaque callback followed by lengthy drain; a long solve inspected after local event retention fills; a managed worker awaiting its service; and successful science with failed diagnostic persistence. These challenge the intended operation rather than require a broad benchmark campaign.

No overall architectural or behavioral verdict is issued by this supporting analysis. All findings are source-grounded relative to the new functional target; suggested contract directions are **Proposed**, and no performance benefit or profiler compatibility is claimed as Tested or Measured. Global gate judgments, rule impacts, library selection and final publication remain with the principal review.
