---
title: Plan 22 remaining solver scope — execution packet
status: in-progress
date: 2026-09-28
parent: 22-solver-capabilities.md
adrs: [ADR-0103, ADR-0104, ADR-0106, ADR-0107, ADR-0109, ADR-0110, ADR-0111]
review_sources:
  - ../design_review/reviews/design_review_solver-capabilities_2026-09-27.md
  - ../design_review/reviews/design_review_plan22-target_2026-09-27.md
---

# Plan 22 remaining solver scope — execution packet

This packet owns step progress, binding execution decisions and checkpoints for the solver
scope of [Plan 22](22-solver-capabilities.md) that remained when the
[store and typed-data track](22-store-and-typed-data-execution.md) closed with its W6 scoped
qualification. Packet definitions, sequencing and finding dispositions stay in the plan; its
*Remaining solver scope* section summarizes the packets below and links here. The
[main execution packet](22-solver-capabilities-execution.md) keeps the history of E1–E14 and
points here at E15.

**Scope:**
- S1–S4, Y3–Y5, C3–C5 and N5;
- M2 completion and M5;
- the G4 and G6 remainders;
- the solver docs pass;
- the solver-owned follow-ups:
  - dynamics (E10);
  - G8;
  - the G6 kernel gaps;
  - the review of `82874e7d`;
  - ADR-0103 Outcome 7 `semi(indicator)` for SCIP;
- findings T11, T12 and T14;
- Q1.

**Evidence level: Proposed.** The packets were planned on 2026-09-28 from three code surveys
and three design passes, and every packet was checked against source. The line numbers cited
are observations on `main` at `f87c0b5b`, not contracts.

## Maintainer decisions (2026-09-28)

1. **Topology.** Tracks run in their own worktrees, with at most four compiling at once, and
   merge into `main` at packet boundaries (AGENTS.md *Personal-project checkout workflow*: genuine
   concurrent editing). Each agent sets `CARGO_TARGET_DIR=<worktree>/target`, and each worktree
   agent uses its own Python environment, never the main `.venv`.
2. **Solver image.** The maintainer confirmed that the solver files are final, so the image
   was published ahead of W7 rather than before Q1: `solvers-image.yml` run 36500279506
   pushed `ci-ccfd084f7324` and `dev-ccfd084f7324`, pinned in `b0aed3ef`. Native recipes
   need no `PSE_SOLVER_IMAGE` override any more. Its solver prefix matches the local
   image the earlier Plan 22 work was tested against: 21 of 22 shared libraries are
   byte-identical, and `libscip` differs only in an embedded build-host string. If C4's
   image check needs `libmkl_rt`, the image is rebuilt, re-published and re-pinned with
   ADR-0122.
3. **Improve the target where the principles or library use allow** (the request for this
   plan). Improvements I1–I16 below are adopted with this packet. The ones that change an
   accepted decision are written as ADRs before their dependent packets start.

## Improvements over the target design

| # | Improvement | Principles | Replaces | Route |
|---|---|---|---|---|
| I1 | **One KKT-point analysis for every route.** An active-set KKT in original coordinates, assembled from the model's derivative programs and factored by FERAL. `pounce-sens-core` (`SensApplication`, `parametric_step`, reduced Hessian with eigenvectors) runs over a FERAL `SensBacksolver`. It serves Ipopt, POUNCE, the SCIP fixed-assignment re-solve and QP routes, and removes the presolve restriction | AP-02, DP-01, DP-13, DP-16, PS-12 | Two routes: `pounce-sensitivity` `SensSolve`, which is absent from the registry cache and the lockfile, with its API known only from the `pounce-rs` facade; and an Ipopt barrier replica. S2 is deleted | ADR-0118, superseding ADR-0107 |
| I2 | **One `LocalValidity` named structure in its own relation**, keyed by run, step and quantity. A withheld quantity has no data rows, so its reason needs a home of its own | PS-12, AP-04, DP-02 | Validity columns defined in each relation | ADR-0118 |
| I3 | **One pin transformation (`Pinned`) and one `Commitment` record.** They serve the SCIP conditional re-solve, S1 parameter pins, S3 profile pins and the HiGHS `FixedLp` and SCIP conditional duals | DP-01, AP-03 | A fixed-assignment path private to `execution/factorable.rs` | within ADR-0103, ADR-0118 |
| I4 | **Covariance by one rule.** Exact when the fit used the exact Hessian, Gauss–Newton otherwise, computed from the existing response SVD with right singular vectors, which also gives the identifiable subspace. Profile likelihood runs as adaptive pin chains over a fit prepared once | DP-01, DP-10, DP-11 | Unspecified. JᵀWJ was implied, but it squares the condition number | within ADR-0107/0118 |
| I5 | **N5 narrowed to what it alone adds:** batched parallel QP and SOS bounds. `pounce_convex::QpSensitivity` is excluded as a duplicate of I1 | DP-01, DP-16 | A second QP-sensitivity mechanism | plan edit |
| I6 | **Scheduled inputs carry sensitivities**: one parameter per schedule interval. Today `InputChange` switches them off, which blocks adjoints, shooting and NMPC | PS-11, DP-02 | Implicit | ADR-0119 |
| I7 | **Authored schedules, events, modes and solve intent become kernel fixture data.** The runtime- and Python-only mode and event structs are deleted. An objective-bound `annotation check` reads the certified dual bound instead of starting a solve | PS-11, AP-04, AP-05 | "A check runs a certify solve"; programmatic events and inputs | ADR-0119 |
| I8 | **A shared `IntegratedExperiment`** for the fit oracle and the shooting oracle (two consumers, not one oracle). Gauss–Newton (Y4a) lands before the adjoint work, since it needs only forward sensitivities | AP-03, DP-16 | A new oracle per route; Y4 after Y3 | within ADR-0110 |
| I9 | **A Diffsol linear-solver adapter that can fail.** A singular factorization returns `Err`, so Diffsol reduces the step. This applies to KLU and faer LU alike | PS-10, DP-13, DP-21 | A contained panic (`catch_unwind`, `Termination::Panic`) | none |
| I10 | **Convexity and cones as a compiler fact.** A DCP pass runs over the `FactorableProgram` that preparation already builds, with exact rational LDLᵀ Gram certificates. SCIP curvature detection serves only as a differential test oracle. Per-class automatic ownership (`automatic_classes`) lets Clarabel own LP and QP explicitly without becoming automatic | PS-09, AP-04, DP-15 | The run-time convexity decision in `math/solves.rs`; one automatic rank per adapter | ADR-0121 |
| I11 | **Clarabel excludes `faer-sparse`**: it pulls faer 0.21.9 beside the pinned 0.24.4 and duplicates threaded direct solves. MKL Pardiso is adopted after a loading check. One typed, verified infeasibility certificate covers Clarabel and HiGHS | DP-01, DP-15, PS-10 | The faer backend; string-keyed `certificate.*` metrics | plan edit; ADR-0122 only if the image changes |
| I12 | **Complementarity reuses what exists.** `disjunctive` is SOS1 on slack columns. `smooth(ε)` is the vocabulary's `smooth_min` (CHKS) under the existing continuation. `penalty(l1)` is a structural fact that routing reads | DP-16, PS-06 | An unspecified lowering; the risk of a second smoothing function | within ADR-0104/0109; the choice is recorded at blueprint §19.7 |
| I13 | **Solutions carry a typed `origin`** (output or incumbent). `StoredStart::Latest` skips incumbent captures, captures are pruned with their streams, and durable incumbents are published | DP-02, DP-09, DP-19 | A join that breaks once incumbent rows expire (R-36) | none |
| I14 | **Native caches are budgeted.** The KINSOL session cache is counted in bytes inside the job lease, and the retained KKT factor of Y5c is charged to the job allowance | DP-20 | A count-only, uncharged cache | none |
| I15 | **Multi-objective bounds are generated β parameters** on rows selected by level facts. That gives at most K cached structures, with the bound values rebound, because bounds belong to `CaseStructure::key` | DP-10 | Structural edits per level | within ADR-0111 |
| I16 | **NMPC is one durable attempt** with per-step stored seeds. The plant runs on the controller's `NativeSession`, and the advanced step reuses the retained I1 factor. O7 predecessor chains were rejected: the steps are strictly sequential and share retained state | AP-03, DP-19 | Unspecified | within ADR-0110 |

