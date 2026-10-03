# Controls, selection facts and observations: current-system inventory

Baseline: HEAD `f0b90258` plus uncommitted Plan 25k tree (read as-is, nothing built or run).
Paths below are relative to the repository root; `BN` = `crates/pse-backend-native/src`,
`RT` = `crates/pse-runtime/src`. Observed facts are marked (obs); interpretation is marked (interp).
The uncommitted diff in `BN` (runner.rs, routing.rs, quality.rs, solve.rs, sos.rs) does not alter any
control, setting, fact or metric listed here (checked via `git diff` on routing.rs and solve.rs;
runner.rs/quality.rs diffs only skimmed by stat).

## 0. Shape of the request surface (one-paragraph map)

`SolveSettings` (RT/math/settings.rs:23, serde+schemars, version 1) -> `.profile()` -> `SolverProfile`
(RT/math/solves.rs:40) = { presolve `Policy`, `NumericalPolicy`, `ConvexityPolicy`, `SolveIntent`,
`SolverSelection` (Auto | Explicit(Backend)), `Controls`, `BackendSettings`, optional `SensitivityRequest` }.
`BackendSettings` (BN/execution.rs:~490) is a tagged enum of one pse-owned settings type per adapter.
Accuracy is NOT a control: `ResolvedAccuracy` (BN/solve.rs:70) is derived from `NumericalPolicy`
(BN/solve.rs:150-190). Python types are generated msgspec structs from the schemars schemas
(`python/pse/contracts/documents/__init__.py`: SolveSettings 5353, SolveControls 5320, KinsolSettings 3716,
IpoptSettings 3504, PounceSettings 4560, HighsSettings 3096, ClarabelSettings 2319, ScipSettings 5247,
PounceConvexSettings 4542, NumericalPolicy 4245, SensitivityRequest 5287).

## 1. Solver controls

### 1.1 Shared `Controls` (BN/solve.rs:245-290, validate 338-365)
| Field | Default | Notes |
|---|---|---|
| time_limit (s) | 300 | wall clock incl. callbacks |
| iterations | 3000 | native iteration limit, 1..=i32::MAX |
| threads | 1 | adapter must declare `parallel` |
| history | 256 | bounded retained events (<=1e6) |
| hessian | Exact | registry `HessianMode`: exact / limited_memory / gauss_newton (fits only) |
| reuse | Fresh | `ReusePolicy`: fresh / allow_rebuild / require_reuse |
| start | NoPriorStart | `NativeStartPolicy`: no_prior_start / previous_accepted / explicit |
| options | {} | raw native option map (Bool/Integer/Real/Text), <=4096 keys |
| foreign_bytes | None | foreign-library memory allowance |
`Controls` carries no tolerance. Identity: `Controls::identity` (serde, Frame NativeControlsV3).

### 1.2 Raw native option passthrough, per adapter (obs)
| Adapter | Passthrough of `controls.options` | Refused / reserved |
|---|---|---|
| KINSOL | none: `!options.is_empty()` -> `Unsupported` (BN/kinsol.rs:1022; BN/execution/kinsol.rs:88) | everything |
| Ipopt | any registered Ipopt option not reserved (BN/ipopt.rs:689-722) | reserved list: option_file_name, hessian/gradient/jacobian_approximation, grad_f/jac_c/jac_d/hessian_constant, nlp_lower/upper_bound_inf, max_iter, max_wall_time, max_cpu_time, tol, constr_viol_tol, dual_inf_tol, compl_inf_tol, acceptable_*, bound_relax_factor, honor_original_bounds, warm_start_init_point, nlp_scaling_method, obj_scaling_factor; plus `settings::RESERVED` (linear_solver, hsllib, pardisolib, mumps_pivot_order, spral_*, pardisomkl_*, mu_strategy, bound_push, bound_frac); plus `RESTART_OPTIONS` (mu_init, warm_start_*). Unreserved and therefore open: restoration (`resto_*`), `nlp_scaling_max_gradient`, `max_soc`, `watchdog_*`, `mu_*` other than strategy/init, `alpha_for_y`, `limited_memory_*`, `print_level` (defaults 0), etc. Registration/type is checked only by Ipopt (`AddIpopt*Option` false -> Contract error) |
| POUNCE | any registered option not reserved (BN/pounce.rs:296-340); table is cleared per reused app | PINNED_OFF (mu_strategy_fallback, dual_divergence_retry forced false), RESERVED_METHODS (l1_fallback_on_restoration_failure, l1_exact_penalty_barrier), RESTART_OPTIONS, algorithm, hessian/gradient/jacobian approximations, *_constant, presolve, limits, tolerances, scaling, linear_solver, option_file_name, bounds infinity; any `feral_*`/`ma57_*` key refused (use typed FERAL profile). `mu_strategy` is NOT reserved for POUNCE (read from options at pounce.rs:~400) |
| HiGHS | any HiGHS option not reserved (BN/settings/highs.rs:46-80), text <512 bytes | reserved: threads, parallel, time_limit, solver, infinite_bound/cost, small/large_matrix_value, user_bound/cost_scale, solve_relaxation, mip_max_nodes, *_iteration_limit, qp_regularization_value, primal/dual/mip feasibility tol, mip_abs/rel_gap, simplex_scale_strategy, log_file, blend_multi_objectives. Open: `presolve`, `simplex_strategy`, `mip_*` heuristics, `qp_allow_hot_start` (defaults true unless caller supplies) |
| Clarabel | none (BN/conic.rs:463) -> `Unsupported` | typed settings only |
| SCIP | any SCIP parameter not reserved (BN/scip.rs:1055-1130) | reserved: misc/catchctrlc, limits/time,memory,gap,absgap,totalnodes, numerics/feastol, randomization/randomseedshift, lp/threads, nlpi/ipopt/{linear_solver,hsllib,pardisolib}; prefixes parallel/, concurrent/, iis/, reoptimization/, exact/ |
| POUNCE-convex | not examined in detail (typed settings; QpOptions built at BN/execution/pounce_convex.rs:377) | - |

