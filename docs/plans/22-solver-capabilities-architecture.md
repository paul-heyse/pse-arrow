---
title: Solver capabilities, discrete decisions and the operational store — target architecture
status: draft
date: 2026-09-27
parent: 22-solver-capabilities.md
review_sources:
  - ../design_review/reviews/design_review_solver-capabilities_2026-09-27.md
---

# Solver capabilities, discrete decisions and the operational store — target architecture

**Evidence level: Proposed.** This document states the target that
[Plan 22](22-solver-capabilities.md) implements. Library facts cite the
[solver capability review](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md)
(slot 8) at the evidence level recorded there. The decisions it rests on are ADR-0102–ADR-0113,
accepted on 2026-09-27 after the
[Plan 22 target review](../design_review/reviews/design_review_plan22-target_2026-09-27.md);
the ADRs are authoritative where this text and they differ. Findings T01–T16 of that review
were corrected here before acceptance. Nothing here is implemented until its packet lands.

## 1. Target, drivers and scenarios

**Target.** A process simulator that solves every problem class an engineer can author, from
one model definition. The classes:
- square simulation, local and global optimization;
- mixed-integer linear, quadratic and nonlinear programs;
- generalized disjunctive programs;
- conic programs;
- dynamics and dynamic optimization;
- estimation with covariance, and parametric sensitivity.

Every outcome is truthful, physically scoped and durable. Long or many-case work runs across
processes without losing state.

**Drivers.**
- PS-09 and DP-13: established libraries own iteration, globalization, factorization and search.
- PS-10 and PS-12: typed outcomes; derived quantities carry their validity conditions.
- DP-01 and AP-04: one authority per fact, including across the two stores.
- §E and AP-03: a new backend or capability is additive.
- DP-19 and DP-20: explicit, durable lifecycle with bounded, coordinated resources.
- DP-24: versioned durable contracts.

**Scenarios.** The review's S01–S09 are reused unchanged. New scenarios:

| ID | Stimulus and conditions | Expected response |
|---|---|---|
| <a id="s10"></a>S10 | Authored binary unit commitment with linear operations (price-taker) | Domain facet → `CoefficientProblem` with integer domains → HiGHS MILP; marginal prices from the fixed-integer LP, stated as conditional on the commitment (PS-12) |
| <a id="s11"></a>S11 | Superstructure synthesis: disjunctions over nonlinear unit alternatives | Disjunctions lowered by a declared realization (hull, big-M or indicator) → mixed-integer nonlinear program → SCIP; incumbents stream durably |
| <a id="s12"></a>S12 | Certify a small nonconvex design optimization; prove global phase stability | Explicit `certify` intent → factorable export → SCIP global bound; `GapQualified` with box, tolerance and export fidelity recorded |
| <a id="s13"></a>S13 | Covariance, confidence intervals and output uncertainty after a steady fit | Reduced Hessian from the POUNCE sensitivity machinery; validity typed; withheld when conditions fail |
| <a id="s14"></a>S14 | Rolling-horizon NMPC | Staged-sequence primitive; value-only rebind; SQP working-set or barrier warm restart; advanced-step sensitivity; per-step records in the operational store; one publication per horizon batch |
| <a id="s15"></a>S15 | A 10 000-point study across worker processes | Durable job queue; points claimed with `SKIP LOCKED`; a failed point is isolated; results published once per study |
| <a id="s16"></a>S16 | The process running a long SCIP solve dies | Lease expiry marks the attempt `stale`; a new attempt re-injects the best stored incumbent |
| <a id="s17"></a>S17 | Two processes publish to one workspace, locally or on a remote object store | The catalog transaction serializes head changes; the loser re-prepares against the new parent; nothing is lost or half-visible |
| <a id="s18"></a>S18 | Add another backend | One adapter implementation, one registry value and its capability record; no edits to runtime workflows |

## 2. Discrete decisions and disjunctive modeling

### 2.1 Domain facet

A variable's domain is part of its **meaning**: it changes the problem class. It is therefore
declared on the variable, not carried by an annotation (annotations hold numerical knowledge
under ADR-0101). ADR-0103 settles the syntax:

```
var units_on[u in units] : Indicator in binary;
var trays : Count in integer;
var steam : MassFlow in semicontinuous;
var modules : Count in semiinteger;
```

- **Physical typing (PS-01).**
  - `integer` and `binary` require dimensionless count or indicator quantity kinds.
  - Semi domains keep the variable's physical quantity. The zero branch is exact; the active branch is bounded.
- **Bounds.**
  - `binary` implies [0, 1].
  - `integer` and the semi domains require **finite** case bounds, from `annotation bounds` or case values. Admission refuses otherwise with a typed `Unsupported` cause naming the variable.
  - Non-integral bounds on an integer variable are tightened inward exactly, and the transformation is recorded.
  - Spatial branch-and-bound needs finite boxes. The same admission covers continuous variables that enter a global route (§5).
- **One authority.**
  - A registry enum `ModelingVariableDomain` [continuous, integer, binary, semicontinuous, semiinteger] is generated into `pse-model`.
  - `pse_math::binding::VariableDomain` is deleted; `pse-math` consumes the generated type directly (it already sits above `pse-model`), so no mapping layer exists ([T15](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t15)).
  - `grouped.rs` carries the declared domain instead of hard-coding `Continuous`.
  - `runtime.solve_variables` gains a `domain` column.