**Rejected, with reasons:**
- **A check that starts its own solve.** Analysis selection stays outside the model (PS-11).
- **A pre-factorization singularity check.** It would be bespoke and incomplete.
- **SCIP `bounddisjunction` for semi domains.** ADR-0103 chose the indicator form, and no benefit was measured.
- **Scholtes smoothing.** It is degenerate for interior-point methods.
- **Fischer–Burmeister smoothing.** It would be a second function with the same meaning.
- **Both a Gauss–Newton and an exact covariance for one steady fit.** That gives two answers to one question.

## Decisions to write first (coordinator, start of W7)

Numbers are assigned by `just adr-new`, from ADR-0118 on. No dependent packet starts before its
record is accepted.

| Record | Content | Review | Unblocks |
|---|---|---|---|
| ADR-0118 One KKT-point analysis | Supersedes ADR-0107. It restates Outcomes 1, 4, 6 and 7, and replaces Outcomes 2, 3, 5 and 8 with I1–I3. It records that ADR-0109 item 4 is delivered by this mechanism. It amends blueprint §19.4, §19.8 and §25 | Author review under the process-simulator profile (PS-07, PS-10, PS-12) | S1; S0 may start earlier |
| ADR-0119 Kernel fixtures declare analysis selections | Solve intent; integration schedules; events with direction; modes; the certified-bound basis of objective-bound checks (I6, I7). Within ADR-0106 and ADR-0110 | Short | G6r kernel, Y0c kernel, Y0d |
| ADR-0120 Provider envelope contract | `ProviderFactory::envelope`, the G6 kernel-gap note | Short | — |
| ADR-0121 Convexity and cone recognition as compiler facts | I10, including the reason for a bespoke DCP pass (core §F) and `automatic_classes`. Amends blueprint §18.9 | Short, author review | C5 |
| ADR-0122 (only if needed) | Refines ADR-0108's image composition, if Clarabel's MKL Pardiso needs `libmkl_rt` | Short | The MKL part of C4 |

## Packets

Each packet does four things:
- compiles with `just check-package <pkg>` or `just check`;
- runs its named targeted tests, using `just unit-package <pkg> <filter>`, `just native-test -E '…'` with `PSE_SOLVER_IMAGE` set, or `just db-test`;
- runs `just codegen` after registry edits, plus `just python-stubs` where the Python surface changes;
- deletes what it replaces in the same change.

### Track T-N — NLP analysis and fitting

**R82874 — review of `82874e7d`.**
- The reuse rule is correct: every solve re-applies all of its own option values after the reuse decision.
- Changes:
  - the `RequireReuse` refusal names the dropped option keys as a typed reason, instead of "bounds or layout changed";
  - a new test, `reused_problem_reapplies_changed_values`;
  - the Ipopt/POUNCE option-reset difference is recorded at blueprint §18.3 (DOCS-a).

**S0 — KKT point analysis** (`pse-backend-native`; a refactor within existing contracts).
- New `src/kkt.rs`:
  - layout `[x; active rows; active bound rows]`;
  - `KktPoint` data: strong, weak or inactive activity by original identity; LICQ from the inertia of `[I Aᵀ; A 0]`; curvature, inertia, condition estimate and backsolve residual;
  - `KktFactor` = `Arc<feral::Solver>` plus the matrix and layout; it implements `SensBacksolver` by `solve_refined`.