### 1.3 KINSOL (`settings::kinsol::Method`, BN/settings/kinsol.rs:100-140; adapter BN/kinsol.rs)
Typed controls and defaults:
- strategy: LineSearch (registry vocab: picard / newton / line_search / fixed_point). Newton<->LineSearch
  interchangeable on a retained session; Picard needs explicit constant splitting; FixedPoint needs a declared map.
- linear: Klu (analytic CSC Jacobian). Alternatives: Dense{limit}, Spgmr/Spfgmr/Spbcgs/Sptfqmr{dimension} (matrix-free via analytic JVP).
- anderson (m): 0 (off; must be <= n). anderson_delay: 0. orthogonalization: ModifiedGramSchmidt (4 vocab members). Both refused unless anderson>0.
- damping: 1.0 (applied to both `KINSetDamping` and `KINSetDampingAA`).
- setup_interval: 10 -> `KINSetMaxSetupCalls` (msbset). `KINSetNoInitSetup(0)` always: numeric setup refreshed on every solve (kinsol.rs:1084).
- max_newton_step: None (KINSOL default 1000*||D_u u0||); if given must be >= 1.
- eta: Choice1 (E-W); Choice2{gamma in (0,1], alpha in (1,2]}; Constant{value}. Refused unless a Krylov linear route.
- preconditioner: None; Jacobi (right, from Jacobian diagonal). Refused unless Krylov and not fixed point.
Derived from policy, never user input (`Settings::from_policy`, kinsol.rs:120-140): variable_scales, residual_scales
(`feasibility*s/t`), step_tolerance (=feasibility); `KINSetFuncNormTol` = `accuracy.feasibility`; max iters = `controls.iterations`;
sign constraints from one-sided bounds/guard signs (shifted coordinates).
NOT exposed (absence verified by grep over `crates/`, no hits): `KINSetMaxSubSetupCalls` (msbsetsub), `KINSetNoResMon`/`KINSetResMonParams`/
`KINSetResMonConstValue` (residual monitoring runs at KINSOL defaults), `KINSetNoMinEps`, `KINSetRelErrFunc`, `KINSetMaxBetaFails`,
info/print handlers, `KINSetEtaForm`-driven adaptivity beyond the three forms, scaling of the initial step, line-search parameters.
Reuse: `matches_settings` requires equal linear, anderson, orthogonalization, preconditioner (fixed at allocation); strategy/damping/setup_interval/eta/max step/delay refreshed each solve.
Auto path: `BackendSettings::Default` -> `Method::default()` (BN/execution/kinsol.rs:46); i.e. direct KLU + line search + setup interval 10, always. Nested implicit solves use the same adapter with `setup_interval: 1` (BN/implicit.rs:317-322), default everything else.
Warm start: primal only (`WarmCapability::Primal`, payload `Root(x)`); no Jacobian/factor reuse across solves (KLU symbolic is reused within a retained session; numeric refreshed).