### 2.2 Semantics per analysis mode

| Mode | Discrete variables |
|---|---|
| Steady optimization | Decisions: MILP, MIQP or MINLP routes (§3) |
| Steady square or root | Admitted only when every discrete variable is fixed by the case; the solve is then continuous |
| Initialization | A declared stage fixes discrete variables at start or declared values as a scoped overlay, restored on every exit (PS-08, DP-05) |
| Integrated dynamics | Discrete variables are fixed per segment (piecewise-constant inputs); free discrete decisions over time use the simultaneous route |
| Simultaneous dynamic optimization | Admitted: mixed-integer dynamic optimization lowers to MINLP |
| Fitting | Refused unless fixed |
| Duals, sensitivity, covariance | Computed on the continuous problem with the discrete assignment fixed; the validity record states that assignment (PS-12) |
| Structural and DoF analysis | Discrete variables count as degrees of freedom; each disjunct's equations are analysed under its realization |

### 2.3 Constraint forms

The kernel gains these declarations, each lowered by a named transformation (DP-08):
- **Indicator constraints:** `eq name when y: lhs == rhs;`.
- **SOS1 and SOS2 sets.**
- **Cardinality:** `atmost`, `atleast`, `exactly`.
- **Piecewise-linear functions** with an SOS2 or incremental lowering.
- **Logic propositions** over binary or Boolean variables: `and`, `or`, `xor`, `implies`, `exactly(k)`.

Each lowering has two paths:
- **Native, where the backend has the constraint handler:** SCIP indicator, SOS1/SOS2, and/or/xor, logicor, cardinality.
- **Linear inequalities over finite bounds, where it does not:** HiGHS.

Routing refuses a native-only realization on a backend that lacks it. There is no silent conversion.

### 2.4 Disjunctions (generalized disjunctive programming)

```
disjunction route {
  alternative distill  { eq ...; annotation bounds ...; }
  alternative membrane { eq ...; }
  alternative bypass   { eq ...; }
}
realize route using hull;   // bigm | bigm(derived) | hull | indicator
```

- A disjunction is the **decision-time** counterpart of the existing evaluation-time
  `implicit … regime … eligible(...)`. It reuses its structure (named alternatives, local
  equations and annotations) and its realization declaration (`realize … using …`).
- A regime selects an equation set during evaluation. A disjunction chooses one during optimization.
- **Lowerings are named transformations with declared equivalence** (ADR-0104):
  - `bigm(M)` uses an authored M; its validity is the author's assertion and is recorded as such.
  - `bigm(derived)` takes M from bound propagation over the declared box, using `pounce-presolve` FBBT intervals (library-owned), widened outward by a declared relative margin. Every row of the disjunct must be FBBT-complete with a finite interval; otherwise the lowering is refused, naming the row, and no default M is used. The lowering is exact for bounded disjuncts.
  - `hull` produces the convex-hull reformulation with disaggregated variables. It is tighter and needs finite bounds on every disjunct variable. Linear disjuncts are exact; nonlinear disjuncts use the ε-perspective `(λ+ε)·g(x/(λ+ε))` with a declared ε, and the O(ε) approximation is stated (PS-06).
  - `indicator` emits native indicator constraints and is SCIP only.
- Lowering happens at preparation. The authored revision is unchanged (D13), and the realization with its parameters enters preparation identity.
- Logic propositions between alternatives lower as in §2.3.
- Nested disjunctions are admitted. Their lowering order is explicit.

### 2.5 Complementarity and phase appearance

A declaration `complements(a >= 0, b >= 0)` takes a realization. PS-06 requires each one's
approximation to be stated:

| Realization | Mechanism | Approximation |
|---|---|---|
| `smooth(epsilon)` | Continuation over ε, reusing the existing continuation policy | O(ε) |
| `penalty(l1)` | POUNCE ℓ1 exact-penalty route (§6.3) | Exact at a nondegenerate solution; least-infeasible point otherwise |
| `disjunctive` | SOS1 or indicator; SCIP | Exact, discrete |

An authored `penalty(l1)` is a declared requirement of the formulation, so it is the explicit
selection of POUNCE `L1ExactPenalty` and is recorded as such; it does not conflict with
"ℓ1 is never automatic" (§6.3). Mixing `penalty(l1)` with other realizations in one solve is
admitted; the ℓ1 penalty then applies to every constraint, and the result says so
([T14](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t14)).

Phase appearance can then be authored as smoothing, complementarity or explicit discrete modes,
as package data.

## 3. Problem classes, routing and assurance vocabulary

These changes are ADR-0106, which supersedes ADR-0090. They are registry changes followed by `just codegen`.

- **`NativeProblemClass` gains:**
  - `nonconvex_quadratic`;
  - `mixed_integer_quadratic` (convexity recorded as a fact);
  - `mixed_integer_nonlinear`.
- **`SolveIntent` becomes a registry enum and gains `Certify`.** Global certification is an explicitly selected intent, never an automatic upgrade of a local solve.
- **`NativeAssurance` gains four values** ([T10](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t10)):
  - `global_bound`: a dual bound for the exported program over the declared box, valid within the backend's recorded tolerances and the export fidelity; not interval-rigorous;
  - `proven_infeasible`: the backend's global infeasibility conclusion for the exported program, under the same conditions; not interval-rigorous;
  - `exact_certificate`: exact rational MILP, the only rigorous assurance;
  - `sos_bound_nonrigorous`: a floating-point SOS polynomial bound (POUNCE-convex), never a certificate and never part of a gap claim.