- `conditioning.rs` keeps `jacobian_condition` and the generic `kkt()`. It loses `second_order`, `Active` and `attach_second_order`.
- `Evidence::second_order` becomes `Evidence::local: Option<KktPoint>`. `execution/runner.rs` takes an `Analysis` request, and `presolve/pipeline.rs::finish` runs it.
- `factorable::adopt` carries `local` over from the fixed-assignment re-solve; today it is dropped.
- The factor never enters `SolveReport`. It is dropped after the step, or moved into `Retained` for Y5c and charged to the job allowance.
- Cargo: `pounce-sens-core =0.12.0`.
- **Tests:**
  - adapted: `kkt_inertia_certifies_second_order`, `second_order_verdicts_follow_the_inertia`;
  - new: `kkt_factor_backsolve_matches_dense_reference`, `licq_failure_distinct_from_singular_curvature`, `fixed_assignment_resolve_keeps_local_analysis`.
- **Deletes:** the two-pass assembly and the `second_order.unavailable` text metric.

**S1 — parametric sensitivity and reduced Hessian** (after S0 and ADR-0118).
- **Preparation.** `pse-math` `assembly.rs` gains `CasePlan::parametric`, which keeps the objective and takes as coordinates the free variables plus the requested parameters. `pse-compiler` gains `prepare_modeling_parametric`, cached with the A6 view.
- **Pinning.** New `src/transform.rs` holds `Pinned` and `Unconstrained`, moved out of `execution/factorable.rs`.
- **Computation.** `IndexSchurData` over the pin rows feeds `parametric_step` or `solve_many`. The pin multipliers give df*/dp, `compute_reduced_hessian_eigen` gives the reduced Hessian, and results map back through `Normalization`.
- **Registry** (`catalog/native_math.rs`, `declare_local_analysis`):
  - enums `DerivedQuantity`, `WithheldReason`, `CovarianceApproximation`, `IntervalMethod` and `IntervalEnd`;
  - the named structure `LocalValidity`;
  - relations `runtime.local_validity`, `runtime.parametric_sensitivities` and `runtime.reduced_hessians`;
  - `DualQualification` gains `sensitivity_certified`.
