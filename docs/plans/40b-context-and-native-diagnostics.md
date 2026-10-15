---
title: Context and native diagnostic producers
status: done
date: 2026-10-14
review_sources: [docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md]
scenario_sources: [docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md, docs/design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md]
doc_owner: docs/plans/40-simulation-performance-diagnostics.md
doc_topics: [design, libraries, preparation, validation]
adrs: []
---

# Context and native diagnostic producers

This supporting plan makes actual preparation, numerical and workflow work explainable
through the operation observations owned by [40a](40a-operation-context-and-live-history.md).
It supplies context-specific meaning for [40c](40c-inspection-analysis-and-profiling.md):
which producer is occupied, which work accumulated, what changed recently, and which
scientific or operational outcome followed. It covers the producer contributions to
[principal F03](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f03),
[F04](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f04)
and [F01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f01).
[Plan 40](40-simulation-performance-diagnostics.md) owns their combined disposition and
shared decisions; this document's packet table owns only B-packet progress.

## Boundaries and accepted direction

The operator accepted [principal RC01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#rc01)
on 2026-10-14: ordinary operations collect lightweight current, cumulative and recent
observations from their beginning; explicit opt-out reports unavailable coverage;
deeper capture remains optional. The coordinator owns the blueprint §23.1 decision/design
route before implementation that depends on the changed boundary. This document creates
no ADR and does not duplicate that route or enact architecture amendments.

40a A0 settles the narrow producer interface's placement below `pse-runtime`, transport
and lifetime. A1 supplies typed scoped observations; A2 supplies live/cumulative/recent
state; A3 supplies task/thread/shared-producer context; A4 supplies process/inspection
transport; A5 supplies coverage and interruption semantics. B packages consume these
slices when implemented and usable by their producers, rather than waiting for all of 40a.
No lower math, kernel, compiler, engine or native crate gains a dependency on runtime.
If A0 requires a new crate or changed binding contract, the coordinator settles the
required decision route before that edit; this plan does not select it in advance.

The context owner defines each fact's native meaning and public Rust projection. The
core observation owner defines shared scope/clock/availability/lifecycle mechanics.
Neither takes over mathematical admission, solver policy, cache validity or scientific
qualification. There is no universal iteration, residual or cost unit. 40c composes
these typed projections and does not reinterpret foreign handles or parse metric text.

No environment-equivalence/provenance-proof campaign, cold statistical rerun, timing
threshold, new cancellation mechanism, changed scientific permission or extra model
evaluation is part of this plan. Existing independent original assessment remains required.
Observation may perturb scheduling and wall-time-limited exits within the scientific
contract's allowed nondeterminism; bitwise trajectories and identical termination are
not new acceptance conditions. Capture work and its limitations must remain visible.

## Current foundation and selected reuse

The inspected baseline is `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` plus the concurrent
dirty checkout on 2026-10-14. This is a focused planning assessment, not renewed product
qualification. Source and manifests, the principal review and its four supporting
documents were read; no builds, tests, services, runtime probes or measurements ran.
Unrelated edits are preserved, including concurrent runtime, publication and plan work.

| Existing foundation and current source | Selected use and required improvement |
| --- | --- |
| `pse-compiler/src/workspace.rs`: single-writer workspace, private `with_events`, `PreparedCase` demand/order upgrades; math `execution.rs` construction spans | Reuse the Salsa event hook and owned stage boundaries. Preserve observer attachment through workspace rebuild: current `rebuild` constructs through `Self::new`, so simply supplying a callback once would lose it. Keep diagnostic identity out of tracked inputs and memo keys. |
| Runtime `math/{preparation,artifacts,numerics}.rs`, compiler-issued artifacts and `pse-columnar::Flights` | Observe actual lookup, joined-flight, construction, reuse and retirement decisions at their owners. Existing hit/miss totals cannot distinguish one producer from multiple waiting misses. Do not change consumed scientific identities to improve attribution. |
| Native `callback.rs::{CallbackState,CountedNlp}`, `solve::{Execution,Progress}`, `solve/observations.rs` | Preserve failure classification, callback work-owner suppression, typed unavailable/nonfinite facts and original versus submitted coordinates. Publish compact updates before terminal `finish`; 40a replaces prefix-only retention. |
| `execution.rs::adapter` exhaustively covers eleven registry backends; `Retained` owns native sessions | Integrate every actual adapter, including unavailable-feature paths, with explicit capability/coverage. Session allocation, seed submission, symbolic reuse and numeric reuse remain different facts. Root-isolation/selection-proof work is a separate auxiliary owner. |
| Workflow study/fitting/dynamics/shooting/horizon, surrogate actor and completion/result owners | Project the existing scientific choices, attempts and actual children. Avoid another workflow controller. Publish immutable observations out of band so inspection cannot queue behind occupied actor/native work. |
| Engine `session/{assurance,observation,trace}.rs`, canonical transport and durable progress | Reuse actual-invocation metrics, nonblocking taps, separate diagnostic reservations and effect/drain outcomes. Add explicit collector-loss facts; preserve closed scientific-result admission. |

Blueprint §7 and §14 own mathematics/preparation, §13 and §19 own context/result meaning,
§20.6 owns durable execution/publication and §23.1–§23.2 own observations/failures. The
selected standard is Core 3.4 with Process Simulator 1.5. AP-02/AP-04/AP-06 favor one
producer contract with domain-owned payloads; AP-07/H6/H12/H21/H26 favor compact owner
copies, reuse of existing counters and nonblocking publication. Full vector copying,
per-primitive durable writes and global-counter subtraction would add work without
settling the requested causes and are not selected.

The workspace pins Symbolica 3.0.1, Salsa 0.28.4, faer 0.24.4, Clarabel 0.11.1, vendored
Diffsol 0.16.2, SUNDIALS through vendored sys 0.6.2 and the POUNCE 0.12.0/FERAL 0.18.0 fork
at `d02beafd370fd3815d6c0541fab1630ff839a739`. Native declarations include Ipopt 3.14.20,
PETSc 3.24.0 and the pinned UNO revision. These are source baselines, not assertions
about an attempt's loaded backend. The loaded native-solver, mathematical-library,
Salsa and DataFusion/tracing skills support inquiry; exact selected source governs
version mismatches, including the math skill's Symbolica 3.0.0 corpus. Context7's current
Salsa event documentation was queried; the captured 0.28.4 `source/event.rs` confirms
the relevant execution, memo-validation and blocking distinctions. No stale utilization
catalog or capability map supplies implementation claims.

## Producer meaning and publication

A producer receives the 40a operation context before expensive work, publishes an
owner-known phase on entry, and updates completed counters and latest safe payload on
return/checkpoint. An open phase has observed start and freshness even while its final
duration is unavailable. The reader never calls back into a running native owner. A
stale last checkpoint identifies the observation gap; it does not establish a hang.

Preserve operation/attempt/segment, producing instance and parent/wait links, selected
request/demand, actual coordinate/normalization and native stage. Native wall, CPU and
system time overlap; callback-inclusive time overlaps nested work. Counts of calls,
maps, accepted steps, factors, backsolves, proof work, branch nodes and output batches
retain their units. 40a's compatible-snapshot rules govern recent deltas. A retry,
reinitialization, session replacement or native restart starts the appropriate new
scope instead of subtracting incompatible lifetime totals.

Every supplied field distinguishes measured/observed zero from disabled, inapplicable,
uninitialized, unsupported, omitted, stale or failed collection as applicable. Native
not-run sentinels and nonfinite observations preserve their meaning. A selected feature
may offer only entry/exit plus terminal facts; publish that coverage rather than
manufacturing live detail. A native handle retained after failure is not evidence of
a valid factor or current metric. Large coordinate/incumbent/history copies require
explicit requested detail and bounded capture, separate from compact ordinary state.

Scientific callbacks, properties, derivatives, implicit solves and original checks
run only when their existing scientific owner requests them. Observation consumes their
already-produced values. Entry/exit clocks and counter publication add observation work,
not extra scientific requests. Publish failed/refused work and its typed cause without
turning partial buffers into candidates. `callback_work_owner` and actual producer
identity preserve inclusive nesting; per-waiter elapsed time remains separately scoped.

## Native and library capability routes

These are implementation routes, not another capability/status catalog. **Current**
facts are migrated; **available, unexposed** facts are forwarded at eligible safe points;
**instrumentation needed** adds owner-observed boundaries; **unknown** uses B0's bounded
decision. Conditional detail is implemented when selected and applicable, not enabled
for every ordinary operation. The [library evidence](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/library-capabilities.md)
and [sparse/timer supplement](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/linear-backend-diagnostics.md)
retain the exact field inventory and source limits.

| Supplier and route | Required implementation or bounded decision |
| --- | --- |
| Preparation/Salsa/semantic artifacts — B1 | Current construction/order/support spans and query event hook become scoped observations. Distinguish actual query execution, memo validation, body/scalar/evaluator construction, value bind and shared wait. Observe consumed change, retention eviction, retirement or incompatibility only when its owner knows the cause; otherwise publish unknown miss reason. Affine rows/demand/examined coordinates feed 40d without prescribing its mathematical rewrite. |
| DataFusion and datafusion-tracing — B10 | Integrate selected actual planning/execution/operator/partition metrics and existing tracing lifecycle. Distinguish live metric snapshots from drop/finalization exports; retain executed-plan identity, partition scope, units, interval and partiality. Rich plans/previews remain opt-in; opening a getter must not execute a plan or assume a not-yet-finalized span is complete. B0 settles exact selected activation/finalization contracts before forwarding. |
| SurrealDB query execution statistics — B10, B0 capability decision | Decide and integrate the exact SDK `Query::with_stats` route for actual supported invocations. Retain statement/result association, supplied server execution duration/units and absent/error coverage separately from client transport/wait and upstream work. Source-check supported request routes and safe result handling; mark genuinely unsupported routes explicitly. Never replay a query or write to obtain statistics. |
| Symbolica/Numerica/property/implicit — B2 | Current static `OperationCount`, instruction/storage extent and derivative width stay structural facts. Add actual stage/provider/inner-attempt calls and durations, branch/start/accuracy context, derivative source/order, reconstruction and original assessment boundaries. No static-size estimate becomes dynamic work. |
| Ipopt — B3, B0 timer decision | Migrate intermediate main/restoration, submitted snapshot, normalized errors and final structured counters. Forward selected C++ `TimingStatistics` only with activation and task-state checks: `Total*Time` requires an ended task; start getters require running state; `timing_statistics` defaults false. Keep active intervals separate without ending/restarting native timers for inspection. Serious exits may lack data. Factor-interface forwarding is separately covered by B5. |
| KINSOL — B3 | Migrate residual/map, J*v, Jacobian and preconditioner boundaries and eligible final getters; expose latest raw-map role/failure plus actual inner context. Failed G can increment attempted iterations; no accepted-state implication. Fixed-point linear metrics are inapplicable. Mask stale/uninitialized getters after illegal input; B0 settles safe checkpoint getter consistency. |
| PETSc SNES/KSP/TS — B3/B6, B0 logging decision | Add selected owner monitors and eligible completed logging/linear facts. Logging off makes `PetscLogEventGetPerfInfo` zeroed, not observed zero work. SNES linear totals exclude a failed inner KSP's iterations; collect eligible KSP failure detail separately. Retain native reasons, TS rejection/failure and actual step meaning. Serialized admission remains intact. |
| POUNCE NLP — B3/B5 | Migrate intermediate main/restoration and completed timing, FD/coloring, SQP/restoration and failure statistics. Publish safe completed-factor summary copies at owner checkpoints, retaining attempt versus lifetime/reset semantics. Do not infer factor work from callback/Jacobian counts or wait on the solver's mutable state lock to serve inspection. |
| UNO — B3, B0 ABI decision | Existing oracle/poll callbacks and terminal evaluation/subproblem/CPU/materialization/auxiliary-HiGHS facts become scoped live boundaries and final payloads. Poll is cancellation, not an iteration record. Investigate the exact patched ABI/upstream hook for iteration/subproblem detail; if unavailable, implement explicit opaque-interval coverage with last poll/callback. Do not present the shared Ipopt callback as UNO coverage. |
| HiGHS — B4 | Migrate simplex/IPM/MIP callbacks, incumbent/bound/gap and terminal method-specific counters. Forward eligible actual run time/model/presolved dimensions and starts/settings. QP/PDLP callback cadence remains separately stated; B0 checks any selected internal timer/factor export. Basis submission/allocation retention is not factor reuse. |
| Clarabel — B4, B0 timer decision | Migrate `DefaultInfo` and `LinearSolverInfo`, model update/setup and final status. Nested timer scopes are source-visible, but public typed per-key reading is unresolved and timers move out of the solver during solve. `total_time`/print/Debug do not supply a live typed phase tree; never parse them. Keep public info and owner entry/exit coverage unless a fitting structured interface is established. |
| POUNCE convex and QP — B4/B7 | The separate convex route refuses `RequireReuse` and reports `warm_started=false`; its completion event is not NLP live progress. Forward applicable `SessionStats`, `QpStats` and `PresolveStats`, distinguishing parametric attempt, homotopy/working-set/cold acceptance, refactors and Schur updates. The `kkt/path/qp.rs` active-set consumer is migrated separately; its unavailable iteration admission remains unavailable. |
| SCIP — B4, B0 plugin decision | Migrate owned presolve/node/LP/incumbent/bound events. Add exact stage-eligible current-run/all-run nodes, LP/strong-branch work and selected plugin presolve/propagation/separation/NLP facts where APIs supply them. Resolve plugin stage and reset rules before getters; respect concurrent-copy ownership. Full solution pools/vectors remain selective detail, not mandatory hot-path copies. |
| FERAL — B5 | Forward completed `FactorStats`/`OrderingInfo` subset and applicable tiny-pivot/scaling/ordering/front facts. Establish successful/refused/stale-last-factor status at every return. Actual factors, pattern reuse, structure refresh and retained layout are distinct; synchronized summary access is not in-factor access. |
| faer and KLU — B5/B6 | Observe actual symbolic analysis, numeric factor/refactor and solve boundaries, decomposition-specific storage/fill and failures. KLU refactor attempted/succeeded/conditioning-rejected/full fallback are separate reasons. Existing rcond/conditional condest calls are observable; optional flops/rcond/condest/growth calculations are not newly run to populate a passive view. Rank `-1` and allocator-lifetime peaks retain applicability/scope. KINSOL's KLU lifecycle is separate from Diffsol's wrapper. |
| MUMPS — B5, B0 live decision | Forward eligible per-job INFO/INFOG/RINFO/RINFOG copies through the actual owning wrapper, preserving local/global, max/sum, estimate/actual, encoded large counts and decimal MB. In-factor `RINFO(3)` is a documented candidate compared with local estimated `RINFO(1)`, not elapsed time or a bounded percentage. Synchronization/visibility/ABI/private-instance access are unqualified; no getter thread is selected. Expensive error analysis and ICNTL cancellation are not activated by inspection. |
| SPRAL — B5 | Forward eligible returned `inform`, ordering/rank/front/delay/pivot/factor-entry/flop/status facts through selected wrappers. `num_factor` means entries. Solve can retain factor inform; do not charge it as new solve work. Header/document field mismatches require exact ABI resolution; no in-call monitor or general timing/memory field is promised. |
| oneMKL Pardiso — B5 | Observe actual phases and copy eligible `iparm` output from its owning wrapper with matrix type/settings/status. Conditional flop/nnz diagnostics and memory estimates remain activation-dependent. This ABI has no `dparm`; standalone Pardiso statistics cannot be transferred. No asynchronous timing/getter contract is assumed. |
| Diffsol — B6 | Copy statistics at actual step/segment returns, retaining setup causes: checkpoint/reinit, first/second convergence failure, error-test failure and successful-step heuristic. Attribute internal mesh/current order/proposed step versus returned/root/interpolated time. No invented last-accepted-step getter. Expose segment resets, failed internal work where captured and linear setup versus full-factor distinctions. |
| IDAS — B6 | Migrate IC/backtracks, forward/root/output returns and backward segments. Forward eligible Jacobian/setup/preconditioner-solve/linear-failure/linear-residual plus sensitivity/quadrature/backward getters with initialized memory. Last accepted step/order requires positive integration step count; IC artificial step is not physical progress. Preserve current/last and internal/returned time, reinitialization and per-backward-problem accumulation. |
| Root isolation/selection proofs — B7 | Instrument the actual existing auxiliary request, selected source/chart/interval scope, proof work/refusal and return. Preserve proof allowances and unavailable facts; interval/root certification is not a native accepted-iteration stream. No new proof evaluation for display. |
| Numerica statistical integration and FeOS oracle — N/A production route | No production dynamics/provider use was established for these catalog entries. Numerica's statistical integration accumulator is not Diffsol/IDAS dynamics; FeOS reference generation/log output is not runtime property authority. If a selected existing producer actually consumes one, map its owned calls in B0/B2; otherwise record this bounded non-applicability, without adding a new production integration. |

## Bounded investigations

B0 can settle questions independently as each dependent package becomes ready. Read exact
pinned headers, source and wrapper call sites first; use one small owned behavioral control
only when source cannot settle a material safety/applicability question. Record the chosen
route at the relevant packet, not an evergreen unknowns backlog. An unavailable facility
still delivers entry/exit, current last-checkpoint and honest opaque coverage.

| Question | Evidence and decision outcome | Dependents and stop boundary |
| --- | --- | --- |
| Which exact native facts can be copied safely, at what activation/reset point? | Native/bridge source and lifecycle; choose eligible owner checkpoint/return, guarded structured forwarding, optional detail, or unsupported fact with reason. Resolve Ipopt active-task preconditions, PETSc log activation/failed KSP, KINSOL stale getters, IDAS phase memory and SCIP plugin stages explicitly. | B3–B6. Stop when each selected field has source/stage/availability/reset and a usable producer route; do not inventory every internal native statistic. |
| Can documented MUMPS in-factor work cross the current wrapper safely? | Exact private-instance access and C/Fortran memory/synchronization/lifetime contract, plus bounded control only if necessary. Choose an established synchronized native mechanism or opaque in-call coverage plus returned job facts. | B5 and 40c's explanation. Stop at this facility and wrapper; no concurrent unsynchronized read, cancellation redesign, host proof or blanket claim that the library lacks progress. |
| Can Clarabel nested time and UNO iteration/subproblem detail be exposed structurally? | Exact pinned public interface, patched ABI and upstream hooks. Choose fitting structured forwarding; if an interface change is necessary, bring the concrete cost/contract decision to the coordinator; otherwise complete honest current/terminal coverage. | B3/B4. Stop once a useful selected route or explicit unsupported detail is established; no print parsing, bespoke solver controller or automatic patch commitment. |
| Which wrapper exposes sparse backend outputs actually used by the attempt? | Ipopt/PETSc/Clarabel/POUNCE/direct dynamics call sites, backend selection and returned status. Map FERAL/faer/KLU/MUMPS/SPRAL/oneMKL data to the owning phase; distinguish predictions, optional computations and retained prior outputs. | B5/B6/B7. Stop after selected supported wrappers are classified and integrated; no new factor backend or library-wide performance census. |
| Which compiler/cache transitions can establish a rebuild cause? | Current workspace rebuild/event attachment, consumed dependency/demand, artifact/retention/flights decisions and exact Salsa event semantics. Choose owner-known reasons and unknown remainder; preserve one shared producer/multiple waiters. | B1/B2. Stop once ordinary build/reuse/edit/pressure cases are explainable. Do not replay compilation, prove historical provenance or add diagnostic fields to mathematical keys. |
| Which actual query/operator facts are available before completion? | Exact DataFusion/tracing activation and live-versus-drop-finalization lifecycle, plus selected Surreal SDK `Query::with_stats` routes and statement-result association. Choose supported forwarding and honest partial/unsupported scope, preserving server versus client clocks/units. | B10. Stop after actual-invocation integrations and revealing live/final/error controls; no query replay, new query engine or blanket rich-plan capture. |
| Can surrogate/native-child/managed work publish while its owner is busy? | Actor request loop, immutable publication point and A3/A4 transport; choose compact out-of-band state tied to actual child/request identity. | B8–B11. Stop at live read without waiting for active work and explicit transport gaps; no second scheduler, always-durable primitive log or central service. |

## Context composition and consumers

Initialization, recycle and continuation owners publish attempted stages, selected parameter,
accepted predecessor, rejected step/retry and raw-map role. Inner branch/start/accuracy
permission is supplied context, not achieved uniqueness. Steady sensitivity/KKT/uncertainty
owners publish the requested quantity, actual correction/derivative/factor/RHS work and
premise/withholding reason. Original bounds, stationarity and qualification remain separate
from native residuals. These producers reuse B2/B3/B5 rather than duplicate evaluation counts.

Dynamic owners distinguish IC, forward physical integration, event settlement/reset,
output sampling, sensitivities and adjoint checkpoint/replay/backward work. Observations
retain segment and direction, requested output versus native mesh and available partial
coverage; they cannot reinterpret an output return as an accepted internal step.

Study owners publish occurrence identity, selected/ready/queued/active/settled/unattempted
state, dependency/admission wait, batch membership and actual worker. Equal content may
share preparation without merging occurrences. Fitting descends from outer parameter
trial to experiment/lane, point-cache reuse and derivative-order upgrade, then final
assessment, covariance and profile end/chain. Profile operational failures remain distinct
from scientifically withheld intervals. Caller-local counts never stand in for managed
worker work; A4 carries the worker's own observations.

Surrogate producers distinguish fidelity callback, GP/model/tuning/infill phase, proposal
and original screening/correction. Existing inclusive partial/failed `work` is retained;
history-copy cost is separate. The live view uses published immutable state rather than
the actor's blocking `Work`/`Snapshot` request while occupied. Shooting/collocation retain
outer trial, formulation/mesh, window/action/continuity mappings; horizon producers retain
plant/MHE/NMPC/background prediction, fallback reason and actually applied/held control.
Parallel/background totals are not blindly added to foreground critical-path latency.

Query/publication owners expose actual request/plan/partition identity, client wait,
returned server execution statistics where supplied, rows/batches/bytes, acknowledgment
or uncertain effect, conversion/read and drain. Never replay a query, write or native
evaluation as inspection. Rich plan/operator detail can remain opt-in. An observed query
result's statistics are not the cost of all upstream preparation, transport or publication.
Telemetry-writer failure, scientific completion, publication and join stay independent.

Consumers migrate with their producer: existing progress taps, `RunHandle`/workflow
projections, terminal metric/evidence projection, durable worker messages and 40c's Rust
inspection view consume the same domain facts. Preserve legitimate historical progress
and scientific tables while deleting any superseded metric conversion/collector/caller
that the new projection replaces. Keep no compatibility shim or second mutable authority;
delete obsolete fixtures/tests as soon as their replacement controls pass.

## Execution checkpoint — 2026-10-15

Producer integration is implemented in the working tree across compiler/math preparation,
actual callback and kernel/implicit work, all eleven native adapters, selected factor owners,
dynamics, fitting/profile lanes, study occurrences, surrogate/shooting/horizon roles,
actual engine/query requests, publication/read and native drain. Further source integration
and targeted acceptance are still in progress; no all-adapter or simulator qualification
is claimed.

B0 selects owner callback/return copies. Ipopt forwards selected ended, enabled native timer
tasks through its existing bridge; running or disabled tasks remain unavailable. PETSc
returns failed-KSP reason/iterations independently of SNES totals and marks logging-off
coverage. IDAS getters follow initialized IC/forward/sensitivity/quadrature/backward memory;
Diffsol copies actual step/segment statistics and setup causes. KLU reports attempted and
accepted refactor/full fallback from the owner rather than adding condition estimates.

Clarabel's nested timers and UNO iteration details are unsupported through their selected
public interfaces; existing owner boundaries/current/public return facts remain usable.
Private POUNCE FERAL objects and Ipopt-owned MUMPS/SPRAL/oneMKL in-factor state cannot be read
through an established synchronized public route; those intervals stay opaque, with selected
returned wrapper facts. Direct FERAL and faer/KLU owners have separate safe return routes.
No getter thread, native timer manipulation, diagnostic factorization, print parser or
extra scientific evaluation is introduced. SCIP now copies the selected stage-eligible
dual-inference/pseudo-objective propagation, Gomory separation and nonlinear/Ipopt NLP
plugin census, with run/reset and timing-activation distinctions. Concurrent copied watches
restore their captured context. High-level faer LU does not expose private L/U storage;
actual matrix and already-allocated scratch extents are usable, exact private fill is not.

DataFusion live copies are bounded owner checkpoints, separate from drop/final exports;
node values update independently, so a snapshot is not a coherent all-node instant. Off
leaves operator metrics uncollected; Contract captures compact metrics; Diagnostic adds
bounded text. Canonical transport observes the original SDK response before take/check,
retaining statement status/index and exact supplied server nanoseconds. These server samples
are not anchored into local active intervals. Request errors have explicit absent-statistics
coverage. No query replay is performed.

Default light collection now retains native convergence point history. Generic Progress
uses rolling/current retention with explicit loss; its separately required durable tap and
terminal scientific records remain their real contracts. The focused source coverage pass identified additional actual-owner scopes and quantity/role
correlation gaps; these are implemented and exercised by the selected controls linked in the
Outcome. Unsupported private detail is distinct from missing required producer work.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B0 | Accepted RC01 route; 40a A0 producer placement sufficient for each question | Settle the bounded investigations above incrementally. Each selected capability has implementation, conditional-detail, unsupported or N/A route with exact safety/applicability/reset reason. Decisions unblock the named B packets; no production change is implied by source availability. | Complete — selected targeted acceptance |
| B1 | A1/A2 usable producer interface/state; A3 shared-flight links; relevant B0 answers | Compiler/math/runtime preparation producers plus progress/inspection consumers show demand, actual construction, value bind, reuse and observed miss cause before a run exists. Preserve event attachment across rebuild. Targeted cold/repeat, consumed/unconsumed value, structural edit, pressure and two-waiter cancellation controls; delete replaced collector/caller/tests. | Complete — selected targeted acceptance |
| B2 | A1/A2; B1 preparation identities; A3 nested producer links | Native callback, math stages, kernel providers and implicit producers expose entry/open state, completed calls/time, typed failure and derivative/branch/accuracy context without new evaluations. Migrate terminal/live consumers; targeted nested failed-trial and derivative-demand controls establish work-owner suppression and original-assessment separation; retire superseded accounting paths. | Complete — selected targeted acceptance |
| B3 | B2; A3 thread links; B0 selected Ipopt/KINSOL/PETSc/POUNCE/UNO decisions | Integrate root/NLP adapters and bridge projections, including retained-session/reset and unlinked coverage. Targeted main/restoration/refusal, attempted failed G, PETSc logging-off/failed KSP, native serious exit and UNO poll-only coverage controls; preserve original candidate assessment and remove replaced adapter projections. | Complete — selected targeted acceptance |
| B4 | B2; A1/A2; B0 selected timer/plugin/convex decisions | Integrate HiGHS, Clarabel, SCIP and separate POUNCE convex producers/consumers. Targeted coefficient update/start, cone normalization, MIP incumbent versus bound, restart/stage validity and parametric attempted versus accepted-reuse controls. Retire replaced conversion/collector code and fixtures. | Complete — selected targeted acceptance |
| B5 | A1/A2; B0 wrapper and synchronization decisions; B2 work scopes | Factor/reuse producers for actual FERAL/faer/KLU/MUMPS/SPRAL/oneMKL owners plus native/Rust consumers preserve phase, status, estimates, activation and allocation scope. Targeted factor/refactor/failed-factor/fallback and unavailable/conditional-stat controls; no optional diagnostic recomputation. Delete replaced summaries when callers migrate. | Complete — selected targeted acceptance |
| B6 | B2/B5; A1/A2; B0 dynamic getter decisions | Diffsol/IDAS/PETSc dynamics and runtime trajectory producers project IC, step/setup causes, events, resets, forward/sensitivity/adjoint work and actual return coverage. Targeted IC retry, rejected step, off-grid event/reset and short forward/backward controls; remove replaced segment/progress collectors without losing terminal evidence. | Complete — selected targeted acceptance |
| B7 | B1–B3/B5 as consumed; A3 nested links | Initialization/recycle/continuation, auxiliary root proof and derived/KKT/uncertainty producers plus consumers explain attempted/accepted state, actual RHS/factor work and withholding. Targeted failed stage retry, inner-map failure and two-response/withheld-premise controls; retire duplicate context accounting. | Complete — selected targeted acceptance |
| B8 | B1/B2 and relevant numerical producer packets; A3; A4 for managed route | Study and fitting/profile producers publish occurrences/readiness, actual lanes/workers, experiment/order/cache and chain failure. Local slices proceed before A4; managed delivery waits for implemented transport. Targeted skewed independent/failed causal frontier, two-experiment upgrade and failed profile-chain controls; migrate consumers and delete replaced totals/inferences. | Complete — selected targeted acceptance |
| B9 | B2/B6/B7 as consumed; A3; B0 busy-owner decision | Surrogate, shooting/collocation and horizon producers plus consumers expose fidelity/proposal/original correction, windows/actions and role/prediction/fallback/hold. Targeted occupied-actor read, failed fidelity/original refusal, two-window mapping and short horizon fallback controls. Remove blocking diagnostic-only actor requests and superseded aggregate paths once migrated. | Complete — selected targeted acceptance |
| B10 | A1/A2; A3 actual request context; A4 remote scopes; A5 interruption; B8/B9 as consumed | Engine/canonical request, publication/result-read and actual drain producers join their real facts to the operation. Targeted actual-invocation metrics without replay, collector truncation, completed science with writer/publication failure, connected-read failure and cancellation through native join. Preserve scientific sealing; retire superseded observer inference paths. | Complete — selected targeted acceptance |
| B11 | Delivered B1–B10; A4/A5; 40c typed consumer integration available | Complete public Rust context projections and migrated local/child/managed consumers; audit every actual adapter and catalog route for honest coverage. Targeted ordinary opt-out, recent saturation, reset/retry and nested/shared producer controls with disabled/enabled inspection and scientific request counts. Remove remaining replaced mechanisms/callers/tests; hand scope to 40e without declaring whole-product qualification. | Complete — selected targeted acceptance |

B1/B2 establish widely consumed meaning first; numerical families can then proceed
independently where contracts and file ownership permit. B5 shares adapter/bridge surfaces
with B3/B4/B6, and B7–B9 share workflow execution/completion surfaces: the coordinator
assigns edits explicitly even when logical dependencies permit parallel work. B10 owns
producer facts at transport/effect boundaries; 40a owns transport mechanics and 40c owns
inspection/explanation. Missing A4 blocks remote completion, not useful local slices.

## Scenario coverage and handoff

References below use each source's original anchor; principal S01–S12 and supporting
context S01–S26 are different namespaces. This is an ownership/acceptance map, not a new
scenario definition or finding-disposition ledger. Controls are selected for distinct
contracts and can establish several rows together.

| Original principal scenarios | Original supporting controls | Producer packets and boundary |
| --- | --- | --- |
| [Principal S01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s01) | [Context S01](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s01), [S02](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s02), [S03](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s03), [S04](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s04), [S23](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s23), [S26](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s26) | B1 actual preparation/reuse; B5 native reuse; S26 supplies facts to 40d, which owns mathematical locality changes. |
| [Principal S02](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s02), [S03](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s03) | [Context S05](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s05), [S06](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s06) | B2–B5 method, nested property and transformed-coordinate contracts; B11 consumer integration. |
| [Principal S04](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s04), [S05](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s05) | [Context S07](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s07), [S08](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s08), [S09](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s09) | B7 attempts, committed predecessors, inner-map failure and quantity-specific validity; B2/B5 actual work. |
| [Principal S06](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s06) | [Context S10](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s10), [S11](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s11), [S12](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s12) | B6 IC/events/forward/adjoint and reset/return meaning. |
| [Principal S07](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s07), [S08](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s08) | [Context S13](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s13), [S14](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s14), [S15](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s15), [S16](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s16), [S17](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s17) | B8 occurrence/lane/experiment/profile facts; A4 carries actual managed-owner observations. |
| [Principal S09](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s09), [S10](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s10) | [Context S18](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s18), [S19](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s19), [S20](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s20) | B9 surrogate/window/formulation/horizon roles; 40c explains them without changing scientific permission. |
| [Principal S11](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s11), [S12](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s12) | [Context S21](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s21), [S22](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s22), [S24](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s24), [S25](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s25) | B10/B11 producer effect/drain and projections; A2/A4/A5 history/transport/coverage; 40c owns Rust/Python inspection and 40e assembled interruption acceptance. |

## Verification and completion boundary

**Proposed:** each implementation packet compiles touched packages with
`scripts/pse-env -- just check-package <pkg>` and runs its selected meaningful unit controls
with `scripts/pse-env -- just unit-package <pkg> <filter>`, using linked native placement
where required. Actual Arrow-consuming correctness closures retain explicit force-validation.
Names/fixtures are chosen against the implementing source; this plan claims no executed
test by naming a future control. Reports name command, mode, scope, zero failure baseline
and actual result. Each packet migrates consumers and deletes replaced code/callers/tests
in the same change after its targeted controls pass.

**Proposed:** no-extra-evaluation controls count actual scientific requests, including
failed/refused and nested work, under disabled/enabled inspection. Check preserved scientific
assessment/permission and actual join rather than require identical scheduling, trajectories
or wall-limit exits. Record capture/copy work and explicit unavailable coverage; make no
quantified overhead or speedup claim without a separately selected measurement.

**Proposed:** [40e](40e-qualification-and-closeout.md) alone schedules assembled integration,
component/solver/Python journeys, hygiene and manual checks after functional scope is complete.
Its acceptance includes context composition across placements and truthful partial coverage;
B11 handoff does not close principal findings or qualify the enclosing simulator. Profiler
feasibility/export belongs to 40c, and affine semantic/locality correction to 40d.

Current execution is owned by the checkpoint above and the package table. RC01/G0 and the
shared producer interface are installed; the remaining immediate dependency is combined
compile/targeted functional acceptance. Full 40e qualification remains separately scoped.

## Outcome — selected implementation, 2026-10-15

**Implemented:** actual preparation/query/reuse, callbacks, kernels/implicit work, eleven
native adapters, selected sparse-factor owners, forward/backward/replay dynamics,
initialization/continuation, derived quantities, studies/fitting/profile, surrogates,
shooting/horizons, engine/server query statistics, publication/read and native drain now
supply scoped observations. Selected public getters follow the actual owner's activation,
return and reset contract. No inspection evaluation, diagnostic factorization or query
replay was added. Existing scientific permission and known-subtotal/unknown accounting
remain authoritative.

A mistake made and corrected was treating native factors and aggregate fitting context as
sufficient attribution: the final coverage review found bypassed setup/factor owners,
continuation decisions, quantity-specific withholding and concurrent experiment identities.
These now have their own actual scopes and facts; fitting lane waves and experiments cannot
overwrite one shared identity. Horizon prediction/background work is distinct from its
foreground control. Success and operational failure clear stale withholding causes.

The horizon control exposed a supporting production defect: a valid background factor was
lost when backend-free plant/proposal work replaced the entered native scope. Compatible
non-solve requests now borrow that scope and its existing charged stacks; explicit backend
or thread changes still replace it. Per-request context/trace propagation into the retained
session thread and explicit scope/return/stop/admission facts make this handoff inspectable.
An older scope test expected accumulation of replaced profile pools; its expectation now
checks only the actual new team and selected backend. The original horizon prediction and
scientific-permission assertions remain intact.

The actual managed journey exposed two retention mistakes: idle polling created operation
nodes, and invocation-specific query labels filled the metric/source-reference catalogues.
Only real candidates now create child scopes, with identities seeded when known. Query
statistics use fixed counters, one origin and an atomic typed latest sample with bounded
first/latest history and explicit coalescing. Exact server duration/status/index association
is preserved without claiming a complete query log or a local active server interval.
The typed sample uses external enum tagging: internal tagging buffers through Serde content
and cannot deserialize `u128`. The maximum-value round-trip control preserves exact durations
within the default text bound.

Deliberate deviations: exact private Ipopt MUMPS/SPRAL/MKL instances, Clarabel nested timers,
UNO structured iterations, KINSOL-KLU internal subphase counts and high-level faer private
fill are unavailable through the selected supported interfaces. Existing public summaries,
source-known matrix/scratch extents and actual call boundaries are forwarded instead.
Cold POUNCE convex does not fabricate ActiveSetSession or warm/factor reuse statistics.
SCIP plugin facts and enabled Ipopt timers are conditional on their actual owner contracts.
These are settled capability decisions rather than unimplemented selected producer work.

Focused **Tested** results and their zero-failure baseline are recorded at the
[coordinator's selected verification](40-simulation-performance-diagnostics.md#selected-implementation-verification).
The remaining analysis/profile/Python/CLI, affine locality and assembled qualification
obligations stay with 40c, 40d and 40e; this companion alone closes no composite finding.
