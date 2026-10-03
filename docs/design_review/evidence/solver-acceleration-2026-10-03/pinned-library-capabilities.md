# Pinned solver libraries: acceleration and globalization capabilities vs pse use

Baseline: HEAD `f0b90258` plus the uncommitted working tree on 2026-10-03. This matrix covers capabilities and contracts only. It makes no performance claims.

## Sources and pins

| Library | Pin and build | Evidence source |
|---|---|---|
| Ipopt | 3.14.20 C API through `pse-ipopt-sys`. The solver image (`docker/solvers/build.sh`, `.github/setup/solver-images.json`, ADR-0108) is configured with `--with-mumps --with-spral --enable-pardisomkl --with-asl --without-hsl --disable-linear-solver-loader`. It links MUMPS 5.9.1 with METIS 5.1.0, SPRAL 2025.09.18 and oneMKL 2026.1.0 Pardiso. **No `--enable-sipopt`**, so sIPOPT/`ipopt_sens` is not built. | skill corpus `ipopt@3.14.20` |
| POUNCE | `pounce-rs` =0.12.0 with features `qp, convex` (**not `sensitivity`**). `pounce-nlp`, `pounce-common`, `pounce-feral`, `pounce-sens-core` and `pounce-presolve` are each =0.12.0. | corpus `pounce-*@0.12.0` |
| FERAL | =0.18.0 | corpus `feral@0.18.0` |
| SUNDIALS | `sundials-sys` =0.6.2 bundles SUNDIALS 7.1.1 (`build_libraries`, `static_libraries`, `kinsol`, `klu`). The `idas` feature is added by `pse-backend-native/idas`. ARKODE and CVODES are vendored but not enabled. | corpus `sundials-sys@0.6.2/vendor` |
| Diffsol | `diffsol` =0.16.2 (faer), `diffsol-la` =0.1.1. `diffsol-nl` 0.1.1 is transitive only. | corpus |
| HiGHS | `highs` =2.4.0, `highs-sys` =1.15.0, which bundles HiGHS 1.15.0 | corpus `HiGHS/Version.txt` |
| Clarabel | =0.11.1 (`serde`; optional `pardiso-mkl`, `sdp`) | corpus |
| SCIP | 10.0.2 through `scip-sys` =0.1.28. The image builds it with `IPOPT=ON PAPILO=ON SOPLEX TPI=tny THREADSAFE EXACTSOLVE`. | build.sh |
| faer | =0.24.4 (`std`, `sparse-linalg`) | symbolica-faer-oximo skill index |

Status vocabulary:

- **used**: pse sets or calls it in production code.
- **exposed-typed-unused**: a typed setting exists, but no policy or pipeline drives it beyond defaults.
- **raw-reachable**: not reserved, so a caller can set it through the untyped `Controls.options` passthrough that every adapter admits (`solve::reject_reserved`). It sits outside typed policy, identity semantics and automatic selection.
- **not exposed**: reserved, refused, never called, or the feature or build is not compiled.
- **not provided**: the pinned library does not offer it.

Integration owner = the pse adapter file under `crates/pse-backend-native/src/`.

---

## 1. Ipopt 3.14.20 (C API) — owner `ipopt.rs`, `ipopt/settings.rs`, `settings/ipopt.rs`, `solve.rs::WarmRestart`

