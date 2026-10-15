---
title: Simulation diagnostic contexts and causal analysis
status: proposed
role: supporting-design-review-analysis
baseline: a08f5fa09 plus concurrent uncommitted work
doc_role: evidence
doc_owner: docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
doc_retention: while-dependent
---

# Simulation diagnostic contexts and causal analysis

This document supports the [principal review](../../reviews/design_review_simulation-performance-diagnostics_2026-10-14.md) of causally useful simulation performance diagnostics. It supplies context coverage, proposed analysis recipes and revealing scenarios, with no overall review verdict, finding disposition or implementation authorization. The coordinator reconciles this analysis with the other evidence.

The functional target is to let a developer or scientist understand what an actual operation is doing, what useful progress it has made, where work is repeated or waiting, and what explains a failure or an apparent hang. Correctness and avoidance of unnecessary work are the design-phase aims. Comparative or exact benchmarking, statistical repetition campaigns, hard timing criteria and environment/artifact proof are outside this inquiry. Scientific source identity, model correspondence, physical units, original-space qualification and honest diagnostic coverage remain necessary to interpret observations.

**Evidence boundary.** Existing behavior below is **Implemented, source inspected**, not a claim of a newly executed test or measurement. Recommendations and verification scenarios are **Proposed**. No probes, builds, product edits or new simulation campaigns were performed for this document. The baseline was checked as `a08f5fa09`; the shared checkout contains concurrent edits. Architectural context is blueprint §14.3–14.4, §17, §19.3, §20.6 and §23.1–23.2. The selected standard is Core 3.4 with Process Simulator 1.5 and the pse-arrow binding. This supporting analysis applies the complete-operation and semantic-authority questions; the principal review owns foundation and gate judgments.

## 1. Distinctions that make an explanation useful

A simulation context is an intersection of axes, not a synonym for a solver or a benchmark case:

- Revision and reuse: cold admission, unchanged retained work, value rebind, structural edit, changed physical/provider closure and retention loss.
- Mathematical consumer: steady roots or feasibility, optimization, coefficient/conic/discrete/global work, dynamic integration, sensitivities and postsolve analysis.
- Composition: initialization, recycles, continuation, independent or causal study points, fit experiments, profile chains, shooting windows and closed-loop roles.
- Placement and lifecycle: caller versus numerical worker, local versus managed, ephemeral versus durable, queueing, publication, connected reads and teardown.
- Entry and outcome: Rust or Python, synchronous or asynchronous consumption, compact or full detail, success, refusal, partial completion, failure, cancellation, abandonment and uncertain operational effects.

The matrix is not a demand to execute every combination. It prevents a diagnostic design that explains a successful local steady solve while silently losing the parent, remote, live or failed portions of another supported workflow.

**Proposed explanation contract.** An explanation should distinguish four questions: what work was requested; what work actually occurred; which products or states were reused; and what scientific result or permission was established. A preparation request, elapsed interval or cache key cannot answer all four. A long phase is not by itself proof of wasted work; repeated source traversal unrelated to a row's dependencies can be unnecessary even before its elapsed benefit is measured.

### 1.1 Identity, coordinates and physical interpretation

**Implemented.** `workflow/run.rs::RunHandle` exposes run identity, optional attempt identity and canonical lookup keys. Native `solve/record.rs::AttemptRecord` retains original variable/row correspondence, submitted starts, options, termination, work and actual native-state reuse. `solve/observations.rs` distinguishes submitted-oracle coordinates from physical tears; `NativeSnapshot` records total dimension and partial capture. `workflow/local_analysis.rs` retains local validity, root/KKT withholding reasons and physical quantity correspondence. Study occurrence identity remains distinct from equal binding content.

**Proposed measures.** Associate an observation with the actual immutable model/revision and physical interpretation, the occurrence/role/attempt and its parent work, the selected route, and the body/block or original equation/variable correspondence needed to explain it. Distinguish original, scaled, reduced and native coordinates. Residual magnitudes need their quantity/unit and normalization or allowance; a scaled norm must not be presented as a physical balance defect. Preserve mappings through elimination, aggregation, relaxation, discrete fixing and event resets. Dimension and omission metadata must accompany selected coordinate detail.

These are scientific and causal attribution needs, not exact source/environment rebinding requirements. A diagnostic need not copy a full model or all coordinates into each event. Compact references plus selected original identities can carry the explanation. Neither a partial snapshot nor an incumbent is a resumable native checkpoint or a scientifically usable result merely because it is observable.

### 1.2 Time, work, memory and concurrency

**Implemented.** `math.rs::PreparationCounts` distinguishes structural views, observation programs, sharing rebinds and rebuilding rebinds. `math.rs::counted` provides task-local enclosing-operation counts despite concurrent work. `pse-engine/src/cache_service/metrics.rs::Metrics` counts actual assembly builds/reuses, SQL parsing/binding, analyses, optimizations, physical planning and reusable executions. Native `solve.rs::WorkEvidence` retains optional evaluations, iterations, factorizations and proof steps. `callback.rs` exports method/block callback counts and inclusive callback seconds. `env.rs::SharedRuntime::report` separates pool reservation/peak and top consumers from process-lifetime RSS high-water observations.

