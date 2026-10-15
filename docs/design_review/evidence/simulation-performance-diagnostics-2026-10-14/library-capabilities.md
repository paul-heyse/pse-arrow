---
title: Simulation performance diagnostics library capabilities
doc_role: evidence
doc_owner: docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
doc_retention: while-dependent
---

# Simulation performance diagnostics: library capability evidence

Supporting evidence for [the simulation performance diagnostic design review](../../reviews/design_review_simulation-performance-diagnostics_2026-10-14.md). This is a library research catalog, not the overall review verdict or an implementation proposal.

## Scope and evidence basis

The [sparse-backend and native-timer supplement](linear-backend-diagnostics.md) resolves additional exact field and activation questions for MUMPS, SPRAL, oneMKL, KLU, Ipopt and Clarabel. Wrapper forwarding and safe live access remain separate integration questions.

Baseline supplied by the coordinator: `a08f5fa09` plus the concurrent dirty checkout examined on 2026-10-14. `git status --short` showed existing changes across plans, architecture, validation, scripts and tests. None was modified by this inquiry. No builds, services, benchmarks or runtime probes were run by the researcher, who authored this catalog in coordinator-assigned scratch. The coordinator published it here; the separate tooling inquiry records its own two bounded probes.

The shared worker/library-research contracts and `native-solver-libraries`, `symbolica-faer-oximo`, and Context7 skills were read. Exact checked-out source, vendored source, installed Cargo source and skill-captured source supply interface evidence. Context7 was resolved and queried for SUNDIALS, PETSc, Symbolica, Ipopt, HiGHS and Clarabel; current documentation was not silently transferred to older releases. No utilization ledger or capability map was used.

Relevant architectural authority is blueprint §7 (library-owned mathematics), §14 (compilation/preparation), §19 (workflow/result meaning), §20.6 (durable attempts/progress) and §23.1–§23.2 (observability/failure semantics). Source locations below are evidence pointers, not alternative contract owners.

**Evidence vocabulary:** all interface/catalog conclusions below are **Interface-checked**, with source-observed current adapter behavior. They are not newly **Tested** or **Measured**. An existing test name is a candidate review control, not a receipt that this inquiry executed it.

## Reading the catalog

- **Current:** the examined adapter already collects or publishes the fact. This does not establish end-to-end availability in every simulation context.
- **Available, unexposed:** an exact native/public interface supplies a fact that the examined collector does not project. Stage and mode eligibility still apply.
- **Instrumentation needed:** the adapter owns an operation or state transition that could be observed without adding scientific evaluations, but no existing requested observation was established.
- **Unknown/unavailable:** either the examined interface lacks the fact, or safety, visibility, activation or version coverage remains unresolved. Unknown is never zero.

“Live” means supplied by the solver's executing thread through a documented callback, or copied by the owner at an actual safe return/checkpoint. A getter callable after a solve is not evidence that another thread may call it during that solve. No concurrent native-handle access is justified by this catalog. Terminal facts can be partial or absent following early refusal, cancellation, serious native failure or an operation that never returns.

## Cross-family live and cost boundaries

`crates/pse-backend-native/src/solve.rs::{Progress,Progress::push,Progress::snapshot,Progress::tapped}` exposes a mutex-protected event snapshot and an optional synchronous tap. Retention is a bounded **prefix**, not a rolling latest-minute window. Once full, later events increment `dropped_events`; the tap receives every event before retention/drop. Consequently a stored/snapshotted prefix alone cannot answer “what changed in the last minute” in a long run. Whether the joined job/durable observer maintains a suitable recent window belongs to the review's runtime path, not this catalog.

`callback.rs::CallbackState::evaluate` measures per-demand calls and wall seconds, including rejected trials and contained unwind work. Successful evaluations normally return without emitting a success event. Counts/times are local mutable maps; `finish` projects them to `callback.*.calls`, `callback.*.seconds` and evidence. Failure events and terminal totals do not establish live cumulative callback timing. The elapsed timer surrounds scientific work plus work-observation handling; its definition must remain explicit. `CallbackKey` distinguishes named operations and block ordinals. `CountedNlp` and `Execution::callback_work_owner` prevent selected nested charge duplication; a diagnostic aggregation must preserve that ownership.

Native iterations, callback calls, model expressions, dynamic steps, LP iterations, branch nodes and numeric factorizations have distinct units. CPU, system and wall time overlap rather than sum. Nested phase totals and parent totals overlap. A timer recorded after a call returns cannot show the current duration of that still-running call; “last published work completed at T” is different from “currently executing phase P.” No callback silence by itself establishes numerical stall, deadlock, completion or zero work.

