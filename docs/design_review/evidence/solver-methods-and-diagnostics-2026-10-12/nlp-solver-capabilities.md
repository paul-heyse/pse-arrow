> Supporting evidence for [the solver methods, selection and diagnostics review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, B1 (library-research): NLP solvers — Ipopt, POUNCE, Uno, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Its judgments are the worker's; the principal review owns the verdicts.

# B1 — NLP solver library capabilities: Ipopt 3.14.20, POUNCE 0.12 fork, Uno 805a27ec

## Summary

1. **Ipopt timing is reachable today without parsing journals.** The local C++ bridge already compiles against Ipopt's installed C++ headers. `IpoptApplication::Statistics()` and `IpoptData::TimingStats()` are both public in those headers. Per-task timing needs one option, `timing_statistics=yes`. Register row R-48's trigger ("Ipopt exposes its timings through its C API") does not fit this architecture: the C API has no timing calls, but the code no longer depends on the C API. *Interface-checked.*
2. **Gap list, confirmed against the source:**
   - Ipopt: the bridge never calls `Statistics()` or `TimingStats()`.
   - Uno: `primal_feasibility`, `stationarity` and `complementarity` cross the C boundary and are dropped.
   - POUNCE: `work.iterations` is never set, and only the last inertia is kept.
   - **Refuted:** "Uno has no iteration hook". Uno has a per-major-iteration termination callback that carries the residuals and multipliers, plus a notify-acceptable-iterate callback. The bridge registers the first only to poll for cancellation and ignores its arguments.
3. **Stops are exceptions, not cooperative.** The Ipopt and Uno bridges stop a solve by throwing a C++ exception, never by the library's own stop return (`intermediate_callback` → false, Uno termination callback → 0). The exception path loses the final iterate, and for Ipopt also `SolveStatistics`. A stop decided by an in-solve switch could not hand its iterate to the next solver.
4. **No HSL anywhere; some methods are not built.** The Ipopt image has no HSL and no inexact algorithm. HSL also gates `nlp_scaling_method=equilibration-based` and `linear_system_scaling=mc19`. POUNCE can use MA57 through `pounce-hsl` (feature `ma57`); the adapter uses FERAL only. Uno is built with HiGHS only: it has no sparse symmetric-indefinite solver, so its `ipopt` and `auto` presets cannot run in this build.
5. **Newer upstream:**
   - Ipopt: 3.14.20 is the newest tagged release.
   - POUNCE: 0.12.0 is the newest tagged release; the fork's `[Unreleased]` changelog section is empty.
   - Uno: has moved past the pin. Upstream added fixed-variable handling in the interior-point method and Ipopt-style acceptable termination, and removed the `loose_*` options. The bridge sets `loose_*` (`pse_uno.cpp`, create function), so a bump would break it.

Baseline: HEAD `ff5a4aa2f` plus the Plan 36 working tree, read as-is. Nothing was built or run.

---

## 1. Ipopt 3.14.20 (image: MUMPS 5.9.1+METIS, SPRAL 2025.09.18, oneMKL Pardiso; no HSL)

**Sources:**
- Skill corpus `content/corpus/ipopt@3.14.20/src`.
- `docker/solvers/build.sh` `build_ipopt` (configure flags `--with-mumps --with-spral --enable-pardisomkl --without-hsl --disable-linear-solver-loader`).
- Context7 `/coin-or/ipopt` (3.14 docs).

### 1.1 Methods and the conditions under which each applies (*Interface-checked*)

- **Filter line-search primal-dual interior-point method** (Wächter–Biegler 2006). Includes second-order corrections (`max_soc`), a watchdog, and inertia-correcting regularization δw/δc (`IpPDPerturbationHandler`). The inertia correction requires an inertia-revealing factorization. MUMPS, SSIDS and Pardiso all report inertia. With MKL Pardiso, inertia can be unreliable under perturbed pivots, which matters for nonconvex problems.
- **Restoration phase.** An ℓ1 feasibility sub-IPM, entered automatically when the filter rejects steps. Options: `start_with_resto`, `expect_infeasible_problem`, `required_infeasibility_reduction`, and `resto.`-prefixed option overrides (IpAlgBuilder reads `"resto." + prefix`, e.g. `mu_oracle` at `IpAlgBuilder.cpp:933`).
- **Barrier strategies.**
  - `mu_strategy=monotone`: Fiacco–McCormick; `mu_init`, `mu_linear_decrease_factor`.
  - `mu_strategy=adaptive`: Nocedal–Wächter–Waltz, with `mu_oracle` ∈ {`quality-function`, `loqo`, `probing`}, `fixed_mu_oracle`, and `adaptive_mu_globalization` ∈ {`obj-constr-filter`, `kkt-error`, `never-monotone-mode`}.
  - `mehrotra_algorithm` (for LP/convex QP; forces adaptive/probing).
  - Registration is in `IpAlgBuilder.cpp:364,373,1054,1082`.
- **Inexact (iterative) algorithm: not built.** It requires `--enable-inexact-solver` *and* Pardiso from pardiso-project.org (`configure.ac:355-371`), and the image has neither.
- **Hessian:**
  - `hessian_approximation=exact` or `limited-memory` (L-BFGS/SR1 via `limited_memory_update_type` and `limited_memory_max_history`).
  - `hessian_approximation_space` (`nonlinear-variables`).
  - `hessian_constant`.
- **Initialization:**
  - `bound_push`/`bound_frac`, `slack_bound_*`;
  - `least_square_init_primal`, `least_square_init_duals`;
  - `bound_mult_init_method` (`constant`/`mu-based`);
  - `constr_mult_init_max` (least-squares multiplier estimate).
- **Warm start:**
  - `warm_start_init_point` plus `warm_start_bound_push/frac`, `warm_start_slack_bound_*`, `warm_start_mult_bound_push`, `warm_start_mult_init_max`;
  - `warm_start_target_mu`;
  - `warm_start_entire_iterate` (uses `TNLP::get_warm_start_iterate` for the full internal iterate, slacks included);
  - `warm_start_same_structure` (with `ReOptimizeTNLP`).
- **Termination:** `tol`, `dual_inf_tol`, `constr_viol_tol`, `compl_inf_tol`, `acceptable_*` with `acceptable_iter`, `max_iter`, `max_wall_time`, `max_cpu_time`, `diverging_iterates_tol`.
- **Scaling:**
  - `nlp_scaling_method` ∈ {`none`, `user-scaling`, `gradient-based`, `equilibration-based`}. Equilibration needs HSL MC19 (`IpAlgBuilder.cpp:330-336, 689-691`), so it is **unavailable** in this build.
  - `linear_system_scaling` ∈ {`none`, `mc19` (HSL), `slack-based`}; `linear_scaling_on_demand`.
- **Derivative checking:** `derivative_test` ∈ {`first-order`, `second-order`, `only-second-order`}, `derivative_test_tol`, `derivative_test_perturbation`. Also `jacobian_approximation=finite-difference-values`.

### 1.2 Controls for choosing a method

- Linear solver (§6).
- `mu_strategy` and `mu_oracle`.
- Hessian mode.
- Scaling.
- `fixed_variable_treatment`.
- `bound_relax_factor`.
- Warm-start pushes.

The SPRAL environment requires `OMP_CANCELLATION=TRUE` and `OMP_PROC_BIND=TRUE` (Ipopt install doc, via Context7). The local admission code enforces both (`BN/ipopt/settings.rs` `admit`).

### 1.3 Warm start and reuse (*Interface-checked*)

**Reachable and used:**
- `ReOptimizeTNLP` with `warm_start_same_structure=yes` (bridge `pse_ipopt_sequence.cpp:240,248`).
- `warm_start_init_point` with the full primal-dual seed (`pse_ipopt_sequence.cpp`, solve; `BN/ipopt/sequence.rs:340-395`).
- The six per-attempt restart numerics (`mu_init`, `warm_start_*`), reset between attempts (`pse_ipopt_sequence.cpp`, reset_restart).
- The final μ, taken from the last intermediate callback, is carried in `WarmPayload::Nlp.barrier`.

**Not reachable or not used:**
- `warm_start_entire_iterate` (the TNLP does not override `get_warm_start_iterate`).
- `warm_start_target_mu`.
- Active-set transfer: refused for Ipopt (`sequence.rs`, "active working set is not an Ipopt sequence start").

**What can pass between solvers:** primal x, bound duals zL/zU, constraint multipliers λ, and μ. An interior-point method has no meaningful active set to export, though one can be inferred from the complementarity products.

### 1.4 Statistics and diagnostics API

All of the following is in installed headers (`src/Makefile.am` header list includes `IpSolveStatistics.hpp`, `IpIpoptData.hpp`, `IpIpoptCalculatedQuantities.hpp`, `IpTimingStatistics.hpp`, `IpTimedTask.hpp`, `IpOrigIpoptNLP.hpp`). *Interface-checked.*

- **`IpoptApplication::Statistics()`** returns `SmartPtr<SolveStatistics>` (`IpIpoptApplication.hpp:221`). It provides:
  - `IterationCount`;
  - `TotalCpuTime`, `TotalSysTime`, `TotalWallclockTime`;
  - `NumberOfEvaluations(obj, constr, grad, jac, hess)`;
  - `Infeasibilities(dual, constr, varbounds, compl, kkt)` and `ScaledInfeasibilities(...)`, both five-argument forms since 3.14;
  - `FinalObjective`, `FinalScaledObjective`.

  It is **created only on a normal algorithm return other than `INVALID_NUMBER_DETECTED`** (`IpIpoptApplication.cpp:821-825`). On the exception paths (`NonIpopt_Exception_Thrown`, `Insufficient_Memory`, ...) it is NULL.
- **`IpoptData::TimingStats()`** (`IpIpoptData.hpp:599`). `TimingStatistics` exposes a `TimedTask` per phase: `OverallAlgorithm`, `InitializeIterates`, `UpdateHessian`, `UpdateBarrierParameter`, `ComputeSearchDirection`, `ComputeAcceptableTrialPoint`, `CheckConvergence`, `PDSystemSolverTotal`/`SolveOnce`, `ComputeResiduals`, `StdAugSystemSolverMultiSolve`, `LinearSystemScaling`, `LinearSystemSymbolicFactorization`, `LinearSystemFactorization`, `LinearSystemBackSolve`, `QualityFunctionSearch`, `TryCorrector`.
  - Per-evaluation timers: `f_eval_time`, `grad_f_eval_time`, `c_eval_time`, `jac_c_eval_time`, `d_eval_time`, `jac_d_eval_time`, `h_eval_time`.
  - Aggregates: `TotalFunctionEvaluation{Cpu,Sys,Wallclock}Time`.
  - Each `TimedTask` has `TotalCpuTime`/`TotalSysTime`/`TotalWallclockTime` (`IpTimingStatistics.hpp:67-250`, `IpTimedTask.hpp:118-175`).
  - **Enabling it:** the phase timers are disabled unless `timing_statistics=yes` (`IpIpoptAlg.cpp:126,144-147`). `print_timing_statistics=yes` turns them on *and* prints them (`IpIpoptApplication.cpp:629-633`). The timers are reset at the start of every optimize call, ReOptimize included (`IpIpoptApplication.cpp:620`).
  - **Access:** through `IpoptData*` in `intermediate_callback` and `finalize_solution`, or through `application->IpoptDataObject()` after return.
- **Per-iteration (`intermediate_callback`):**
  - algorithm mode (regular or restoration), iteration, objective, inf_pr, inf_du, μ, ‖d‖, regularization, α_du, α_pr, line-search trials;
  - `IpoptData*` and `IpoptCalculatedQuantities*`;
  - `get_curr_iterate` and `get_curr_violations` (3.14).
  - `IpoptData` also gives `info_string()` (single-character flags such as inertia correction or quality escalation), `info_regu_x`, `info_alpha_primal_char`, `info_ls_count`, `tiny_step_flag`, `curr_mu`, `tol` (`IpIpoptData.hpp:347-535`).
- **NLP evaluation counts:** `OrigIpoptNLP::f_evals()` … `h_evals()` (`IpOrigIpoptNLP.hpp:322-346`).
- **Linear-solver internals are not public.** The MUMPS, SPRAL and Pardiso interface headers are not installed (only `IpSymLinearSolver.hpp` and `IpTSymLinearSolver.hpp`). MUMPS INFO(9)/INFO(10) and errors are written to the journal only, at detailed or linear-algebra levels (`IpMumpsSolverInterface.cpp:458-551`). INFOG, Pardiso iparm outputs and SPRAL inform are not exposed. The only structured trace of inertia correction is δw (`regularization`) and the `info_string` flags.

### 1.5 Bound versus unbound in the local adapter

- **Confirmed:** no `Statistics()`, `TimingStats()`, `IpoptDataObject` or `timing_statistics` use anywhere in `crates/pse-ipopt-sys` or `crates/pse-backend-native/src`.
  - *Search coverage:* grep for `Statistics|TimingStats|IpoptDataObject|timing_statistics|print_timing` over both trees, excluding the generated `bindings.rs`. The only hits are POUNCE and Diffsol.
- **What the adapter does record:**
  - Iterations come from the last intermediate callback (`bridge :113-130`; `sequence.rs:577-579`).
  - Per-iteration events `ipopt.iteration`: mode, objective, inf_pr, inf_du, μ, ‖d‖, regularization, α, line-search trials, ‖x‖∞, and the stationarity norm from `get_curr_violations` (`BN/uno/callback.rs:324-405`, shared dispatch).
  - Final μ (`sequence.rs:620-624`).
  - Callback counts and wall time come from the Rust `CallbackState`, not from Ipopt.
- **Mismatch.** The capability string `diagnostics: "native current iterate, violations, callback counts and timing"` (`BN/execution/ipopt.rs:39`) claims "timing", but no Ipopt-native timing is read. *Interpretation:* "timing" here is the event elapsed time.
- **Untyped controls.** `mu_oracle`, `start_with_resto`, `least_square_init_*`, `max_soc`, `linear_system_scaling`, `derivative_test` and `warm_start_target_mu` are reachable only as raw `controls.options`. Raw options are rejected only for the reserved lists (`sequence.rs:252-290`; `ipopt/settings.rs:RESERVED`). The `MuStrategy` vocabulary has only `monotone` and `adaptive` (`pse-model` generated enums).
- **Fixed by the adapter:** `nlp_scaling_method` is either `gradient-based` or `none` (`sequence.rs:316-326`).

### 1.6 Newer upstream

3.14.20 (2026-08-27) is the newest tag in the GitHub releases listing as of 2026-10-12. Its changes include:
- a SPRAL use-after-free fix on structurally singular re-solves;
- `max_filter_resets` now being enforced;
- `WallclockTime()` now measured since the Epoch.

Nothing newer was found. Unreleased master was not inspected.

### 1.7 Observation, interrupt and outcome mapping

**What Ipopt provides:**
- `intermediate_callback` returning false gives `USER_REQUESTED_STOP` → `User_Requested_Stop`. `finalize_solution` still runs with the current iterate, and statistics are still created (`IpIpoptAlg.cpp:461,536`; `IpIpoptApplication.cpp:803-825`).
- An `eval_*` returning false at a trial point makes Ipopt cut the step, which is the recoverable route.
- A thrown non-Ipopt exception gives `NonIpopt_Exception_Thrown` with no statistics (`IpIpoptApplication.cpp:872-878`; `rethrow_nonipoptexception` defaults to false).

**What the bridge does:**
- Evaluation codes: 0 = ok, 1 = recoverable (returns false to Ipopt), 2 = terminal (throws `Terminal`).
- Poll codes 1–4 throw `Terminal` and latch CANCEL, DEADLINE, ABANDON or TERMINAL (`bridge :52-71`).
- `intermediate_callback` **always returns true** and stops only by throwing (`:113-130`).
- Rust maps CANCEL→`Cancelled`, DEADLINE→`TimeLimit`, ABANDON→`Limit`, MEMORY→`ResourceExhausted`, else `Evaluation` (`sequence.rs:497-512`).
- The candidate is vetoed under the terminal latch.

**Interpretation:** library hooks are enough to *observe* each iteration and to *stop cooperatively*. But Ipopt reads its options only at `Initialize`, so in-solve switching *within* Ipopt is limited to stopping and resolving with new options: `ReOptimizeTNLP` for the same structure, or a fresh handle for structural options. Changing μ or regularization through `IpoptData` setters from the callback would mean reproducing Ipopt's own μ and regularization control logic. *Proposed:* add a cooperative-stop code that returns false from `intermediate_callback`, so the iterate and statistics survive a switch decision.

---

## 2. POUNCE (fork `ff638694`, 0.12.0; FERAL 0.18.0)

**Sources:**
- Fork checkout `~/.cargo/git/checkouts/pounce-8316ac8e070882e5/ff63869` (upstream repository `jkitchin/pounce`).
- The skill route `nlp.pounce`; its corpus is upstream 0.12.0, not the fork.

### 2.1 Methods (*Interface-checked*)

- **Algorithm selection** (`upstream_options.rs:317-330`):
  - `algorithm` ∈ {`interior-point` (the Ipopt-lineage filter IPM), `active-set-sqp` (via `pounce-qp`)}.
  - The ℓ1 exact penalty-barrier method (`pounce-l1penalty`).
  - `solver_selection` routes LP, QP, QCQP and SOCP to the convex arm (`pounce-convex`, with crossover).
- **Barrier strategies:** `mu/monotone.rs`, `mu/adaptive.rs`, oracles `quality_function`, `loqo`, `probing`; `mu_strategy_fallback` (a barrier-schedule retry, on by default since 0.11).
- **Hessian modes** (`hess/`): exact, limited-memory quasi-Newton, partitioned quasi-Newton (BFGS/SR1, per-constraint or block elements), finite-difference Hessian (CPR/star coloring, declared or Jacobian pattern, reuse tolerance).
- **KKT solvers** (`kkt/`): standard augmented system, Schur augmented system, low-rank augmented system (SMW for L-BFGS), slack scaling.
- **Robustness:**
  - restoration (`pounce-restoration`);
  - the second-opinion ladder (`second_opinion.rs`), up to three or four alternate trajectories, promoted only if one converges;
  - the dual-divergence retry;
  - FERAL quality escalation (`feral_increase_quality`);
  - start conditioning and jitter (`start_conditioner::jitter_start`).
- **Unimplemented options are refused rather than ignored.** This covers the inexact-algorithm family, `least_square_init_duals`, `dependency_detector`, `gradient_approximation`/`jacobian_approximation`, `hsllib`, the corrector knobs and others (`unimplemented_options.rs:184-348`).
- **Linear solvers:**
  - `linear_solver` registers every Ipopt name, but only `feral` and `ma57` are implemented. MA57 requires the `ma57` feature and libcoinhsl (`application.rs:5630-5647`; `crates/pounce-hsl`). Every other name is refused up front.
  - FERAL orderings include `auto`, AMD, METIS, SCOTCH and KaHIP. Its scalings include MC64-symmetric and inf-norm. It also offers static pivoting and iterative refinement (`pounce-feral/src/lib.rs:201-585`).
- **Batch:** independent NLPs on a rayon pool, outer-parallel and inner-serial (`batch.rs`).
- **Sensitivity:** `pounce-sens-core` provides `step_along_path`, `path_direction`, fix-relax and the Schur stack (`boundcheck.rs:1275,1974`). The convex arm adds a reduced Hessian.
- **Presolve:** `pounce-presolve`, including FBBT, linear-equality elimination and auxiliary elimination (owned by the `symbolica-faer-oximo` reference).

### 2.2 Controls for choosing a method

- `algorithm`, `hessian_approximation`, `mu_strategy`/`mu_oracle`, `nlp_scaling_method` (includes `curvature-based`).
- FERAL ordering, scaling and parallelism.
- `presolve`.
- **Upstream guidance (0.12.0 release notes):** `feral_ordering=metis` is 2–3.3× faster on collocation, optimal-control and PDE-in-time models (GasLib-40: 56k–112k variables), and slower on grid QPs and simply structured models. A limited-memory Hessian makes `mu_strategy` default to adaptive.

### 2.3 Warm start, reuse and transfer

**Library hooks:**
- `set_sqp_warm_start(SqpIterates)` and `last_sqp_working_set()` carry the active set between SQP solves (`application.rs:1833,1937`).
- `set_warm_start_iterate(IterateSnapshot)` (`application.rs:1863`).
- `warm_start_diagnostics()`.
- `TnlpPresolveSession` keeps the presolve transform across warm re-solves (`pounce-rs/src/session.rs:377-510`).
- `final_mu`, intended for threading into `mu_init`/`warm_start_target_mu` in predictor–corrector path following.
- FERAL symbolic pattern reuse (`FactorStats.pattern_reused`).

**Used locally:**
- the SQP warm start and working set (`pounce.rs:616,901`);
- IPM primal-dual via `warm_start_init_point` and typed restart (`pounce.rs:414,504-520`);
- second-opinion rung policy (`pounce/profiles.rs`);
- path and sensitivity (`BN/kkt/path.rs`, `kkt/sensitivity.rs`).

**Not used:** batch, `TnlpPresolveSession` and `set_warm_start_iterate`. *Search coverage:* grep for those symbol names across `crates/`.

### 2.4 Statistics and diagnostics

- **`SolveStatistics`** (`pounce-nlp/src/solve_statistics.rs:51-271`) contains:
  - Ipopt-equivalent totals (iterations, CPU/sys/wall time, evaluation counts, final scaled and unscaled infeasibilities and KKT error);
  - `final_declared_constr_viol`/`box_viol`, `final_kkt_error_above_noise`, `final_mu`;
  - restoration calls, inner iterations and wall time;
  - `quality_escalations`;
  - dual-divergence flags;
  - SQP QP solves and working-set changes;
  - partitioned and finite-difference Hessian census fields;
  - `iterations: Vec<IterRecord>` (a full per-iteration trajectory including `alpha_primal_char`).
- **`timing_stats()`** gives the Ipopt-style `TimingStatistics` (`application.rs:1171`).
- **FERAL `FactorStats` per factorization:** nnz_a, nnz_l, fill, inertia, min/max |pivot|, pattern_reused, scaling and ordering info (`vendor/feral/src/numeric/solver.rs:102-128`). The `pounce-feral` summary folds these and keeps **only `last_inertia`** (`pounce-feral/src/lib.rs:995-1023`). Each factorization is also emitted on a tracing span (pounce#71).

### 2.5 Bound versus unbound locally

- **Bound:**
  - the whole `SolveStatistics`, serialized into metrics and kept as `pounce_statistics`, with iterations truncated to `controls.history` (`pounce.rs:763-781`);
  - 22 phase timers (`pounce.rs:782-806`);
  - FERAL factor count, pattern reuse and changes, fill and pivot extrema, last inertia, last nnz (`pounce.rs:808-843`);
  - warm and crossover records;
  - per-iteration `pounce.iteration` events (`tnlp.rs:556-566`).
- **Confirmed: `report.evidence.work.iterations` is never set for POUNCE.** The only setters are in uno, petsc, conic, ipopt and a test (`grep 'work.iterations\s*='`). The count does exist as `statistics.iteration_count`.
- **Confirmed: inertia history keeps only the last value.** This is a library limitation (the summary type), and the local `Lease::record` copies only `after.last_inertia` (`BN/pounce/retained.rs:159-176`). *Interpretation:* the local `Lease` wrapper sees every factor call (before and after the summary), and the tracing span carries each factorization. So a per-factorization inertia history is obtainable without a library change. *Proposed.*

### 2.6 Newer upstream

0.12.0 (2026-09-19) is the newest release. The fork's `[Unreleased]` changelog section is empty. Known issues in 0.12.0:
- `feral_ordering=scotch` regresses on `deb7`;
- two MA57 batched-backsolve tests fail against CoinHSL.

### 2.7 Hooks and stops

- `intermediate_callback` returning false → `UserRequestedStop`, which keeps the result (`ipopt_alg.rs:1640,2156`).
- Locally, it returns false only when `state.evaluate` fails (`tnlp.rs:556-566`).
- Because POUNCE is Rust, a controller could also read `IpoptData` and `IpoptCq` in the callback (the adapter ignores both).

---

## 3. Uno (commit `805a27ec`, 2026-10-03; reports version 2.9.0)

**Sources:**
- Pinned archive fetched to scratch, sha256 `55618cdb…` matches the `Cargo.toml` pin.
- Cached install header `~/.cache/pse-arrow/native/uno/.../Uno_C_API.h`.
- `nm` on the cached `libuno.a`, most recent generation, dated 2026-10-10.
- GitHub releases.

### 3.1 Methods (*Interface-checked*)

Uno is built from "ingredients" (`uno/ingredients/`):
- **Constraint relaxation:** feasibility restoration or none.
- **Inequality handling:** inequality-constrained (SQP/SLP) or interior point.
- **Hessian models:** exact, L-BFGS (direct and inverse), identity, zero.
- **Globalization mechanisms:** trust region or backtracking line search.
- **Globalization strategies:** Fletcher filter, Wächter filter, funnel, merit.
- **Inertia correction:** none, primal, primal-dual.
- **Subproblem solvers:** HiGHS (LP/QP), BQPD, BoxLP, EQP/Woodbury, and symmetric-indefinite solvers MA27, MA57, MA86 (HSL), MUMPS and SSIDS.

**Presets** (`uno/options/Presets.cpp`):
- `filtersqp`: trust region + Fletcher filter + exact Hessian.
- `funnelsqp`.
- `filterslp`: zero Hessian.
- `ipopt`: interior point + Wächter filter line search + primal-dual inertia correction.
- `auto` (since 2.8): picks `ipopt` when n+m ≥ 2000 or Jacobian+Hessian nnz ≥ 50 000, otherwise `filtersqp`. The thresholds are marked "TODO: calibrate".

**This build:** `libuno.a` contains `HiGHSSolver`, `LBFGSHessian`, `InteriorPointMethod` and `PrimalDualInertiaCorrection` symbols, but **no** MUMPS, SSIDS, BQPD or MA* solver classes. *Search:* `nm -C | grep` for those class names. CMake detection is on `find_package(MUMPS)` and `find_library(SPRAL/BQPD/HSL)` (`CMakeLists.txt:137-292`); the pipeline passes only HiGHS and BLAS (`scripts/native_pipeline_cache.py:324-345`).

**The bridge allows only two configurations** (`pse_uno.cpp:207-221`):
- `filtersqp` with `hessian_model=LBFGS`, `inertia_correction_strategy=none` and HiGHS QP;
- `filterslp` with a zero Hessian.

No Hessian callback is bound, even though the C API has `uno_set_lagrangian_hessian` and a Hessian-operator callback.

### 3.2 Controls for choosing a method

- Preset or ingredient options.
- `hessian_model`.
- `QP_solver`/`LP_solver`.
- `linear_solver` (for the interior point method).
- `TR_radius`.
- `use_function_scaling`.
- Tolerances.

Upstream guidance is the `auto` rule above: dense active-set QP "stops scaling" at roughly 2000 rows plus columns.

### 3.3 Warm start

- The C API has `uno_set_initial_primal_iterate` and `uno_set_initial_dual_iterate` (header :286-291). The bridge sets primal only (`pse_uno.cpp:242`), and the declared capability is `WarmCapability::Primal`.
- Re-solves are not reused: the bridge refuses a second solve on a handle (`:236`).
- Within one solve, HiGHS QP hot start between subproblems is driven by `WarmstartInformation` (`HiGHSQuadraticProgram.cpp:38-47`). The patch's `qp_hot_start` flag is recorded.

### 3.4 Statistics and diagnostics

- **Result fields:** optimization status, solution status, objective, primal feasibility, stationarity, complementarity, primal/dual/constraint vectors, iterations, evaluation counts (objective, constraints, gradient, Jacobian, Hessian), subproblem count, `Timers{wallclock}` only (`Result.hpp:16-40`, `Statistics.hpp:15-18`).
- **Accessors:** `uno_get_solution_{primal_feasibility,stationarity,complementarity}`, `uno_get_cpu_time`, `uno_get_number_*`.
- **Per-iteration data:** comes only through the callbacks (§3.7) or the logger stream, which is text.

### 3.5 Bound versus unbound locally

- **Confirmed:** `pse_uno_result.primal_feasibility`, `stationarity` and `complementarity` are declared (`crates/pse-uno-sys/src/lib.rs:114-117`) but never read. `BN/uno.rs:412-470` records iterate_status, iterations, evaluation counts, subproblems, cpu_seconds, materialization, HiGHS feasibility counters and the hot-start flag. *Search:* grep for those field names in `uno.rs` and `uno/`. The only hits are `accuracy.*`.
- **Refuted as a library claim, confirmed for the adapter:** "no iteration hook". The termination callback runs every major iteration (`Uno.cpp:220-236`) and carries primal, bound and constraint multipliers, the objective multiplier, and the three residuals. The bridge registers it (`pse_uno.cpp:223`) but uses it only to poll and return 1 (`:131-135`). `notify_acceptable_iterate` is passed as `nullptr`.

### 3.6 Newer upstream (after 2026-10-03)

- #972: fixed variables in `PrimalDualInteriorPointProblem`.
- #974: Ipopt-style tight and acceptable termination; **removes the `loose_*` options** that the bridge sets.
- #928: BQPD workspace fix.

v2.9.0 (2026-09-08) brought second-order corrections, least-squares multipliers, a restoration filter, function scaling, and a `DIVERGING_ITERATE` status. The pin is a post-2.9.0 commit, so it includes these.

### 3.7 Hooks and stops

- The C termination callback returns **0 to request termination** → `USER_TERMINATION`, and Uno post-processes and returns the current iterate (`Uno_C_API.cpp:351-359`; `Uno.cpp:232-235`).
- Uno catches `std::exception` in its major loop as `ALGORITHMIC_ERROR` (`Uno.cpp:129-137`). The bridge's `Terminal` derives from `std::exception`, so a bridge stop surfaces as an algorithmic error that the bridge then overrides through its latch. That loses the cooperative status and the iterate.

---

## 4. Cross-library hook summary (*Interface-checked*)

| | Per-iteration observation | Cooperative stop | Iterate after stop | Native statistics read locally |
|---|---|---|---|---|
| Ipopt | yes: callback, `IpoptData`/`IpoptCq` | `intermediate_callback`→false (unused; bridge throws) | yes if cooperative; lost on throw | none (iterations and μ from the callback only) |
| POUNCE | yes: `IterStats` plus Rust data | →false (used on failure) | yes | complete statistics and timing; inertia last-only |
| Uno | termination callback with residuals and multipliers (args ignored) | return 0 (unused; bridge throws) | yes if cooperative | counts and CPU time; residuals dropped |

**Interpretation:** all three expose enough per-iteration state (residuals, μ or trust region, regularization, step lengths, multipliers) to drive observation-based switching *outside* the library. None allows changing algorithmic options in the middle of a solve. So the clean pattern is: stop cooperatively, keep the iterate, then re-solve (same-structure ReOptimize, POUNCE warm or working set, a different solver). This needs no copying of library controllers. *Proposed.*

---

## 5. Probes that would settle remaining doubts (not run)

1. **Ipopt timing through the bridge.**
   - *Question:* does `IpoptData::TimingStats()` return non-zero per-phase times after `timing_statistics=yes`?
   - *Method:* a C++ probe linked against `/opt/pse-solvers` that runs HS071 with `SetStringValue("timing_statistics","yes")`, then reads `app->IpoptDataObject()->TimingStats().LinearSystemFactorization().TotalWallclockTime()` and `app->Statistics()->IterationCount()`. Command: `scripts/pse-env --native -- g++ -std=c++17 probe.cpp $(pkg-config --cflags --libs ipopt) && ./a.out`.
   - *Expected:* factorization, back-solve and evaluation times are each greater than 0 and sum to at most `OverallAlgorithm`.
2. **Statistics after a stop.**
   - *Question:* is `Statistics()` NULL after a stop?
   - *Method:* the same probe, throwing from `intermediate_callback` in one run and returning false in another.
   - *Expected:* NULL with `NonIpopt_Exception_Thrown`; non-NULL with `User_Requested_Stop`, and `finalize_solution` called.
3. **Uno termination-callback cadence.**
   - *Method:* count termination-callback invocations against `uno_get_number_iterations` on a small NLP (`scripts/pse-env --native=uno -- …`).
   - *Expected:* the two counts are equal or differ by one.
4. **Uno build contents.**
   - *Method:* confirm the active cache generation matches the build the bridge links, by comparing the `PSE_UNO_BUILD_ID` source against the cache key.
   - *Status:* my `nm` result is from the most recent generation only.

---

## 6. Decision guidance (upstream documentation and literature; *Proposed* as selection inputs)

**Problem size, density and structure → which solver.**
- *Small or medium, modest inequality count, active set expected to be stable (n+m ≲ 2·10³, nnz ≲ 5·10⁴):* active-set SQP (Uno filtersqp/funnelsqp, POUNCE active-set-sqp). Its warm starts transfer the working set and typically need few iterations on re-solves.
- *Large and sparse, or many inequalities with combinatorial active-set change:* interior point (Ipopt, POUNCE IPM, Uno ipopt preset). This is Uno's own `auto` rule; its thresholds are not yet calibrated.
- *Square systems (zero degrees of freedom):* an NLP solver is the wrong class. Use a Newton root solver; interior-point methods with n = m degenerate to feasibility.
- *Few degrees of freedom (n − m small):* reduced-space and SQP methods are comparatively cheap. The reduced Hessian is small enough for quasi-Newton approximations.

**Linear solver for the KKT system** (Ipopt documentation; Tasseff et al. 2019 linear-solver parallelism study):
- MA27 for small problems.
- MA57 is the usual single-thread best for medium to large problems.
- MA86 and MA97 (OpenMP; MA97 bit-reproducible) and Pardiso for large problems with dense supernodes, where threads pay off.
- MA77 is out-of-core.
- MUMPS is robust but usually slower than MA57.
- SPRAL SSIDS is roughly in the MA97 class.
- On collocation and optimal-control structure, nested-dissection ordering (METIS) gives large gains: POUNCE 0.12 reports 2–3.3× with `feral_ordering=metis` on GasLib-40. It is a loss on grid QPs.
- HSL is admissible under the maintainer rule. It would also unlock MC19 equilibration scaling in Ipopt.

**Nonconvexity:**
- Needs inertia-revealing factorization plus inertia correction: Ipopt or POUNCE IPM, or Uno with `primal_dual` inertia correction. Uno's trust-region SQP with an exact Hessian handles indefinite QPs only if the QP solver accepts nonconvex QPs; HiGHS QP needs a convex Hessian, which is why the bridge pins L-BFGS.
- Local-infeasibility verdicts on nonconvex problems are trajectory artifacts often enough that POUNCE runs a second-opinion ladder.

**Hessian mode:**
- Exact when available: interior-point convergence and iteration counts depend strongly on it.
- L-BFGS when second derivatives are unavailable or costly, accepting more iterations. POUNCE notes that limited-memory mode makes adaptive μ the better default.
- Partitioned quasi-Newton when the Lagrangian is element-separable.
- Finite-difference Hessian when the coloring compresses well (groups ≪ n).

**Barrier strategy:**
- Adaptive μ (quality-function oracle) usually needs fewer iterations from a cold start.
- Monotone μ is the documented choice for warm starts (small `mu_init`, `warm_start_bound_push`/`mult_bound_push` around 10⁻⁶–10⁻⁹) and is more predictable.
- Probing (Mehrotra) suits LP and convex QP.

**Re-solve sequences:**
- Interior-point warm starts help only when the seed is well-centered: pushes and μ must be reduced together, or the method re-centers and loses most of the benefit (Gondzio and others). Same-structure re-solve (`ReOptimizeTNLP`, FERAL pattern reuse) saves the symbolic analysis regardless.
- Active-set SQP warm starts are strong when few active-set changes are expected between neighboring problems: parametric sweeps, continuation, NMPC.
- Sensitivity steps (`step_along_path`) provide a first-order predictor across parameter changes when the active set is stable.

**Observable cues for in-solve switching, all available through the hooks in §4:**
- repeated restoration entry or long restoration time (restoration calls and wall time; Ipopt mode = restoration);
- large or growing δw (regularization), meaning strong nonconvexity or rank deficiency;
- `quality_escalations` or `q` flags, meaning a numerically difficult KKT matrix;
- α_pr collapsing with inf_pr flat, meaning a stall that suggests a feasibility-first or SQP alternative;
- the dual-divergence signature;
- SQP working-set churn (`sqp_qp_working_set_changes`), meaning the interior point method would likely be the better choice.

---

## Files

No repository or skill file was written.

Scratch, outside the repository: `/tmp/claude-1000/-home-paul-pse-arrow/5335360e-031d-4d0d-ac32-d1df2e1106de/scratchpad/B1/uno.tgz` and the extracted `src/` (the pinned Uno archive, sha-verified).

Key paths:
- `crates/pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`
- `crates/pse-uno-sys/bridge/pse_uno.cpp`
- `crates/pse-uno-sys/src/lib.rs`
- `crates/pse-backend-native/src/ipopt/sequence.rs`
- `crates/pse-backend-native/src/uno.rs`
- `crates/pse-backend-native/src/uno/callback.rs`
- `crates/pse-backend-native/src/pounce.rs`
- `crates/pse-backend-native/src/pounce/retained.rs`
- `crates/pse-backend-native/src/tnlp.rs`
- `crates/pse-backend-native/src/execution/ipopt.rs`
- `docker/solvers/build.sh`
- `docs/adr/register.md` (R-34, R-48)