**Proposed analysis.** Use elapsed time to locate an interval and actual work counters/structure to explain it. Distinguish active computation, admission or dependency wait, I/O/publication wait and cancellation/drain. Inclusive parent and child times overlap; CPU, system and wall times also overlap. Concurrent work totals do not sum to wall duration. Reused allocation does not establish reused numeric factors; a reused seed does not establish reused multistep history. Pool reservation is neither actual heap allocation nor process RSS. A process-lifetime high-water mark cannot be attributed to one late case, and global cache counters cannot be assigned to one concurrent operation by subtraction without appropriate ownership.

## 2. Live, recent and completed views

The user needs to distinguish “what happened in the last minute?” from “what has this run done?” and “what is it doing now?” These require related but different views.

**Implemented.** `solve.rs::Progress::push` keeps the earliest events that fit its admitted limit and increments a dropped count thereafter. It is bounded prefix history, not a rolling recent window. `RunHandle::progress` and Python `_runs.py::RunHandle.progress` expose that history and loss count. The durable tap in `workflow/durable.rs` uses nonblocking queue admission, sequence and wall timestamps; `ProgressCoverage` separately records observed, queued, dropped-before-queue, acknowledged and unacknowledged events and writer outcome. `workflow/progress.rs::ProgressStream` reads an exact terminal attempt, not a provisional live tail. `NativeObservations` contains a bounded current native snapshot, but its contract is distinct from the generic event history. Benchmark `native_process/phases.rs` accumulates closed spans; benchmark phase JSON is written only after the case group finishes.

**Proposed views and analysis rules:**

| View | Information needed | Honest interpretation |
|---|---|---|
| Current activity | Owning phase/child, phase entry observation, active/waiting/draining state when known, last observation and source, native checkpoint/physical time where available | “Last observed in X” is valid when freshness is known. A stale callback is not proof that execution still occupies X or has hung. |
| Recent window, for example the last minute | Window boundaries, observed coverage, counter deltas with reset/attempt identity, recent phase transitions, native progress and blocking observations | Rates require valid denominator and coverage. A short run has a shorter observed interval. Missing recent events after prefix saturation is unknown activity, not zero work. |
| Cumulative operation | Actual work since operation/attempt admission, completed and active child accounting, preparation/reuse counts, method-specific work, memory scope | Inclusive totals and exclusive categories must be labeled; resets, retries and process changes cannot be silently merged. |
| Bounded explanatory history | Selected transitions, refusals/retries, latest failure and success, representative or exceptional samples with omissions | Bounded history need not retain every primitive call. It must say whether it is a prefix, suffix, sampled stream or selected transition history. |
| Terminal explanation | Completion/qualification, original typed causes, produced/withheld members, work and coverage through join/drain and publication | Scientific completion, publication completion and diagnostic-writer completion remain separate. Partial diagnostics do not create a success or invalidate a scientific result. |

**Proposed causal recipe.** First establish observation scope, freshness and loss; then locate the active owner and its parent. Compare recent deltas with cumulative work: a run can have a costly past phase and a different current bottleneck. Inspect the governing work dimension and reused products before blaming a library or hardware. Follow nested work to the producer whose counters actually advanced. Check original-space residuals, physical progress and rejected trials separately from callback traffic. If evidence only identifies a long inclusive phase, retain competing hypotheses and request one smaller, specific observation capable of distinguishing them. Do not infer a root cause from a timing correlation.

For hangs, the smallest settling evidence is often an entry/exit distinction, a worker/admission state and the latest actual native checkpoint—not a new statistical timing campaign. A timer sampled independently of a blocked worker must not call into that worker synchronously and inherit its hang. The review should assess how inexpensive live observations remain available while compilation, a native callback, publication or an owning actor is occupied.

## 3. Contexts, measures and causal recipes

Each subsection specifies a useful explanation, identifies source-backed suppliers and names the smallest revealing verification. All proposed measures describe capabilities to assess; they do not prescribe a universal trace schema, exporter or always-on event stream.

### 3.1 Cold admission, retained warm execution and edits

**Decision/questions.** Is repeated work necessary because relevant structure changed, because a retained product was evicted, or because an interface lost an already available product? Does an unchanged operation reuse mathematical construction, native allocation, factors or only source declarations? Does a value-only change rebuild only its actual dependents?

**Required measures, Proposed.** Separate source/document loading, physical/provider admission, specialization, structural analysis, class/affine proof, support/derivative demand, scalar/evaluator construction, case binding, native preparation, actual solve, assessment and cleanup. Record actual construction/reuse counts, selected body/output/coordinate extents, derivative orders and native-state/seed reuse. Attribute cache miss, eviction or changed dependency only where the owner observes it. Distinguish diagnostic requests from mathematical work they cause.