### 1.4 Ipopt (`settings::ipopt::Settings`, BN/settings/ipopt.rs; native options BN/ipopt/settings.rs)
Typed: linear = Mumps{ordering: Metis default} | Spral{ordering, scaling, pivot} | PardisoMkl{ordering, matching}
(HSL and runtime Pardiso unrepresentable); mu_strategy Monotone (alt Adaptive); bound_push 0.01; bound_frac 0.01 (<=0.5);
restart: `WarmRestart` {barrier: Seed | Value{value}, bound_push 1e-9, bound_frac 1e-9, slack_bound_push 1e-9, slack_bound_frac 1e-9, mult_bound_push 1e-9} (BN/solve.rs:682-760).
Derived and always set (BN/ipopt.rs:~725-770): accuracy.nlp_options() (tol, constr_viol_tol, dual_inf_tol, compl_inf_tol, bound_relax_factor 0, honor_original_bounds yes,
acceptable_* (acceptable_iter 15 only when an acceptable budget is declared, else 0), max_iter, max_wall_time, hessian_approximation exact|limited-memory (from `HessianMode != LimitedMemory`),
`*_constant` flags from `NlpOracle::derivative_facts()`, nlp_scaling_method gradient-based|none (from `NumericalPolicy.native_scaling`), warm_start_init_point yes only for a complete primal-dual seed,
print_level 0 unless overridden).
Warm start: `WarmCapability::PrimalDual`; payload Nlp{primal, bounds, rows, barrier}; `mu_init` is set from the seed's recorded final barrier only under monotone mu (restart.apply). Not set by pse: cold `mu_init`, restoration options, `nlp_scaling_max_gradient`, `bound_mult_init_*`, `warm_start_mult_init_max` (open via passthrough only).
`DroppedOptions` rule (BN/ipopt.rs:~825-870): retained C problem reused only if coordinates/profile stamps, patterns and bounds match AND the new option key set contains every key ever set on the problem (Ipopt C API cannot unset); else rebuild, or under `RequireReuse` -> `ProblemError::Reuse{refusal: DroppedOptions(keys) | Structure}`. All option values are re-applied each solve. (interp) A cold step followed by a warm step reuses (adds keys); the reverse rebuilds.
Admission refuses unlinked solver, threads>1 for MUMPS, SPRAL w/o OMP_CANCELLATION or with OMP_PROC_BIND false, Pardiso off pinned CBWR or MKL_DYNAMIC on. No substitution.
Hessian: exact Lagrangian, limited-memory, or Gauss-Newton Gram (`HessianMode`); finite differences never enabled silently.