- **`GapQualified`** extends to SCIP, recording the box, tolerances, export fidelity and the sources of the dual bound and the primal candidate. A `Relaxed` export yields bound and infeasibility claims only (§5.2).
- **`CandidateUse`** stays the single typed decision, owned by the workflow completion owner (§16.6), and is extended rather than replaced: `usable`, `qualified_unclosed`, `seed_only` (feasible in original coordinates, but the native stop forbids use as a result; may seed a later step), `diagnostic_only` (a relaxed or least-infeasible point; an observation only) and `unusable`. Every workflow consumes it ([F13](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f13), [T01](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t01)).

**Target routing** (automatic order first; explicit alternatives never substitute silently):

| Class | Automatic | Explicit |
|---|---|---|
| linear | HiGHS | Clarabel, POUNCE-convex, SCIP |
| mixed_linear | HiGHS | SCIP, including exact mode |
| convex_quadratic | HiGHS | Clarabel, POUNCE-convex, SCIP |
| nonconvex_quadratic | Ipopt, POUNCE (local) | SCIP (`Certify`) |
| mixed_integer_quadratic | SCIP | — |
| mixed_integer_nonlinear | SCIP | — |
| continuous_cone (explicit or recognized, §8) | Clarabel | POUNCE-convex |
| smooth_nlp | Ipopt, POUNCE | SCIP (`Certify`) |
| square_root, declared_fixed_point | KINSOL, Ipopt, POUNCE | — |
| ode, semi_explicit_index1 and its extensions (§7) | Diffsol, IDAS by profile | explicit method |

## 4. Backend execution adapter and shared runners

The seam that makes S18 additive, correcting [F27](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f27)
and [F28](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f28).

**`BackendExecution` trait**, owned by `pse-backend-native`. Each backend implements:

| Item | Contents |
|---|---|
| `Settings` | A pse-owned, serde-versioned type. Its identity is derived from serde, never hand-listed (F09) |
| `capability()` | The one capability record. Routing eligibility and the published `solver_capabilities` rows both derive from it (F21) |
| `admit(facts, controls, settings)` | Returns typed eligibility reasons |
| `session` | Prepare, solve and retain, with native lifecycle on the owning worker |
| `WarmPayload` | Its own typed variant, with a coordinate-compatibility stamp separate from the profile stamp (F24) |
| `evidence` | A typed evidence struct consumed by `quality.rs`; metrics remain observation only (F16) |

The adapter table is a static map from `Backend` to its adapter. There is no string lookup.

**Shared runners in `pse-runtime`:**

| Runner | Replaces | Serves |
|---|---|---|
| One NLP runner (pipeline → solve → finish → KKT → qualify) | Four copies (F28) | Solve, initialization, fitting and certification |
| One staged-sequence primitive (value-only rebind, retained sessions, typed `StartSource`) | Two initialization engines and two multi-case engines (F29) | Homotopy, studies, rolling horizons |

**Typed failures (F17, F18, F19).** `ProblemError` gains Provider, Unsupported, Numerical,
Limit, Cancelled and Internal variants, each keeping its cause. The registry diagnostic
vocabulary gains severity plus the DP-21 categories (invalid model, unsupported, numerical,
infrastructure, inconclusive).

## 5. Factorable projection and SCIP

### 5.1 `FactorableProgram` (pse-math)

A library-neutral DAG projected from Symbolica atoms. It sits beside the FBBT projection and
never replaces the evaluator.

- **Nodes:**
  - Var, Const (rational or f64);
  - n-ary Sum and Product;
  - Pow with a rational exponent;
  - Exp, Log, Abs, Sin, Cos;
  - Min and Max, exported exactly through the absolute-value identity;
  - `Aux`, a free auxiliary variable standing for an opaque output.
- **Fidelity.** Every row and the objective carry `Exact`, `Relaxed` or `Unavailable`.
- **Obligations** (`Require`, `Domain`) become closed bounds or constraints.
- **Implicit blocks** export their residual equations and declared bounds whatever their realization. The residual atom exists (`implicit_cubic.rs`), so the PR cubic is exact.
- **Providers** become `Aux` within an *enforced* envelope, so the row is `Relaxed`.
- **Branches with proven continuity** become disjunctions (exact, but mixed-integer) or `Aux` (relaxed), by a declared policy.
- **Validity guards must not poison projection.** The census traced the only non-physics opacity in the PC-SAFT cases to a `valid(...)` conditional lowered into a `Domain` stage. The projection treats `Domain` stages as obligations, not as expression dependencies.

### 5.2 Relaxation-soundness rule

- A `Relaxed` export may support a **dual bound** and a **global infeasibility proof**, never an optimality or solution claim.
- Solutions from any export are re-qualified in original coordinates by `quality.rs` (PS-10).
- An `Unavailable` objective yields no bound, and `Certify` is refused with that reason.
- **Mixed-integer programs with relaxed rows** ([T07](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t07)). SCIP's incumbent is an assignment proposal, recorded `diagnostic_only`. The candidate comes from the fixed-assignment continuous re-solve (§2.2) through the one NLP runner and is qualified in original coordinates. A gap may combine the relaxation's dual bound with that qualified candidate's objective; both sources are recorded.