**Current owners/gaps.** `workflow/modeling/{declared,cases,preparation}.rs` own admission/binding; `math/artifacts.rs` owns scalar/artifact retention and flights; `pse-math/src/execution.rs` records support and compiled-body/order construction spans; `PreparationCounts` supplies view/rebind observations. The existing `declared_execution(...).analysis` followed by `prepare_analysis` can discard the already admitted model; `prepare_declared` retains it. That source sequence is a reuse question, not proof that every subsequent call recompiles. Public view/rebind counts alone do not distinguish every actual library construction, cache flight wait or eviction. Preparation can precede the public run handle.

**Recipe.** Compare immutable consumed structure and selected demand, then actual construction events, then native reuse receipts. Investigate a value rebind that creates new evaluator support; investigate a structural edit whose unrelated bodies rebuild; investigate unchanged execution after pressure separately from ordinary warm reuse. Expected reuse is not established by a faster second call.

**Explainable/unresolved.** Current task-local counts can explain which kinds of preparations the caller performed. Construction spans can expose work to an installed subscriber. Whether the normal user can recover a complete live-to-terminal causal account of all preparation is unresolved in this source inspection.

**Smallest verification, Proposed.** S01–S04: one original operation and one purposeful repeat/edit with actual construction and rebind observations. These are correctness/causal comparisons of work, not comparative latency measurements.

### 3.2 Steady algebraic methods and solver classes

**Decision/questions.** Is the solver making useful numerical progress, repeating costly evaluation, rebuilding linear algebra or spending time in an inappropriate mathematical representation? Which method-specific work increased? Was the original model assessed after a native stop?

**Implemented owners.** `pse-backend-native/src/execution.rs` is the adapter capability/representation owner, including roots, NLP, coefficients, cones and trajectories; current backend variants cover KINSOL/PETSc, Ipopt/POUNCE/Uno, HiGHS/POUNCE-convex, Clarabel, SCIP, Diffsol and IDAS. This lists interfaces, not a claim that every adapter is linked or qualified in the inspected run. `SolveReport` separates metrics, typed evidence, preprocessing, native termination, original qualification and progress. `NativeCounters` currently has dedicated Ipopt and KINSOL variants; other adapters also supply method-owned metric maps and typed evidence. `WorkEvidence` fields are optional and not interchangeable cost units.

| Method context | Required method-specific measures, Proposed | Analysis that could change a decision |
|---|---|---|
| Nonlinear roots, fixed point, Newton–Krylov | Residual/map calls, attempted nonlinear versus accepted outer iterations, J*v, Jacobian/preconditioner setups and solves, linear iterations, actual step/rejection state | Distinguish poor contraction or noisy inner maps from many linear iterations or repeatedly rebuilt preconditioners. KINSOL fixed-point iterations do not mean accepted physical states. |
| NLP and SQP | Objective/constraint/gradient/Jacobian/Hessian callbacks and inclusive work, trial rejection/regime changes, restoration/main state, factor/setup work where observable, scaled and original infeasibility, seed/state reuse | Distinguish repeated failed trials, derivative cost, globalization/restoration and linear algebra. High callback cost does not itself establish poor convergence. |
| LP/QP and coefficient formulations | Export/lowering dimensions and updates, native model/basis reuse, simplex/IPM/crossover/active-set iterations, presolve effects, solve and original readback/qualification | Distinguish rebuilding a coefficient model from legitimate new solves. HiGHS allocation/basis retention does not promise factor reuse. |
| Conic methods | Cone/block dimensions and transformations, native iterations and residual conventions, setup/factor work where available, original-coordinate readback and assessment | Distinguish expensive conic lowering/setup from progress on the native conic problem; preserve which residual belongs to which representation. |
| MIP/MINLP/global certification | Nodes, relaxations/subsolves, cuts/propagation/proof work where supplied, incumbent/bound/gap history in declared sense and units, original assessment, exact stop category | A node is not an iteration. More callbacks may reflect useful global search. An incumbent, local feasible candidate and global bound are separate observations. |
| Reduced, implicit and nested formulations | Original/reduced dimensions, reconstruction work, inner solve/evaluation/factor counts, branch and accuracy context, original reassessment | Distinguish a cheap outer iteration with expensive repeated inner work from outer stagnation. Native coordinates must remain attributable to original variables/rows. |

**Gaps to assess.** Native progress/getter availability and cancellation fidelity differ by adapter; a common presentation cannot invent missing counters. Terminal callback metrics do not ensure recent live visibility. Aggregate factorization counts must not be manufactured from iterations or Jacobian calls. The review should assess adequate per-method diagnostics rather than force every method into the Ipopt shape.

**Recipe.** Establish method, representation, effective controls and scope. Locate the native versus callback versus original-assessment interval. Relate work counts to actual numerical progress and the original criterion. Investigate which producer performs repeated construction or evaluations. Leave a cause unresolved when the adapter exposes only aggregate solve duration.

**Smallest verification, Proposed.** S05 selects one revealing existing small fixture for each materially different representation/coverage contract; this is not a request for all-backend statistical campaigns. S06 checks transformed coordinates and original assessment.

### 3.3 Initialization, recycles and continuation