Inspection must not invoke residuals, derivatives, original feasibility checks, properties or implicit solves merely to populate diagnostics. Consume already-computed native data or owner-observed boundaries. An intermediate iterate, raw fixed-point map output, MIP incumbent, returned native candidate and independently assessed original-space candidate carry different authority. Keep coordinate/scaling/phase, model/profile/attempt, sample completeness and accepted-state meaning attached.

## Ipopt

**Baseline:** supported solver image declares Ipopt 3.14.20; actual host link may come through `IPOPT_DIR`/pkg-config and is separately identified. Exact sources: `docker/solvers/Dockerfile`; `crates/pse-ipopt-sys/build.rs`; skill corpus `native-solver-libraries/content/corpus/ipopt@3.14.20/src/Interfaces/{IpSolveStatistics.hpp,IpTNLP.hpp}`. Current consumer: `pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`, `pse-backend-native/src/ipopt/{sequence.rs,runtime.rs}`, shared `uno/callback.rs`.

**Current live:** `Sequence::intermediate_callback` polls cancellation/deadline and supplies mode, iteration, objective, native primal/dual infeasibility, barrier, step norm, regularization, primal/dual step lengths and line-search trial count. The bridge obtains current iterate/violations within this callback and supplies available vectors to the Rust callback. Rust emits `ipopt.iteration`, including regime crossings and optional normalized iterate/stationarity observations; latest bounded snapshot records submitted coordinates and main/restoration/unknown phase. This is a native iteration observation, not a certified accepted original-space solution. Current bridge allocates full temporary current/Lagrangian vectors per callback before Rust's bounded retained prefix: inspection cost is not necessarily proportional to retained size.

**Current completion:** structured `SolveStatistics` iteration count, objective/constraint/gradient/Jacobian/Hessian evaluation counts, CPU/system/wall seconds; explicit availability flags and optional `NativeCounters::Ipopt`. Callback-inclusive work/times and failure witnesses are collected separately. Terminal native counts can differ from callback counts because callbacks include adapter assessments/other named work.

**Available/unexposed or instrumentation needed:** native timing machinery and `print_timing_statistics` are source leads for algorithm, evaluation, symbolic factor, factor and backsolve phase costs, not an established live structured export. Current intermediate ABI supplies no running phase-cost totals or numeric factor/reuse census. Review exact `IpTimingStatistics`/application ownership and each linked linear interface before extending claims. Session compatibility, update/rebuild and warm-start decisions can be investigated at current owner boundaries.

**Failure/hang:** a non-returning linear operation cannot produce new intermediate callbacks or completion getters. Cancellation is cooperative at evaluations/iteration polls; no asynchronous getter contract was established. Preserve last callback timestamp/phase and mark intervening work unknown. Possible external sampling is a separate evidence source, not a native counter.

**Semantic cautions:** Ipopt primal/dual/step quantities can have different internal/scaled/restoration meanings. Backtracking count does not include every second-order correction operation. Distinguish requested warm start, compatible retained application, actual native reuse and achieved solution quality.