### 5.3 SCIP 10.0.2 adapter (`pse-backend-native::scip`, feature `scip`)

**Build** (a component of the solver image, §5.4 and ADR-0108):
- the checksummed scipoptsuite 10.0.2 source;
- SoPlex LP (`LPS=spx`);
- `IPOPT=ON` against `/opt/pse-solvers` (one Ipopt, 3.14.20);
- `PAPILO=ON`;
- GMP, MPFR and Boost for exact mode;
- `THREADSAFE=ON`, and `TPI=tny` for concurrent solving, so SCIP's threads do not share the OpenMP runtime settings SPRAL requires (§5.4).

The `bundled` and `from-source` scip-sys profiles are rejected (review §8.1).

**Binding:** `scip-sys` against the image's `SCIPOPTDIR`. At run time the adapter checks
`SCIPmajorVersion/minor/tech`, `SCIP_APIVERSION`, `SCIP_Real = f64` and the index width.
`russcip` is not used: its conversions panic and its safe surface adds nothing the adapter needs.

**Lifecycle** follows §18.8: one `SCIP*` per attempt, created and freed on the owning worker.

**Reserved options:**
- `misc/catchctrlc = FALSE` (mandatory under PyO3);
- `limits/time` from the deadline;
- `limits/memory` from the foreign allowance;
- seeds recorded;
- concurrent mode deterministic, with thread count equal to the admitted permits;
- SCIP's nested Ipopt ([T08](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t08)): `nlpi/ipopt/linear_solver` is set from the typed Ipopt linear-solver setting (default `mumps`); `nlpi/ipopt/hsllib` and `nlpi/ipopt/pardisolib` are refused; the nested Ipopt runs serial unless threads are admitted for it.

**Cancellation** runs through an event handler that polls the attempt flag and calls
`SCIPinterruptSolve` on the owning thread.

**Status mapping:** raw `SCIPgetStatus` → `NativeTermination`, as tabled in the review (§8.1, Q5).

**Capabilities:**
- incumbent injection (`SCIPaddSolFree`) from Ipopt or POUNCE;
- export-equivalence readback;
- native indicator, SOS and logic handlers (§2.3);
- solution pool;
- reoptimization for MIP sequences;
- exact rational MILP (`SCIPenableExactSolving`, which excludes reoptimization);
- IIS on the true problem (`SCIPgenerateIIS`);
- an incumbent and bound event stream into the operational store (§9).

### 5.4 One solver image: Ipopt linear algebra, MKL and OpenMP

ADR-0108 owns this contract; it supersedes ADR-0028. Findings
[T04](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t04)–[T06](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t06)
shaped it.

- **Composition.** One digest-pinned image, built from checksummed sources: Ipopt 3.14.20 (the
  only Ipopt, which SCIP also links), sequential MUMPS rebuilt with METIS, SPRAL SSIDS (CPU,
  OpenMP, hwloc, METIS), oneMKL, METIS once for MUMPS and SPRAL, ASL for the parity executable,
  and the SCIP components of §5.3. HSL is excluded.
- **One BLAS/LAPACK, one OpenMP runtime.** Ipopt offers `pardisomkl` only when MKL is its
  BLAS/LAPACK, so oneMKL (LP64, GNU threading layer) is the single BLAS/LAPACK provider of every
  process that loads the native profile — MUMPS, SPRAL and Clarabel SDP included — and libgomp
  is the single OpenMP runtime.
- **Typed selection.** `linear_solver ∈ {mumps, spral, pardisomkl}` with explicit `ordering`
  (default `mumps` with `metis`, stated explicitly because `mumps_pivot_order = 7` would pick
  METIS silently once linked). `linear_solver`, `hsllib` and `pardisolib` are reserved; HSL
  values are refused. An absent library or unmet precondition is a typed `Unsupported`
  refusal, never a fallback.
- **Threads.** SPRAL and MKL threads are admitted under §18.8: permits for the requested count,
  `omp_set_num_threads` and `mkl_set_num_threads_local` on the owning worker, `MKL_DYNAMIC=FALSE`.
  MUMPS stays serial.
- **SPRAL preconditions.** `OMP_CANCELLATION=TRUE` and `OMP_PROC_BIND=TRUE` are process-level;
  the image environment and `pse-worker` set them, and admission refuses SPRAL when
  `omp_get_cancellation()` or the bind policy shows they are unmet.
- **Determinism.** `MKL_CBWR` is pinned to a branch recorded in the image manifest and checked
  with `mkl_cbwr_get`. The profile key records library versions, CBWR branch, linear solver,
  ordering and thread counts. The claim is run-to-run reproducibility for one image, branch and
  thread count, and numerical — never bitwise — agreement otherwise.

## 6. Sensitivity, covariance and uncertainty

1. **NLP parametric sensitivity and reduced Hessian, POUNCE route:**
   - `pounce-rs` feature `sensitivity`, via `SensSolve` with deltas and reduced Hessian.
   - Results map back through presolve column maps and the normalization back-map.
   - PS-12 validity is typed: activity classification, reduced-Hessian eigenvalues for second-order sufficiency, LICQ from the Schur step, strict complementarity.