**Decision/questions.** Is effort spent finding a valid initial state, solving a tear map, retrying a failed stage, tightening an inner solve or reconstructing the original model? Did continuation advance from an accepted state or repeat a rejected state? Did a discrete initialization fix escape its temporary scope?

**Required measures, Proposed.** Named stage/homotopy parameter, accepted and rejected attempts, step changes, submitted and committed starts, original checks, discrete assignment, tear/map residual in physical allowance, map calls and inner work with parent identity, inner accuracy/start/branch context, reconstruction and final original solve. Separate outer accepted state from raw map trial and from inner native iterations.

**Current owners/gaps.** `workflow/modeling/engines.rs` retains `ModelingInitializationAttempt`, committed values and failure/interrupt causes. `math/initialization.rs`, `math/solves/{causal,blocks,paths}.rs` and native `recycle.rs` own composition. `RawMapSample` retains role, input/mapped/residual values, physical error ratio, omissions and failure; `RawMapInnerContext` distinguishes frozen, inapplicable and unavailable inner context. These records already prevent interpreting an absent map output as zero. Full live attribution of nested work and recent history still needs assessment; a raw context hash describes supplied permissions/settings, not a proof of achieved branch or uniqueness.

**Recipe.** Follow the sequence of attempted stages and committed predecessors. Within a slow outer attempt, follow raw-map roles and actual inner work; compare contraction in original permitted coordinates with changes in inner accuracy or branch. Determine whether the controller is legitimately adapting or redoing already failed work. Preserve a final original-specification assessment independently of temporary stages.

**Explainable/unresolved.** Attempt and raw-map records can explain the scientific sequence and known supplied inner context. Attribution is unresolved where an opaque producer supplies no inner facts or bounded history omits the relevant trials; do not reconstruct them from text or elapsed time.

**Smallest verification, Proposed.** S07–S08: one failed stage followed by a legitimate retry, and one recycle map containing an inner failure or branch/accuracy change. Check identities and counters, not convergence-rate thresholds.

### 3.4 Steady sensitivities, KKT and requested postsolve analyses

**Decision/questions.** Is the requested result unavailable because stationarity/rank/active-set premises failed, or because producing derivatives/factors was expensive or refused? Was the primal point recomputed unnecessarily? Did a new RHS reuse a compatible factor or merely a starting point?

**Required measures, Proposed.** Requested output/parameter identities and action/order; primal correction and original assessment; derivative construction/evaluation; Jacobian/KKT dimensions and sparsity; factor constructions and backsolves only when observed; rank/conditioning/active-set qualification; requested versus produced/withheld quantities; factor retention/reuse and its consumed coordinates. Keep original and normalized tolerances and finite/nonfinite observations distinct.

**Current owners/gaps.** `math/solves/{derived,output_program,kkt_accuracy}.rs` and native `kkt/sensitivity.rs` supply derived work. `workflow/local_analysis.rs` projects validity and withholding reasons, including multipliers, stationarity, LICQ, complementarity, second-order and backsolve failures. `workflow/uncertainty.rs` composes parameter covariance with the derivative response and preserves upstream withholding. Strategy product records retain actual actions and derivative order. These outputs explain why a quantity is valid/withheld; they do not by themselves provide every live phase or factor-cost attribution.

**Recipe.** Start with the requested scientific quantity and its premises. Follow the primal point and derivative/action product actually consumed; then locate factor/backsolve and assessment work. A large condition estimate is a numerical lead, not proof of the performance cause. An unavailable derivative is not a slow successful derivative or a zero response.

**Smallest verification, Proposed.** S09: a tiny primal problem with two requested responses and one deliberately withheld premise, exposing which factors/actions actually occurred and preserving original-coordinate meanings.

### 3.5 Dynamics, consistent initialization, events and adjoints

**Decision/questions.** Is elapsed work advancing physical time or retrying initialization, internal steps, events or sensitivities? Is output-grid density causing repeated work? Are forward, backward and second-order solves distinguishable? Is a stop before the requested horizon explained with valid partial coverage?

**Required measures, Proposed.** Requested/consistent initial state and IC attempts/backtracks; current physical/internal/returned time with method semantics; accepted steps, rejected/error-test failures, nonlinear/linear iterations, RHS/Jacobian/J*v/preconditioner work where supplied; method order and current/last step distinctions; segment/event/mode/reset identity; output-sampling work and coverage; forward sensitivity directions, adjoint checkpoints/replays/backward segments, Hessian directions and postsolve original/conservation checks. Record actual reuse versus reset/reinitialization of multistep history.

**Current owners/gaps.** `workflow/modeling/dynamics*.rs`, `workflow/dynamics.rs`, native `dynamics/{idas,integrator,anchored}.rs` own this flow. `ConsistentStateReport` retains callback/native evidence, backtracks and separate native/assessment failures. Native `Report` retains completed time, endpoint, sample coverage, actual events, conservation, per-finished-segment statistics, progress and dropped counts. `NativeStepObservation` carefully distinguishes IDAS internal versus returned time and current versus last step/order; Diffsol does not claim getters it lacks. Per-segment statistics and completed evaluation events do not necessarily expose the active inner segment during a hang.