### 1.5 POUNCE (`settings::pounce::Settings`, BN/settings/pounce.rs)
Typed: method = InteriorPoint (default) | ActiveSetSqp | L1ExactPenalty (explicit only; Auto presolve resolves Off under it); linear = full FERAL config (cascade_break, fma, refine, increase_quality,
refine_max_steps, refine_target, singular_pivot_floor, inertia_pivot_floor, pivtol, ordering {amd, amf, metis_nd, scotch_nd, kahip_nd, auto, auto_race}, scaling {inf_norm, mc64_symmetric, identity, auto},
parallel, min_par_flops, static_pivoting; defaults = FERAL's own); restart `WarmRestart` as Ipopt. Hidden second solves pinned off. Warm: primal-dual + SQP working set (`PrimalDualAndWorkingSet`);
working set transfers only under the same presolve transformation identity. No typed SQP-specific controls (QP solver choice, trust-region/filter parameters); only the raw passthrough (unreserved POUNCE options, readback snapshot of all registered options is recorded as `report.options`).
POUNCE-convex (`settings::pounce_convex`): self_dual true, equilibrate true, crossover false, FERAL linear. Explicit-only (`automatic_classes: &[]`), `batch: true`, no warm.

### 1.6 HiGHS (`settings::highs::Settings`, BN/settings/highs.rs:20-135)
Typed: method Choose (simplex / ipm / pdlp; mixed-integer or quadratic requires Choose), nodes None (-> mip_max_nodes i32::MAX), diagnostics (rays, iis, ranging, relaxation, fixed_lp, basis_inverse, presolve, cut_pool; all off), sparse_start (partial MIP start).
Derived: feasibility/dual/mip tolerances, mip gaps from policy, `qp_regularization_value` from gap, `simplex_scale_strategy 0` iff `native_scaling` false, iteration limits = `controls.iterations`, threads/parallel from controls. Warm: primal+dual+basis; QP hot start enabled by default. HiGHS `presolve` left to the native default unless passed raw.

### 1.7 Clarabel (`conic::Settings`, BN/conic.rs:248-330)
Typed and complete (mode SingleSolve|ReusableData; direct Qdldl|MklPardiso; max_step_fraction, tol_infeas_*, tol_ktratio, reduced_*, equilibrate_*, linesearch_backtrack_step, min_switch/terminate_step_length,
static/dynamic regularization, iterative_refinement_*, presolve_enable, input_sparse_dropzeros, chordal_decomposition_*); defaults = Clarabel's. Iteration/time/tolerances/thread count derive from controls+accuracy. No warm start. Auto owns `ContinuousCone` only.

### 1.8 SCIP (`execution::ScipSettings`, BN/execution/scip.rs:22-60)
nlp_linear_solver (Mumps), seed 0, nodes None, pool 0, iis false, exact false, reoptimize false. Auto owns MixedLinear, MixedIntegerQuadratic, MixedIntegerNonlinear, SmoothNlp (rank 5, last); continuous LP/QP explicit only.

### 1.9 Dynamics (BN/dynamics.rs:41-300, `Profile` 743-835; generated from registry vocab in `crates/pse-schema/src/catalog/solve_settings.rs:341-440`)
`Profile`: method Auto (diffsol|idas; IDAS when recoverable trials needed), trial_failures Terminal, rtol, atol[], out_rtol/out_atol (conservation), initial_step, max_steps, max_events, max_cells, sensitivity none|forward|adjoint, adjoint {steps_between_checkpoints 250, max_checkpoints 400}, parameter_scales, schedule, diffsol {method bdf|tr_bdf2|esdirk34|tsit45, linear faer_lu|klu}, idas {linear klu | spgmr{dimension, preconditioner} | spfgmr, sensitivity simultaneous|staggered, initialization algebraic_and_rates|steady_states}, plus full native Diffsol `InitialConditionSolverOptions` and `OdeSolverOptions` as remote-serde documents (passthrough of the library's own option structs). IDAS counters recorded (idas.rs:1974-1983: steps, residual_evaluations, nonlinear_iterations, nonlinear_convergence_failures, error_test_failures, initialization_backtracks, root_evaluations, linear_iterations, preconditioner_evaluations, jacobian_products). Not mapped in depth: Diffsol metric names.

## 2. Selection facts

### 2.1 What the selector consumes (obs)
`routing::Requirements` (BN/routing.rs:558) = facts (`pse_math::facts::ProblemFacts`, crates/pse-math/src/facts.rs:25), intent, `numerical_psd`, `least_squares`, controls, settings, `sensitivity` flag, `Context` (snapshot of linked builds + process env, pending class evidence, per-backend refusals, structure witness, oracle contract, guard signs, budgets, coefficients/certificate/cone/factorable, prepared artifacts).
`ProblemFacts` fields: class_status; variables, rows; objective/objectives; equalities; per-variable domains; derivatives (available order) and prepared_derivatives; per-variable `BoundShape` {Free, Nonnegative, Nonpositive, Lower, Upper, Boxed}; `guarded: bool`; coefficients; affine_rows[]; objective_degree; quadratic; native constraint forms; structural requirements (l1); `Convexity` (exact Gram certificate or curvature-pass fact or cone summary).
Selection algorithm (BN/routing.rs:1174-1275 `select_assessed`): problem classes most specific first (SquareRoot, then Linear/MixedLinear/Convex-/Nonconvex-/MixedInteger-Quadratic/MINLP/ContinuousCone, then SmoothNlp), first class with an eligible adapter whose `automatic_classes` contains it wins, ties broken by `BackendExecution::automatic()` rank: KINSOL 0 (SquareRoot, DeclaredFixedPoint), HiGHS 1 (Linear, MixedLinear, ConvexQuadratic), Ipopt 2 (SmoothNlp), POUNCE 3 (SmoothNlp), Clarabel 4 (ContinuousCone), SCIP 5. Dynamics/Diffsol/Idas/POUNCE-convex are explicit-only. A sensitivity request first restricts to `Capability.sensitivities` adapters (KINSOL, Ipopt, POUNCE, SCIP true; HiGHS, Clarabel, convex, dynamics false).
So in practice Auto = class -> fixed preference rank; for smooth NLP it is always Ipopt when eligible (POUNCE only by explicit selection or if Ipopt ineligible), for square roots always KINSOL.
`Capability` record (BN/execution.rs:68-130): structural policy, lexicographic degradation, classes, automatic_classes, derivatives (ExactHessianOrLimitedMemory | JacobianOrProduct | Coefficients | Factorable | Forward/SecondOrderAdjointSensitivities), warm (None|Primal|PrimalDual|PrimalDualAndBasis|PrimalDualAndWorkingSet), general_bounds, sign_bounds, parallel, certifies, native_forms, requirements, lexicographic, batch, sensitivities, and prose reuse/cancellation/diagnostics. `batch` is true only for POUNCE-convex. `sign_bounds` is true for all algebraic adapters; KINSOL has `general_bounds: false` (one-sided only via shifted signs).
Within an adapter, the SETTINGS are not selected from facts: Auto leaves `BackendSettings::Default` (native defaults). The one fact-driven setting choice found is `BackendSettings::for_requirements` (POUNCE l1 method under an authored `penalty(l1)`; BN/execution.rs:~560) and `Auto` presolve resolving to Off under l1.
Other facts consumed outside routing, per solve: `NlpOracle::derivative_facts()` (gradient/Jacobian/Hessian constant) -> Ipopt `*_constant` options (BN/lib.rs:425); `native_scaling` policy flag; HiGHS convexity evidence; presolve `Auto` (qualified passes only, never auxiliary reduction).

### 2.2 Where Auto is resolved and recorded
Resolved by `Requirements::decision`/`select` (BN/routing.rs:966-1275) called from solve preparation (RT/math/solves.rs prepare path) and, per block, from `PreparedInitialization::strategies` (RT/math/initialization.rs:42-130, same solver profile and backend settings for every block). Recorded as `runtime.route_decisions` rows (`Decision::row`, BN/routing.rs:228-295): intent, selection Auto|Explicit, classes, state, snapshot id, eligibility per backend with reason codes, structural mode/admission, evidence/artifact demands, selected backend, representation, refusal. Written by RT/workflow/modeling_results.rs (~line 120-140) with `runtime.structural_assessments`. No retry/fallback to another backend: AGENTS/§18.7 and `PINNED_OFF` enforce this.

### 2.3 Facts that exist elsewhere but are NOT inputs to selection or settings (obs, verified by grep over routing.rs and execution/*.rs)
- Structure: `StructuralAnalysis` (matching, DM parts, prerequisite-first `blocks`) is carried in `Context.structure` and used for structural ADMISSION (rank/over/under-determination, mode) and for block initialization planning; BTF block count/size and block structure are not used to choose an adapter or settings in routing.rs (no `blocks` reference).
- Conditioning/rank: `conditioning::jacobian_condition` (Hager-Higham estimate through FERAL sparse LU; BN/conditioning.rs:19) and `conditioning::kkt` (inertia, condition) are called only from KKT-point analysis (BN/kkt.rs) and the diagnostics workflow (RT/workflow/modeling/diagnostics.rs:780), i.e. post hoc or on request, never before a solve to choose method. `jacobian_diagnostics::analyze` (degeneracy LPs/MILPs) is likewise diagnostics-only.
- Domain risk: `pse_math::presolve::Facts` holds FBBT tapes (interval enclosures per row), guard signs (`signs`), per-instance obligations, `has_guards`, affine row proofs, `objective_linear`; only `guarded: bool`, bound shapes and guard signs (KINSOL sign constraints) reach routing. Property validity envelopes surface only as runtime recoverable trial rejections (`ProviderError::Envelope`/`RegimeCrossing`), counted after the fact.
- Prior state: `Compatibility {layout, profile, data, backend}` (BN/solve.rs:643) decides seed fit and retained-session fit; the choice to use a seed is made by the explicit `StartPolicy` and, in sequences, by typed `Start` + candidate-use. Nothing computes parameter distance between cases or ranks candidate predecessors; study predecessors are authored dependency edges (RT/workflow/study.rs:127-155, `StudyStartKind`: fresh/not_needed/continuation/explicit/fresh_fallback).
- Retained-factor prediction: `kkt::advance::{Advance, predict}` (BN/kkt/advance.rs) predicts via one backsolve with active-set screening and returns a `Fallback` reason; its only consumer in `RT` is the horizon driver (RT/workflow/horizon/driver.rs:1252). Studies, homotopy and initialization do not call it.
- Derivatives: availability is a routing fact (`derivatives`, `prepared_derivatives`); Hessian vs limited-memory is a user control (`HessianMode`), with `Capability.derivatives` and `derivative_demand` (BN/routing.rs:727) converting the control into a compilation demand; no automatic downgrade to limited-memory when Hessian is expensive or unavailable beyond refusal/artifact demand. Jacobian-vector product availability: KINSOL Krylov routes assume it (analytic JVP).
- Size: variables/rows present in `ProblemFacts` but no size-based method choice (e.g. dense vs KLU vs Krylov) except KINSOL Dense's explicit `limit`.

## 3. In-solve observations and metrics

### 3.1 Recorded per attempt (obs)
- Callback boundary (`CallbackState::finish`, BN/callback.rs:199-230): `callback.<demand>.calls`, `callback.<demand>.seconds` (including rejected trials), `callback.trial_rejections`, `callback.regime_crossings`, `callback.terminal_failure`; typed `Evidence.callback` (trial_rejections, regime_crossings, terminal_failure).
- KINSOL counters (kinsol.rs:1120-1152): KINGetNumNonlinSolvIters, NumFuncEvals, NumBetaCondFails, NumBacktrackOps; for non-fixed-point: NumJacEvals, NumLinFuncEvals, NumPrecEvals, NumPrecSolves, NumLinIters, NumLinConvFails, NumJtimesEvals, LastLinFlag; reals KINGetFuncNorm, KINGetStepLength. Plus `start.submitted`, `reuse.native_model`. No per-iteration events (KINSOL has no iteration callback wired). No count of numeric setups other than NumJacEvals/NumPrecEvals; no setup-call counter (`KINGetNumLinSolvSetups`) found.
- Ipopt (ipopt.rs:351-455): per-iteration `ipopt.iteration` events (iteration, restoration flag (mode==1), objective.normalized, primal.native, dual.native, barrier, step.norm, regularization, alpha.dual, alpha.primal, line_search.trials, stationarity.normalized, iterate.normalized.infinity_norm, regime.crossings), bounded by `controls.history` (default 256) with `dropped_events` counted; final: reuse.native_model, linear.threads, execution.seconds, provenance. No final aggregate statistics (iteration count, restoration count, factorization count, timing) from Ipopt; they would have to be reconstructed from retained events, which are truncated beyond history. (no `IpoptGetCurrentStatistics`-like call found; verified by grep.)
- POUNCE (pounce.rs:500-600): full serde projection of `pounce_rs::SolveStatistics` as `statistics.*` metrics (including per-iteration trajectory truncated to history), complete `timing.*` (alg phases, factorization, backsolve, callback totals, per-callback eval times), `linear.factors`, `linear.pattern_reuse`, `linear.pattern_changes`, `linear.fill_ratio`, `linear.min_pivot/max_pivot`, inertia counts, nnzA/nnzL, `warm.diagnostics`, crossover report; effective option table and defaults.
- HiGHS: simplex/ipm/pdlp/qp/crossover iteration counts, status, basis_validity, mip_node_count, bounds, gap, infeasibility and residual summaries, complementarity; callbacks events (simplex.iterations, ipm.iterations, MIP improving solution as typed incumbent).
- Clarabel: iterations and residual/gap info (BN/conic.rs:828-850). IDAS: counters listed in 1.9.
- Quality (post solve, in original coordinates): `quality::Quality`, `Observation`, `KktEvidence`, `local` KKT analysis (inertia, condition) and parametric sensitivity if requested.
### 3.2 Publication
`runtime.solve_runs` (state, backend, termination, assurance, qualification), `runtime.solve_metrics` (run_id, step, namespace, name, typed value; `progress` namespace holds `dropped_events`; events become `event.<i>.<phase>` namespaces) via RT/workflow/results.rs:150-215; `runtime.route_decisions`, `runtime.structural_assessments`, `runtime.resolved_numerics` (frozen budgets and provenance), `runtime.candidate_assessments`, `runtime.solver_capabilities` (one row per linked adapter), `runtime.incumbents` (durable only). Progress also via `RunHandle::progress`. Spans (`pse.*`) stop at preparation/assembly boundaries; no per-evaluation spans (operations-and-validation §23.1).
### 3.3 What consumes observations at runtime (obs)
Policy in code: "metrics remain observations and are never an input to a decision" (BN/solve.rs `Evidence` doc). A grep for `.metrics.get`/index in non-test `RT`/`BN`/`pse-compiler` found no production reader. The only runtime consumers of in-solve evidence are:
1. `CallbackEvidence` via `callback::retryable_evaluation` (BN/callback.rs:267): an `Evaluation` termination is retryable only with positive trial-rejection evidence and no terminal failure; used by initialization `retryable()` (RT/workflow/modeling/engines.rs:181-230).
2. Homotopy in `initialize_model` (engines.rs:655-715): value-only continuation on authored endpoints; on acceptance step *= `growth` (1.5), on retryable failure step *= 0.5 until `minimum_step` (1e-6), first point failure is terminal; retryability is a function of `Termination` category (success/acceptable/feasible_only/infeasible/inconclusive/iteration_limit/numerical retry; unbounded/limits/cancelled/panic/invalid do not), not of iteration counts or conditioning. Acceptance is the candidate-use decision.
3. Sequence seeding: typed `Start`, `Predecessor`, candidate-use (`CandidateDecision::permits_use`) in RT/workflow/staged.rs.
4. Horizon driver consumes `kkt::predict` and its `Fallback`.
No strategy change inside a native solve is driven by pse (KINSOL/Ipopt/POUNCE own their own globalization); pse changes nothing mid-solve and does not adapt settings between attempts from counters.

## 4. User-facing options surface and its source of truth

### 4.1 Types
| Surface | Rust owner | Registry/codegen |
|---|---|---|
| `SolveSettings` (version, intent, backend, settings, presolve, presolve_options, required_passes, controls, numerics, convexity_absolute/relative, sensitivity) | RT/math/settings.rs:23 (serde + schemars) | JSON Schema -> `docs/generated/schema/` -> msgspec types in `python/pse/contracts/documents/` (`pse-codegen::codegen::documents`); enums are registry vocab (`crates/pse-schema/src/catalog/solve_settings.rs`, `native_math.rs`, `modeling_analysis.rs`) |
| `Controls` ("SolveControls") | BN/solve.rs:245 | same (schemars) |
| `BackendSettings` (tag `backend`) | BN/execution.rs:~490 | same; per-adapter types in BN/settings/*, BN/conic.rs, BN/execution/scip.rs |
| `NumericalPolicy` | `crates/pse-model/src/numerics.rs:41` | same; defaults: strict_nominals false, native_scaling true, kkt {1e-8,1e-8}, acceptable None, integrality 1e-8, gap_absolute 1e-8, gap_relative 1e-8, mip_absolute_gap 1e-6, mip_relative_gap 1e-4, closure RequireClosed, incumbent Refuse, plus ID-keyed `requirements` (per quantity/unit nominals and budgets) |
| `Policy` (presolve) | BN/presolve.rs:21 | kinds off/auto/explicit; explicit = full `pounce_presolve::PresolveOptions` via raw options and `required_passes` (Pass vocabulary) |
| `SensitivityRequest` | RT/math/settings.rs:~75 | parameters, reduced_hessian, propagation |
| Start/reuse | `StartPolicy` (registry `NativeStartPolicy`), `ReusePolicy` | in Controls; `WarmRestart` in Ipopt/POUNCE settings |
| Initialization (authored model) | `ModelingInitialization` (RT/workflow/modeling/engines.rs:42): stages, homotopy bool, initial_step 0.25, minimum_step 1e-6, growth 1.5, maximum_attempts 128, time_limit 60 s, discrete {Refuse, FixAtStart, FixAt}; user override via `InitializationOverrides` (RT/workflow/modeling/declared.rs:155) which may not contradict authored values; authored fixtures own stages/homotopy endpoints (`Fixture.initialization`, `continuation` endpoints) | Python `InitializationOverrides` struct in contracts documents (generated); Python `ModelingInitialization` in `python/pse/_modeling.py` is a result handle only |
| Structural block initialization | `InitializationProfile{solver: SolverProfile, stages: Vec<overlay>}` (RT/math/initialization.rs:266); intents Root/Initialize only; no explicit presolve/convexity; `threads == 1`; Explicit start and RequireReuse refused; PreviousAccepted seeds stage k from stage k-1 |
| Study start/dependency | `StudyPointPolicy` with authored predecessor edge and role (RT/workflow/study.rs:61-155) | `StudySeedNeed`, `StudyContinuationPermission` (require_usable|allow_seed_only), `StudyUnavailableSeedPolicy` (refuse|fresh_on_unavailable) |

### 4.2 User controls vs derived or reserved
User: Controls (limits, threads, history, hessian, reuse, start, options, foreign_bytes); typed backend settings listed in section 1; presolve policy and passes; numerical policy fields; sensitivity request; initialization/homotopy policy; backend selection (Explicit vs Auto).
Derived and reserved (refused if passed raw): all stopping tolerances (from policy), KINSOL scales and step tolerance, Ipopt/POUNCE scaling, bound relaxation, derivative-constancy flags, iteration/time options, linear solver identity (typed), mu_strategy/bound_push (Ipopt), warm restart options, HiGHS tolerances/gaps/QP regularization/scaling, SCIP limits/seed/threads, POUNCE retries and l1 switch.
Identity: serde-derived hashes (`Controls::identity`, `ResolvedAccuracy::key`, `NumericalPolicy::key`, profile key) feed the `profile` stamp of `Compatibility`; changing a setting changes session fit (not seed fit).
Python: no hand-written mirror; typed msgspec documents decode in Rust (`SolveSettings.profile()` applies cross-field rules: presolve options only with explicit policy, both-or-neither convexity budgets, sensitivity admission).

## 5. Numerical accuracy tiers

Observed concepts of accuracy distinct from final:
- `NumericalPolicy.acceptable: Option<KktTolerances>` (BN/solve.rs:70-140): relaxed KKT termination budgets for Ipopt/POUNCE (`acceptable_*`, `acceptable_iter` 15) that must be >= the strict KKT budgets; absent by default; this is a native stopping relaxation, qualified separately (qualification "acceptable").
- `IncumbentPolicy` (Refuse | AcceptFeasible | AcceptWithinGap) and `ClosurePolicy` (RequireClosed | AllowUnclosed): acceptance of limit-terminated candidates; completion-side, not an intermediate-accuracy mechanism.
- MIP gaps (`mip_absolute_gap` 1e-6, `mip_relative_gap` 1e-4) differ from continuous gaps (1e-8): a class-specific final tolerance, not a stage tier.
- Nested implicit solves (BN/implicit.rs:253-330, RT/workflow/modeling/implicit.rs:~100-190): inner KINSOL is given the SAME resolved per-row/per-variable budgets as the outer problem (targets from `numerics::resolve`), `derivative_tolerance = policy.kkt.stationarity`, shared controls iterations/time, default `ResolvedAccuracy::from_policy(Default)`. There is no looser early-iteration tolerance, no tolerance tied to the outer iterate's progress, and no inexact-evaluation contract; inner accuracy equals final accuracy.
- Homotopy/stage steps: each step is judged by the same candidate-use decision and the same numerical policy as the final solve (spec §17.5: "stage and homotopy attempts answer to the model's checks, not to final-fixture expectations"); only the unchanged original specification may commit. No separate stage tolerance exists.
- Inner Krylov solves: KINSOL forcing term (`Eta`) is the only intermediate-accuracy knob exposed, Krylov routes only, defaults to choice 1 but default linear is KLU so it is inactive by default.
- `ResolvedAccuracy.feasibility` is a "conservative scalar projection" (min over normalized variable and row budgets), with explicit refusal to use raw max/min of differently dimensioned tolerances.
(interp) There is no concept of an "intermediate accuracy" tier outside the native solver's own forcing terms and `acceptable` termination.

## 6. Uncertainties and unexamined edges
- Diffsol metrics and POUNCE-convex/Clarabel/IDAS metric key lists only partly inspected (names for IDAS counters read; Clarabel `metrics()` at conic.rs:828 not read in full).
- The runner (BN/execution/runner.rs) is modified in the working tree (165 lines); I did not read its diff, so any new retry/orchestration there is unchecked. Likewise RT/math/solves.rs prepare path was not read line by line for where `Requirements` is built (only the initialization path read).
- Absence claims (no KINSOL msbsetsub/ResMon exposure; no Ipopt summary statistics; no metric consumers; no use of conditioning/BTF in selection) are based on grep over `crates/` (Rust sources) and reading routing.rs/execution/*.rs; generated code and Python were not searched for these.
- Python wrappers beyond the generated documents (`python/pse/_workflow.py`, `_strategies.py`) not read; whether they add Python-side defaults is unchecked.
- Whether `StudyContinuationPermission`/`fresh_on_unavailable` allow automatic start choice beyond author edges: only the enum declarations read.