2. **Ipopt route:** `pounce-sens-core` (`SensApplication`, `parametric_step`) over a barrier-replica KKT with a FERAL LDLᵀ backsolver. This replaces Plan 20 §6's faer sparse LU, which cannot report inertia.
3. **ℓ1 exact penalty** (`pounce-l1penalty`, already linked) is an explicit POUNCE method. It serves complementarity (§2.5) and whole-model infeasibility explanation, replacing the whole-model part of Plan 20 §5's bespoke elastic policy.
4. **Covariance and confidence intervals:**
   - steady fits: from the reduced Hessian;
   - transient fits: Gauss–Newton from the response SVD that already exists, recorded with `approximation = gauss_newton` (valid under the declared statistical model with small residuals; residual curvature neglected), and exact from §7's second-order sensitivities when available, recorded as such ([T11](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t11));
   - profile-likelihood intervals: a study over fixed parameters.
5. **Uncertainty propagation** (the counterpart of IDAES `sens.py`): parameter covariance propagated through output sensitivities.
6. **Conditional quantities.** For mixed-integer problems, sensitivities and duals are computed with the discrete assignment fixed, and say so.
7. **Parity oracle.** `ipopt_sens` in the parity container serves only as an independent test oracle.

## 7. Dynamics and root-finding extensions

These amend ADR-0084 and ADR-0093.

**IDAS:**
- scheduled input changes with recoverable trials (`IDAReInit`, `IDASensReInit`, `IDACalcIC`, `IDAGetSensConsistentIC`);
- events and roots without sensitivities (`IDARootInit`, `IDASetRootDirection`). Events *with* sensitivities stay on Diffsol, which owns reset sensitivities;
- `IDASetConstraints`;
- staggered sensitivities;
- SPGMR and SPFGMR with preconditioner hooks;
- `IDA_Y_INIT`.

**Diffsol:**
- SDIRK (`tr_bdf2`, `esdirk34`) and explicit `tsit45` for mass-free ODEs;
- the KLU backend through the `suitesparse` feature;
- all selected by a typed method in the profile.

**Adjoint sensitivities:**
- Diffsol: adjoint with checkpointing. Primary; operator trait implementations over faer.
- IDAS: `IDAAdjInit` through `IDASolveB` and `IDAQuadInitB`, for the recoverable-trial profile.
- A gradient-only fit mode consumes either.

**Transient Hessians:**
- `HessianMode::GaussNewton` (JᵀWJ from forward sensitivities);
- exact second-order sensitivities through IDAS forward-over-adjoint (`IDAInitBS`, `IDAQuadInitBS`).

**Dynamic optimization:**
- simultaneous collocation (the existing Plan 20 target) remains primary;
- single and multiple shooting are explicit routes over Diffsol forward or adjoint sensitivities;
- NMPC and MHE are compositions over the staged-sequence primitive (§4).

**KINSOL:**
- `KINSetMaxNewtonStep`, eta forms, preconditioner hooks;
- SPFGMR, SPBCGS and SPTFQMR;
- Anderson orthogonalization and delay;
- one-sided nonzero bounds shifted to sign constraints;
- `Settings::from_policy` as the single constructor (F22);
- a per-worker session cache for nested `InnerSolver` calls;
- solve tests for the Dense, SPGMR and Picard paths.

## 8. Coefficient and conic extensions

**HiGHS 1.15.0** (`highs-sys` bump), with `qp_allow_hot_start = true`. It also brings:
- fixed-integer LP duals (`Highs_getFixedLp`), conditional on the commitment;
- incumbent capture from MIP callback kinds 3 and 4;
- presolve and postsolve views and basis-inverse operations for diagnostics;
- the cut pool;
- a MIP node budget separate from `iterations`;
- the QP regularization value derived from the requested gap.