- **Tests:**
  - `sensitivity_matches_analytic_nlp` and `sensitivity_withheld_when_sosc_fails`;
  - `sensitivity_withheld_when_weakly_active` and `sensitivity_withheld_when_licq_fails`;
  - `sensitivity_backend_independent` (Ipopt, POUNCE and the SCIP re-solve);
  - `sensitivity_survives_presolve`;
  - `reduced_hessian_sign_pinned` (gh#937);
  - `sensitivity_withheld_when_multiplier_fails_complementarity`, a negative control for a postsolved multiplier that fails original-coordinate complementarity (review finding F01, ADR-0118 Outcome 7);
  - `sensitivity_agrees_with_ipopt_sens` runs in Q1, in the parity container.
- **Packet-start checks:** postsolve recovers the multipliers of rows that presolve removed; each parameter's scale has a known source.

**S3 — covariance and intervals** (`pse-runtime` fitting; after S1).
- **SVD.** `fitting/oracle.rs` also returns right singular vectors. Gauss–Newton Σ = S·V·diag(s⁻²)·Vᵀ·S, and the same SVD gives the identifiable subspace.
- **Exact covariance.** The exact Σ is the B·K⁻¹·Bᵀ block of the fit's KKT analysis.
- **Reporting.** `FitReport.covariance` carries its approximation.
- **Intervals.** Wald intervals, plus a new `fitting/profile.rs`: 2·np adaptive pin chains with homotopy step control and `statrs` quantiles.
- **Relations:** `runtime.parameter_covariances` and `runtime.parameter_intervals`.
- **Tests:**
  - `linear_regression_covariance_analytic`;
  - `unidentifiable_fit_withholds_covariance`, on the vessel-fitting fixture;
  - `gauss_newton_covariance_labelled` (T11);
  - `profile_likelihood_matches_wald_on_linear_model`;
  - `profile_chain_seeds_from_predecessor`;
  - `covariance_withheld_without_declared_sigma`.
- **Statistical model** (review finding F02, ADR-0118 Outcome 2): covariance requires unit importance for every included observation, and is withheld otherwise. Test: `covariance_withheld_with_nonunit_importance`.

**S4 — uncertainty propagation** (after S3).
- New `workflow/uncertainty.rs` computes Σ_y = J·Σ_θ·Jᵀ, taking J from S1 or from `FitReport.responses`.
- Validity is the conjunction of the upstream validity rows.
- New relation: `runtime.propagated_covariances`.
- **Tests:** `uncertainty_propagation_linear_exact` and `propagation_withheld_when_upstream_withheld`. The `sens.py` parity case runs in Q1.

### Track T-Y — dynamics

**Y0a — typed factorization failure** (I9).
- New `dynamics/linear.rs`: a pse-owned Diffsol `LinearSolver` over faer LU and KLU (`suitesparse_sys`) that returns `Err` from `solve_in_place`.
- **Deletes:** the `diffsol::KLU` and `FaerSparseLU` choices.
- **Tests:** `diffsol_singular_factorization_is_typed_numerical` (both solvers) and `diffsol_klu_matches_faer_lu`.
- An upstream issue is filed at the maintainer's call.

**Y0b — KINSOL cache in bytes** (I14).
- `Session::retained_bytes()`. The cache evicts least recently used sessions by bytes, up to `MathPolicy.inner_session_bytes`, which joins the job and session leases.
- **Deletes:** `CAPACITY`.
- **Tests:** `inner_session_cache_bounded_by_bytes` and `nested_inner_solver_reuses_session`.

**Y4a — `HessianMode::GaussNewton`** (I8).
- A new registry value. The Ipopt, POUNCE and routing adapters treat any mode other than `LimitedMemory` as a supplied Hessian.
- Admitted only for least-squares fits.
- `FitOracle::hessian` = the `gram.refill` JᵀWJ plus the constraint-multiplier Hessians, without residual curvature. The derivative source is recorded (PS-07).
- **Tests:** `gauss_newton_hessian_matches_jtwj` and `gauss_newton_fit_admits_transient`.

**Y0c — scheduled inputs with live sensitivities** (I6). Its backend part runs in T-Y; its kernel part runs in T-L after ADR-0119.
- **Backend.** `Contract` and `Profile` gain `schedule`. Per-interval parameter columns are mapped per segment in the Diffsol `Operator` and in IDAS `IDASensReInit`. One `Profile::parameters_at(t)` replaces the two current copies.
- **Kernel.** The fixture gains `integration.schedules`, with the grammar `schedule u at(...) values(...)`. The frame `DynamicProfileV3` becomes V4.
- **Deletes:** `InputChange`, `Profile.changes` and `parameter_active`.
- **Tests:** `scheduled_input_sensitivities_cross_changes` (Diffsol and IDAS against finite differences) and `kernel_fixture_schedules_inputs`. The PETSc PID test and the p09 fit move to the new form.

**Y3a — Diffsol adjoint** (after the Y0c backend).
- **Operator traits.** `NonLinearOpAdjoint` and `NonLinearOpSensAdjoint` are built from the one CSC by faer transpose products, overriding the dense defaults. `ConstantOpSensAdjoint` goes on `Init` and `LinearOpTranspose` on `Mass`.
- **Registry:** `DynamicSensitivity{none, forward, adjoint}` and `AdjointSettings`.
- **Checkpoints.** Our driven loop records them, at least two per segment, and chains segments backward through `*_solver_adjoint_from_state`.
- **API.** `dynamics::gradient(...)`. Events with resets are refused.
- **Fitting.** `FitProfile.derivatives{responses, gradient}` (`pse.fit.profile.v3`). The gradient mode requires `LimitedMemory` and reruns forward once for rank (PS-12).
- New `workflow/integrated.rs` (`IntegratedExperiment`).
- **Tests:** `adjoint_gradient_equals_forward_on_transient_fit` (also against finite differences) and `checkpoint_memory_bounded`.

**Y3b — IDAS adjoint.**
- Uses `IDAAdjInit(Nd, IDA_HERMITE)`, `IDACreateB`/`IDAInitB`, `IDASetLinearSolverB` (KLU over the transposed pattern), `IDASetJacFnB`, `IDAQuadInitB` and `IDACalcICB`.
- `IDAReInitB` at measurement jumps. Each segment runs `IDAReInit` + `IDAAdjReInit` + `IDASolveF`.
- A checkpoint estimate above `Execution.memory` is refused before any native work.
- **Tests:** the Y3a tests, run on IDAS.

**Y4b — exact transient Hessian** (after Y3b).
- `Oracle::weighted_hessian` and `Contract.derivatives`. `DynamicWorker` prepares at second order on request.
- IDAS `IDAInitBS`/`IDAQuadInitBS`. `FitOracle` = the gram plus the second-order adjoint term. Exact is refused on Diffsol.
- **Deletes:** the transient refusal in `fitting/oracle.rs` `hessian()`.
- **Test:** `second_order_adjoint_matches_finite_difference`.
- **Packet-start check:** backward Newton with a block-diagonal Jacobian versus SPGMR.

**Y5b — shooting** (after Y3a).
- New `dynamics/anchored.rs`, which treats the start state as parameters.
- New `workflow/shooting.rs` on `IntegratedExperiment`: node states and continuity rows, with path bounds at samples.
- **Registry:** `ModelingFixtureExecution::shooting` and `ShootingMethod`.
- **Tests:** `shooting_matches_simultaneous_optimum` (single and multiple shooting) and `multiple_shooting_continuity_closes`.

### Track T-C — conic and routing

**C4 — Clarabel** (I10, I11).
- **Settings.** `ConicProblem::from_coefficients`; `Settings.direct ∈ {qdldl, mkl_pardiso}`; threads above 1 are refused with QDLDL; the stale "serial-netlib" strings are fixed.
- **Classes.** The classes become [Linear, ConvexQuadratic, ContinuousCone], with a new `Capability.automatic_classes`. `routing::problem_classes` orders classes most specific first. The coefficients runner lowers to cone form when the adapter is conic, and keeps the original re-check.
- **Certificates.** `InfeasibilityCertificate{kind, accuracy, ray, verification}` is checked in original coordinates, for Clarabel and HiGHS rays alike.
- **Registry and Cargo:** `solver_capabilities.automatic_classes`, `runtime.infeasibility_certificates`, and the Cargo feature `clarabel/pardiso-mkl`.
- **Tests:**
  - `clarabel_lp_matches_highs`;
  - `clarabel_qp_farkas_certificate`;
  - `farkas_certificate_verified_in_original_coordinates`;
  - `almost_infeasible_is_not_certified`;
  - `clarabel_mkl_pardiso_matches_qdldl`, which replaces the faer test;
  - `clarabel_chordal_matches_undecomposed`;
  - `explicit_only_classes_never_automatic`.
- **Deletes:** the routing assertions that Clarabel refuses LP, the `certificate.*` metrics and `Certificate.kind: String`.
- **Packet-start checks:**
  1. Run `nm -D` on the image's `libmkl_intel_lp64` and `libmkl_core` for `pardiso_` and the thread symbols.
  2. If they are exported, point `MKL_PARDISO_PATH` at the loaded library.
  3. Otherwise, add only `libmkl_rt`, with `MKL_THREADING_LAYER=GNU` and `MKL_INTERFACE_LAYER=LP64`, rebuild the image and write ADR-0122.
  4. Extend `single_blas_provider_in_process` to assert that no `libiomp5` is loaded.

**C5 — cone recognition** (after C4 and ADR-0121).
- **DCP pass.** New `pse-math/src/curvature.rs` applies DCP rules over `FactorableProgram`, with signs from the FBBT box:
  - exp, log, entropy and relative entropy → K_exp;
  - pow → power cone;
  - abs;
  - norms → SOC.

  Only `Exact` fidelity counts.
- **Gram certificates.** An exact rational LDLᵀ with symmetric pivoting in `convexity.rs` (`MathGramV2`, factors kept).
- **Facts and routing.** `ProblemFacts.convexity` is carried by `ValueProducts`, so it rebinds with values. Routing emits `ConvexQuadratic` and `ContinuousCone` from the fact.
- **Deletes:** the run-time exact branch in `math/solves.rs` and `Requirements.convex`. `ConvexityPolicy::Numerical` stays opt-in and never becomes a fact.
- **Tests:**
  - `gram_certificate_yields_soc`;
  - `exact_ldlt_certifies_nondiagonal_psd`;
  - `recognized_exp_cone_routes_to_clarabel`;
  - `unrecognized_problem_not_routed`;
  - `convexity_fact_rebinds_with_values`;
  - `numerical_psd_only_under_explicit_policy`;
  - `curvature_sound_against_scip_oracle`.
- **Packet-start check:** rerun the census for coverage and preparation-time cost.

**M5b — ℓ1 selection by fact** (after M5a).
- `Ineligible::Method` (`NativeIneligibility::method`).
- POUNCE admission records the author's ℓ1 selection, and the result states that the penalty covers every constraint (T14).
- **Tests:** `authored_l1_realization_selects_route` and `l1_never_automatic`.

**N5 — POUNCE-convex** (I5).
- **Backend.** The pounce-rs `convex` feature; `NativeBackend::pounce_convex`; `Capability.batch`; new `execution/pounce_convex.rs` and `settings/pounce_convex.rs` with an IPM for LP, QP and cones, warm start, explicit selection only.
- **Batching.** A batch entry calls `solve_qp_batch_parallel_warm` with the admitted permits. `engines.rs::study` groups points that share a view key, and each point is still qualified on its own.
- **SOS bounds.** New `execution/sos.rs` applies `sos_constrained_lower_bound` over G2 polynomials, labelled `sos_bound_nonrigorous`.
- **Tests:**
  - `pounce_convex_qp_matches_highs`;
  - `pounce_convex_batched_study`;
  - `sos_bound_labelled_nonrigorous`;
  - `pounce_convex_never_automatic`;
  - `qp_sensitivity_through_kkt_analysis`.
- **Packet-start check:** the capability record states the cancellation granularity.

### Track T-G — SCIP and global

**G-semi — ADR-0103 Outcome 7.**
- **Lowering.** `execution/factorable.rs` replaces `Refusal::SemiDomain` with a binary z and the rows x − u·z ≤ 0 and x − l·z ≥ 0, plus `Origin::SemiLink`.
- **Surrounding changes.** The fixed-assignment re-solve fixes the branch, `assess()` checks semi membership, and the transformation is recorded (A4).
- **Tests:**
  - `semi_indicator_lowering_matches_highs_native`;
  - `semiinteger_lowering_keeps_integrality`;
  - `semi_minlp_fixed_assignment_resolve`;
  - `semi_transformation_recorded`;
  - `exact_mode_accepts_semi_lowering`.

**Epigraph fix.**
- With a nonlinear objective, three values are evaluated at the solution instead of read with `SCIPgetSolOrigObj`: the candidate objective, the incumbent event and the solution pool.
- Gap claims already re-evaluate the objective.
- **Test:** `epigraph_incumbent_reports_function_value`.

**G4r.**
- Adds `heater_optimization_certified` with a declared `math_foreign_bytes`; the default is 64 MiB.
- Measure peak memory and time first. If the gap cannot close in bounded time, the test asserts `global_bound` with its recorded gap (PS-12).
- It becomes an `intent certify` fixture after the G6r kernel.

**G6r tests** (after the G6r kernel).
- **Tests:**
  - `tpd_detects_known_instability` (Peng–Robinson, with teqp as the reference);
  - `tpd_certifies_stable_feed` as an authored fixture;
  - `fixture_intent_selects_certify`;
  - `fixture_policy_intent_conflict_refused`;
  - `objective_bound_check_uses_certified_bound`.
- **PC-SAFT.** Measure the body slots it needs, then add `SpecializationLimits.body_slots`, capped by `MAX_FORMAL_SYMBOLS`. If the need exceeds 4096 at an unacceptable start-up cost, record an excluded capability with a revisit trigger.

**G8f — store follow-ups** (I13).
- **Store changes.** New column `operational_solutions.origin`; `latest_compatible` returns only `output`. The expiry sweep prunes unreferenced captures, which narrows R-36. `runtime.incumbents` is published into Delta.
- **Regeneration.** `just --yes db-reset`, then Cornucopia regeneration in bootstrap order.
- **Tests:** `latest_start_skips_incumbent_captures`, `captured_solutions_pruned_with_streams` and `published_incumbents_equal_stream_snapshot`.

**SCIP concurrency.**
- **Test:** `scip_concurrent_streams_incumbents`.
- The blueprint §18 text changes from "serial".

### Track T-L — language and kernel

This track owns `parser.rs`, `catalog/modeling.rs`, `pse-modeling/src/specialize` and compiler modeling.

**M2a — domain admission.**
- `admit_domains` tightens integer and semi-integer bounds to ceil and floor, and records the change as `DomainTightening` with the info diagnostic `modeling.domain.tightened`.
- Binary bounds outside [0, 1] are refused (`DomainRefusal::ConflictingBound`) instead of clamped.
- **Tests:** `integer_bounds_tightened_inward_and_recorded` and `binary_bounds_outside_unit_box_refused`.

**G6r kernel** (ADR-0119, ADR-0120).
- An optional fixture clause `intent certify;`. A conflict with `ModelingFixturePolicy` is a typed refusal.
- The compiler classifies objective-bound checks. `results.rs` evaluates them against the certified bound, with `runtime.modeling_checks.basis ∈ {point, global_bound}`.

**C3 authoring** (ADR-0111).
- `annotation objective` gains priority, weight, normalization and per-level tolerances.
- The single-objective refusals are deleted.
- An `objective_bounds` transformation generates fⱼ − βⱼ ≤ 0 rows, selected by `objective.level` facts (I15).

**M5a — complementarity** (I12).
- **Declaration.** `complements name[idx]: (a >= 0, b >= 0);` is a registry declaration kind with the realizations `smooth`, `penalty_l1` and `disjunctive`, and no default.
- **Lowerings:**
  - `smooth(ε)` → `smooth_min(a, b, ε) == 0`, with ε continued;
  - `disjunctive` → slack columns plus a native SOS1;
  - `penalty(l1)` → rows plus a requirement fact (case-structure v4).
- **Package data.** A `ComplementarityVLE` in `equilibrium.pse`, with behaviour read from IDAES `smooth_VLE_2.py` only, and flash fixtures in all three forms.
- **Tests:**
  - `complements_parses_and_renders`;
  - `smooth_complementarity_product_equals_eps_sq_over_4`;
  - `disjunctive_complementarity_refused_on_highs`;
  - `flash_phase_disappearance_agrees_across_realizations` (3 feeds × 3 realizations).

**ENV — provider envelope enforcement** (ADR-0120 items 5–7, review finding F04; `pse-kernels`).
- Each envelope interval must contain a real number: (+∞, +∞) and (−∞, −∞) are refused.
- The host checks every provider evaluation against the declared envelope; a violation is a typed `Contract` error.
- The host includes the envelope in the provider's configuration key.
- **Tests:** `envelope_rejects_empty_interval`, `envelope_violation_is_typed_contract_error`, `envelope_in_provider_key`.

**Y0c kernel and Y0d** (ADR-0119).
- **Schedules** as in Y0c.
- **Events.** The registry `EventDirection` replaces `Crossing`. Fixture clauses `mode …` and `event guard direction tolerance reset(...) next(...)`.
- **Sign constraints.** IDAS sign constraints come from constant-zero bound annotations; the guard stays the validity authority. Authored bounds are the only source: the per-state `IdasSettings.constraints` vector is derived and is deleted as a runtime and Python setting (review finding F03, ADR-0119).
- **Deletes:**
  - `ModelingDynamicMode`/`ModelingDynamicEvent` as inputs;
  - `FitProfile.modes` and `declared_simulation_modes`;
  - the Python mode and event settings and `modes=`.
- **Tests:**
  - `authored_directional_event_routes_to_idas`;
  - `idas_sign_constraints_from_authored_bounds`;
  - `simultaneous_route_refuses_authored_events`;
  - the kernel events test, rewritten on the fixture.

### Track T-M — staged engines

This track owns `workflow/staged.rs`, `math/staged.rs`, the objectives in `highs.rs`, and the objective parts of the `pse-math` binding and assembly.

**M2b — initialization.**
- `ModelingInitialization.discrete ∈ {Refuse, FixAtStart, FixAt(values)}` is merged into every stage and homotopy `Overlay`, and restored on every exit.
- **Tests:** `initialization_fixes_and_restores_integers` and `initialization_refuses_nonintegral_discrete_start`.

**M2c — conditional results** (after S0).
- One `Commitment` covers HiGHS `FixedLp` and the SCIP re-solve.
- `solve_runs` v5 gains a `commitment` column.
- `Pinned::discrete` replaces `AlgebraicOracle::with_fixed_assignment`.
- **Tests:** `duals_conditional_on_assignment` (HiGHS and SCIP) and `sensitivity_conditional_on_assignment`.

**C3 engine.**
- **Binding.** `binding.rs` gets `objectives: Vec<ObjectiveMember>` and `Target::Objective(index)`, with `MathCaseStructureV4`.
- **Native lexicographic route.** `CoefficientProblem.objectives` is passed through `Highs_passLinearObjectives`.
- **Staged route.** New `workflow/objectives.rs` runs the levels on `Staged`.
- **Relation:** `runtime.objective_levels`.
- **Tests:**
  - `lexicographic_milp_native`;
  - `lexicographic_nlp_staged_matches_weighted_limit`;
  - `lexicographic_degradation_tolerance_respected` (T12);
  - `dimensional_weighted_sum_refused`;
  - `competing_objectives_without_priority_refused`;
  - `zero_tolerance_refused_on_staged_level`;
  - `objective_levels_prepare_once_per_level`.
- **Packet-start check:** compare HiGHS's equal-priority handling and tolerance formula with ADR-0111 item 3.

**Y5a — simultaneous dynamic optimization.**
- A new `SaturatedProcess` in `control-fixtures.pse`: a bounded control with an integral tracking objective. The same definition is also integrated with the control scheduled.
- **Tests:** `simultaneous_dynamic_optimization_matches_analytic`, and `…_with_discrete_decision` after M2.

**Y5c1 — NMPC and MHE** (I16).
- **Driver.** New `workflow/horizon.rs` and `Runtime::start_horizon`: one durable attempt with per-step O6 seeds. `Staged::step` gains a native `Predecessor` (N2).
- **Stepping.** The overlay rebinds x0 and the setpoints or measurements. The plant step runs on the same `NativeSession`.
- **MHE** uses an arrival cost.
- **Progress** is reported as O5 `horizon.step` events.
- **Tests:** `nmpc_closed_loop_on_antiwindup`, `horizon_reuses_prepared_view` and `mhe_recovers_initial_state`.

**Y5c2 — advanced step** (after S1).
- S1's parametric step runs on the retained factor with respect to x0.
- An active-set change falls back to a full solve, which is recorded.
- **Test:** `advanced_step_matches_full_resolve`.

### Track T-D — documentation, decisions and closure (coordinator)

**DOCS-a** (W7, one blueprint revision) covers work that has already landed:
- §18.3: N1–N3 and the reuse rule;
- §18.6 and §18.7: the A5 reason codes;
- §18.8: MKL and SPRAL threads;
- §18.9: the capability matrix;
- §18.10: pse-owned conic types, with the netlib text removed;
- §18.10.1: G4–G7 and SCIP concurrency;
- the §3.3 and §25 rows;
- identity versions: case-structure v3, backend settings v3, native seed v2;
- `validate_profile` admission at §17.1.

Architecture edits use `PSE_DESIGN_EDIT=1`, with one revision row per wave.

**After each wave:** a docs step for that wave's packets.

**DOCS-z** (W12):
- consolidation;
- the plan's Outcome draft;
- image publication (maintainer decision 2).

## Waves and coordination

The coordinator:
- reviews each merge;
- after any merge that touches the registry, runs `just codegen` (and `just python-stubs`) — generated conflicts are resolved by regeneration, never by hand;
- reruns the packet's targeted tests on `main` with no concurrent builds;
- records the checkpoint here.

**One registry token.** Only one track at a time edits `pse-schema`, the root `Cargo.toml` or
`Cargo.lock`.

**Files edited by more than one track**, in separate hunks, merged by the coordinator:
- `execution/runner.rs` (S0, C4);
- `execution.rs` (C4, N5);
- `solve.rs` (S0, C4);
- `catalog/native_math.rs` (S1, C4, N5, C3).

| Wave | Track 1 | Track 2 | Track 3 | Track 4 | Coordinator |
|---|---|---|---|---|---|
| W7 | T-N: R82874 → S0 | T-Y: Y0a → Y0b → Y4a → Y0c backend | T-C: C4, with the image check first | T-L: M2a → G6r kernel | ADR-0118–ADR-0121 (and ADR-0122 if needed); DOCS-a |
| W8 | T-N: S1 | T-Y: Y3a → Y3b | T-G: G-semi → epigraph → G4r → G8f | T-L: ENV → C3 authoring → M5a | docs step |
| W9 | T-N: S3 → S4 | T-Y: Y4b → Y5b | T-C: C5 → M5b → N5 | T-M: M2b → M2c → C3 engine | docs step |
| W10 | T-G: G6r tests, PC-SAFT slots, SCIP concurrency | T-L: Y0c kernel and Y0d | T-M: Y5a | — | docs step |
| W11 | T-M: Y5c1 → Y5c2 | — | — | — | docs step |
| W12 | — | — | — | — | DOCS-z; re-publish and re-pin the image only if C4 changed it |
| W13 | Q1 | | | | |

**Environment at the start of W7:**
- the doctor passes on the native build (fixed in `9cab2ca2`); rerun `just py-sync-native`
  if a plain `uv sync` has replaced the native extension;
- the pinned images are pulled (`just bootstrap-solvers`), so no override is needed;
- `just db-status`.

**Agents** run as `general-purpose` with the skills they need and Context7:
- `native-solver-libraries` and `symbolica-faer-oximo`;
- `pyomo-and-solvers` for IDAES behaviour;
- `datafusion` and `deltalake` for G8f.

## Verification

**Per packet:**
- the named targeted tests, run through the recipes that own their features and paths;
- `just adr-lint` for the decision records.

No formatting, lint or integration suites run mid-plan (AGENTS.md *Execution rhythm*).

**Q1** (W13) runs on the pinned published image. Each check is reported against the zero
baseline with its command and conditions.
- **Formatting and lint:** `cargo fmt --check`, `just clippy`, `just quality`.
- **Static checks:** `just governance`, `just codegen-check`, `just adr-lint`, `just docs`.
- **Suites:**
  - native-acceptance conformance, including `authored_publication_resource` and global certification;
  - the invariant harness;
  - `just db-test`, `just worker-test` and `just publication-test`;
  - Python component and integration tests.
- **Parity** (`just parity`, on this machine): DegeneracyHunter, PETSc PID, `sens.py` (`sensitivity_agrees_with_ipopt_sens`) and parmest covariance.
- **New `.config/process-cases.json` cases:**
  - authored MILP;
  - small MINLP and GDP;
  - heater certify;
  - Peng–Robinson TPD;
  - flash complementarity in three forms;
  - a recognized cone case;
  - a semi-domain MILP on SCIP;
  - IDAS PID;
  - an NMPC horizon;
  - sensitivity and covariance;
  - a large conic case;
  - a 10 000-point study on four workers.
- **Measured:**
  - the large-KKT comparison (MUMPS+METIS, SPRAL, MKL Pardiso and FERAL at n_KKT ≥ 10⁴);
  - Clarabel QDLDL against MKL Pardiso threads.
- **Closing:** the Outcome, moving enduring meaning into the architecture sections, and retiring the plan and its reviews (ADR-0096).

## Progress

| Step | Packets | State |
|---|---|---|
| W7 | ADR-0118–ADR-0121; DOCS-a; R82874 → S0; Y0a → Y0b → Y4a → Y0c backend; C4; M2a → G6r kernel | complete: decision records `68f4ea6d`, DOCS-a `5143f567`, R82874 and S0 `407103b1`, C4 `7cffc2e9`, Y0a–Y0c `fbd03320`, M2a and G6r kernel `4e0d2107` |
| W8 | S1; Y3a → Y3b; G-semi → epigraph → G4r → G8f; ENV → C3 authoring → M5a | complete: S1 `30adcd0a`, Y3a–Y3b `58fd9e66`, ENV, C3 authoring and M5a `ff65741d`, T-G with G6r tests, PC-SAFT slots and SCIP concurrency `55b75e70`; DOCS-b `8e696985` |
| W9 | S3 → S4; Y4b → Y5b; C5 → M5b → N5; M2b → M2c → C3 engine | complete: S3–S4 `1c4e18e3`, Y4b and Y5b `8c25bebb`, C5 and M5b `91683d83`, M2b, M2c, C3 engine and Y5a `064d204a`, N5 `f021f0c6` |
| W10 | G6r tests, PC-SAFT, SCIP concurrency; Y0c kernel and Y0d; Y5a | complete: Y0c kernel, Y0d and authored shooting `667b9fae`; the rest landed with W8 and W9 |
| W11 | Y5c1 → Y5c2 | complete: Y5c1 `8da0cff5`, Y5c2 `5a37033d` |
| W12 | DOCS-z; re-publication only if C4 changed the image | complete: DOCS-z (blueprint revision 73); the image did not change after `ci-2893f027e09f` |
| W13 | Q1 | running |

## Current checkpoint (2026-09-29)

**State.** Planned; nothing implemented. The solver image is published and pinned
(`b0aed3ef`), and the doctor recognizes the native build (`9cab2ca2`). W7 starts with the
decision records.

**W7 started (2026-09-28).** Five worktrees branch from `0fac35d9` (`plan22/w7-<track>`):
- T-D writes ADR-0118–ADR-0121 and their change-tier review first; DOCS-a follows after the records land;
- T-N, T-Y, T-C and T-L compile in parallel.

Each worktree has its own target directory and dependency venv. The shared agent brief sits
beside the worktrees; it is coordination scaffolding and is not tracked.

**Decision records landed (merge `68f4ea6d`).** ADR-0118–ADR-0121 were accepted after the [change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md) returned Accept. Its findings F01–F05 were corrected in the records before acceptance and fed into these packets:
- F01 → an S1 negative control;
- F02 → an S3 test, replacing that packet's start check;
- F03 → a Y0d deletion;
- F04 → the new ENV packet;
- F05 is settled in ADR-0121 itself: an explicit numerical-convexity request stays a per-request routing input.

Blueprint revision 69. `just adr-lint` exits 0 on `main`.

**W7 complete; build infrastructure changed (2026-09-29).**
- **Code tracks.** T-N, T-C, T-Y and T-L merged through a warm scratch worktree. Generated conflicts were resolved by regenerating from the merged registry (`codegen-contracts`, then `codegen`) plus `conformance-fixtures`. The only semantic conflict was C4's routing test literal missing Y4a's `least_squares`.
- **Compiler cache** (`b3fa3141`). Shared sccache hits across checkouts were 0% for Rust, because every checkout exported an absolute `CARGO_TARGET_DIR` and sccache keys `CARGO_*` env. The recipe environment no longer exports the default.
- **Toolchain and features** (ADR-0122, merge `a75bcf71`): pinned nightly-2026-09-29, `-Z feature-unification` with `feature-unification = "workspace"`, and cargo-hakari (`pse-workspace-hack`). Distinct builds of symbolica, datafusion and arrow-array across the seven recipe selections drop from 7 to 4.
- **Shared build directory rejected.** It was tried and dropped: Cargo keys workspace members by workspace-relative path with mtime freshness, so diverging worktrees reused each other's units. Intermediates stay per checkout.
- **Validation.** Compile checks only, by maintainer direction.
- **Solver image.** The dev stage now bakes the pinned nightly. Re-publication run 36514323160 is in flight, and then the pin moves.
- **Follow-ups carried:**
  - the `libmkl_rt.so` alias and `MKL_PARDISO_PATH` move into the image, replacing C4's runtime environment write;
  - fetching HiGHS rays automatically on an infeasible LP;
  - the declared-analysis path and fixture intent (T-L W7 report);
  - hakari's hand-kept skip list (arrow, the native-solver crates, pse-ids, pse-diagnostics, diffsol-la);
  - nothing compiles on the stable 1.98.1 floor any more.

**W8–W12 complete (2026-09-29).**
- **Merges.** Every track merged into `main` (commits in the progress table). Generated conflicts were regenerated, never hand-edited; semantic conflicts were fixed on the merge (C5's `numerical_psd` against shooting and routing literals; one `solver_capabilities` v3 carrying T-C's `requirements` and T-M's `lexicographic_classes`; M2b's `CaseOverrides` in the horizon driver).
- **Coordinator takeover.** From W9's end the coordinator finished the remaining work sequentially in the main checkout, at the maintainer's direction, after parallel agents' broad test runs overloaded the machine. Worktrees and merged branches were removed.
- **Y5c2** found and fixed an S0 defect: the KKT-point analysis treated an interior-point candidate's active bound as inactive when its slack `μ/z` exceeded the acceptance budget. A limit whose strong multiplier exceeds its slack is now active (`interior_point_active_bound_beyond_its_tolerance_is_active`).
- **N5** adds `batch` and `sensitivities` to the capability record; a sensitivity request is an automatic-routing preference, not an eligibility rule, so an explicit coefficient adapter keeps S1's withheld-with-reason behaviour.
- **Stale comments** listed by DOCS-b were fixed (`c5df295a`), including the Diffsol/IDAS derivative vocabulary.

**Next:** Q1 (W13), then the Outcome and retirement (ADR-0096).