**Recipe.** Distinguish wall clock from scientific time and output samples from native steps. Locate initialization versus forward integration versus event settlement versus backward/action work. Compare recent physical advancement with work counters and rejected steps. If work advances while scientific time does not, inspect the method's legitimate local phase before inferring a hang. Correlate resets with history invalidation and reinitialization. Validate conservation/endpoint coverage separately from performance explanation.

**Smallest verification, Proposed.** S10–S12: a small IC retry, one event/reset and an off-grid terminal endpoint, then one forward/adjoint request. Use known causal structure and counters; no speed or minimum step-size gates.

### 3.6 Independent, causal, lazy and durable studies

**Decision/questions.** Are points waiting on scientific dependencies, capacity, source activation, preparation, storage or slow numerical work? Are equal inputs sharing preparation without merging occurrences? Are ready independent points batched lawfully, and are causal points using only permitted predecessors? Where does managed work occur?

**Required measures, Proposed.** Occurrence key/index, dependency readiness and reason, selected versus unselected lazy points, source activation/admission, attempts/retries and start provenance, preparation/construction/rebind counts by execution owner, batch membership/size and actual parallelism, ready/queued/running/settled populations, child work and completion, publication/acknowledgment state, cancellations and unattempted points. Separate total work from critical path and observer-local counts from worker-local counts.

**Current owners/gaps.** `workflow/{study,study_execution,study_operations,study_variants}.rs` compose the existing operation owners under shared study policy. `StudyReport` preserves outcomes, available results and decision. Its preparation counts explicitly cover the calling process and exclude the external managed primary; Python repeats that scope. Task-local counts prevent simple concurrent-operation contamination locally. Durable worker/occurrence and progress owners preserve attempts and publication effects. This source inspection does not establish a joined cross-process performance account. A caller seeing zero preparations cannot conclude that the worker reused everything.

**Recipe.** Traverse the actual readiness/dispatch path rather than counting the declared Cartesian space. Find the critical predecessor or capacity wait, then attribute each selected occurrence's actual preparation and numerical work. Distinguish fresh fallback authorized by policy from a continuation whose seed is unavailable. For durable work follow worker attempt and terminal effect state before diagnosing a retry or apparent duplicate execution.

**Smallest verification, Proposed.** S13–S15: two independent points with one skewed child, a short causal chain with a failed predecessor, and the same small operation under a managed observer. Inspect identity/coverage and scheduling facts rather than throughput.

### 3.7 Fitting and profile likelihood

**Decision/questions.** Is the outer optimizer slow because each experiment is expensive, derivatives repeat integration, original reassessment duplicates work, or profile chains repeat full fits? Which experiments or parameter ends determine the critical path? Is an interval withheld scientifically or missing because a worker never started?

**Required measures, Proposed.** Outer trial/callback and exact parameter identity, experiment identity/type and actual parallelism, per-experiment steady/integrated/response work, point-cache reuse and achieved derivative order, atomic publication of completed experiment products, final original assessment, response/rank/covariance work, pinned parameter/end/chain identity, accepted profile points and adaptive choices, scheduling/worker failures. Distinguish uncertainty analysis from primal fitting and a profile chain from independent study points.

**Current owners/gaps.** `workflow/fitting/{preparation,oracle,experiments,profile,covariance}.rs` own these stages; `FitReport` separates solve, strategy, checks, responses, rank, covariance and profiles. `fitting/oracle.rs` caches an evaluated point and emits `fit.evaluation` with experiment count after the complete evaluation. `ProfileChain` retains each pinned fit, actual parallelism and typed scheduling/environment/panic failures. Existing total experiment counts do not identify a currently occupied slow experiment or explain every derivative reconstruction. Aggregate native fit callbacks must not be mistaken for individual experiment solves.

**Recipe.** Descend from outer optimizer trial to the experiment/action creating work, preserving derivative order and cache state. Compare cumulative evaluations with distinct actual input points without merging scientifically distinct attempts. At interval work, inspect the chain's adaptation, accepted seeds and stop reason; separate unstarted chains and worker failure from scientifically invalid endpoints. Preserve both failed/partial experiment work and original final checks.

**Smallest verification, Proposed.** S16–S17: one small fit with two distinguishable experiment types and a derivative upgrade at an unchanged point; one profile chain with a controlled worker or inner-fit failure. Check causal attribution and retained outcomes, not confidence-interval performance.

### 3.8 Surrogate and multifidelity proposals

**Decision/questions.** Is effort spent evaluating fidelity callbacks, fitting/tuning the surrogate, optimizing infill, screening an approximate proposal or correcting it on the original model? Does the approximation preserve the original scientific target and coordinate interpretation? Can live inspection proceed while the owning actor is occupied?

**Required measures, Proposed.** Original/model target and fidelity correspondence, coordinate/output identity, inclusive successful/failed/repeated callback evaluations, model/iteration phase, GP/tuning/infill work when observed, retained sample/state extent, proposal production, original screening/correction and its independent work/acceptance, cancellation and actor join. Recent activity must come from a safe observation path, not require a blocking request to the busy actor.