**Multi-objective contract** (ADR-0111, refining ADR-0101's objective clause).
`annotation objective` gains priority and weight members, and each priority level declares
absolute and relative degradation tolerances
([T12](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t12)):
- Within a level the objective is a weighted sum; each member must be dimensionless or carry an explicit declared normalization.
- Across levels the order is lexicographic: each later level bounds every earlier objective by its optimum plus the declared tolerance. A zero tolerance is admitted only on the native LP/MILP path; staged NLP, QP and MINLP levels need a positive tolerance, because an exactly active objective bound breaks constraint qualification.
- LP and MILP use HiGHS's native lexicographic objectives (`Highs_passLinearObjectives`) with the declared tolerances.
- NLP and MINLP compose lexicographic solves as staged-sequence steps with objective-bound constraints: composition, not a bespoke solver.

**Clarabel:**
- explicit eligibility for LP and certified convex QP, lowered from `CoefficientProblem`. It adds Farkas certificates for QP and a second convex-QP owner that avoids HiGHS's regularization gap;
- `faer-sparse` and Pardiso backends and threads under §18.8 admission;
- chordal decomposition when data reuse is not requested.

**Automatic cone recognition:**
- second-order cones from the existing exact Gram certificate;
- exponential and power cones from factorable DAG patterns (§5.1);
- a recognized convex problem becomes automatically eligible for Clarabel. The recognition is a compiler fact, so routing stays derived (PS-09).

**POUNCE-convex backend** (`pounce-rs` feature `convex`):
- IPM for LP, QP and cones, with warm start;
- batched parallel QP solves for studies;
- QP parametric sensitivity;
- SOS polynomial lower bounds, labelled with a distinct non-rigorous assurance.

## 9. Operational store and publication catalog (PostgreSQL 18)

### 9.1 Why an operational store, and why PostgreSQL

**Today** (persistence map, 2026-09-27):
- Delta publication is the only durable state.
- Runs, jobs, progress, warm starts and sessions exist only in memory and are lost on exit.
- Job admission refuses rather than queues, and only within one process.
- Publication is qualified for a single local writer (§26; register R-10), with one head row per workspace and no automatic conflict retry.
- Each publication creates about 15 small Delta tables with two commits each.

**The workloads this plan adds cannot be served that way:**
- long MINLP and global solves that must survive a crash (S16);
- studies across worker processes (S15);
- rolling horizons with frequent small records (S14);
- warm starts reused across processes;
- concurrent publication (S17).

DP-19 requires planned, running, completed, partial, stale, cancelled and failed to be
distinguishable. Today they are distinguishable only in memory.

| Candidate | Fit for mutable operational and control state | Verdict |
|---|---|---|
| Delta only | Immutable columnar versions. Every small update is a commit plus files; no multi-table transactions; no row locks, queues or notifications; single-writer qualification | Keep for data; reject for control state |
| SQLite | Transactions, but one writer per database file, no cross-host access and no `SKIP LOCKED`/`LISTEN` | Too narrow for workers and remote stores |
| Embedded KV (redb, sled) | Single process | Reject |
| **PostgreSQL 18** | Transactions and serializable compare-and-set; row locks and `FOR UPDATE SKIP LOCKED` queues; advisory locks for leases; `LISTEN/NOTIFY` for cancellation and progress; native `uuidv7()`; mature Rust client (sqlx); installed locally (18.6, cluster `main`, port 5432) | **Adopt for the operational and control plane** |

The division is: **PostgreSQL owns what changes; Delta owns what is published.**

### 9.2 One authority per fact

| Fact | Authority | Other representations (derived, with identity) |
|---|---|---|
| Authored sources | Package files (unchanged) | Content-addressed source bundles in PostgreSQL for job execution, keyed by the §6.1 content hash; authored snapshot tables in each Delta publication (existing) |
| Attempt existence and lifecycle state | `pse_ops.attempts` | Published `runtime.computation_runs` and `runtime.run_lineage`: the immutable record of a published attempt's final state |
| Jobs, leases, cancellation requests | PostgreSQL | — |
| Live progress, incumbents and bounds | PostgreSQL (retention-bounded) | The final snapshot in published `runtime.solve_metrics` and the new `runtime.incumbents` |
| Reusable solutions (primal, dual, working set, basis) | PostgreSQL solution store, keyed by coordinate-compatibility stamp and preparation identity | The seed identity recorded in lineage (F25) |
| Study and point status | PostgreSQL | The published study relation |
| Publication heads, membership and settlement | **PostgreSQL catalog**, replacing the Delta control table | A derived, read-only export manifest for offline readers |
| Scientific result relations | Delta member tables | — |
| Meaning of operational relations | The registry (D1) | SQL migrations as the physical representation, checked by a migration-conformance test |

Enums are stored as text and parsed into registry enum types at the boundary (DP-02).
There are no database enum types that could drift.

### 9.3 Schema sketch (`pse_ops`, versioned migrations)

| Table | Contents |
|---|---|
| `source_bundles`, `source_documents` | Keyed by content hash |
| `attempts` | `attempt_id uuid` minted by the runtime (no database default), `run_id`, `kind`, `request_identity`, `preparation_identity`, `state`, `parent_attempt`, timestamps, `worker`, `lease_expires_at`, `cancel_requested`, typed termination |
| `attempt_transitions` | Append-only audit of every state change |
| `jobs` | `job_id`, `attempt_id`, versioned `payload jsonb` (source bundle, case, profile), `priority`, `state`, `claimed_by`, `lease_expires_at`, `tries`, `idempotency_key unique` |
| `progress_events` | `attempt_id`, `seq`, `at`, `phase`, `values jsonb`; batched inserts or `COPY` |
| `incumbents` | `attempt_id`, `seq`, `objective`, `dual_bound`, `gap`, `at`, `solution_id` |
| `solutions` | `solution_id`, `compatibility_stamp`, `preparation_identity`, `kind`, `payload bytea` (Arrow IPC, versioned), `created_by` |
| `studies`, `study_points` | Point `binding_hash`, `state`, `attempt_id`, `result_ref` |
| `workspaces`, `publication_heads`, `publications` (attempt identity unique), `publication_members`, `settlements`, `reader_leases`, `retention_marks` | The catalog, reader leases and two-phase deletion state |

### 9.4 Lifecycle, queue and leases

- **State machine:** planned → queued → running → {completed, partial, failed, cancelled}. From running, lease expiry gives stale, and a stale attempt is superseded by a new attempt.
- **Transitions** ([T13](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t13)). One pure Rust transition table in `pse-operations` is the only authority for legality and is testable without a database. Repository functions apply it inside a transaction under row locks, so illegal transitions are rejected (DP-03). The database enforces only the state-value domain (a `CHECK` from registry values) and append-only history (the application role cannot update or delete `attempt_transitions`).
- **Identity.** The runtime mints attempt, run and publication identities on its existing UUIDv7 `SemanticId` path before any effect; the store records them, and no database default mints a domain identity (DP-04).
- **Claim:** `UPDATE jobs … WHERE job_id = (SELECT … FOR UPDATE SKIP LOCKED LIMIT 1) RETURNING`.
- **Heartbeat** extends the lease. On expiry the attempt goes to stale, and the job is requeued under an explicit retry policy (maximum tries, backoff).
- **Idempotency:** each try is a new attempt identity, and publication is idempotent per attempt (DP-19).
- **Cancellation** sets `cancel_requested` and sends `NOTIFY`. The column is the authority and the heartbeat returns it; `NOTIFY` only shortens latency, because notifications are not durable across a dropped listener. A worker issues `LISTEN`, commits, then re-reads state, as PostgreSQL's delivery rule requires, and re-reads again whenever `PgListener::try_recv` reports a lost connection ([T03](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t03)). The owning worker maps cancellation onto the existing cooperative cancellation lease.
- **`MathService` admission** queues through the store instead of refusing, when durability is requested.

### 9.5 Publication through the catalog

1. **Members are unchanged:** written to Delta at attempt-scoped paths, with provisioned and written receipts.
2. **Commit** is one PostgreSQL transaction. It checks that `publication_heads.publication_id` is the expected parent, inserts the publication and its members (location and version), and advances the head. A lost race is a typed conflict; the loser re-prepares against the new parent. This keeps today's "never rebase" rule.
3. **Readers** resolve a publication id, or an explicit named query such as the latest head (with the resolved version recorded), to member locations and versions. A reader takes a lease row with an expiry in a short transaction and releases it when done; no reader holds a database session open while reading Delta files ([T02](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t02)).
4. **Retention and maintenance.** The catalog computes the protected versions (SQL in `pse-operations`), replacing the DataFusion query over the Delta control relations ([T16](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t16)). Deletion is two-phase: mark the publication `expiring` so no new lease is granted, wait until every lease is released or expired, let `pse-catalog` delete the member files, then mark it `deleted`. Transaction-scoped advisory locks serialize maintainers. This replaces `.pse-retention.lock`, which works for local files only, and makes remote object stores qualifiable (R-10, decided by ADR-0112).
5. **Deletion without migration.** Existing Delta publications are regenerated by rerun. The system is in design, not production (maintainer, 2026-09-27), so there is no importer. A store written under the control table is refused as an unsupported historical format, with a migration-required diagnostic that names regeneration by rerun.
   - The Delta control table, its code and the file leases are deleted once the catalog path is proven (DP-16).
   - An export command writes a read-only manifest with the member tables for offline readers.

### 9.6 Durability classes

| Class | Registry | Can publish | Queue and progress | Use |
|---|---|---|---|---|
| `Ephemeral` | None | No | In-memory, as today | Library calls and unit tests that need no database (AP-06) |
| `Durable` | Registered attempt | Yes | Durable | Everything that publishes, queues, studies or runs long |

The class is an explicit policy, not a fallback (DP-15). Publication without the catalog is refused.

### 9.7 Deployment

- **Server:** PostgreSQL 18.6, cluster `18/main` on port 5432. The stopped `16/main` cluster on 5433 is outside this plan; dropping it is a maintainer decision.
- **Role and databases:** a login role `pse` (SCRAM) owns database `pse`. For `#[sqlx::test]`, `pse` holds `CREATEDB` so each test gets an isolated database.
- **Connection:** `PSE_DATABASE_URL`, or the libpq service `pse` in `~/.pg_service.conf`. No credential is ever committed.
- **`just` recipes:**

| Recipe | Does |
|---|---|
| `db-bootstrap` | Idempotent; runs as the `postgres` superuser via `sudo -u postgres psql`, behind the same confirmation as other recipes that reach outside the working copy |
| `db-migrate` | Applies the embedded `sqlx::migrate!` migrations |
| `db-status` | Reports server version ≥ 18, connectivity and pending migrations |
| `db-backup` | `pg_dump -Fc` to a configured directory |
| `db-restore` | Restores such a dump |

- **Doctor:** reports server version ≥ 18, connectivity and pending migrations.
- **Configuration:** defaults, plus `idle_in_transaction_session_timeout` and a pool size below `max_connections`. `pg_stat_statements` is optional, for measurement.
- **Backups** are dump-based. Point-in-time recovery is out of scope until a remote deployment needs it.

### 9.8 Code placement

**A new crate, `pse-operations`** (ADR-0112):
- **Owns** the store contract, migrations, repositories, the catalog and reader leases.
- **Client:** `sqlx` =0.9.0, in `pse-operations` only, with `default-features = false` and features `postgres`, `runtime-tokio`, `tls-rustls-ring`, `macros`, `migrate`, `uuid`, `chrono` and `json` (matching the workspace's chrono, uuid and rustls/ring pins; the exact pin lives in `Cargo.toml`).
- **Queries are runtime-typed:** `query` and `query_as` with `FromRow`. There are no `query!` macros and no `.sqlx` offline metadata, so there is no build-time database and no generated path (DP-16). SQL stays SQL, in the repository modules.
- **Migrations:** `sqlx::migrate!` embeds versioned `.sql` files with LF line endings.
- **Connections and coordination:** `PgPool`; `PgListener`, which reconnects transparently, for cancellation and progress notifications; `PgAdvisoryLock`, or `pg_advisory_xact_lock` inside catalog transactions.
- **Batches:** typed batch inserts with `UNNEST($n::type[])`; `copy_in_raw` only if a measurement shows the need.
- **Errors** derive `thiserror` plus `miette` codes (§23.2). SQLSTATE 40001, 40P01, 23505, 55P03 and 57014 map into typed store errors.
- **Tests:** `#[sqlx::test(migrator = "pse_operations::MIGRATOR")]` against the local PostgreSQL 18 server, one isolated database per test with library-owned cleanup; the `just` recipe maps `PSE_DATABASE_URL` to `DATABASE_URL`.
- **Server features:** `uuidv7()` for surrogate keys of rows without a runtime-minted domain identity, `FOR UPDATE SKIP LOCKED`, advisory locks, `LISTEN`/`NOTIFY`, `idle_in_transaction_session_timeout`; `pg_stat_statements` optional for measurement; no extension required.
- **Dependency direction:** `pse-runtime` depends on it; it depends on `pse-model`, `pse-schema` and `pse-diagnostics` for meaning and on `pse-engine` for its read-only DataFusion provider, as `pse-catalog` does; `pse-catalog` keeps Delta data I/O; the domain core never depends on it (DP-17).
- **Workers.** `pse-worker` is a binary target of `pse-runtime`, the composition root that already owns `MathService` and the workflows (decided in ADR-0112). It sets the process-level OpenMP environment SPRAL needs (§5.4).

**Client stacks not adopted** (the maintainer's [external review](../external-review-postgresl-options.md), assessed in ADR-0112):
tokio-postgres with deadpool-postgres, refinery and tokio-postgres-rustls (four seams where one serves; no library-owned per-test database or reconnecting listener); Cornucopia or clorinde (code generation against a live database); SeaQuery (dynamic querying belongs to DataFusion); Diesel and SeaORM (ORMs); pgvector (no vector data); pgrx (no server-side computation); testcontainers (the local server with `#[sqlx::test]` suffices). Each carries its revisit trigger in ADR-0112.

**Query surface.** Operational tables appear to DataFusion sessions as read-only providers. O9
chooses between the ADBC PostgreSQL driver (`adbc_core`/`adbc_driver_manager` 0.24, which accept
arrow-array ≥58 <60 and so fit arrow =59.3) and typed row-to-Arrow builders for the small fixed
operational tables. `datafusion-table-providers` 0.13.1 is blocked: its postgres crate requires
datafusion ^54 and arrow ^58, breaking the one type universe, and brings a second driver stack;
revisit when a release matches `datafusion =55.1.0`. Python gains runs, jobs, studies and a
progress stream.

### 9.9 Failure behaviour

- **Database unavailable:** durable operations fail with the infrastructure class, naming the connection target. Ephemeral solves are unaffected. There is no fallback to Delta control.
- **Crash:** leases expire, attempts become stale, and unpublished members remain reclaimable through the catalog.
- **Catalog commit ambiguity** (lost acknowledgement): settle by querying the catalog for the attempt's publication.

## 10. Alternatives

| Alternative | Why not |
|---|---|
| Keep global MINLP out of scope | Contradicts the maintainer's direction (2026-09-27) and the S10–S12 workloads |
| Couenne or Bonmin for MINLP | Duplicates SCIP. Couenne needs NL/ASL (D12); Bonmin has no C API and is heuristic on nonconvex problems |
| Bespoke outer approximation over HiGHS and Ipopt | Reimplements solver machinery (DP-13, G8) |
| sIPOPT for sensitivity | C++ only; duplicates §6. Kept as a parity oracle |
| Keep the Delta control table beside a PostgreSQL run registry | Two authorities for publication visibility, or a registry that cannot coordinate multi-writer publication (DP-01, R-10) |
| An in-memory fake of the operational store for tests | A second implementation of the contract; `#[sqlx::test]` isolates real databases |

## 11. Risks and open questions

- **SCIP build.** Build time, image size and the GMP/MPFR/Boost toolchain. Mitigated by the image build and a skill runtime receipt before adoption.
- **HSL.** No licence route exists for the maintainer. SPRAL SSIDS and oneMKL Pardiso (both licensed for personal use) and MUMPS with METIS replace it (ADR-0108; register R-34).
- **Process-wide numerical environment.** One BLAS/LAPACK provider, one OpenMP runtime, a pinned `MKL_CBWR` and SPRAL's OpenMP settings apply to the whole process (§5.4). Mitigated by admission checks and the recorded profile key.
- **`pounce-sensitivity` 0.12.0 API.** Unverified: it is in neither the corpus nor the registry. Verified at packet start; the fallback is `pounce-sens-core` on both routes.
- **Plan 21 overlap.** The kernel changes (§2) touch grammar, registry and compiler that Plan 21 built; K8 is complete (maintainer, 2026-09-27), so the M packets follow Plan 21's kernel discipline without waiting.
- **Operational store as a deployment dependency** for durable work. Mitigated by the `Ephemeral` class and the `just` recipes.
- **Catalog cut-over.** There is no import; existing publications are regenerated by rerun. The risk is limited to proving the catalog path before deleting the Delta control table.
- **Event volume.** Mitigated by batched or `COPY` inserts, retention policy and measurement in the final qualification.