| Family | Capability | Status | Notes and caveats |
|---|---|---|---|
| 1 Warm start | `warm_start_init_point` with primal, bound and row multipliers | **used** | Set only for a complete primal-dual seed. A partial dual seed is refused (`ipopt.rs`). |
| 1 | `mu_init`, `warm_start_bound_push/frac`, `warm_start_slack_bound_push/frac`, `warm_start_mult_bound_push` | **used** | Typed `WarmRestart`, reserved from raw options (`RESTART_OPTIONS`). `mu_init` is applied only under the monotone `mu_strategy`. |
| 1 | `warm_start_entire_iterate` (restart from Ipopt's internal iterate) | not exposed in practice | C++ `IpoptApplication` only. The C `IpoptSolve` always calls `OptimizeTNLP`, never `ReOptimizeTNLP` (`IpStdCInterface.cpp:273`). |
| 1 / 2 | `warm_start_same_structure` (keep the linear-solver symbolic analysis) | raw-reachable but **unusable via the C API** | `TSymLinearSolver` asserts `have_structure_` (`IpTSymLinearSolver.cpp:124`). The C API builds a fresh algorithm on each `IpoptSolve`, so setting it would raise INVALID_WARMSTART. It is not reserved in pse, so a raw setting fails loudly. Consider reserving it. |
| 1 | `warm_start_target_mu` | raw-reachable | |
| 2 Sensitivity | sIPOPT (`ipopt_sens`, `sens_*`, KKT factor retention) | **not provided by the build** | No `--enable-sipopt`, and sIPOPT is C++ and AMPL only, not part of the C API. pse uses the POUNCE `pounce-sens-core` route instead (see §2). |
| 2 | Retaining the KKT factor across C-API solves | not provided | Factors are internal to the algorithm object. |
| 3 Inexact Newton | `inexact_algorithm` (Curtis–Schenk–Wächter, built by `InexactAlgorithmBuilder`) | **not usable** | Needs Pardiso from pardiso-project with its iterative mode. The `pardisolib` loader is disabled and `pardisomkl` lacks it. Treat as unavailable. |
| 4 Globalization | Filter line search, SOC (`max_soc`, `kappa_soc`), watchdog, inertia correction (`perturb_*`, `first_hessian_perturbation`), restoration phase, `soft_resto_pderror_reduction_factor`, `max_resto_iter`, `required_infeasibility_reduction` | **used at defaults**, raw-reachable | None of these are typed. `restoration` is observed in the intermediate callback metrics. |
| 4 | `start_with_resto`, `expect_infeasible_problem` (+`_ctol`, `_ytol`) | raw-reachable | Natural levers for a "bad start" preparation stage. No typed or policy route exists. |
| 4 | `line_search_method` (`filter`/`cg-penalty`/`penalty`), `accept_every_trial_step`, `alpha_for_y`, `recalc_y`, `bound_mult_init_method`, `least_square_init_primal/duals`, `constr_mult_init_max` | raw-reachable | `least_square_init_*` gives library-owned least-squares initialization of primal and duals, relevant to target A. |
| 4 | `mu_strategy` monotone/adaptive, `mu_oracle` (probing/loqo/quality-function), `adaptive_mu_globalization`, `corrector_type`, `mehrotra_algorithm` | `mu_strategy` **exposed-typed** (monotone default); the rest raw-reachable | `mu_init` is not applied under adaptive. |
| 4 | `jacobian_regularization_value`, `dependency_detector` (`mumps` only in this build) | raw-reachable | |
| 5 Continuation | Built-in homotopy or continuation | **not provided** | |
| 7 Structure | MUMPS (ICNTL Schur, null space, sparse RHS), SPRAL, Pardiso Schur (`iparm[35]`) | not exposed | Reached only through Ipopt's internal linear-solver interfaces. The C API gives no direct MUMPS or Pardiso handle. Ordering, scaling and matching are **exposed-typed** (`settings::ipopt::Linear`). `mumps_permuting_scaling`, `mumps_scaling`, `mumps_dep_tol`, `mumps_mem_percent` and `pardisomkl_redo_symbolic_fact_only_if_inertia_wrong` are raw-reachable. Within one solve Ipopt reuses the symbolic analysis automatically. |
| 7 | Parallel factorization | **used** (typed) | SPRAL on OpenMP, Pardiso on MKL threads, under admission rules. MUMPS is sequential. |
| 8 Evaluation reduction | `hessian_approximation=limited-memory` (L-BFGS / SR1) | **used** (automatic when no Hessian is supplied) | `hessian_approximation` is reserved. `limited_memory_update_type`, `limited_memory_max_history`, `limited_memory_initialization` and `hessian_approximation_space` are raw-reachable. |
| 8 | `grad_f_constant`, `jac_c_constant`, `jac_d_constant`, `hessian_constant` | **used** | Derived from oracle derivative facts. |
| 8 | Finite-difference `jacobian_approximation` / `gradient_approximation` | not exposed (reserved) | Deliberate: "never silently enables finite differences". |
| 8 | Gauss–Newton | **used** (pse-supplied Hessian, `HessianMode::GaussNewton`, fits only) | pse-owned Hessian, library-agnostic. |
| 9 Cancellation | Intermediate callback returning false gives `User_Requested_Stop` | **used** | Cancellation and deadline checked (§18.3). |
| 9 | `max_wall_time`, `max_cpu_time` | `max_wall_time` **used**; both reserved | |
| 9 | Recoverable evaluation error: a callback returning false makes the line search cut back the trial step | **used** | `callback.rs::classify` marks `Failure::Trial` for domain, applicability and range errors. |
| — | Problem scaling `SetIpoptProblemScaling` | not called | `nlp_scaling_method` is reserved and derived from policy (gradient-based or none). |

## 2. POUNCE 0.12.0 — owner `pounce.rs`, `tnlp.rs`, `settings/pounce.rs`, `kkt/*`, `presolve*`, `execution/pounce_convex.rs`

POUNCE registers an Ipopt-compatible option set plus extensions (`pounce-algorithm/src/upstream_options.rs`). pse sets options per solve and reserves a list of them (`pounce.rs` ~L296–330).

| Family | Capability | Status | Notes |
|---|---|---|---|
| 1 | IPM primal-dual warm start (`warm_start_init_point` + `WarmRestart`) | **used** | |
| 1 | SQP working-set warm start (`set_sqp_warm_start`, `sqp::WorkingSet`) | **used** | Carried through presolve and keyed by transformation hash (§18.3). |
| 1 | `warm_start_same_structure`, `warm_start_target_mu`, `set_warm_start_iterate(IterateSnapshot)` (entire-iterate restart), `WarmStartRecentering` | raw-reachable / **not exposed** | The `IterateSnapshot` API is not called. A reused `IpoptApplication` has its options cleared each solve. The linear backend factory is re-installed each solve, so FERAL symbolic state is not carried across solves. |
| 1 | `TnlpPresolveSession` (warm points mapped through retained presolve) | not exposed | pse has its own presolve pipeline that projects seeds (`presolve/pipeline.rs`). |
| 1 / 6 | POUNCE-convex warm start (`QpWarmStart`, `solve_qp_batch_parallel_warm`, `solve_socp_ipm_warm`) | **used** | Same-layout previous solution. `WarmCapability::None` is declared for the lifecycle, and no native state is retained. |
| 1 / 6 | Convex active-set session with warm (`active_set_session.rs`), `project_warm` through convex presolve | not exposed | |
| 2 | `pounce-sens-core` `SensApplication`, `IndexSchurData`, `parametric_step`, `compute_reduced_hessian` over pse's FERAL-backed backsolver | **used** | `kkt/sensitivity.rs` (S1/S3), `kkt/advance.rs` (retained-factor advanced step). |
| 2 | `boundcheck::step_along_path` (piecewise-linear path following through active-set breakpoints; Schur holds without refactorization), `refine_step_onto_bounds` (sIPOPT fix-relax), `path_direction` (directional derivative at a kink), `activity_kernel`, `rowlimit` | **not exposed** | **Most consequential.** `kkt/advance.rs` refuses the prediction whenever the active set changes. The library already supplies the breakpoint-walking predictor (exact for QPs, a predictor for NLPs). Contract: `weak_rows`, `eps` and `release_eps` semantics and the `max_iter` remainder rule (`boundcheck.rs` ~L1230–1275). |
| 2 | `pounce_rs::sensitivity::SensSolve` (full sIPOPT port) | not compiled | The `sensitivity` feature is off. pse composes sens-core directly, so this is likely unnecessary. |
| 2 / 6 | `pounce-convex::QpSensitivity::parametric_step`, `parametric_step_bounded`, `parametric_step_path` | not exposed | Parametric QP sensitivity with a bounded multi-segment path. |
| 2 / 6 | `pounce-qp` parametric active-set QP (`solve_parametric` homotopy `(1−t)QP₀+tQP₁`, Schur-complement rank-2 working-set updates without refactor, `elastic` ℓ1 mode, `negcurv` second-order certification) | used internally by `ActiveSetSqp`; direct API not exposed | A library-owned QP-homotopy engine for sequences of related QPs and for bounded trust-region or Levenberg–Marquardt subproblems on indefinite H. |
| 3 | Inexact / Krylov step | not provided in the NLP path | `wsmp_iterative` is experimental, and WSMP is not linked. |
| 4 | IPM filter line search, SOC, watchdog, inertia correction, restoration (`pounce-restoration`), `start_with_resto`, `expect_infeasible_problem`, `adaptive_mu_*`, `mu_oracle`, `neg_curv_escapes`, `theta_max_adaptive_*`, `filter_reset_trigger` | **used at defaults**; mostly raw-reachable | The restoration factory provider is installed explicitly. `mu_strategy` for POUNCE is raw only (no typed field; `settings/pounce.rs` has none). |
| 4 | SQP globalization `sqp_globalization` = filter / l1-elastic; `sqp_hessian` = exact / damped-bfgs / lbfgs | raw-reachable (`algorithm` is reserved; `sqp_*` keys are not) | |
| 4 | ℓ1 exact penalty-barrier (`l1_exact_penalty_barrier`) | **used** (typed `Method::L1ExactPenalty`, explicit only) | `l1_fallback_on_restoration_failure` is reserved and pinned off. |
| 4 | Second-opinion ladder (`run_second_opinion_ladder`: rescaling, barrier change, displaced start), `mu_strategy_fallback`, `dual_divergence_retry` | **deliberately not exposed / pinned off** | Policy F03 forbids hidden second solves. These are library-owned retry trajectories that a pipeline could invoke as declared attempts rather than hidden ones. |
| 4 | `infeasibility_refutation` (feasible-point refutation of an infeasible verdict) | not exposed directly | Check whether it is active inside the POUNCE path by default. |
| 5 | Continuation | Only the QP homotopy in `pounce-qp`. NLP continuation is not provided. | |
| 7 | KKT Schur block (`set_kkt_schur_block`, `SchurAugSystemSolver`, `FeralSchurSolver`) | not exposed | A library-owned Schur-complement KKT path for designated indices. |
| 7 | Presolve: auxiliary square-block elimination (`block_solve::{DampedNewtonSolver, RelaxedNewtonSolver, LargeBlockSolver}`), BTF/DM, linear-equality elimination, FBBT, redundant rows, LICQ | **used** through `PresolveOptions` qualification (`presolve.rs`) | `AuxiliaryCouplingPolicy::Aggressive` is refused. `auxiliary_max_block_dim` ≤ 1024. This is library-owned reduced-space elimination before the NLP, but it is presolve-time only, not iterative nonlinear preconditioning. |
| 7 | NLP batch `solve_nlp_batch_parallel_warm` / `NlpWarmStart` | not exposed | Parallel parametric sweeps and multistart, with per-instance warm starts. |
| 8 | `hessian_approximation` = `partitioned` (element-wise SR1/BFGS blocks, `partitioned_elements` per-constraint/blocks) and `finite-difference` (CPR/star-coloured sparse FD of the analytic Jacobian, `fd_hessian_reuse_tol`) | **not exposed (reserved)** | `HessianMode` maps only exact / limited-memory / Gauss–Newton. Partitioned quasi-Newton is a structure-preserving alternative to L-BFGS. FD-of-Jacobian is "exact up to FD error". Both conflict with the "never silently FD" rule unless typed explicitly. |
| 9 | `max_wall_time` and the intermediate callback | **used** | |
| 9 | Recoverable trial evaluation | **used** | Shared `CallbackState`. |
| 9 | POUNCE-convex: solve-wide deadline only, no interrupt | used | It overshoots by at most one factorization. |

## 3. FERAL 0.18.0 — owner `conditioning.rs`, `kkt/*`, POUNCE backend factory

| Family | Capability | Status | Notes |
|---|---|---|---|
| 2 | Symbolic reuse when the pattern is unchanged (`Solver::factor` caches `last_symbolic` / `last_pattern_fingerprint`) | used within one POUNCE solve; **not across solves** | The backend factory is rebuilt on each solve. |
| 2 | `solve_many`, `solve_many_refined` (multiple RHS) | partly used (sensitivity backsolves) | |
| 2 / 7 | `factorize_multifrontal_with_schur` (partial factorization returning a Schur block) | not exposed directly | Reached through POUNCE `FeralSchurSolver`. Constraint: the Schur columns must be a contiguous tail with a single root. |
| 2 | Unsymmetric LU `SparseLu` with product-form `update` / `update_sparse` / `refactor` (basis-update engine), `analyze_triangularized` (BTF-ordered LU; only under `LuPivoting::GilbertPeierls`) | `factor_markowitz` **used** (conditioning); updates not used | Rank-one column-replacement updates suit active-set or basis changes. The README warns that the BTF path changes rounding trajectories. |
| 7 | Parallel multifrontal, ordering race (`AutoRace`), MC64, static pivoting, cascade break | **exposed-typed** (full `FeralConfig` remote serde) | |
| — | Certified inertia, `estimate_condition_1norm`, `increase_quality` | **used** | Conditioning and inertia (Plan 22 N4). |

## 4. KINSOL (SUNDIALS 7.1.1) — owner `kinsol.rs`, `settings/kinsol.rs`, `implicit.rs`

APIs called (grep): `KINSetLinearSolver/JacFn/JacTimesVecFn/Preconditioner/Constraints/MAA/OrthAA/DelayAA/DampingAA/Damping/MaxSetupCalls/NoInitSetup(0)/MaxNewtonStep/EtaForm/EtaConstValue/EtaParams/FuncNormTol/ScaledStepTol/NumMaxIters`, plus statistics getters.

| Family | Capability | Status | Notes |
|---|---|---|---|
| 1 / 2 | `KINSetNoInitSetup(SUNTRUE)`: keep the previous Jacobian/preconditioner factor on the next `KINSol` call | **not exposed** | Always 0 ("refresh numeric setup", `kinsol.rs` ~L1084), even in the cached nested session (`implicit.rs`). This is the native cross-solve Jacobian-reuse lever for sequences, continuation and recycle loops. |
| 3 | Modified Newton: `KINSetMaxSetupCalls` (msbset) | **exposed-typed** (`Method.setup_interval`, default 10) | Not policy-driven. |
| 3 | `KINSetMaxSubSetupCalls` (msbsetsub, residual-monitoring interval), `KINSetResMonParams`, `KINSetResMonConstValue`, `KINSetNoResMon` | **not exposed** (library defaults in force: monitoring on, msbsetsub=5) | Matrix-based solvers only (KLU/dense). Docs say msbset should be a multiple of msbsetsub. pse admits any `setup_interval`. |
| 3 | Krylov SPGMR/SPFGMR/SPBCGS/SPTFQMR with analytic Jv; Eisenstat–Walker choice 1/2 or constant η | **exposed-typed** | Selected method, not automatic (as the colleague noted). |
| 3 | Preconditioner hook (`KINSetPreconditioner`) | **exposed-typed**, Jacobi only | The library provides only the hook. Block-Jacobi/ILU, or a KLU-of-blocks preconditioner, would be pse-owned. KINBBDPRE exists in SUNDIALS but is MPI-oriented and not linked. |
| 3 | `KINSetMaxBetaFails`, `KINSetNoMinEps` | not exposed | |
| 4 | `KIN_LINESEARCH` (backtracking with β-condition), `KIN_NONE`, `KIN_PICARD`, `KIN_FP`; damping; Anderson (`MAA`, delay, damping, orthogonalization); `KINSetMaxNewtonStep`; sign constraints | **exposed-typed / used** | Trust region or dogleg is **not provided** by KINSOL. FP and Picard forbid constraints (checked). |
| 4 | `KINSetReturnNewest` (FP/Picard: newest vs best iterate) | not exposed | |
| 5 | Continuation | not provided | |
| 7 | KLU with BTF (KLU default `btf=1`), `SUNLinSol_KLUReInit`, KLU ordering | KLU **used** at defaults; ordering and BTF not explicitly controlled | |
| 9 | Recoverable sysfunc error (>0): KINSOL retries with a halved step up to `MAX_RECVR`=5 (`kinsol.c` ~L1751) | **used** (`kinsol.rs::result` returns 1) | Recovery applies in the Newton/line-search paths. |
| 9 | Time budget | not provided natively | pse stops through callbacks returning −1 when the deadline or cancellation latches. |

## 5. IDAS (SUNDIALS 7.1.1) — owner `dynamics/idas.rs`

| Family | Capability | Status | Notes |
|---|---|---|---|
| 5 | `IDACalcIC` (`IDA_YA_YDP_INIT`, `IDA_Y_INIT` "SteadyStates"), `IDAReInit`, `IDASetStopTime`, `IDASensReInit`, adjoint `IDACalcICB` | **used** | |
| 5 | IC solver tuning: `IDASetMaxNumItersIC`, `IDASetMaxNumJacsIC`, `IDASetMaxNumStepsIC`, `IDASetLineSearchOffIC`, `IDASetStepToleranceIC`, `IDASetNonlinConvCoefIC`, `IDASetMaxBacksIC` | **not exposed** | IDAS's IC solve is a damped Newton with line search, usable as a library-owned consistent-initialization stage. |
| 5 | Pseudo-transient continuation / steady-state by integration | **not provided as a feature**; not implemented in pse | IDAS supplies the integrator (`IDASolve` to a stop time, `IDAReInit`). Building the pseudo-time system and the convergence test would be pse-owned. |
| 3 | `IDASetDeltaCjLSetup`, `IDASetEpsLin`, `IDASetLSNormFactor`, `IDASetLinearSolutionScaling`, `IDASetJacTimesResFn`, `IDASetNonlinearSolver` (SUNNonlinSol Newton / FixedPoint with Anderson), `IDASetMaxNonlinIters`, `IDASetNonlinConvCoef`, `IDASetMaxStep` | not exposed | `IDASetJacTimes`, `IDASetPreconditioner` (SPGMR/SPFGMR, left precondition) and `IDASetInitStep` are **used**. |
| 9 | Recoverable residual error (returns 1, so IDAS reduces the step) | **used** (`Context::failure`) | |

## 6. Diffsol 0.16.2 / diffsol-nl 0.1.1 — owner `dynamics/integrator.rs`, `dynamics/linear.rs`, `execution/dynamics.rs`

| Family | Capability | Status | Notes |
|---|---|---|---|
| 3 | `OdeSolverOptions.update_jacobian_after_steps`, `threshold_to_update_jacobian` (Jacobian lagging), nonlinear tolerance, step-growth limits | **exposed** (native struct copied in full, `copy_native`) | |
| 4 / 5 | `InitialConditionSolverOptions` (backtracking line search, Armijo, `max_linear_solver_setups`) | **exposed** | |
| — | Bdf / Sdirk / ExplicitRk | **used** | pse-owned faer `SymbolicLu`/`Lu` Newton factorizations implement `diffsol-la` traits (symbolic reused). |
| 4 | `diffsol-nl` `NewtonNonlinearSolver` with `BacktrackingLineSearch` as a general root solver | used only in adjoint (`NoLineSearch`) | A small library-owned Newton; no trust region. |

## 7. HiGHS 1.15.0 — owner `highs.rs`, `highs/*`, `settings/highs.rs`

| Family | Capability | Status | Notes |
|---|---|---|---|
| 1 | Retained instance with model modification (`changeColBounds`, `changeRowBounds`, `changeCoeff`, `changeColCost`), so simplex hot-starts from the retained basis | **used** | |
| 1 | `Highs_setBasis` / `getBasis` (basis transport) | **used** | |
| 1 | `Highs_setSparseSolution` (MIP start) | **used** (`Settings.sparse_start`) | |
| 1 | `Highs_setSolution` (full primal/dual start, e.g. LP crossover start) | not called | |
| 1 / 6 | `qp_allow_hot_start` (active-set QP hot start, advanced, default false) | raw-reachable | Relevant if HiGHS QP serves as a bounded LM/trust-region subproblem. HiPO (QP IPM) is not built. |
| 2 | Ranging, `getBasisInverseRow`, fixed-LP duals | **used** (opt-in diagnostics) | |
| 9 | `Highs_setCallback` interrupt, `time_limit` | **used** | |

## 8. Clarabel 0.11.1 — owner `conic.rs`, `execution/clarabel.rs`

| Family | Capability | Status | Notes |
|---|---|---|---|
| 1 | Warm start | **not provided** (no warm-start API at 0.11.1; grep of `src` finds none) | |
| 2 | `update_data` / `update_settings` with `is_data_update_allowed` (keeps the symbolic KKT factorization) | **used** | Structural and settings restrictions apply. |
| 6 | Convex QP/SOCP local models (bounded least squares as a QP) | available; used as a solver class | No hot start, so it is a poor fit for many tiny LM steps. |
| 7 | Chordal decomposition (`sdp`), direct solver (qdldl; `pardiso-mkl` optional; faer-sparse refused, since it would add a second faer) | **exposed-typed** | |
| 9 | `set_termination_callback`, `time_limit` | **used** | |

## 9. SCIP 10.0.2 — owner `scip.rs`, `execution/scip.rs`

| Family | Capability | Status | Notes |
|---|---|---|---|
| 1 | Reoptimization (`SCIPenableReoptimization`, `SCIPchgReoptObjective`, `SCIPfreeReoptSolve`) | **used** (typed) | Over sequences. |
| 1 | Partial and complete start solutions (`SCIPcreatePartialSol`, `SCIPaddSolFree`) | **used** | |
| 4 / 7 | Concurrent solve (`SCIPsolveConcurrent`), PaPILO presolve, nested Ipopt NLPI (`nlpi/ipopt/linear_solver` is reserved and typed) | **used** | |
| 9 | `SCIPinterruptSolve`, `limits/time`, `limits/memory` | **used** | |
| — | Heuristic, separation, propagation and branching parameters | raw-reachable (free-form after reserved keys) | |

## 10. faer 0.24.4 — owner `dynamics/linear.rs`, `dynamics.rs`; general

| Family | Capability | Status | Notes |
|---|---|---|---|
| 2 | Sparse `SymbolicLu` + numeric `Lu` (symbolic reuse) | **used** (dynamics Newton) | |
| 2 | Sparse `SymbolicLlt`/`Llt`, LDLᵀ/Bunch–Kaufman, sparse `Qr` | not used | Sparse QR is the library-owned least-squares factor for Gauss–Newton or LM local models. |
| 2 | Dense LDLᵀ/LLᵀ `rank_r_update_clobber`, `insert_rows_and_cols`, `delete_rows_and_cols` | not used | Factor updates for small active-set or Schur blocks. |
| 3 | `matrix_free::{bicgstab, conjugate_gradient, lsmr}`, `partial_eigen`, `partial_svd` | not used | No GMRES in faer. LSMR gives matrix-free least squares (Jacobian-free LM). `partial_svd` supports conditioning or rank diagnostics. |

---

## Cross-cutting findings

1. **Raw passthrough is wide.** Every adapter admits untyped `Controls.options` minus a reserved list. Many globalization levers (Ipopt and POUNCE `start_with_resto`, `expect_infeasible_problem`, `mu_oracle`, `least_square_init_*`, `limited_memory_*`, SQP globalization, HiGHS `qp_allow_hot_start`) are therefore reachable but outside typed policy and automatic selection. Under the "seamless automatic selection" target, each one a pipeline drives needs a typed home. The raw surface also lets callers set things that conflict with the C-API lifecycle (`warm_start_same_structure`).
2. **Library-owned mechanisms that would otherwise be bespoke:**
   - active-set-crossing sensitivity prediction (`pounce-sens-core::boundcheck::step_along_path`, `refine_step_onto_bounds`, `path_direction`);
   - parametric QP homotopy with Schur updates (`pounce-qp`);
   - convex parametric path (`QpSensitivity::parametric_step_path`);
   - cross-solve Jacobian reuse in KINSOL (`KINSetNoInitSetup`), plus residual-monitoring control;
   - IDAS IC solver controls;
   - partitioned quasi-Newton and coloured FD Hessians (POUNCE);
   - Schur KKT blocks (POUNCE/FERAL);
   - NLP batch warm (POUNCE);
   - declared retry trajectories (POUNCE second-opinion ladder);
   - faer sparse QR and LSMR for LM local models.
3. **Library-provided nowhere in the pinned set:** NLP or nonlinear-equation homotopy or continuation; trust-region or dogleg nonlinear-equation globalization (KINSOL, diffsol-nl and Ipopt are line-search only); pseudo-transient continuation as a feature. Integrators exist (IDAS, Diffsol), but the PTC formulation would be pse-owned. Clarabel has no warm start. Ipopt has no sIPOPT or factor retention through the C API.
4. **Version and build uncertainties:**
   - sIPOPT absent: verified from configure flags, not from a library listing.
   - Ipopt `inexact_algorithm` was not executed. It is judged unusable from the build (no Panua Pardiso).
   - Whether POUNCE `infeasibility_refutation` runs by default was not traced.
   - `pounce-qp`'s `solve_parametric` stub history: the module docs say the homotopy is now implemented. This is observed in source, not executed.
   - FERAL symbolic caching across a rebuilt backend factory was inferred from code, not measured.
   - The KINSOL msbset/msbsetsub multiple rule is documented but not enforced by pse.
   - All library claims are source or doc observations at the pinned versions. None was executed in this investigation.