**Current owners/gaps.** `math/surrogate.rs` owns a task-admitted retained actor; `math/surrogate/library.rs` preserves fidelity correspondence, explicit controls and library-owned iteration. `SurrogateHandle::work` reports inclusive partial/failed callback/iteration work; `snapshot` explicitly copies history under a separate allocation, whereas compact advancement does not. A proposal is explicitly not original-result permission. `math/strategy.rs::RungProducts` retains statistical/native products separately. Actor commands use request/reply synchronization, so live-read behavior while a phase is executing warrants focused inspection rather than assuming every getter is nonblocking. This inquiry did not qualify GP internals or any specific live sampler.

**Recipe.** Follow approximate evaluation/model work separately from original correction. Identify which fidelity and scientific observable were evaluated; an improvement in approximate criterion does not establish improved original feasibility. Distinguish history-copy overhead from model advancement. If live inspection waits behind the active phase, mark that visibility limitation explicitly.

**Smallest verification, Proposed.** S18: a tiny declared-fidelity proposal with one failed callback and an original screening refusal; verify inclusive work and separate approximate/original outcomes without a convergence or speed claim.

### 3.9 Shooting, simultaneous formulations and horizons

**Decision/questions.** Is a dynamic optimization spending work in formulation assembly, outer NLP trials, repeated window integrations, continuity/path assessment or sensitivities? In a horizon, which work belongs to plant, estimator, controller, background prediction or fallback, and which lies on the actual decision path?

**Required measures, Proposed.** Formulation/mesh/window dimensions, original-to-transcribed coordinate maps, outer trial and derivative order, window/segment work and replay, terminal versus integral objective actions, continuity/path original assessment, role/sample identity, measurement/window readiness, state/seed/factor reuse, prediction/backsolve versus full-solve fallback and its reason, applied/held moves, plant advancement and role-specific failures. Scientific sample time and diagnostic wall time stay distinct.

**Current owners/gaps.** `workflow/modeling/dynamics.rs` supplies simultaneous/collocation operations. `workflow/shooting.rs` composes single/multiple anchored windows with forward and adjoint actions and emits total windows after a completed evaluation. `workflow/horizon.rs` and `horizon/driver.rs` compose sequential plant/MHE/NMPC roles; advanced-step prediction can retain a sensitivity factor and fall back to a full solve. `horizon.step` events and retained modeling requests are useful causal anchors. They do not alone establish live visibility inside a current role or exclusive accounting between background and foreground work.

**Recipe.** Start at the physical operation and its formulation. Trace each outer trial into windows/actions and back to original assessment. For horizons follow sample order, role readiness, predecessor state and actually applied controls; attribute background work without adding it blindly to critical-path time. A prediction failure followed by a full solve is not two unexplained solves, and a held control after rejection is not successful optimization.

**Smallest verification, Proposed.** S19–S20: a small two-window shooting case with a rejected trial and a simultaneous counterpart used for mapping/coverage, not timing comparison; a short horizon with one valid prediction and one recorded fallback/hold.

### 3.10 Entry points, placement, outputs and lifecycle outcomes

**Decision/questions.** Does a user-visible wait occur before dispatch, in numerical work, after scientific completion during publication, or in result conversion/read/export? Do Rust and Python describe the same underlying operation? Can a cancelled observer detach without losing the supervisor's native join? Which partial result remains valid when another stage fails?

**Required measures, Proposed.** Request/preparation/run/attempt and worker identity; admission/dispatch/execute/join/publication/read/teardown boundaries; actual execution-owner counters; transport/queue/acknowledgment state; compact/full/streamed demand and produced/withheld members; observer cancellation versus native cancellation/abandonment; terminal cause and last checkpoint; live and terminal diagnostic loss/freshness. Host/placement conditions provide context without exact host equivalence proof. Account result bytes/batches and repeated reads separately from compute; distinguish observer RSS from primary RSS and process-lifetime from operation memory.

**Current owners/gaps.** `workflow/{run,worker,durable,completion,progress,connected_results,result_projection,live_simulation}.rs` own these boundaries; Python `_runs.py`, `_modeling.py` and `_workflow.py` project the Rust execution/results. Python synchronous wait releases Python and joins cancellation before propagating a signal; async cancellation requests stop while retaining the supervised report. Durable progress coverage records writer outcomes independently of scientific completion. The same complete causal explanation across all placements and output demands remains a review question, not established by shared type names or a local benchmark. Preparation failure before a run exists and storage failure after native success need independent attribution.

**Recipe.** Locate the currently owning boundary, then inspect its actual child/worker and completion state. A heartbeat proves authority renewal, not numerical progress. No progress event can mean uninstrumented work or dropped telemetry. Separate declined scientific permission, completed science with failed publication, cancellation still draining, worker loss and uncertain effect. Preserve valid earlier members without claiming whole-operation success.

