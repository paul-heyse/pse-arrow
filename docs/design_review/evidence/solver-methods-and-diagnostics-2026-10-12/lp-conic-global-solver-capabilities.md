> Supporting evidence for [the solver methods, selection and diagnostics review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, B4 (library-research): HiGHS, Clarabel, SCIP, IBEX, egobox, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Its judgments are the worker's; the principal review owns the verdicts.

# B4 review evidence: LP, MILP, QP, conic, global and auxiliary solver libraries

## Summary

- **HiPO is not built into HiGHS.** `highs-sys` 1.15.0 builds HiGHS with `HIPO` left at its default, OFF. HiGHS 1.15 can still load HiPO at run time, but only from a separate `libhighs_extras.so` whose version matches exactly. Without that library, `solver=ipm` quietly runs IPX, not HiPO, and `choose` always picks simplex for an LP. *Interface-checked*
- **HiGHS statistics.** The adapter records 26 of the 38 `HighsInfo` values. It never calls `Highs_getRunTime`. The callback has `mip_total_lp_iterations` and `pdlp_iteration_count` available but does not read them. *Interface-checked*
- **HiGHS basis reuse (most important finding).** Every solve calls `Highs_clearSolver` first (`BN/highs.rs:716-720`). Upstream says that when a model is changed through HiGHS's own methods, the stored basis is updated so the next solve can hot-start. Clearing first throws that away. A basis now survives between solves only if the caller sends the exported `WarmStart` back in. *Interface-checked*
- **SCIP statistics.** The final report has status, node count, primal and dual bounds, and gap. It has no final solving time, LP iterations, maximum depth, or presolve, separator, heuristic or nested-NLP statistics. SCIP 10.0.2 offers a structured statistics API (`SCIPcollect*Statistics` into a `SCIP_DATATREE`, and `SCIPprintStatisticsJson`), and none of it is bound. Emphasis settings are not exposed as a typed control. *Interface-checked*
- **Clarabel** captures every public `DefaultInfo` field. The only gap is the per-phase `timers` breakdown, which has no public per-phase getter in 0.11.1. Clarabel has no warm start. *Interface-checked*
- **IBEX** reports proof-cell and box-classification counts but no timing. **egobox** does keep its full library state on the worker and offers it as an explicit `SurrogateSnapshot`. The routine results carry a compact summary and do not include GP hyperparameters. *Interface-checked*
- **Newer releases.** HiGHS 1.15.1 is the latest; there is no 1.16. There is no Clarabel 0.12; 0.11.1 is the latest. SCIP 10.0.3 and 10.1.0 bring fixes and APIs that matter here. egobox 0.38.1 is the latest.
- **Global fallback after a local failure.** Using SCIP automatically after a local NLP fails is a different problem class, not a cheaper retry. Its cost can grow exponentially. It needs finite boxes and a factorable structure, and it buys a certificate the user did not ask for. It is sensible only when certification was requested or the class requires it, as the current routing already does.

Everything here comes from reading source, headers and documentation. Nothing was built or run.

---

## 1. HiGHS 1.15 (`highs` =2.4.0, `highs-sys` =1.15.0)

### 1.1 Methods and when each applies

**Build facts.** *Interface-checked*
- The `solver` option accepts `choose`, `simplex`, `ipm`, `ipx`, `hipo`, `pdlp`, `qpasm` and `hipdlp` (`HiGHS/highs/lp_data/HighsOptions.h:745-750`).
- **Dispatch in `HighsSolve.cpp`:**
  - `useIpm()` is true only for `ipm`, `hipo` or `ipx` (`:682`). So `choose` on an LP always means dual simplex (`:41-44`).
  - `useHipo()` (`:691-720`) returns `HighsExternalApi::isAvailable<hipo>()` for `ipm` or `hipo`. Otherwise, IPX runs.
  - `hipdlp` and `pdlp` (cuPDLP-C) are separate PDLP implementations (`:96-114`).
- **How HiPO is made available:**
  - `CMakeLists.txt:84` sets `BUILD_SHARED_EXTRAS_LIB` to ON by default, and `:97` sets `HIPO` to OFF.
  - `highs-sys/build.rs` never defines `HIPO`.
  - `HighsExternalApi.cpp:37-90` loads `libhighs_extras.so` with `dlopen` and rejects it unless `HighsExtras_getVersion` matches 1.15.x.
  - **Result:** HiPO is absent unless such a library sits on the loader path. The adapter's comment that "HiPO is not built" (`BN/settings/highs.rs:113`) is correct about the build. Nothing in the adapter checks or reports which IPM actually ran.

**Method profiles.** These are upstream practice, labelled *Proposed* as interpretation.
- **Dual simplex:**
  - best for small to medium sparse LPs and for any LP that has a valid or nearly valid basis.
  - the only method that hot-starts.
  - produces a vertex, which ranging and basis-inverse views need.
  - suffers from heavy degeneracy (stalling), which the dual algorithm and perturbation reduce.
- **IPX** (interior point with basis crossover): wins on large LPs, on highly degenerate LPs, and on LPs whose factor fill stays modest. It cannot hot-start. Crossover (`run_crossover`, default `on`) restores a basis; `crossover_iteration_count` is captured.
- **HiPO:** a newer interior point method for LP and, since 1.14, convex QP. It needs METIS and BLAS through the extras library.
- **PDLP:** a first-order method. Suitable for very large LPs, where a factorization is too expensive, and where low accuracy (1e-4 to 1e-6) is acceptable. Its accuracy and conditioning sensitivity are poor for the tight tolerances a simulator needs. It never polls the interrupt callback (`BN/execution/highs.rs:41-43`).
- **QP:**
  - QPASM is a convex-only active-set method, and 1.15 improved its non-convexity detection.
  - An MIQP is not supported.
  - The adapter refuses an explicit LP method on a QP (`BN/settings/highs.rs:104-119`). That matches the build, because QPASM is the only QP solver present.
- **MIP:**
  - branch-and-cut with presolve, symmetry detection, cuts, RINS, RENS, feasibility jump, zero-objective rounding (`zi_round`), shifting and root reduced-cost heuristics.
  - `mip_lp_solver` and `mip_ipm_solver` choose the method for subproblem LPs.
  - 1.15 is the first parallel MIP. Parallel simplex is disabled inside MIP (1.15 release notes, PR #3042).
  - semi-continuous and semi-integer domains are supported.

### 1.2 Method-choice controls

*Interface-checked.*
- **Typed controls the adapter sets** (`BN/highs.rs:620-695`):
  - `solver` from `HighsMethod` (`choose`, `simplex`, `ipm` or `pdlp`).
  - `parallel` (`on` or `off`, keyed to the thread count) and `threads`.
  - the four iteration limits, `mip_max_nodes`, tolerances, `qp_regularization_value`.
  - `qp_allow_hot_start=true`, and `simplex_scale_strategy=0` when native scaling is off.
- **Controls not modelled as types:** `presolve`, `simplex_strategy` (primal or dual), `run_crossover`, `ipx` versus `hipo`, `hipdlp`, `mip_lp_solver`, `mip_ipm_solver`, `mip_heuristic_effort`, `mip_detect_symmetry` and the PDLP restart and scaling options. They pass through only as free-form `controls.options` (`BN/highs.rs:620`).
- **Upstream guidance** (Context7, `docs/src/guide/further.md`):
  - "When a basis is known, the simplex solver is applied without presolve."
  - IPM after presolve can be better when the modifications are extensive.

### 1.3 Warm start and reuse

*Interface-checked.*
- **Bound:**
  - `Highs_setSolution` with primal and dual values (`BN/highs.rs:750-757`).
  - `Highs_setBasis` (`:773`).
  - `Highs_setSparseSolution`, the partial MIP start (`:796`).
  - in-place `changeColCost`, `changeColBounds`, `changeCoeff` and `changeRowBounds` (`:540-590`).
  - the QP hot start is turned on by default.
  - the basis is exported after simplex or QP (`:985-1000`).
- **`Highs_clearSolver` before every solve** (`:716-720`) clears the retained basis and solution, and with them HiGHS's internally updated hot-start state. The capability record still claims "native coefficient/bound updates with compatible layout" (`BN/execution/highs.rs:40`).
  - *Interpretation:* a basis survives only if the runtime round-trips `WarmStart`. HiGHS must then re-factorize from the submitted basis, where it could have continued with its retained factor.
  - Search coverage: `highs.rs` only. I did not trace whether every sequential runtime caller feeds `WarmStart` back.
- **Unbound:**
  - `Highs_setLogicalBasis`.
  - callback kind 9, `kHighsCallbackCallbackMipUserSolution`, which injects heuristic solutions during a MIP.
  - callback kind 8, lazy constraints.
  - `Highs_changeColIntegrality` and `addRows`/`deleteRows` for growing a model.

### 1.4 Statistics and diagnostics API, against what is bound

*Interface-checked.*
- **`HighsInfo` has 38 names** (`HighsInfo.h`). The adapter reads 26 (`BN/highs.rs:864-891`).
- **Not read (12):**
  - `primal_dual_integral`.
  - the three `*_semi_infeasibilit*` values.
  - the four relative primal and dual infeasibility values.
  - the four relative residual-error values.
- **C API functions not called:** `Highs_getRunTime`, `Highs_zeroAllClocks`, `Highs_writeModel`, `Highs_writePresolvedModel`, `Highs_writeSolution(Pretty)`, `Highs_getIisLp`, `Highs_crossover`.
  - Search: `grep` across `crates/**/*.rs` found no calls.
- **`getKappa`** (basis condition number) exists only in C++ (`Highs.h:751`). It is not in the C API.
- **Callback output struct** (`HighsCallbackStruct.h:24-48`):

  | Field | Status |
  |---|---|
  | `running_time` | Captured |
  | simplex and IPM iteration counts | Captured |
  | MIP primal and dual bound, gap, node count | Captured |
  | incumbent (kind 4) | Captured, with a throttled primal |
  | `objective_function_value` | Not read |
  | `mip_total_lp_iterations` | Not read |
  | `pdlp_iteration_count` | Not read |

- **Diagnostics run on copies** (`BN/highs/diagnostics.rs`):
  - rays (`:412-415`), ranging (`:493`), feasibility relaxation (`:615`).
  - fixed-commitment LP (`:851`), basis-inverse rows (`:947`), presolved LP and postsolve (`:991-1053`).
  - IIS (`:1139`) and the cut pool through callback kind 7.
- **Presolve log:** the C++ `HighsPresolveLog` (counts per reduction rule) is not reachable through the C API. The presolve view reports dimensions only.

### 1.5 Exploration claims checked

- **"About 26 info values"** is confirmed: exactly 26 (`BN/highs.rs:864-891`).
- **"Final runtime only in events"** is confirmed: `seconds` comes from callback `running_time` (`:1406`), and `Highs_getRunTime` is never called.
- **"No typed work counters"** is confirmed for the report: the iteration counts are string-keyed metrics. MIP LP iterations are missing entirely.

### 1.6 Newer releases

- **1.15.1** (2026-07-02) is the latest.
  - It fixes release binaries that lacked HiPO, an OpenBLAS performance setup problem, and C++20 builds.
  - String option values (except file names) are now lower-cased.
- **There is no 1.16.**
- **1.14** brought the HiPO convex QP and moved HiPO to infinity-norm equilibration.
- **1.15** brought the first parallel MIP, the QP hot start and IIS fixes.

### 1.7 Observation and interrupt hooks

*Interface-checked.*
- **Kinds started:** 1 to 6, plus 7 on request (`BN/highs.rs:1261-1268`). `user_interrupt` is written only for kinds 1, 2 and 6 (`:1458-1476`).
- **QPASM and PDLP** can stop only at their native time limit.
- **Status mapping:** every model status maps to a typed `Termination` (`termination()`, `:1480+`). A cancellation overrides the category, with assurance `None` (`:851-858`).
- **Contract wording mismatch:** §18.10 calls the improving-solution callback "kind 4" and the merely feasible one "kind 3". The code uses `kHighsCallbackMipImprovingSolution` (4) for incumbents, which agrees. It uses kinds 3, 5 and 6 for bound events. That is consistent, but bound events also fire on `MipLogging` (5).

---

## 2. Clarabel 0.11.1

**1. Methods.**
- An interior point method on a homogeneous self-dual embedding. Infeasibility is detected through τ/κ and Farkas or recession certificates; statuses include `Almost*`, `InsufficientProgress` and `CallbackTerminated`.
- **Cones:** zero, nonnegative, second-order, exponential, power, generalized power, and PSD triangle (the last behind `sdp`).
- Ruiz equilibration (`equilibrate_*`).
- Static and dynamic KKT regularization and iterative refinement.
- Presolve that drops infinite bounds, and chordal decomposition for SDP.
- Direct solver: QDLDL by default, MKL Pardiso optional, or `auto`.
- Suitable for LP, convex QP, SOCP, exponential, power and SDP cones at moderate accuracy, and for detecting infeasibility with a certificate. *Interface-checked*

**2. Controls.** The adapter mirrors every `DefaultSettings` field it admits (`BN/conic.rs:250-351`, `:470-504`). The iteration limit comes from the shared controls (`:518`). *Interface-checked*

**3. Warm start.** There is none.
- A grep for `warm_start|warmstart` in `clarabel-0.11.1/src` found nothing.
- What does exist is `update_data`/`update_P`/`update_q`/`update_A`/`update_b`, which reuse the symbolic factorization when the sparsity pattern is unchanged and presolve and chordal decomposition are off. The adapter's `ReusableData` mode uses it (`BN/conic.rs:632`).
- *Interface-checked*

**4 and 5. Statistics.**
- The adapter records every public `DefaultInfo` field, both per iteration through the termination callback (`BN/conic.rs:656-664`) and at the end (`:685-693`, `:832-870`): μ, σ, step length, costs, residuals, infeasibility residuals, gaps, κ/τ ratio, `solve_time`, iterations, and linear-solver name, threads, direct flag, `nnzA` and `nnzL`.
- `DefaultSolver::timers` (`solver/core/solver.rs:139`) has phase keys for setup, equilibration, kktinit, kkt update, kkt solve and presolve. Its only public methods are `total_time()` and `print()` (`timers/timers.rs`). There is no per-phase getter, so those timings can only be printed.
- **"Essentially complete" is confirmed.** The one gap is the timer breakdown, which is limited upstream. *Interface-checked*

**6. Newer releases.** None. The GitHub releases list shows v0.11.1 (2025-06-11) as the latest.

**7. Interrupt.** The termination callback returns `stopped()` and becomes `CallbackTerminated`, which maps to a typed category. *Interface-checked*

---

## 3. SCIP 10.0.2 (`scip-sys` =0.1.28, raw C API)

### 3.1 Methods

Background practice, *Proposed* as interpretation.
- **Spatial branch-and-bound** for factorable MINLP: outer approximation of nonlinear expressions through nonlinear handlers (quadratic, bilinear, convex/concave, perspective, SOC), bound tightening by FBBT and OBBT, and branching on continuous variables.
- **Nonconvex QP** goes through the same route.
- **Plugins:** presolvers (including PaPILO when built), separators (Gomory, MIR, flow cover, clique and others), primal heuristics (RENS, RINS, undercover, sub-NLP using the nested Ipopt) and propagators.
- **Exact rational MILP** (`SCIPenableExactSolving`) is linear only.
- **Concurrent solving** races several configurations (`parallel/mode` selects opportunistic or deterministic). Deterministic mode makes runs reproducible at some throughput cost.
- **Reoptimization** retains the tree across objective changes only.
- **IIS** works through sub-SCIP filters.

### 3.2 Controls

*Interface-checked.*
- **Emphasis values:** `SCIP_PARAMEMPHASIS_{DEFAULT, CPSOLVER, EASYCIP, FEASIBILITY, HARDLP, OPTIMALITY, COUNTER, PHASEFEAS, PHASEIMPROVE, PHASEPROOF, NUMERICS, BENCHMARK}` (`type_paramset.h:70-82`).
- **Meta-setting functions:** `SCIPsetEmphasis`, `SCIPsetPresolving`, `SCIPsetHeuristics` and `SCIPsetSeparating` (`scip_param.h`).
- **None is called** (grep over `crates/**/*.rs`).
- **What the adapter does set:** reserved keys only (`BN/scip.rs:1055-1180`): limits, `numerics/feastol`, seed, `lp/threads=1`, the nested Ipopt's linear solver, and IIS keys. With more than one thread it adds `parallel/mode=1`, which is deterministic. Other parameters pass as free-form options.
- **Consequence:** an emphasis, which is a bundle of parameter changes, cannot be requested as one value.

### 3.3 Warm start and reuse

*Interface-checked.*
- A submitted incumbent goes in through `SCIPcreateOrigSol`/`SCIPcreatePartialSol` followed by `SCIPaddSolFree`.
- `SCIPenableReoptimization` with `SCIPchgReoptObjective` is used for finite MIP sequences with an unchanged constraint system.
- The solution pool (`SCIPgetSols`) is reported as ranked solutions (`BN/scip.rs:3000-3025`). It is not fed back automatically as a start.

### 3.4 Statistics API against what is bound

*Interface-checked.*
- **Bound.** In the final report (`BN/scip.rs:2950-3170`): `SCIPgetStatus`, `SCIPgetPrimalbound`, `SCIPgetDualbound`, `SCIPgetGap`, `SCIPgetNTotalNodes`, the pool, IIS, the exact objective, and counts of events and interrupts.
- **`SCIPgetSolvingTime` appears only in incumbent events** (`:476`, `:547`), not in the final report.
- **Available in the 10.0.2 headers but unbound:**
  - **Scalar counters:**
    - `SCIPgetNLPIterations`, `SCIPgetNRootLPIterations`, `SCIPgetNNodeLPIterations`, `SCIPgetNStrongbranchLPIterations`
    - `SCIPgetMaxDepth`, `SCIPgetNNodes`, `SCIPgetNRuns`, `SCIPgetNReoptRuns`
    - `SCIPgetNSepaRounds`, `SCIPgetNCutsApplied`, `SCIPgetNConflictConssApplied`
    - `SCIPgetPrimalDualIntegral`, `SCIPgetDualboundRoot`, `SCIPgetFirstLPDualboundRoot`
    - `SCIPgetNSolsFound`, `SCIPgetNBestSolsFound`
    - `SCIPgetDeterministicTime`
  - **Timing:** `SCIPgetPresolvingTime` and `SCIPgetSolvingTime` (`scip_timing.h`).
  - **Structured statistics (new in 10):** `SCIPcollect{Status, Timing, OrigProblem, TransProblem, Presolver, Constraint, ConstraintTiming, Propagator, Conflict, Separator, Cutselector, Pricer, Branchrule, Heuristic, Compression, LP, NLP, Relaxator, Tree, Root, Solution, Concsolver, Benders, ExpressionHandler, NLPI}Statistics`. They fill a `SCIP_DATATREE`, read with `SCIPdatatreeGet{Bool, Long, Real, String, Tree, *Array}` (`pub_datatree.h`). `SCIPprintStatisticsJson` is also available (`scip_solvingstats.h:2695`).
  - **Nested Ipopt statistics:** available through `SCIPcollectNLPIStatistics` and `SCIPcollectNLPStatistics`.
  - **Model export:** `SCIPwriteOrigProblem` and `SCIPwriteTransProblem` are unbound.
- **Exploration claim confirmed:** there is no final solving time, no LP iterations, no maximum depth, and no cut, separator or presolve statistics.

### 3.5 Interrupt and observation

*Interface-checked.*
- **Events caught** (`BN/scip.rs:66-78`): presolve round, node focused, node solved, LP solved, best solution found, dual bound improved.
- **Cancellation:** `poll()` calls `SCIPinterruptSolve` (`:606-620`). `UserInterrupt` and `Terminate` map to `Cancelled` (`:302`). Every status maps to a typed category and assurance (`:296-322`). Gap limit maps to `Success` plus `GlobalBound`.

### 3.6 Newer releases

- **10.0.3** (2026-07-06):
  - IIS fixes: it now disregards the deletable flag, and it restores the infeasible original problem when interrupted. These may touch the IIS workarounds at `BN/scip.rs:1144-1157`.
  - `SCIPsolveConcurrent` now errors when reoptimization is enabled. The adapter already refuses that combination.
  - It creates a node-branched event when a node solve is interrupted.
- **10.1.0** (2026-09-18):
  - `SCIPgetSolvingPhase` and `SCIPgetSolvingPhaseFlags` (feasibility, improvement and proof phases). These directly support observation-driven switching inside a solve.
  - `SCIPiisGetSubscipCons`/`SCIPiisGetSubscipVar` and the IIS maps.
  - `concurrent/symmetrybefore`.
  - Interrupted nodes are no longer copied to a child, which simplifies continuing a solve.
  - `SCIPcalcBestCut`.

---

## 4. IBEX 2.9.1 (`BN/root_isolation.rs`, `native/root_isolation.cpp`)

**Build.** *Interface-checked*
- `Cargo.toml:466-484`: filib 3.0.2.2 for intervals, `LP_LIB=soplex` with an LP version of 4.0.2, and the SoPlex paths forced to `NOTFOUND`.
- *Interpretation:* IBEX builds its bundled SoPlex for the polytope hull. I did not verify this build path.

**1. Methods.** *Interface-checked*
- A `Covering : ibex::Solver` subclass owns bisection with a `CellStack` (`.cpp:218-330`).
- Contractors: `CtcHC4` (forward-backward), `Ctc3BCid` (shaving), `CtcNewton` (interval Newton), and `LinearizerXTaylor` with `CtcPolytopeHull` composed with HC4 inside a `CtcFixPoint`.
- A `LargestFirst` fallback bisector.
- `inflating_newton` for existence and uniqueness proofs (`:569`, `:716`).
- **Applicability:** a small square system (at most 128 inputs), factorable, with finite boxes, when an existence or uniqueness *certificate* is wanted, or an enclosure of all roots in a box.

**2. Controls.** Cell budget (`MAX_CELLS` = 2^20), a time budget passed in as `seconds`, and cancellation. *Interface-checked*

**3. Warm start.** None. There is a process-global lock and no retained session (`BN/root_isolation.rs:43-45`). *Interface-checked*

**4 and 5. Statistics.** *Interface-checked*
- `NativeResult` has `cells`, `solution`, `boundary`, `unknown` and `pending` (`BN/root_isolation.rs:197-205`).
- The chain result has `proof_cells`, `charts` and `connections` (`:188-194`).
- Point arithmetic reports evaluation counts (`:207+`).
- The `seconds` field is an input budget (`:1296`), not an output. No measured elapsed time is returned.
- IBEX's `Solver` time and per-contractor counts are not exported.
- **"No timing" is confirmed.**

**6. Newer releases.** Not researched; there is no skill coverage. The 2.9.x line is the current stable IBEX as far as I know. *Unverified*

**7. Interrupt.** A `cancelled` C callback and exceptions map to status codes 0 to 6, which become `RootPointEvidence::{Interrupted, Incomplete, ...}` (`.cpp:37`, `:641-647`). *Interface-checked*

---

## 5. egobox 0.38.1 (`RT/math/surrogate/library.rs`)

**Resolved versions** (`Cargo.lock`): egobox-ego 0.38.1, egobox-moe 0.35.5, egobox-gp 0.36.4, egobox-doe 0.35.1.

**1. Methods.**
- EGO with infill criteria such as log-EI; log-EI is the default.
- TREGO alternates trust-region local steps with global EGO steps.
- A mixture-of-experts GP clusters the inputs and fits local GPs.
- Mixed-integer variable types.
- **When warranted** (*Proposed*): an expensive black-box evaluation (seconds to hours each), roughly 20 or fewer dimensions, and no usable derivatives. It is not warranted for a cheap, differentiable algebraic model.

**2. Configuration used** (`library.rs:494-520`). *Interface-checked*
- An explicit initial design of experiments, Cobyla for infill optimization, and GP regression, correlation and θ tuning set from the options.
- `configure_trego` always activates `TregoStrategy` (`egor_config.rs:641-642`).
- Clustering is left at `NbClusters::fixed(1)` (`egobox-moe/src/parameters.rs:40-44`). The mixture-of-experts capability is therefore not used: there is one GP per output.

**3. Warm start.**
- Within a run, the worker keeps the full solver and `EgorState` (`:445-448`).
- 0.38 now updates the GP incrementally between iterations (release notes).
- The library's `warm_start` and `hot_start` from files are unused, because of `no_outdir()`.

**4 and 5. Statistics.** *Interface-checked*
- Each advance returns `SurrogateStateSummary` (initialized flag, iteration, termination status, disabled flag), `SurrogateWork` and a proposal. The proposal carries the infill statistic, the TREGO radius `sigma`, and the global and local iteration counts (`:640-662`).
- An explicit `SurrogateSnapshot` exposes the cloned `EgorState` (`:690-700`). That includes `surrogate.theta_inits` (the learned θ), DOE data, `counts` and `time`.
- **The exploration claim is partly refuted.** History arrays are left out of the routine `SurrogateAdvance`, but they are kept and can be requested through the snapshot. GP hyperparameters (θ, variance, likelihood) are not in the typed summary.

**6. Newer releases.** 0.38.1 (2026-10-07) is the latest. It fixed mixed-integer enum folding, constraint warm start and DOE sampling fairness.

**7. Interrupt.** The evaluator callback carries a scope and limit, and the library also gets `timeout(remaining)`. Panics become `SurrogateFailure::Panic`.

---

## 6. Decision guidance (*Proposed*; mathematics and solver practice)

### 6.1 Choosing simplex, IPM or PDLP for LP and QP sequences

**Signals that favour simplex:**
- A valid basis from a nearby problem: a tear MILP's LP relaxations, reconciliation or MPC QPs with changed right-hand sides or bounds, planning re-solves.
  - Few changes mean very few pivots.
  - HiGHS also skips presolve when a basis is present.
- A need for vertex duals, ranging or the basis inverse.
- Small to medium size, or a moderate number of rows.

**Signals that favour IPM (IPX now, HiPO when the extras library is present):**
- A cold start.
- Large size (on the order of 10^5 rows or more) with a factorization that stays sparse.
- Heavy degeneracy.
- A request for an analytic centre.
- The iteration count grows weakly with size. Crossover adds cost but returns a basis for later hot starts.

**Signals that favour PDLP:**
- Very large size, or factor fill that makes IPM memory-bound.
- Tolerance of about 1e-4 relative is acceptable.
- Avoid it for ill-conditioned process LPs and for tight feasibility budgets.

**MILP:**
- Tear selection is small and combinatorial, so default HiGHS branch-and-cut is right.
- A MIP start from the previous tear set (sparse is enough) usually helps more than parameter tuning.

**QP sequences** (MPC, reconciliation):
- With a fixed active set and small changes, an active-set method with a hot start (QPASM plus the basis) is fastest.
- With large changes, or when there are many inequalities and a cold start, an interior point is more robust. HiPO QP, Clarabel or POUNCE-convex fit there.

**When a warm basis pays:**
- the structure is unchanged.
- the changes are small relative to the problem.
- the sequence is solved in order on one session.

**What follows for the observation layer.** This is the a priori and in-solve signal set:
- A basis is available, so pick simplex.
- A cold start with large m·nnz, so pick IPM.
- PDLP only on request, or when memory admission fails.
- In-solve, a simplex iteration count far above about 3m, or long stalls, supports switching to IPM with crossover.
- **Prerequisites:**
  - The `clearSolver` behaviour (§1.3) must be fixed, or the runtime must always round-trip the basis.
  - The IPM actually used (IPX or HiPO) must be recorded.

### 6.2 When to use SCIP

- **Use SCIP for:**
  - MIQP and MINLP, which no other adapter represents.
  - native logical forms (indicator, SOS, and/or/xor, cardinality).
  - an explicit `certify` intent: a global bound, proven infeasibility, or an exact MILP.
  - IIS on the true MINLP.
  - deterministic multi-start through concurrent mode, when certification is wanted.

### 6.3 Is global branch-and-bound a sensible automatic fallback after a local NLP fails?

As a default, no.
- **Cost:**
  - Spatial branch-and-bound is exponential in the worst case, in the number of variables that need branching.
  - Its relaxations weaken with wide boxes and with nonconvex terms such as trilinear products, fractional powers and thermodynamic correlations.
  - Practical size limits are tens to low hundreds of nonconvex variables. Process flowsheets regularly exceed that by orders of magnitude.
- **Requirements:** finite variable boxes (`execution::admit_program` refuses otherwise) and a factorable structure. A local NLP has neither need, so failure at the local level says nothing about whether the problem fits SCIP.
- **Guarantees:** SCIP gives a certificate relative to `numerics/feastol` and the exported relaxation. A relaxed export (`Fidelity::Relaxed`) certifies a relaxation, not the model.
- **Why the local solve fails matters:**
  - Local failures are mostly caused by poor initialization, bad scaling, degeneracy or genuine local infeasibility.
  - The usual remedies are cheaper and more targeted: restoration, homotopy or continuation, rescaling, a different local method, or multi-start.
  - A global solver addresses only "is it infeasible everywhere?" and "where is the global optimum?".
- **Paid-solver practice:** BARON and Gurobi's nonconvex mode are invoked by the user's choice of problem class, never as hidden retries behind a local solver.
- **Where SCIP does belong** after a local failure:
  - as an explicit, budgeted diagnostic step to certify infeasibility on a small sub-block (for example a tear block or one unit model).
  - as a feasibility-emphasis solve to find a starting point for the local solver.
- **Fit with the current design.** The current routing makes SCIP automatic only by class and eligibility, never by failure (`numerical-execution.md` §18.7 "Selection"). That is consistent with this practice.
  - Search coverage for "no failure fallback": a grep for `fallback|escalat` near `scip|global|factorable` in `crates/pse-runtime/src` found nothing. That search is not exhaustive.

### 6.4 Where IBEX and egobox fit

- **IBEX:** post-solve certification of a local root, proving it exists and is unique in a box. Root enclosure for small square sub-blocks, such as an equation of state or phase-equilibrium inner solves. Proving that a box contains no root. It is not a primary solver for systems of flowsheet size.
- **egobox:** outer loops over expensive black boxes, such as flowsheet-level design or calibration where one evaluation is a full converged simulation, with few decision variables. Its proposals must be re-evaluated by the original model, which the adapter already does with "proposal for original correction". It is not a fallback for algebraic NLP.

---

## 7. Probes that would settle the remaining doubts (not run)

1. **Which IPM runs.**
   - Question: which IPM does `solver=ipm` run in the linked build?
   - Command: a small LP solved with `solver=ipm` and `output_flag=true`, then read the log for "IPX" or "HiPO"; or check `nm` on the binary for `HighsExtras_getApi`.
   - Expected: IPX, with no `libhighs_extras.so` found.
2. **Whether the basis survives in-place updates.**
   - Question: does a basis survive an `update()` followed by a re-solve when no `WarmStart` is passed?
   - Command: solve an LP, change one bound with `Session::update`, re-solve, and compare `simplex_iteration_count` with and without the exported `WarmStart` resubmitted.
   - Expected: without resubmission, the count roughly equals a cold solve.
3. **SCIP structured statistics.**
   - Question: does SCIP 10.0.2 structured statistics return LP iterations and maximum depth after a MINLP solve?
   - Command: call `SCIPcollectTreeStatistics` and `SCIPcollectLPStatistics` into a `SCIP_DATATREE` (or `SCIPprintStatisticsJson`) in the native SCIP test fixture.
   - Expected: populated keys.

## 8. Scope notes

- **Sources read:**
  - adapters: `BN/highs.rs`, `BN/highs/diagnostics.rs` (by grep), `BN/settings/highs.rs`, `BN/execution/highs.rs`, `BN/conic.rs`, `BN/scip.rs`, `BN/execution/scip.rs`, `BN/root_isolation.rs` and `.cpp`, `RT/math/surrogate/library.rs`.
  - contract: §18.7 and §18.10–18.10.1.
  - registry sources for `highs-sys` 1.15.0, `clarabel` 0.11.1, `scip-sys` 0.1.28 and egobox; SCIP 10.0.2 headers from the local solver cache.
  - GitHub release notes, and Context7 for HiGHS.
- **Not read in depth:** `BN/tears.rs` beyond its tests, `BN/certificate.rs`, `BN/execution/clarabel.rs`, `BN/execution/pounce_convex.rs`, `BN/highs_lifecycle.rs`.
- **Files written:** none.