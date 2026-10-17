> Supporting evidence for [the solver methods, selection and diagnostics review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Principal reviewer's analysis. Current-state facts cite the wave-1 evidence in this folder: [observation-and-persistence-map.md](observation-and-persistence-map.md) (A2) and the B1–B4 library returns. Targets are *Proposed*. Baseline: `ff5a4aa2f` plus the Plan 36 working tree, diff fingerprint `45ceea51b0698679`.

# Observation, diagnostics and measurement

## 1. Three consumers, one evidence stream

Solver observations serve three consumers. Their needs differ:

| Consumer | Needs | Latency | Granularity | Retention |
|---|---|---|---|---|
| **Layer-2 switching** (in-solve or between attempts) | Typed, semantically defined observations of *why* the trajectory is failing: stagnation, contraction, ill-conditioning, bound truncation, restoration, error-test failures | Inside the solve or at attempt end | Per iteration (bounded window) and per attempt | Attempt lifetime |
| **Engineer diagnosis** | Failure localized to model elements: rows, variables, units, providers; trajectory; replay | After the solve | Per attempt, plus an on-failure deep dive | Run lifetime, plus snapshots on request |
| **Measurement and the history layer** | Comparable work and time per method, per rung, per phase; outcome; structure key and features; environment | After the solve, aggregated | Per attempt or rung, aggregated across studies and runs | Long-lived; invalidated by library or build change |

The current design separates typed `evidence`, which decisions read, from untyped `metrics`, which "decisions read never". That separation guards against one real hazard: a decision depending on an ad hoc string key with no declared meaning, which would be a second authority with silent drift (DP-01). But it conflates two properties:

- **untyped**: a name/value pair with no declared semantics;
- **diagnostic-only**: an observation decisions must not use.

The rule's intent is preserved by "decisions read only *typed* observations with declared semantics". Under that rule a contraction estimate, a condition estimate or a stagnation flag can be typed evidence and can feed layer 2, while ad hoc metrics stay diagnostic. This is a rule impact on the `SolveReport` doc contract, §18.6.

## 2. Current capture versus availability (summary; matrix in §6)

- **Typed:** `WorkEvidence` holds four optional counters (evaluations, iterations, factorizations, proof steps). Iterations are set only by some adapters (A2 §1).
- **Untyped but captured:** callback calls and seconds per demand; Ipopt per-iteration events; POUNCE full `SolveStatistics` and 22 timing phases; KINSOL totals; IDAS per-segment counters (in memory only); HiGHS info; Clarabel info per iteration; SCIP bounds, gap and nodes.
- **Not captured:** listed per library in §6. Examples: Ipopt `Statistics()` and timing, Uno residuals (read then dropped), PETSc and KINSOL per-iteration histories, IDAS order, step and error weights, HiGHS and SCIP final runtimes, SCIP LP iterations, depth and plugin statistics, all sparse-factorization statistics (MUMPS INFOG, KLU Common, QDLDL, Pardiso iparm).
- **Time:** no attempt or phase wall time is stored. Only event `elapsed`, callback seconds and POUNCE timing exist.
- **Aggregation:** none across a study or across runs.
- **Tracing:** none in native code. Preparation spans have no production sink.

## 3. Target observation model (Proposed)

### 3.1 Per-attempt typed record (`AttemptObservation`)

- **Common, cross-adapter fields with declared semantics:**
  - `wall` and `cpu` time per phase: prepare, native, callback (per demand), linear algebra (where the library reports it), assessment;
  - `native_iterations` with an adapter-declared meaning (Newton step / IPM iteration / simplex pivot / BDF step / B&B node);
  - evaluation counts per demand; factorizations; rejected trials;
  - peak foreign allocation, where the library reports it;
  - termination category plus a typed **cause**.
- **Adapter statistics** as a typed sum type: `IpoptStatistics`, `KinsolStatistics`, `IdasStatistics`, `HighsInfo`, `ScipStatistics`, `ClarabelInfo`, `PounceStatistics`, `UnoStatistics`, `PetscStatistics`, `FactorStatistics{kind, nnz_factor, flops, inertia, null_pivots, condest, rgrowth, mempeak}`.
  - Adding a library statistic is then an adapter-local typed change plus generated schema.
  - It is not a free-form string, and it is not a cross-cutting edit.
- **Bounded per-iteration trajectory**, typed per family:
  - NLP: `k, f, inf_pr, inf_du, mu, ‖d‖, reg, α_pr, α_du, ls_trials, restoration`;
  - Newton: `k, ‖F‖, ‖Δx‖, λ (step length), backtracks, linear iterations, setup`;
  - fixed point: `k, ‖x−G(x)‖, ρ̂ₖ`;
  - integrator: `t, h, q, netf, ncfn, worst-error variable`;
  - MIP: `nodes, primal, dual, gap`.

  The existing `Progress` bound (`Controls.history`) applies.
- **Derived trajectory diagnoses**, computed once at attempt end by a pure function of the trajectory:
  - observed convergence order estimate: q from log‖Fₖ₊₁‖/log‖Fₖ‖, distinguishing quadratic, superlinear, linear and stalled;
  - contraction ρ̂;
  - stagnation window;
  - restoration share;
  - regularization share;
  - error-test failure clustering in time.

  These are typed evidence usable by layer 2 and the history layer. Linear convergence with constant contraction where an exact Jacobian is claimed is a classic signal of a wrong or inexact Jacobian, or of a singular root (PS-07). The remedy is a derivative check, not more globalization.
- **Method-specific monitors** come from the library that owns them, not from reconstruction:
  - Deuflhard's simplified-correction monitor Θ comes from PETSc's NLEQ-ERR line search, through `SNESMonitorSet` or a convergence history. KINSOL does not compute it.
  - Ipopt's δw (primal regularization, signalling nonconvexity) and δc (constraint regularization, signalling LICQ failure) are distinct signals.
  - Ipopt restoration slacks attribute infeasibility (Wächter & Biegler 2006 §3.3), but they are not exposed today.
- **A normalized error/tolerance per tested quantity**, the same quantity as §15.4's violation divided by tolerance.
  - It gives a unit-free signal comparable across methods.
  - Aspen Plus keys its convergence views and remedy suggestions on it (max Err/Tol, the arg-max variable, nesting depth; B5b).
  - Recycle and specification convergence should report it per tear.

### 3.2 Ladder and strategy record

- The existing `solve_strategy_events` gains per-rung wall time and the typed cause.
- The selection record gains:
  - the admissible set;
  - the ranked order, with the rule identifiers and feature values that produced it;
  - the history-snapshot identity, if advisory history reordered anything.

  This satisfies PS-09's "selection is recorded", including *why*.

### 3.3 Detail tiers (H26 proportionality)

These tiers must stay consistent with Plan 36b P-B6, which addresses efficiency-review F14:
- per-iteration telemetry is a bounded tap that does not interfere with the solve, with partiality and drop evidence;
- a guaranteed small channel carries phase, failure and terminal events;
- a diagnostic stream never cancels or back-pressures a scientific solve.

The trajectory diagnoses below are computed locally at attempt end from the bounded in-memory trajectory. They do not depend on the durable progress stream.

| Tier | Cost | When | Content |
|---|---|---|---|
| T0 always | Counters and timers already on hand; O(1) per iteration | Every attempt | Common fields, adapter statistics, trajectory diagnoses, ladder record |
| T1 bounded trajectory | O(iterations) memory, bounded by history | Default on; the bound is configurable | Per-iteration trajectory |
| T2 on failure | One extra analysis at the final or best iterate, bounded by explicit work and memory policy | Automatically, after a failed or limited attempt | See §4 |
| T3 on request | Expensive analyses: SVD over large systems, degeneracy MILP, derivative checks over all rows, replay export | Explicit request | Existing §15.5 analyses, plus additions |

### 3.4 Pipeline and phase measurement

Record timings per operation, keyed by the existing identities:

- source admission;
- preparation (compile, evaluator build, structural analysis, class proof);
- routing;
- per-rung native execution, split into callback and native-internal time;
- assessment;
- publication.

Also record preparation cache hits and rebuild reasons, which explain an edit → re-solve, and admission waiting time.

These are runtime telemetry, distinct from `case-measure`. That campaign qualifies controlled benchmarks; runtime telemetry explains individual runs and feeds history. The two share the identity vocabulary, not the storage.

### 3.5 Study and history analytics

- **Study level:** success rate per method and rung; iteration, work and time distributions; failure causes ranked; which rung succeeded per point; continuation and prediction effectiveness (predicted-start residual against cold start).
- **History store** (advisory layer 3), keyed by:
  - a **structure-only key**: incidence and structure hash, excluding values, bounds, settings and build. A2 shows no persisted structure-only key exists. `seed_preparation_identity` comes closest but includes layout and backend.
  - a **feature vector**, for similarity when exact keys miss;
  - the **method identity**: adapter, settings profile hash, mechanism;
  - the **environment identity**: native build, library versions, host class.
  - A library or build change invalidates or discounts records (concept drift).
  - Outcome measures are machine-independent work (`WorkEvidence`) plus wall time labelled with the environment.
- **Determinism:** the selection record names the history-snapshot identity, so a run can be reproduced with the same reordering (PS-11).
- **Censoring and exploration:** a method that is never tried has no record. A bounded exploration policy, for example an occasional alternative order inside the admissible set at no correctness risk, prevents an early ordering from locking in. It must never apply to explicit selections.
- **Lessons from the algorithm-selection literature** (B5b Part 3: Rice 1976; SATzilla; AutoFolio; ASlib; Kerschke et al. 2019; Tornede et al. 2022; Berthold & Hendel 2021):
  - **Two key tiers.** An exact structure key answers replay ("this flowsheet converged with X"). Cheap features place unseen problems near seen ones. Features must cost far less than a solve, and they never confer admissibility.
  - **Lexicographic scoring.** First verified original-space acceptance (§15.4), then the cost to acceptance as PAR-k over machine-independent work or wall time labelled with its environment.
  - **Failures and timeouts are right-censored observations.** Pick estimators that treat them that way, such as censored bandits or survival imputation.
  - **Exploration** is a bounded, *deterministic*, seeded permutation recorded in the run. It is safe because the layer-1 fallback chain still runs the rest. It never skips an option layer 1 would try.
  - **Cold start:** the layer-1 order plays the role of the single best solver. Reorder only with a minimum effective sample count and a confidence margin.
  - **Drift:** partition history by solver-build identity, and treat cross-version data as a decayed prior.
  - **Time-sliced schedules:** the ordered fallback with per-attempt budgets is already a time-sliced schedule (3S). History may tune slice budgets as well as order, with a floor so no admitted option is starved.
  - **Robustness labels:** a numerical-robustness label can be a better training target than raw time. Berthold & Hendel's learned MIP scaling choice is the Xpress default.
  - **Offline configuration** of per-class option profiles (SMAC/irace style) is a legitimate source of *authored* profiles, but it must be validated on held-out flowsheets.

## 4. Automatic failure diagnosis (T2)

After an attempt fails or stops at a limit, run a bounded diagnosis of the final or best iterate. All of it reuses existing machinery or library statistics:

1. **Residual localization.** Report the worst k scaled residual rows, mapped to model elements (units, streams, equations). §15.4 quality already computes per-row residuals; the step adds ranking and attribution.
2. **Bound activity.** List variables at or within ε of their bounds, mapped to elements. This is an IDAES `DiagnosticsToolbox` analogue.
3. **Scaling health.** Compare the row and column norm spread of J at the final iterate with the declared nominals, and list the worst rows and columns.
4. **Conditioning and near-singularity.**
   - Use the condition estimate from the factorization the solver already did, when exposed (KLU condest, MUMPS ICNTL(11)), or the existing `conditioning::jacobian_condition`.
   - For moderate sizes, use the existing dense-bounded SVD near-null modes (`MatrixPolicy`), mapped to elements.
   - For large sizes, use MUMPS null-pivot detection (ICNTL(24)) and KLU `singular_col` to name a dependent column.
5. **Convergence-rate diagnosis** from the trajectory (§3.1). Report an inexact-derivative suspicion, or contraction ρ̂ ≥ 1 for fixed points.
6. **Domain and evaluation errors.** Report where recoverable trial failures occurred (provider, envelope, domain), counted per equation and provider. The callback counts exist but are not attributed per source.
7. **Structural recheck** for the failed block, and DM parts if the failure is in initialization.
8. **IDAS.** Report the step-limiting variables from `IDAGetErrWeights` × `IDAGetEstLocalErrors` at the failure time.

The output is typed findings referencing model elements (PS-10), each with its evidence and the work it consumed. Diagnosis never changes the outcome classification.

**Convergence-effort regression.** Over the typed attempt observations, compare iterations, restoration and regularization shares, rungs used and time against a stored baseline per structure key and environment. This is the IDAES `IpoptConvergenceAnalysis` behaviour (B5b), but over typed records rather than parsed logs. It turns "the solver got slower after the upgrade" from an investigation into a query.

**Diagnostics catalogue gaps** against IDAES 2.13 (B5b Part 1; most checks already exist in the 25-rule `FindingKind` catalogue):
- a second severity tier for the Jacobian, variable and residual thresholds;
- a "constraint with no free variables" check;
- a sparse smallest-k SVD path beyond the dense bound;
- scaling-method profiling.

**Replay snapshot**, explicit or on failure by policy:

- full resolved `Controls` (A2 found only the hash is persisted);
- granted threads;
- host class;
- the start vector;
- values and bounds (already with the revision);
- the native model export where the library offers one (HiGHS `writeModel`, SCIP `SCIPwriteOrigProblem`; for NLPs, the prepared artifact identity suffices).

Retention and privacy: snapshots hold model values, so they follow the run's retention, not a global log.

## 5. Gaps against the profile

- **PS-08** requires that "Recycle convergence is a declared policy … with its iteration history recorded". The recycle route records KINSOL totals only, with no per-iteration history (Interface-checked: no progress events in `BN/kinsol.rs`, none in `BN/recycle.rs`). This is a behavioural-adequacy defect under PS-G3.
- **PS-07** requires that "conditioning diagnostics are model attributes". Conditioning is computed only on request (§15.5) or within the KKT analysis on sensitivity requests. A failed solve gives no conditioning diagnosis. This is partial.
- **§15 documentation drift (G7, narrow).** §15 says DiagnosticsToolbox parity "is not a current capability", but the catalogue and an `idaes-2.13` profile exist (B5b).
- **PS-10** requires that failures name model elements. Post-solve quality does so for feasibility. Trajectory failures (stall, restoration failure) carry native status and termination only, with no localization. This is partial.
- **PS-09** requires that the selection is recorded. Eligibility is recorded; preference reasons and the `complete_context` producer substitution are not. This is partial.

## 6. Captured-versus-available matrix

Category-level completeness, from the wave-1 evidence. **Captured** means it is read by the adapter and reaches `SolveReport` (typed or as a metric). **Persisted** means it reaches a stored relation (A2). Everything else is *Interface-checked* by the cited worker. "Δ" marks capability gained by a newer upstream release.

| Library | Per-iteration trajectory | End-of-solve statistics | Timing | Linear-algebra / factor statistics | Health signals (conditioning, inertia, step-limiting) | Stop and iterate retention | Where captured data goes |
|---|---|---|---|---|---|---|---|
| **Ipopt 3.14.20** (B1) | Captured: `intermediate_callback` fields, plus ‖x‖∞ and stationarity from `get_curr_violations`. Available but unread: `IpoptData::info_string()` flags, `info_regu_x`, `tiny_step_flag`. | Not read: `IpoptApplication::Statistics()` (iterations, evaluation counts, final scaled and unscaled infeasibilities). Only iterations from the last callback and the final μ. | Not read: `TimingStats()` per-phase timers, which need `timing_statistics=yes`; reachable through the existing C++ bridge, so the premise of R-48 is obsolete. Rust-side callback seconds are captured. | Not exposed: MUMPS INFOG, Pardiso iparm and SPRAL inform stay inside Ipopt; journal text only. | δw regularization per iteration is captured; inertia corrections only through `info_string` (unread) | The bridge throws `Terminal` instead of returning false, so the iterate and `SolveStatistics` are lost on a stop | Events and metrics → `runtime.solve_metrics` (untyped) |
| **POUNCE 0.12 fork** (B1) | Captured: `IterRecord` trajectory, truncated to the history bound | Captured: full `SolveStatistics` (typed `pounce_statistics` plus flattened metrics) | Captured: 22 phase timers | FERAL factor count, reuse, fill, pivot extrema, last inertia and nnz captured. `n_tiny`, `ordering_info`, `scaling_info` and refinement not read. Inertia history keeps only the last value; obtainable from the local `Lease`. | Quality escalations and restoration statistics captured | Cooperative (returns false on failure) | Typed field plus metrics; `work.iterations` unset |
| **Uno 805a27ec** (B1) | Available but unused: the termination callback carries residuals and multipliers every major iteration; the bridge polls only | Iterations, evaluation counts, subproblems, CPU time captured. `primal_feasibility`, `stationarity`, `complementarity` cross the boundary and are dropped. | CPU seconds only | n/a; HiGHS only in this build | — | The bridge throws (surfacing as ALGORITHMIC_ERROR), so the iterate is lost; cooperative return 0 unused | Metrics |
| **KINSOL 7.1.1** (B2) | None. SUNLogger info lines are compiled out (`LOGGING_LEVEL=2`); 7.1.1 has no info handler. For `KIN_FP`, an exact history is reconstructable from the residual callback at zero cost; for line search, it needs iteration-boundary marking. Δ7.3: `KINDampingFn`/`KINDepthFn` per-iteration hooks. Δ7.8: in-process logger callbacks. | 12 counters plus `FuncNorm` and `StepLength` captured | Callback seconds only | KLU: `memusage` only. `rcond`, `condest`, `rgrowth`, `numerical_rank`, `singular_col`, `mempeak`, `lnz`/`unz` and `nblocks`/`maxblock` are not read. | — | Callback ±1 returns are correct; cancellation is polled | Metrics; recycle history totals only (PS-08 gap) |
| **PETSc 3.24 SNES/TS** (B2, B3) | Received in `convergence()` and discarded. `SNESSetConvergenceHistory` and `SNESMonitorSet` are unbound. | Reason, iterations, function evaluations, linear iterations and final norm captured. Linear-solve and step failures are not read. | None (no `-log_view` stages) | `MatGetInfo` fill, `MatFactorGetErrorZeroPivot` and `KSPComputeExtremeSingularValues` unbound | Trust-region radius only through check hooks (unbound) | `SNES_USER_DIVERGED` through the convergence test | Metrics |
| **IDAS 7.1.1** (B3) | None; IDAS has no monitor function, so `IDA_ONE_STEP` polling is needed | Per segment: steps, residual evaluations, nni, ncfn, netf, IC backtracks, root evaluations, linear iterations, preconditioner evaluations, Jv captured. `IDAGetIntegratorStats` (orders, steps, h), `nsetups`, `njevals`, `nlcfails`, step-solve failures, sensitivity and checkpoint statistics not read. | None | KLU through the SUNLinSol handle: rcond and condest unread | Not read: `IDAGetErrWeights` × `IDAGetEstLocalErrors`, the step-limiting variables; no error handler, so `NativeTermination.message` is always None | Recoverable and terminal callback returns are correct | In-memory `Report.statistics` only; **not persisted** |
| **Diffsol 0.16.2** (B3) | `simulation.step` event (time, steps). `order()` and `state().h` are available free but unread. | Full `OdeSolverStatistics` per segment (steps, error-test failures, Newton iterations and failures, setups split by cause) | None | Local faer and KLU: rcond and condest computed as a refactor gate, not exported | — | Abort through unwind; no recoverable error channel | In memory only; not persisted |
| **HiGHS 1.15** (B4) | Callback events (running time, iteration counts, MIP bounds, gap, nodes). `mip_total_lp_iterations`, `pdlp_iteration_count` and `objective_function_value` not read. | 26 of 38 `HighsInfo` values captured | Final `Highs_getRunTime` never called; time only in events | Basis condition `getKappa` is C++ only and unbound; the presolve log is unreachable through the C API | IIS, rays, ranging and the basis inverse on request (§15.5) | Interrupt callback for simplex, IPM and MIP; QPASM and PDLP only at the time limit | Metrics plus the typed `CoefficientEvidence`; HiGHS diagnostics relations on request |
| **Clarabel 0.11.1** (B4) | Captured: full `DefaultInfo` every iteration | Captured | `solve_time`; the per-phase timer breakdown is print-only upstream | QDLDL statistics are private upstream | Infeasibility certificates verified (§18.10) | Callback termination is cooperative | Metrics plus typed `ConicEvidence` |
| **SCIP 10.0.2** (B4) | Incumbent and bound events | Status, nodes, bounds, gap captured. Not bound: LP iterations, maximum depth, runs, separation rounds, cuts, primal-dual integral, deterministic time, and the structured `SCIPcollect*Statistics` datatree or `SCIPprintStatisticsJson`. Δ10.1: `SCIPgetSolvingPhase`. | `SCIPgetSolvingTime` only in events; presolve time unbound | Nested Ipopt statistics through `SCIPcollectNLPIStatistics` (unbound) | — | `SCIPinterruptSolve` | Metrics plus typed `GlobalEvidence`/`GlobalRecord` |
| **IBEX 2.9.1** (B4) | — | Proof cells, charts, box classification captured | None (elapsed time not returned) | — | Proof evidence typed | Cancel callback | Typed proof evidence |
| **egobox 0.38.1** (B4) | Compact advance summary | Full `EgorState` through the explicit snapshot request | Inside the state, through the snapshot | — | GP hyperparameters not in the typed summary | Timeout and scope | Summary in strategy events |
| **Linear solvers, overall** (B2) | — | — | — | Only FERAL-under-POUNCE statistics reach reports. MUMPS, SPRAL, Pardiso and QDLDL are inaccessible behind their solvers. KLU and faer are directly owned, and their statistics could be read but are not. | Free condition estimates (KLU rcond after refactor; FERAL κ₁, min\|pivot\|, `n_tiny`) are computed and discarded | — | — |

**Pipeline-level gaps (A2):**
- no attempt, rung or phase wall time anywhere in storage;
- typed `WorkEvidence` limited to four counters, with iterations unset by several adapters;
- no study-level aggregation;
- no structure-only identity for history;
- resolved `Controls` persisted only as a hash;
- no host identity;
- no model export;
- no native tracing or exporter;
- on-request diagnostics only, with nothing triggered automatically on failure.