**Smallest verification, Proposed.** S21–S25: Rust/Python projections of one small run; bounded progress saturation; preparation failure before start; cancellation while a child is active; publication/telemetry failure after numerical completion. Test only the revealing ownership/coverage boundaries, without a cross-environment proof campaign.

## 4. The cold 32-block example as a diagnostic scenario

**Source-inspected facts.** `.config/process-cases.json` declares `cold-large-1` with 32 heater blocks, one worker thread and 192 expected unknowns. `benches/benches/native_process.rs` cold mode skips the retained pre-solve. Each operation creates a runtime, admits the shared reference packages, constructs the 32-child authored workload, obtains declared execution and analysis preparation, starts and joins one top-level solve request, validates original authored checks/results and cleans up. One control is not one primitive evaluation: native iterations, callbacks, preparation phases and possible composed work remain inside it. Warm mode separately performs a retained setup solve before its measured/control operation. Neither cold repetition nor a completed cold control establishes reuse.

The interrupted log at `build/assessment/p39-practical-science-and-measurements-20261015/measurements/process-cost/cold-large-1/process.log` shows Criterion warmup followed by an estimated 6222.7 seconds for ten iterations. It contains no completed current phase JSON. Roughly 622 seconds per iteration is the estimator's implication, not measured attribution of that interval to preparation, Ipopt or any particular derivative owner. The interrupted enclosing command is not a successful completed campaign.

**Structural lead, not established timing cause.** Default automatic presolve with the explicit Ipopt NLP route can request unresolved affine-row evidence in `math/solves.rs::discover_class`. `pse-compiler/src/workspace.rs::PreparedCase::prepare_affine_rows` calls `pse-math/src/class_evidence.rs::CasePlan::affine_row_facts`. Its aggregation creates a global formal vector from all plan columns, rebinding each contribution through sequential temporary substitutions. `prove_affine` differentiates a row against the global formal list until it encounters a nonconstant derivative. Independent heater blocks involve local coordinates, yet later rows can traverse many unrelated global formals and repeatedly visit substantial PC-SAFT expressions. This is a concrete amplification mechanism worth examining under AP-07; the present evidence does not establish its fraction of the current duration. Whole-program coefficient-class admission is skipped for explicit Ipopt, so that is not the same hypothesis as the automatic affine-row request.

The selected Helmholtz pressure/enthalpy responses already contain potential derivatives. Numerical derivative demand and compilation therefore deserve separate observation. However `CasePlan` uses local demand products and the runtime retains scalar/artifact families. The evidence does not justify asserting a single dense 192-axis Hessian evaluator or 32 independently compiled copies of every body.

Historical `build/assessment/p16-p18-final-costs/process-cost/cold-large-1/cold-large-1-memory.json` records 96 variables, not the current declared 192. Its short preparation and native/Hessian timings are historical leads only. They cannot support a current before/after comparison, a regression magnitude or a conclusion that the same phase now dominates.

**Proposed settling observation, S26.** Retain the exact scientific model/assertions and inspect one operation only if the source question remains material. Observe coarse admission/preparation/native-join/assessment/drain boundaries and the affine-row entry/exit plus relevant actual row/formal/dependency work. The revealing criterion is whether observed work follows local incidence or needlessly scans unrelated coordinates and where that work sits in the operation. No wall-time threshold, repeated latency sample or environment proof is required. A smaller source-derived fixture with two independent blocks can settle the locality mechanism without rerunning the 32-block cost; it cannot quantify the full run's benefit.

## 5. Revealing scenarios and smallest verification

Every scenario below is **Proposed**, not executed here. These are independently selectable correctness and explanation scenarios. They are not a mandatory all-context qualification campaign. Stable IDs belong to this supporting document; the coordinator may link them from the principal review without creating a second disposition ledger.

