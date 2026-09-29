---
title: Target design — initialization, scaling, diagnostics, solving and studies
status: draft
date: 2026-09-26
parent: docs/plans/20-idaes-capability-target.md
review_sources: [docs/design_review/reviews/design_review_idaes-capability-target_2026-09-26.md]
---

# Target design — initialization, scaling, diagnostics, solving and studies

> **Superseded in part by [Plan 21](https://github.com/paul-heyse/pse-arrow/blob/0e725de269f18dd08331158a07b38a7d92ea0b5e/docs/plans/21-modeling-kernel.md) (2026-09-26).** Wherever this
> document assigns scientific concepts to code, the
> [modeling kernel architecture](21-modeling-kernel-architecture.md) governs. Those concepts
> are:
>
> - property kinds as a registry;
> - identity rules and closure algorithms in `pse-properties` or `pse-thermo`;
> - property-package, state-definition, costing-binding, preset or slot *types*.
>
> They are package data on the kernel's generic concepts: definitions, interfaces with default
> members, functions, sets and tables, accumulators, implicit blocks with realization policies,
> and annotations. The scientific content, coverage and scenarios here remain the target.

> **Amended by [Plan 22](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-capabilities.md) D0 (2026-09-27).** §4–§6 name the
> mechanisms decided in ADR-0102–ADR-0111: POUNCE sensitivity and `pounce-sens-core` with FERAL
> instead of a faer KKT LU, the explicit POUNCE ℓ1 route for whole-model infeasibility,
> discrete domains for price-taker and MatOpt, and DegeneracyHunter on HiGHS.

**Evidence level: Proposed.** This document is the target architecture for the numerical
knowledge and workflows that make idaes-pse models usable. It covers:

- initialization and homotopy;
- scaling;
- the diagnostics toolbox and model statistics;
- solver configuration and formulation policies;
- dynamic optimization;
- sensitivity and uncertainty;
- parameter estimation;
- sweeps and convergence evaluation;
- surrogate training;
- the applications (grid integration, NMPC/MHE, MatOpt).

It is a companion to [Plan 20](20-idaes-capability-target.md). IDAES behaviour is cited from
the `pyomo-and-solvers` skill characterization of idaes-pse 2.13.0 (`skill:`).

## 1. What must be reproduced

**Initialization.**

- `InitializerBase`/`ModularInitializerBase` lifecycle: current-state capture, fix
  initialization states, precheck, `initialization_routine`, restore, and `InitializationStatus`.
- 15 default initializers across the model records, including:
  - `BlockTriangularizationInitializer`, `SingleControlVolumeUnitInitializer` and
    `ModularPropertiesInitializer`;
  - the unit-specific Feed, HX0D, HX1D, HXNTU, Mixer, MSContactor,
    `IsentropicPressureChanger`, Separator and ShellAndTube initializers;
  - the solvent-reboiler, cross-flow HX and Heater1D initializers.
- Legacy per-unit `initialize()` step sequences, which the characterization records as ordered
  `steps`.
- `propagate_state`, `fix_state_vars`/`revert_state_vars`, `initialize_by_time_element` and
  homotopy (`core/solvers/homotopy.py`).

**Scaling.**

- `CustomScalerBase` with nominal-value constraint scaling. `ConstraintScalingScheme` offers
  harmonicMean, inverseSum, inverseRSS, inverseMaximum and inverseMinimum.
- `DefaultScalingRecommendation`.
- 21 default scalers across the model records, and about 50 `CustomScalerBase` subclasses
  overall.
- `AutoScaler`, `ScalingProfiler` and `nominal_value_tools`.
- The legacy `iscale` API with its per-package `default_scaling_factors`.

**Diagnostics.** `DiagnosticsToolbox` has 31 report, display, assert and compute methods. Its
CONFIG declares 18 thresholds (`diagnostics_tools/diagnostics_toolbox.py`):

| Threshold | Default |
|---|---|
| `variable_bounds_absolute_tolerance`, `variable_bounds_relative_tolerance` | 1e-4 each |
| `variable_bounds_violation_tolerance` | 0 |
| `constraint_residual_tolerance` | 1e-5 |
| `constraint_term_mismatch_tolerance` | 1e6 |
| `constraint_term_cancellation_tolerance` | 1e-4 |
| `max_canceling_terms` | 5 |
| `constraint_term_zero_tolerance` | 1e-10 |
| `variable_large_value_tolerance` | 1e4 |
| `variable_small_value_tolerance` | 1e-4 |
| `variable_zero_value_tolerance` | 1e-8 |
| `jacobian_large_value_caution` / `jacobian_large_value_warning` | 1e4 / 1e8 |
| `jacobian_small_value_caution` / `jacobian_small_value_warning` | 1e-4 / 1e-8 |
| `warn_for_evaluation_error_at_bounds` | true |
| `parallel_component_tolerance` | 1e-8 |
| `absolute_feasibility_tolerance` | 1e-6 |

The toolbox also provides:

- Dulmage–Mendelsohn under- and over-constrained sets;
- extreme Jacobian rows, columns and entries;
- near-parallel constraints and variables;
- mismatched and canceling terms;
- potential evaluation errors;
- infeasibility explanation;
- SVD toolbox, DegeneracyHunter (MILP), ill-conditioning certificate;
- `IpoptConvergenceAnalysis`.

`model_statistics` adds 98 counting and set functions.

**Solvers.**

- `get_solver` with `idaes.cfg` defaults. Ipopt uses `nlp_scaling_method=gradient-based`,
  `tol=1e-6` and `max_iter=200`; `ipopt_v2` adds `linear_solver=ma57` and presolve.
- `ipopt_l1`.
- PETSc SNES, TS and TAO wrappers with DAE time stepping.

**Studies and workflows.**

- `parameter_sweep`; convergence evaluation and search (`core/util/convergence`).
- Parameter estimation (Pyomo parmest usage in the examples).
- Uncertainty propagation (`apps/uncertainty_propagation/sens.py`, sIpopt/k_aug based).
- Surrogate training and sampling (PySMO, ALAMO, Keras/OMLT; LHS, Halton, Hammersley, CVT
  sampling).

**Applications.**

- `grid_integration`: bidder, tracker, coordinator, forecaster, multiperiod and pricetaker
  (design, operation and storage models).
- `caprese` (NMPC/MHE) and `nmpc` (dynamic data, cost expressions).
- `matopt`.

The IDAES *design* elements not reproduced are:

- initialization by fixing and unfixing variables in place, restored on a best-effort basis;
- scaling factors written as mutable suffixes on the model;
- diagnostics as prose reports with Python-object references;
- solver defaults hidden in a global config;
- sweeps that rebuild or re-solve without structural reuse.

## 2. Initialization

The architecture has three parts: a generic, transactional engine; declared knowledge from
models and packages; and library solvers.

The **engine** exists today
([§17](../authoritative_design/sections/numerical-execution.md#section-17)). Structural
predecessor-first block initialization, transactional stage overlays, explicit starts, tear
selection and KINSOL fixed-point recycles are kept. They are already stronger than IDAES's
in-place fix/unfix: a failed stage cannot leave the specification altered (PS-08, D13).

What is missing is a home for **initialization knowledge**. Today the only starting values are
template defaults and case values (`lower.rs:991, 1065`). Template initializers are refused
(`lower.rs:78`), and `FeosPackage::initialize_npt` has no caller. The target adds three kinds
of declaration. None of them is an imperative routine.

| Declaration | Where declared | Contract | Replaces (IDAES) |
|---|---|---|---|
| **Start estimators** | State definitions, property packages, model forms and unit templates | A pure typed expression or closure-algorithm call that produces a *proposed start* for named variables from already available values. Examples: ideal-K Rachford–Rice phase split, Wilson-K bubble and dew temperature estimates, outlet equals inlet, `T_out = T_in + Q/(F cp)`, nested flash as an estimator for EO VLE variables. Output is a start with provenance (estimator, inputs), never a fix | `ModularPropertiesInitializer` bubble and dew and flash estimates; `estimate_outlet_state`; parts of legacy `initialize()` |
| **Relaxation stages** | Templates and packages | Named, ordered formulation overlays that simplify the problem for early stages: ideal-K for rigorous VLE, isothermal energy, fixed outlet temperature or duty, deactivated pressure-drop correlation. Each stage is an overlay on the immutable case (§17.1), selected by name in the initialization policy | Unit-specific initializer sequences (the HX initializer's per-side initialization, the pressure changer's isentropic pre-solve) |
| **Homotopy policies** | Analysis policy | Continuation over declared parameters or formulation weights (ideal → rigorous blending) with **adaptive step control**, a declared minimum step, reduction factor and acceleration. The inner solves are the ordinary native routes | IDAES `homotopy()`; today only a supplied finite stage list ([§17.5](../authoritative_design/sections/numerical-execution.md#section-17-5)) |

The **initialization policy** of an analysis composes these declarations per flowsheet. Its
parts are:

1. apply start estimators in structural predecessor order;
2. run relaxation stages in their declared order;
3. solve blocks through the existing engine;
4. use a homotopy where a stage fails, if the policy allows it.

Every applied estimator, stage and continuation step is recorded in the `InitializationReport`.
The failure path keeps the original specification intact. That is the current contract. The
difference from the retired "per-unit initializer templates"
([§17.2](../authoritative_design/sections/numerical-execution.md#section-17-2)) is that the knowledge
is declared data composed by one engine, not a plug-in protocol per unit. That needs an
authority change to §16.4, §17.2 and §17.3 (review
[F10](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f10)).

The adaptive homotopy step logic is a small domain algorithm. No Rust continuation library was
found (review slot 8). Its inner solves stay library-owned (PS-09).

## 3. Scaling

The target keeps the single resolved numerical policy and reversible normalization
([§16](../authoritative_design/sections/numerical-execution.md#section-16)). It adds two sources
of magnitude, each recorded with provenance and rank:

| Source (new rank) | Contract | Replaces |
|---|---|---|
| **Package and template hints** (between Model and PropertyDefault) | Nominal magnitudes declared on state definitions (`flow_mol ~ 1e2 mol/s` by default, overridable), property kinds, model forms and unit templates. They are consumed like property defaults but do not need a per-use binding row | `default_scaling_factors`; the `DefaultScalingRecommendation` of each unit scaler |
| **Derived nominals** (above QuantityNominal) | A prepared artifact evaluates each row's **additive terms** at variable nominals. Symbolica splits the top-level sum at preparation; the compiled program gives term magnitudes. The declared `ConstraintScalingScheme` then computes the row scale: inverse maximum, inverse sum, inverse RSS, harmonic mean or inverse minimum, keeping IDAES names. Variable nominals may propagate through defining equations of materialized properties the same way | `CustomScalerBase.scale_constraint_by_nominal_value`, `nominal_value_tools`, `AutoScaler` |

- The *scheme* is policy (per law, per template or per analysis), recorded like every other
  numerical source.
- Term magnitudes are also the input of the "mismatched and canceling terms" diagnostics (§4),
  so one artifact serves both.
- A **scaling profile** is a study (§6): prepare the same case under several policies and
  compare conditioning estimates (faer) and native iteration counts. This covers
  `ScalingProfiler`.

[Blueprint §16.3](../authoritative_design/sections/numerical-execution.md#section-16) retired
"nominal value algebra" as an expression-walking bespoke mechanism. The target uses the
compiled program (library-owned) rather than a walker. This is an authority change (review
[F11](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f11)).

## 4. Diagnostics

Diagnostics are a **catalogue of named analyses** over the prepared case and, where values are
needed, a candidate. Thresholds are a declared policy profile. The IDAES defaults are preserved
as the named profile `idaes-2.13`. Every finding is a structured `BoundaryDiagnostic` naming
model identities (DP-21, PS-04, PS-10).

| IDAES check | Target analysis | Mechanism and owner |
|---|---|---|
| DM under- and over-constrained sets; structural singularity; `degrees_of_freedom` | Structural admission (exists, §15.2–15.3) plus named reports in model terms (unit, port, law member) | pounce-presolve (`pse-structural`) |
| Components with inconsistent units | Physical admission (exists; stronger than IDAES, since basis and reference are checked) | `pse-quantity` |
| Variables fixed to zero, at or outside bounds, near bounds, with extreme values or no value | Candidate or case value analysis against the resolved bounds and nominals | `pse-backend-native::quality` (extend) |
| Large residuals | Original-space quality (exists) | `quality` |
| Extreme Jacobian rows, columns and entries | Scaled Jacobian at the candidate, with norms per row and column and extreme entries | Compiled derivative programs + faer |
| Near-parallel constraints and variables | Normalized row and column dot products over the sparse Jacobian | faer sparse products |
| Mismatched and canceling terms; problematic constraint terms | Term magnitudes from the derived-nominal artifact (§3), evaluated at the candidate | `pse-math` |
| Potential evaluation errors | Domain obligations of each body (exists in the compiler, §7.3) reported as potential failures before solving | `pse-compiler` |
| SVD toolbox and ill-conditioning certificate | Smallest singular values and vectors of the scaled Jacobian, mapped back to variables and rows; an ill-conditioning certificate LP; KKT and Jacobian condition estimates and certified inertia | faer: dense SVD under a size budget, or shift-invert through faer's sparse LU behind a linear-operator adapter. `faer::operator::partial_svd` returns only the *largest* singular values, while IDAES asks for the smallest. Sparse 1-norm condition estimates and certified KKT inertia from FERAL (Plan 22 N4). Certificate LP on HiGHS (IDAES uses cbc) |
| DegeneracyHunter | MILP for irreducible degenerate sets over the Jacobian | HiGHS, with one `Session` reused per LP/MILP family (Plan 22 C2). IDAES defaults to SCIP; SCIP gives no advantage on this small MILP and is not used for it (ADR-0105) |
| Infeasibility explanation (`compute_infeasibility_explanation`) | Whole-model relaxation plus deletion filter to a minimal infeasible set of named constraints, as in Pyomo's `contrib.iis.mis`; a certified IIS where a global route applies | The whole-model relaxation is the explicit POUNCE ℓ1 exact-penalty route (`L1ExactPenalty`, ADR-0109), whose least-infeasible point is recorded `diagnostic_only` with its violated constraints named; the deletion filter runs Ipopt/POUNCE solves over authored elastic overlays on the candidate constraints; HiGHS IIS (`Highs_getIis`, already wrapped) for linear parts; SCIP `SCIPgenerateIIS` as the certified route for nonlinear programs and true MIPs (ADR-0105, Plan 22 G5); bounded orchestration in the runtime |
| `IpoptConvergenceAnalysis`, convergence evaluation | Study over sampled cases recording outcomes and statistics (§6) | Study runner |
| `model_statistics` counts (98 functions) | Inspection queries over the specialized model: counts by kind, activity, fixedness, bounds, equality type, incidence | `pse-modeling` projection + DataFusion inspection over the admitted relations |

`assert_no_structural_warnings`/`assert_no_numerical_warnings` become policy checks that a test
or workflow can require. Diagnostics never change a scientific outcome (§15.6 kept).

## 5. Solving and formulation policies

- **Class-specific native execution is kept**
  ([§18](../authoritative_design/sections/numerical-execution.md#section-18)): Ipopt and POUNCE
  for NLP, KINSOL for roots, HiGHS for LP/MILP/QP, Clarabel for cones, Diffsol and IDAS for
  DAEs. So are the truthful outcome envelope, original-space qualification and physical
  closure. These already exceed IDAES, whose `check_optimal_termination` trusts the solver
  status. Plan 22 adds SCIP for MIQP, MINLP and explicit global certification (ADR-0102,
  ADR-0105), POUNCE-convex as an explicit alternative (ADR-0109), and typed Ipopt linear
  solvers — MUMPS with METIS, SPRAL SSIDS and oneMKL Pardiso (ADR-0108).
- **IDAES solver defaults** (`idaes.cfg` Ipopt options) become a named *solver profile*
  `idaes-2.13` for parity runs. That profile carries a relative `tol=1e-6`, `max_iter=200` and
  gradient-based native scaling. IDAES's default linear solver, MA27, is unavailable because
  HSL is excluded (ADR-0108), so the profile names MUMPS and its ordering explicitly. The
  default profile remains the resolved numerical policy.
  Reserved options are still refused
  ([§16.6](../authoritative_design/sections/numerical-execution.md#section-16-6)).
- **MPCC formulations.** A `complements(a >= 0, b >= 0)` declaration takes one of three
  realizations (ADR-0104): `smooth(ε)`, whose *relaxation schedule policy* (ε sequence) is a
  continuation policy (§2) and whose tightening is recorded with the result (PS-06);
  `penalty(l1)`, the POUNCE ℓ1 route below; and `disjunctive`, an exact discrete form on SCIP.
  The phase-equilibrium and pressure-minimization formulations default to `smooth(ε)`.
- **POUNCE's ℓ1 exact-penalty route** (`pounce-l1penalty`, pinned 0.12.0) is the typed POUNCE
  method `L1ExactPenalty` (ADR-0109), explicit only. It is never an automatic fallback (§18
  forbids fallbacks); an authored `penalty(l1)` realization is the explicit selection. Its own
  documentation does not recommend it for MPCC benchmarks, so it is not the default realization.
- **`ipopt_l1`** (exact penalty with ℓ1 elastic variables) splits in two. The **whole-model**
  part — infeasibility explanation of a complete model — is the POUNCE ℓ1 route, replacing the
  bespoke whole-model elastic formulation this section previously proposed. **Selected-constraint**
  elastic penalties stay an authored elastic overlay that the analysis applies to chosen
  constraints (ADR-0101), for robust initialization and the deletion filter. Neither is a
  separate solver wrapper.
- **PETSc SNES, TS and TAO** are not re-created. Roots go to KINSOL, time stepping to Diffsol
  and IDAS, and optimization to Ipopt and POUNCE (PS-09). The capability they provide in IDAES
  (DAE time stepping) is covered by §13 and the modeling document §7.

## 6. Studies and workflows

A **study** is a composition over prepared cases:

- one preparation;
- many value bindings;
- per-point attempts with typed outcomes;
- failures isolated per point;
- results as Arrow relations with lineage (PS-11, DP-10).

Today a sweep is a caller loop that re-freezes a revision per point
([§19.3](../authoritative_design/sections/workflows-and-results.md#section-19-3)). Freezing re-runs
lowering in the runtime, which is not Salsa-tracked (review
[F07](../design_review/reviews/design_review_idaes-capability-target_2026-09-26.md#f07)). The target
has a **study runner** that executes `authored.case_sets` with the following types.

| Study | Contract | IDAES counterpart |
|---|---|---|
| Parameter sweep | Declared sample set, from explicit values or a sampler. Value bindings only; structure reused; warm-start policy per point, recorded as a dependency (PS-11 MUST) | `parameter_sweep` |
| Convergence evaluation | Sampled cases through the declared initialization and solve policy; statistics of outcomes, iterations and times | `convergence/*`, `IpoptConvergenceAnalysis` |
| Parameter estimation | The existing fitting route ([§19.4](../authoritative_design/sections/workflows-and-results.md#section-19-4)), extended with **covariance and confidence intervals** from the reduced Hessian at a qualified estimate, computed by the sensitivity mechanism in the next row. Transient fits use Gauss–Newton covariance from the response SVD, labelled as that approximation, or exact second-order sensitivities when available (ADR-0107, ADR-0110). Validity conditions are stated: second-order sufficiency, full rank, statistical model (PS-12). Profile-likelihood intervals are a study over fixed parameters | parmest |
| Parametric sensitivity and uncertainty propagation | NLP parametric sensitivity and the reduced Hessian at a qualified optimum through library-owned sIPOPT semantics (ADR-0107): POUNCE `sensitivity` on the POUNCE route; `pounce-sens-core` (`SensApplication`, `parametric_step`) over a barrier-replica KKT with a FERAL LDLᵀ backsolver on the Ipopt route. FERAL reports inertia, so second-order sufficiency is checked; a faer sparse LU, which cannot report it, is not used. Valid under second-order sufficiency, LICQ, strict complementarity and a stable active set, and withheld otherwise. Propagation of parameter covariance to outputs. `ipopt_sens` is a parity oracle only | `sens.py` (sIpopt, k_aug) |
| Dynamic optimization, NMPC and MHE | Simultaneous discretized dynamics (modeling document §7) with path constraints, and single or multiple shooting over Diffsol sensitivities as explicit routes (ADR-0110); a **rolling-horizon workflow** on the staged-sequence primitive: shift, update measurements, warm start (SQP working set or interior-point restart) as a recorded dependency, advanced-step sensitivity (ADR-0107), solve; MHE arrival cost as declared objective terms | `caprese`, `nmpc` |
| Multiperiod | Period-indexed children of a flowsheet template with declared linking constraints (ramping, storage) | `grid_integration.multiperiod` |
| Market and price-taker studies | Multiperiod designs with price-series data; unit commitment authored with binary domains (ADR-0103): MILP on HiGHS with fixed-integer LP duals stated as conditional on the commitment, and MIQP or MINLP on SCIP when operation models are quadratic or nonlinear (ADR-0102); bidding and tracking as workflows over prepared models; Prescient coupling through the Python boundary as an external co-simulation | `pricetaker`, `bidder`, `tracker`, `coordinator` |
| Surrogate training | Sampling plus regression. Samplers: LHS via `egobox-doe`; Sobol via `sobol_burley`; Halton and Hammersley as trivial generators; CVT bespoke if needed. Seeds are recorded. Linear-in-parameter polynomial and RBF fits use faer least squares or the existing fitting route. Kriging uses `egobox-gp` behind a study adapter, or a faer plus Ipopt/POUNCE likelihood fit if exporting its private parameters is too coupled. Neural networks are imported with `tract-onnx` and translated layer by layer into authored expressions. ALAMO stays an external executable. Output is a published surrogate data package with a training-domain envelope and fit metrics | PySMO, ALAMO, Keras/OMLT, `sampling` |
| Materials optimization (MatOpt) | Lattice and design MILP formulations authored as templates with binary domains, cardinality and logic declarations (ADR-0103, ADR-0104); HiGHS, with SCIP as an explicit alternative | `matopt` |

## 7. Journeys (profile slot 4 additions)

| Journey | Target behaviour |
|---|---|
| Edit → re-solve (value) | A value change binds new case values. Template specialization, property resolution, bodies, plans and artifacts are reused (Salsa); only value-reading facts recompute. The warm start is the prior accepted result under `PreviousAccepted`, recorded in the `StartReceipt` |
| Edit → re-solve (structure) | Adding a unit or changing a balance type re-specializes the affected instances only (DP-09). Property programs for unchanged packages and state contexts are reused |
| Study over many cases | One preparation; per-point bindings; per-point outcomes; a failed point does not contaminate the others |
| Recycle that will not converge | Tear selection (HiGHS MILP or heuristic), KINSOL fixed point, recorded history; escalation to a simultaneous solve with tears as initial values; specification intact |
| Infeasible or ill-posed problem | Structural refusal before solving, naming units, ports and law members. After solving: infeasibility explanation study, a minimal set of named constraints |
| Out-of-envelope property evaluation | Recoverable trial failure attributed to the property kind, method and package; or a selected extrapolation policy recorded with the result |
| Dynamic start and event | Consistent initialization by the integrator from generated initial conditions; events declared; partial trajectories published as partial |

## 8. Numerical stage columns (profile slot 5)

| Stage | Formulation policy | Derivative source and order | Scaling | Problem class and solver capability | Status → outcome | Tolerances and post-solve check |
|---|---|---|---|---|---|---|
| Property resolution and closures | EO or nested per package; smoothing ε declared | Symbolica exact to order 2 for EO; implicit-function derivatives from converged nested closures | Package hints; derived nominals | — (preparation) or provider trials | Typed provider failures (existing classes) | Closure residuals; mechanical and phase stability checks |
| Initialization stages | Relaxation overlays; estimators | As the stage's program | Stage policy | Root (KINSOL) or NLP per block | Existing `InitializationReport` | Per-block original-quality acceptance before commit |
| Main solve | Selected formulation | Exact to order 2; limited-memory only when declared | Resolved policy plus native scaling when permitted | Derived class; eligible adapters only | `NativeTermination` → `Qualification` → `CandidateUse` | Original-space residuals, bounds, domain obligations, closure per subject member; VLE stability check |
| Dynamics (integrated) | Mode-generated accumulation | First order for the integrator, plus sensitivities | State and residual scales | ODE or index-1 DAE | `TrajectoryTermination` | Integrated closure per balance |
| Dynamics (simultaneous) | Discretized time | Exact to order 2 | Resolved policy | NLP | As main solve | Discretization residuals; closure per element |
| Studies | Per study | Per point | Per point | Per point | Per-point outcomes; study completeness | Per point; aggregate only over qualified points |

## 9. Risks

| Risk | Control |
|---|---|
| Declared start estimators diverge from what the model needs | Estimators are validated as starts only. A bad estimate costs iterations, never correctness. Convergence evaluation studies expose weak estimators |
| Derived nominals at poor nominal points | Nominals rank below explicit declarations; the scheme and provenance are recorded; profile studies compare policies |
| Homotopy adds bespoke control logic | Bounded step control only; inner solves library-owned; no Newton or line search of our own (PS-09) |
| Diagnostic SVD or MILP cost on large models | Opt-in, bounded analyses with reservations (existing §15.5 pattern) |