Upstream: [3.14 interfaces](https://github.com/coin-or/Ipopt/blob/stable/3.14/doc/interfaces.dox), [3.14 output meanings](https://github.com/coin-or/Ipopt/blob/stable/3.14/doc/output.dox).

## KINSOL

**Baseline:** `sundials-sys =0.6.2`, vendored bundled SUNDIALS 7.1.1, KINSOL/KLU/static build features. Sources: `Cargo.toml`, `vendor/sundials-sys/vendor/include/kinsol/{kinsol.h,kinsol_ls.h}`, corresponding bundled implementation; consumer `pse-backend-native/src/kinsol.rs`.

**Current live:** residual/map, Jacobian/J*v and preconditioner setup/solve callbacks are safe owner checkpoints, timed/countable by `CallbackState`; failed evaluations emit failure events. Raw map capture distinguishes fixed-point evaluation, original assessment and restart screening, bounded physical/submitted coordinates, partial capture, failure and frozen inner scientific context. The raw history is attached after the solve, not established as a live shared snapshot. No iteration monitor exporting current nonlinear convergence was found in the examined adapter. KINSOL's internal residual-monitoring option controls refresh policy; it is not this application's live observer.

**Current completion:** nonlinear iterations, function evaluations, beta-condition failures/backtracks; for non-fixed-point methods Jacobian evaluations, finite-difference linear function evaluations, preconditioner evaluations/solves, linear iterations/failures, J*v evaluations and last linear flag; function norm/step length only when current meaning is established. Typed `NativeCounters::Kinsol` and callback totals remain available independently of progress retention.

**Instrumentation needed/unknown:** callback entry/current duration, safe per-checkpoint cumulative timing and refresh/reuse reason can be investigated at existing owners. Native getter reads during arbitrary callback execution require exact consistency/reentrancy analysis; none is authorized from another thread. Last setup point and setup-call observations exist but do not imply that every native linear factor is counted. Fixed-point mode has no applicable linear counter; a failed G evaluation can already increment native attempted iteration count. Retained counters after illegal input or before current linear initialization can be stale, and adapter explicitly masks them.

**Causal questions:** are additional residual evaluations caused by globalization, FD/J*v, restart screening or original assessment? Are preconditioner rebuilds native refresh policy, changed structure or repeated outer applications? Does residual stagnation mean native scaled norm, physical G(x)-x or failed inner solve? Keep these separate.

Upstream: [KINSOL optional outputs](https://github.com/llnl/sundials/blob/main/doc/kinsol/guide/source/Usage/index.rst); exact 7.1.1 local source governs applicability.

## PETSc

**Baseline:** PETSc 3.24.0, MPIUNI, real double, Int32, serial/non-OpenMP build declaration and repository patch in `Cargo.toml` metadata. Consumer: `crates/pse-petsc-sys/bridge/{pse_petsc.h,pse_petsc.c}`, `pse-backend-native/src/petsc.rs`, `petsc/{flow.rs,blocks.rs,derived.rs}`.

**Current live checkpoints:** SNES function/Jacobian/convergence callbacks; TS implicit function/Jacobian, domain, pre-stage and post-step callbacks. The SNES convergence callback invokes native default convergence under `CallbackState`; it does not itself emit a numeric iteration progress event in the examined function. TS safe-owner callbacks can inspect the live TS within execution ownership; current stats are used by flow callbacks. Callback presence establishes a checkpoint, not that live numeric facts reach user progress.

**Current completion:** SNES converged reason/string, nonlinear iterations, function evaluations, linear iterations and norm. TS reason, step count, SNES iterations, step rejections, SNES failures, internal time/dt. Effective options/native/build/profile and numerical-family provenance are separate facts.

**Available/unexposed:** native SNES/KSP/TS monitors and reason getters; KSP iterations/reason, SNES linear failures, TS KSP totals; default event/stage logging of function/Jacobian, setup/solve and PC/KSP work. Examine 3.24.0 declarations and execution paths before promising any particular monitor's accepted-step or cumulative semantics. PC/Jacobian lag/persistence settings describe a policy, not proof that a factor/preconditioner was reused. Named logging can expose count/time/flops for enabled events; event overlap and backend-specific activity remain relevant.

**Exact zero/partiality traps:** PETSc 3.24.0 `PetscLogEventGetPerfInfo` returns zeroed statistics if default logging was not started. `SNESGetLinearSolveIterations` excludes a failed inner linear solve's iterations; its source directs callers to `KSPGetIterationNumber` after failure. Zero without activation evidence is uncollected, not no work. Current bridge contains no event-performance export found in the examined C API.

**Safe access/hang:** current bridge uses serialized native admission. Do not query a running SNES/TS from another thread. An owner monitor/checkpoint or exported immutable copy is the live route; an indivisible factor can remain opaque until return. On-demand logging activation during a run is not established and process/global-handler effects need review.

Pinned upstream: [3.24.0 logging implementation](https://raw.githubusercontent.com/petsc/petsc/v3.24.0/src/sys/logging/plog.c), [3.24.0 SNES implementation](https://raw.githubusercontent.com/petsc/petsc/v3.24.0/src/snes/interface/snes.c). Current release pages returned 3.26.0; their claims were checked against these pinned files rather than transferred.

## HiGHS

**Baseline:** Rust `highs =2.4.0`, `highs-sys =1.15.0`; use the latter's captured native headers/source, not Rust wrapper version as native version. Consumer `pse-backend-native/src/highs.rs`, `highs/diagnostics.rs`, `highs_lifecycle.rs`; skill corpus `highs-sys@1.15.0/HiGHS/highs/{interfaces/highs_c_api.h,lp_data/HighsInfo.h,lp_data/HighsCallback.h}`.

**Current live:** registered simplex/IPM/MIP interrupt/progress callbacks, running time and relevant iteration fields, MIP nodes/bounds/gap; incumbents and root cut-pool capture have distinct callback kinds and admitted copy costs. QP/PDLP lack the examined adapter's iteration interruption path; native time limit is retained. A field appearing in generic callback output is not proof that every solver emits it at useful cadence.

**Current completion:** simplex/IPM/PDLP/QP/crossover iterations, primal/dual solution status, basis validity, MIP nodes/bounds/gap, objective, integrality and feasibility/residual/complementarity measures; effective options and start provenance. Native info getters can omit unavailable fields. No per-factor/live phase-time collection was established in these adapter paths.

**Available/unexposed:** C API `Highs_getRunTime`, model rows/columns/nonzeros/Hessian nonzeros, presolved rows/columns/nonzeros and typed native info queries. Presolved-model availability and solver-stage validity must be checked. Native internal timers, factor reinversion or update statistics are source leads, not established stable C exports here.

**Causal questions:** did presolve shrink/expand work, crossover dominate, a basis/hot-start get used, or a QP re-solve start cold? Keep start submission/effective hot-start options separate from reuse outcome; existing tests exercise cold/hot iteration differences but were not run. No unsynchronized `Highs_get*` on a concurrently running instance is justified. Native callbacks supply already-computed progress without additional model evaluation.

Upstream: [callback contracts](https://github.com/ERGO-Code/HiGHS/blob/master/docs/src/callbacks.md); pinned headers qualify current-doc details.

## Clarabel

**Baseline:** 0.11.1, default features disabled, serde enabled; optional SDP and MKL Pardiso profiles. Workspace avoids `faer-sparse` because it would split the pinned faer family. Sources: skill corpus `clarabel@0.11.1/src/solver/{implementations/default/info.rs,core/solver.rs,core/kktsolvers/mod.rs}`, `src/timers/timers.rs`; consumer `pse-backend-native/src/conic.rs::{metrics,Session}`.

**Current live:** termination callback receives `DefaultInfo`: iteration, barrier/centering/step, primal/dual costs, residuals/infeasibility residuals, gaps/ktratio and current solve time. Adapter pushes `clarabel.iteration`, checks original cancellation and makes no extra residual evaluation. Linear solver name/thread/directness/nnzA/nnzL is included by `metrics`. The callback observes solver info, not a separately certified authored candidate.

**Current completion:** same info fields, solver status/candidate/certificate where admitted, effective settings and backend profile. Reused allocation/data is distinguished from external iterate warm start: adapter states there is no external iterate seed API.

**Available but extraction unresolved:** solver owns `pub timers: Option<Timers>` and times default start, IP iteration, cone scaling, KKT update, KKT solve and post-process. During solve it takes timers out of the field and restores them afterwards. `Timers` internals are private; examined public access is total time/print plus control methods, not a structured per-key getter. Thus nested timing is source-visible but not established as directly extractable structured live data. Do not report those names as currently exported metrics. No public numeric factor/reuse count was established from `DefaultInfo`/`LinearSolverInfo`.

**Cost/safety:** iterations/callbacks may be separated by an opaque KKT operation. Public mutable solver ownership does not permit concurrent reads while `solve(&mut self)` runs. Printing/timer parsing is not chosen as a remedy. Programmatic exact-source interface review is required first.

Upstream: [solver source](https://github.com/oxfordcontrol/Clarabel.rs/blob/v0.11.1/src/solver/core/solver.rs), [timers source](https://github.com/oxfordcontrol/Clarabel.rs/blob/v0.11.1/src/timers/timers.rs).

## POUNCE NLP, convex and QP; FERAL integration

**Baseline:** coherent 0.12.0 POUNCE family and 0.18.0 FERAL at fork `d02beafd370fd3815d6c0541fab1630ff839a739`; `qp`, `convex`, serde selected where declared. Skill release summaries are discovery, not authority for this changed fork. Exact local checkout: `/home/paul/.cargo/git/checkouts/pounce-8316ac8e070882e5/d02beaf/`; corresponding [pinned fork](https://github.com/paul-heyse/pounce/tree/d02beafd370fd3815d6c0541fab1630ff839a739). Consumer `pse-backend-native/src/{pounce.rs,tnlp.rs,pounce/}`.

**Current NLP live:** TNLP intermediate callback emits iteration, native objective/primal/dual residuals, barrier, step, regularization, primal/dual alpha, line-search trials and regular/restoration mode, with cooperative stop. No full live timing-statistics snapshot was established. Linear summary sink is synchronized and contains completed factor observations, but the examined adapter reads it at completion; safe publication to user/runtime live inspection is not established.

**Current completion:** serializes native SolveStatistics, retains bounded iteration trajectory and dropped count; separate timers for overall algorithm, initialization, Hessian update, barrier update, search direction, acceptable-trial computation, trial acceptance, convergence checks, intermediate callback, symbolic factorization, numeric factorization, backsolve, quality search, all callbacks/all function evaluations, objective/gradient/constraint/Jacobian/Hessian evaluations. Native statistics cover separate evaluation totals, restoration calls/inner/outer iterations/time, quality escalation, finite-difference Hessian pattern/nnz/columns/color groups/row width/fallback, partitioned elements/storage, dual-divergence/retry flags and SQP QP solves/working-set changes. Per-mode not-run sentinels and nonfinite values must retain their native meaning.

Current linear collector includes actual factor count, pattern reuse/changes, matrix/factor nnz, fill ratio, pivot extrema/inertia where available, structure refreshes, retained-pool compatibility/layout bytes/thread count. It explicitly labels lifetime extrema unavailable for this attempt. Separate summary totals, per-attempt differences and application-reset observations must not be merged.

**Current separate convex adapter:** the ordinary NLP adapter explicitly calls `app.set_convex_routing_available(false)`. The `PounceConvex` backend is a separate `pse-backend-native/src/execution/pounce_convex.rs` route. Its `report` publishes iteration count, objective, status, batch and `warm_started=false`, and emits `pounce_convex.solve` after obtaining a solution. This event is completion of the member solve, not a live iteration stream. `RequireReuse` is refused by this route. Native time limit and cooperative ownership remain distinct from live observation cadence. Do not transfer NLP timing statistics or its intermediate callback to the convex route.

**Convex/QP exact available:** `pounce-qp/src/problem.rs::QpStats` records working-set changes, refactors, Schur updates, phase-1 use, solve duration and parametric source. `pounce-convex/src/active_set_session.rs::SessionStats` distinguishes solves/parametric attempts, homotopy accepted, working-set accepted, engine-cold accepted and cold ladder. A successful parametric call need not mean homotopy reuse. `pounce-convex/src/presolve.rs::PresolveStats` supplies reduction evidence. These available native capabilities are not all projected by the separate convex adapter. A third actual consumer, `pse-backend-native/src/kkt/path/qp.rs`, uses `ParametricActiveSetSolver`; its owned linear `multi_solve` counts backsolve calls, refreshes summary at completed return, and explicitly leaves iteration admission unknown because QP/refinement loops do not expose it there. Review that sensitivity/continuation path separately from both primary backends.

**FERAL exact available:** `vendor/feral/src/numeric/solver.rs::{FactorStats,OrderingInfo,last_factor_stats}` records nnz, fill, inertia, pivot extrema, pattern reuse, scaling and ordering outcome, tiny perturbed pivots; ordering information includes supernodes/front sizes. Repository currently projects a subset through the POUNCE summary. Whether last-factor fields describe successful factor, refused factor or stale previous state must be checked at each status-return boundary.

**Causal questions/cost:** separate restoration from useful outer progress; numeric refactor from symbolic analysis and allocation reuse; accepted working-set reuse from attempted parametric reuse; coloring probe groups from actual evaluation totals; refinement/residual checks and failed factors from successful backsolves. All phase/callback totals are inclusive/overlapping unless source establishes otherwise. Snapshot locks/copies must not impede solver locks. No live access to mutable application internals outside owner checkpoints is established.

## UNO

**Baseline:** revision `805a27ecf802eec33d1aea85ae73efa8a24f71be` plus repository patch; `Cargo.toml` metadata, `docker/solvers/uno-pse.patch`, `pse-uno-sys/bridge/{pse_uno.h,pse_uno.cpp}`; consumer `pse-backend-native/src/uno.rs` and shared callback module.

**Current live:** objective/constraint/gradient/Jacobian callbacks and a poll callback. Poll checks terminal/cancellation/deadline without evaluating the oracle. The UNO ABI does not carry an iteration-record callback. The shared callback module's `ipopt.iteration` implementation serves the Ipopt bridge; its presence is not UNO progress coverage.

**Current completion:** optimization/iterate status, iterations, separate evaluation counts, subproblems, objective/primal/stationarity/complementarity, CPU seconds, dense-Hessian materialization count/columns/peak numeric bytes, HiGHS auxiliary feasibility calls/iterations/status and effective QP hot-start/tolerances. Availability flags guard auxiliary results. Handles are single-use; ABI requires exclusive access and copied input ownership.

**Unknown/instrumentation needed:** exact upstream iteration/subproblem callbacks, safe native timer/working-set/factor detail, current versus rejected trial and globalization/restart reasons require upstream/patch review. Materialization counters and auxiliary LP work are useful causes already available at terminal return, not live cumulative observations. Process-global logger permit acquisition can consume elapsed time before native work. A hung native subproblem has no promised asynchronous introspection; last poll/callback and external sampling are separate evidence.

Upstream baseline: [pinned UNO](https://github.com/cvanaret/Uno/tree/805a27ecf802eec33d1aea85ae73efa8a24f71be).

## SCIP

**Baseline:** SCIP 10.0.2 through raw `scip-sys =0.1.28`, supported linked image ABI checked at runtime; no bundled/from-source profile in this consumer. Exact source: skill corpus `scip@10.0.2/src/scip/{scip_solvingstats.h,scip_solvingstats.c,type_event.h}`; consumer `pse-backend-native/src/scip.rs::{Watch,ColumnMap,watch_init,watch_initsol,poll}`.

**Current live:** registered presolve-round, focused/solved node, LP-solved, best-solution and dual-bound events supply polling and selected progress. Incumbent observations capture native objective/bound/gap/nodes/solving time, optional admitted primal vector and owner identity. Bound events are separate. Concurrent SCIP copies have their own instance/thread and synchronized reporting; never borrow an arbitrary sibling's native handle. Stage checks guard native queries.

**Current completion:** status, total nodes, primal/dual bound/gap, event count, interrupt/callback failure, threads/nested TPI state, exact mode, reoptimization, pools/certificates/IIS where admitted. “Native incumbent” does not establish repository original-space qualification or publication acceptance.

**Available/unexposed:** exact statistics headers expose current-run versus all-run nodes, run count, LP counts/iterations, strong-branch LP iterations and structured statistics printing. Further solver/plugin timing and propagation/presolve/separation/NLP census requires selected exact APIs and stage conditions; source coverage is not a guarantee that every metric applies to every solve mode. Existing final collector does not establish broad census coverage.

**Cost/partiality:** copying full incumbent vectors and solution pools can dominate observation cost; keep copy allowance/cadence explicit. Event-triggered queries are owner-thread reads at allowed stages, not permission for external concurrent getters. A long indivisible plugin/LP/native operation can delay event polling. Current-run counters reset at native restarts; all-run totals differ. Live numerical gap trends, node growth and LP work are distinct explanations of apparent stall.

Upstream: [SCIP 10.0.2 source](https://github.com/scipopt/scip/tree/v1002/src/scip).

## Diffsol

**Baseline:** vendored `diffsol =0.16.2`, `diffsol-la =0.1.1`, faer feature and repository-owned linear adapters. Sources `vendor/diffsol/src/ode_solver/{mod.rs,bdf.rs,method.rs}`, consumer `pse-backend-native/src/dynamics/{integrator.rs,linear.rs}`; inspect vendored modifications rather than assuming crates.io behavior.

**Current live:** owner-controlled returns from `step` allow `simulation.step` events with returned time/step count, plus existing function callbacks/checkpoints. An internal rejected step/Newton cycle inside one `step` call does not necessarily produce such an event. Sampling/interpolation and root rewinds are separate from accepted native mesh steps.

**Current completion/segment:** serialized OdeSolverStatistics: steps, local-error failures, nonlinear iterations/convergence failures, linear setups and setup causes (checkpoint/reinitialization, first convergence failure, second convergence failure, error-test failure, successful-step heuristic). NativeStepObservation captures current method order/proposed step/internal mesh and returned time before rewind/reset. No last-accepted-step getter is exposed by this inspected route; do not invent one from StateRef.h.

**Instrumentation needed:** owner may copy cumulative statistics at safe step return without an extra scientific evaluation. Last-minute timing/counters inside an in-progress step require finer owner/native checkpoints; not established. Current faer/KLU wrappers expose separate symbolic setup/numeric factor/solve calls. KLU tracks full factors internally and tests check it, but this is not a stable published user counter. Linear setup totals can include symbolic/numeric work differently and are not synonymous with successful full factors.

**Failure/causality:** segment unwind catches record statistics before propagating abort where possible; reinitialization can reset native totals. Review checkpoint/adjoint replay/event reset accounting, sensitivity column costs and interpolation work. A returned root time can precede internal mesh time. Concurrent reading of a mutably borrowed Rust solver is not permissible.

## IDA/IDAS

**Baseline:** bundled SUNDIALS 7.1.1 through vendored sys 0.6.2; repository integration selects IDAS for residual DAE/sensitivity/adjoint routes. Sources `vendor/sundials-sys/vendor/include/{ida,idas}/{ida*.h,idas*.h}` and corresponding implementation; consumer `pse-backend-native/src/dynamics/idas.rs`.

**Current live:** residual/Jacobian/J*v/preconditioner/sensitivity-related callbacks are owned checkpoints. After forward IDASolve returns, `idas.output` reports returned time and cumulative steps; this is an output return boundary, not necessarily each internal accepted step. Forward/backward calls and root/event transitions differ. No arbitrary live external getters are authorized.

**Current segment/completion:** steps, residual evaluations, nonlinear iterations/failures, local-error failures, initialization backtracks, root evaluations, linear iterations, preconditioner evaluations and J*v; method/order/step/mesh/return observations. Last accepted-step fields are guarded by actual positive integration step count so IDACalcIC artificial step is not misreported. Backward statistics accumulate selected counters over each problem's reinitializations.

**Available/unexposed:** exact headers additionally declare `IDAGetNumJacEvals`, `IDAGetNumLinSolvSetups`, `IDAGetNumPrecSolves`, `IDAGetNumLinConvFails`, `IDAGetNumLinResEvals`; sensitivity/quadrature/backward outputs need their own eligible memory/phase. Current segment count list does not collect all these facts. Getters can fail if linear/sensitivity memory is uninitialized or inapplicable. Native counters exclude some user callback/evaluation work; keep adapter totals separate.

**Causal questions:** error-test rejection versus nonlinear convergence failure, repeated consistent initialization, root search, sensitivity/adjoint replay and preconditioner refresh. Segment reset and backward checkpoint boundaries need explicit deltas; total requested samples is not native step count. Safe between-return snapshots can show cumulative work but cannot reveal a hung in-call linear operation without an additional established checkpoint or external sample.

Upstream: [SUNDIALS 7.1.1](https://github.com/LLNL/sundials/tree/v7.1.1), [IDA usage](https://sundials.readthedocs.io/en/latest/ida/Usage/index.html); local pinned source governs.

## Sparse linear backends and math/property evaluation

| Owner/baseline | Exact capability/current use | Remaining distinction and safe observation route |
|---|---|---|
| FERAL 0.18.0 fork | FactorStats/OrderingInfo, completed-factor summary and current POUNCE projection described above | Thread-safe summary sink is a completed-operation observation, not safe access to an in-progress factor. Tiny pivots/ordering/scaling are available source leads beyond current subset. |
| faer 0.24.4 | Separate symbolic analysis, numeric factorization, triangular solve APIs and scratch requirements; current `dynamics/linear.rs` owns these calls. Installed `faer-0.24.4/src/sparse/linalg/{lu,qr,cholesky}.rs`; trait-level solve wrappers also exist. | No universal built-in time/counter stream established. Owner boundaries can distinguish phases; exact factor storage/fill and failure partiality differ by decomposition. Shape/nnz is structural, not measured dynamic work. |
| KLU / SuiteSparse through selected sys build | `dynamics/linear.rs::Klu::refactor` checks refactor return, rcond and conditional condest before fallback to full factor. Common/factor memory is owned and retained. KINSOL also reads owned KLU allocation between solves. | Reused symbolic/numeric storage, attempted refactor and accepted refactor differ. Conditioning diagnostics/refactor failure can themselves add work. No published per-reason live stream established. External reads of native common while mutating factor are unsafe. |
| MUMPS 5.9.1 | Exact per-job INFO/INFOG/RINFO/RINFOG fields, stage/activation, memory/fill/pivot/work meanings and documented in-factor progress are identified in the supplement. | Current Ipopt forwarding, synchronization and safe in-call access remain unqualified. Local/global and estimate/actual observations differ. Do not infer factor calls from flops or expose unactivated arrays as current user metrics. |
| SPRAL 2025.09.18 | Exact inform fields distinguish rank, fronts, delays, factor entries/flops and status; analysis/factor/solve semantics are identified in the supplement. | `num_factor` means entries, not factor calls. Some online documented fields differ from the examined C header. Runtime linkage and wrapper forwarding remain separate from source availability. |
| oneMKL 2026.1.0 Pardiso | Exact header and official iparm reference identify phase, memory, refinement, pivots, inertia and conditional work outputs; see supplement. | This oneMKL ABI has **no dparm argument**; do not transfer standalone Pardiso fields. Activation costs and wrapper access remain explicit. No broad per-attempt live projection is established. |
| Symbolica/Numerica 3.0.1 | Installed Symbolica evaluator OperationCount (add/multiply/invert/function calls), exported instructions, input/output shape. Repository compilation consumes these for allowance/storage/reset decisions in `pse-math/src/execution.rs::operation_count` and compile-stage logic. `pse-math/src/library.rs` owns bounded evaluator construction. | Skill pins 3.0.0; **version mismatch is explicit**. Live 3.0.1 source was inspected for named methods. Static operations/jet width/instruction bytes do not count actual calls, branch selections or provider work. Need existing compiler/runtime identities to attribute optimization/recompilation/reset and scientific evaluation; do not rerun evaluator just for inspection. |
| Numerica numerical integration 3.0.1 | Installed `numerica-3.0.1/src/numerical_integration.rs::StatisticsAccumulator` has iteration/sample/zero-evaluation counts, estimates/error/chi-square and extrema | No direct use of this numerical-integration accumulator found in bounded `crates` search. Do not conflate its statistical integration with Diffsol/IDAS process dynamics or Numerica jets. |
| FeOS/feos-core 0.10.1 | Workspace pin reason and frozen oracle consumer identify reference generation; FeOS `SolverOptions` has max_iter/tol/verbosity, and phase/density routines have textual iteration/result output | Not established as live production property evaluator. No general structured property-call/time/inner-iteration output found in examined SolverOptions/phase/density paths. Investigate authored property/provider/implicit/evaluator owners first; do not parse oracle-library logs as process runtime authority. |
| Authored properties and nested implicit solves | Symbolica/Numerica expression/jet stages, external providers and explicit inner solver callbacks are distinct current owners (`pse-math/src/execution.rs::evaluate_stages`, implicit modules, `pse-kernels` provider scope) | Cost must attribute actual provider/inner attempt with model/profile/branch/start/accuracy context, avoid nested double counting and identify non-reused/repeated work. No single library-wide counter explains all property contexts. State dimension, expression operation count, derivative width and actual call/time need separate meanings. |

## Bounded review controls to consider; none executed

1. **Zero versus uncollected:** PETSc log handler off/on; invalid/uninitialized KINSOL linear getter; fixed-point inapplicability; IDAS before consistent initialization/first step; POUNCE mode-not-run sentinels; HiGHS unavailable info. Verify explicit availability and scope rather than numeric defaulting.
2. **Live versus final:** long solve exceeding retained prefix, progress tap active/inactive, live last-minute changes versus terminal totals. Show what is known at each callback and during an intentionally long native factor/callback. No concurrent native getter thread.
3. **Failure work:** controlled trial refusal, failed inner linear solve, early cancellation, panic-contained callback and serious native error. Verify accepted candidates remain separate, failed work is not dropped, terminal-only counters remain partial and observation adds no scientific evaluations.
4. **Reuse causes:** same structure/value update/pattern update/settings update across fresh/retained sessions; reconcile analysis, factor/refactor, solve, working-set/homotopy outcomes and rebuild reason. Compare native counters to owner operations without calling a missing field zero.
5. **Dynamic causes:** local-error rejection, nonlinear failure, root/event reset and checkpoint/adjoint replay; reconcile setup cause counters and segment resets, accepted internal mesh versus returned/interpolated output and sensitivity dimensions.
6. **Nested properties:** one property/implicit-stage dominated outer solve; trace actual expression/provider/inner attempt and timing ownership, derivative-jet width and failed/repeated evaluation. Check observer workload itself does not add original assessment/property calls.
7. **On-demand profile feasibility:** establish owner-approved attach/checkpoint and runtime feature/privilege access for external sampling, measure observer overhead separately, retain exact time/attempt identity and mark unsampled gaps. Sampling may explain a stalled opaque phase, but is not accepted-candidate authority or a substitute for semantic work counts.

## Bounded negative claims and open work

Absence conclusions concern the named adapter collector/ABI and exact source structs inspected, not every internal library facility. No exhaustive native plugin census, host linkage inventory, subscriber/exporter audit or end-to-end Python/durable workflow transport audit was performed. Sparse-backend numerical statistics, UNO iteration access and some convex-mode forwarding remain source-led research questions. This catalog does not choose a new metrics framework, instrumentation strategy or optimization remedy; the review must first establish which user questions current safe observations answer and where they lose causal meaning.