| ID | Small revealing stimulus | Revealing criterion; what remains unproved |
|---|---|---|
| <a id="s01"></a>S01 | Cold operation followed by unchanged retained execution | The explanation distinguishes actual new construction from reuse and still shows original validation. A faster repeat alone is insufficient. |
| <a id="s02"></a>S02 | Change one value consumed by a product and one value not consumed by it | Rebuild/share observations follow the consumed dependency; no assumption that every value change is structurally free. |
| <a id="s03"></a>S03 | Structural edit in one independent model region | Changed and reused bodies/maps remain attributable; unchanged regions do not become invisible or falsely classified as rebuilt. |
| <a id="s04"></a>S04 | Retention pressure or explicit retirement before an unchanged request | An observed eviction/rebuild is distinguished from semantic invalidation and native-session reuse; no invented eviction cause. |
| <a id="s05"></a>S05 | Small representative root, NLP, coefficient, conic and discrete/global operations where available | Counter definitions, missing getters, native progress and original qualification remain method-specific. This establishes diagnostic contracts, not all-adapter qualification. |
| <a id="s06"></a>S06 | Scaled/reduced model with an original-row violation or withheld reconstruction | Native and original residuals, mappings and permission remain distinct; partial native coordinates cannot appear as complete original state. |
| <a id="s07"></a>S07 | Initialization stage rejection followed by a permitted smaller homotopy step | All attempted work and committed predecessor state remain visible; rejected starts are not reported as accepted progress. |
| <a id="s08"></a>S08 | Recycle map with one inner failure or changed supplied branch/accuracy context | Outer map role, physical residual and actual inner work are distinguishable, with unavailable context explicit. |
| <a id="s09"></a>S09 | One steady point, two requested response RHSs and a deliberately withheld KKT/root premise | Actual derivative/factor/backsolve work and quantity-specific validity are explainable without repeated primal solves being hidden. |
| <a id="s10"></a>S10 | Consistent-initialization retry before a short DAE trajectory | IC/backtrack work cannot be mistaken for forward physical-time progress or a zero-length successful simulation. |
| <a id="s11"></a>S11 | One event/reset with off-grid terminal completion | Internal time, returned endpoint, output-grid coverage, reset and history changes preserve their native/physical meanings. |
| <a id="s12"></a>S12 | Short trajectory with forward and adjoint requests | Forward/backward/checkpoint/replay and assessment work remain attributable; more derivative work is not automatically a convergence defect. |
| <a id="s13"></a>S13 | Two independent study points with one occupied child | Readiness, actual parallelism, child work and total versus critical-path explanation survive skew; no throughput threshold. |
| <a id="s14"></a>S14 | Three-point causal/lazy frontier with a failed predecessor | Dependency waits, unselected/unattempted points and actual seed permission are visible without eagerly preparing the declared space. |
| <a id="s15"></a>S15 | One small study operation executed by a managed durable worker | Observer-local zero counts are not presented as worker reuse; attempt/worker/publication and diagnostic coverage remain attributable. |
| <a id="s16"></a>S16 | Two-experiment fit with a derivative-order upgrade at one point | Distinct experiment and action work, point reuse and final original assessment remain visible beneath the outer callback. |
| <a id="s17"></a>S17 | Profile chain with an inner rejection and a scheduling/worker failure | Scientific endpoint validity and operational chain failure remain separate; started and unstarted work are accounted honestly. |
| <a id="s18"></a>S18 | Surrogate phase with a failed fidelity callback and original screening refusal | Inclusive approximate work, proposal and original correction/permission remain separate; history copying is observable as copying. |
| <a id="s19"></a>S19 | Two-window shooting and a small simultaneous formulation | Window/mesh and original-variable mappings explain nested work and failed continuity/path assessment; no latency comparison is implied. |
| <a id="s20"></a>S20 | Short horizon with prediction, fallback and rejected-control hold | Plant/estimator/controller/background roles and actual applied controls can be followed causally; background work is not blindly added to critical-path time. |
| <a id="s21"></a>S21 | Rust and Python sync/async consumers of one small operation | They preserve underlying operation identity, availability and coverage; different waiting/transport costs remain separate from science. |
| <a id="s22"></a>S22 | Saturate a tiny progress buffer while work continues | Recent/current activity remains distinguishable from retained prefix history; drop/reset/freshness facts prevent “no events means idle.” |
| <a id="s23"></a>S23 | Preparation refusal or occupied construction before a run handle exists | User can identify the last observed phase, known work or waiting state, original cause and limits of observation without a fabricated running attempt. |
| <a id="s24"></a>S24 | Cancellation during nested native work, then native join/drain | Request, checkpoint/stop, child teardown and join remain distinguishable; loss of a waiter is not proof the producer stopped. |
| <a id="s25"></a>S25 | Native completion followed by telemetry-writer failure, publication uncertainty or connected-read failure | Scientific result, publication effect, diagnostic coverage and read outcome remain separate; valid partial members are neither discarded nor promoted to whole-run success. |
| <a id="s26"></a>S26 | Independent-block affine proof with local incidence, retaining original assertions | Actual scanned/differentiated coordinates and repeated substitutions reveal or refute the proposed locality amplification; this establishes no 622-second attribution or speedup. |

## 6. Integration questions for the principal review

The source supports several independent questions, rather than a conclusion that one universal profiler fixes every context. Preparation construction and invalidation need one explanation; nested scientific work and numerical validity need another; live progress, bounded loss and cross-process ownership need compatible but distinct treatment. These concerns intersect at actual operation/child identity and explicit coverage.

The principal review should settle whether existing spans, method-owned evidence, task-local counters and durable progress can compose into the target without duplicate authorities or making scientific consumers understand every backend's implementation. It should compare compact common navigation with method/context-specific detail, and recent/current observations with optional bounded history. It should also assess observation overhead at the scale of property callbacks, large studies and remote publication: richer diagnostics need not mean copying every model, tracing every scalar operation, storing unbounded history or blocking numerical work on a telemetry sink.

A useful diagnosis can establish necessary work, unnecessary amplification, a named long phase, a failed scientific premise, an unavailable observation or an unresolved causal hypothesis. It should state which of those it has actually established. Neither exact benchmark statistics nor complete environment proof is a prerequisite to making those distinctions.

This document leaves the final architectural judgment, selected alternative, rule impacts, finding IDs/disposition and publication with the coordinator. There are no new implementation or campaign obligations created here.
