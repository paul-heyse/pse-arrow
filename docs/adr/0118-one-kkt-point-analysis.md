---
id: ADR-0118
title: Serve every NLP and QP route with one KKT-point analysis
status: accepted
date: 2026-09-28
deciders: [paul-heyse]
level: decision
principles: [AP-02, DP-01, DP-13, DP-16, PS-07, PS-10, PS-12]
blueprint: [§15.5, §19.4, §19.8, §25]
review: docs/design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0118
evidence: Proposed
supersedes: [ADR-0107]
superseded-by: null
revisit: sensitivity_agrees_with_ipopt_sens fails beyond its declared tolerance on a nondegenerate fixture, or a backend's solution cannot be re-observed in original coordinates with a multiplier for every original row.
verification: Architecture scenarios S13 and S14 and review scenario S02, settled by the Plan 22 tests. S0 kkt_inertia_certifies_second_order, second_order_verdicts_follow_the_inertia, kkt_factor_backsolve_matches_dense_reference, licq_failure_distinct_from_singular_curvature and fixed_assignment_resolve_keeps_local_analysis. S1 sensitivity_matches_analytic_nlp, sensitivity_withheld_when_sosc_fails, sensitivity_withheld_when_weakly_active, sensitivity_withheld_when_licq_fails, sensitivity_backend_independent (Ipopt, POUNCE and the SCIP re-solve), sensitivity_survives_presolve, reduced_hessian_sign_pinned, and sensitivity_agrees_with_ipopt_sens (Q1, parity container). S3 linear_regression_covariance_analytic, unidentifiable_fit_withholds_covariance, gauss_newton_covariance_labelled, profile_likelihood_matches_wald_on_linear_model, profile_chain_seeds_from_predecessor and covariance_withheld_without_declared_sigma. S4 uncertainty_propagation_linear_exact and propagation_withheld_when_upstream_withheld. Also N5 qp_sensitivity_through_kkt_analysis and M2c sensitivity_conditional_on_assignment.
standard: core-3.1/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s13, docs/plans/22-solver-capabilities-architecture.md#s14, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s02]
---

# ADR-0118: Serve every NLP and QP route with one KKT-point analysis

## Context

[ADR-0107](0107-sensitivity-covariance-uncertainty.md) admitted parametric sensitivity, the
reduced Hessian, covariance and uncertainty propagation under PS-12 validity, through two
mechanisms:
- on the POUNCE route, `pounce-sensitivity`'s `SensSolve`;
- on the Ipopt route, `pounce-sens-core` over a barrier-replica KKT.

Planning the remaining solver scope on 2026-09-28 found that:
- `pounce-sensitivity` 0.12.0 is in neither the registry cache nor the lockfile, and its API is known only from the `pounce-rs` facade;
- a barrier replica reproduces Ipopt's internal, barrier-perturbed system as a second KKT assembly;
- neither mechanism serves the SCIP fixed-assignment re-solve or the QP routes, which also need sensitivities (ADR-0103 item 6, ADR-0109 item 4);
- ADR-0107 restricted sensitivity to presolve `Off` or kept parameter columns.

Plan 22 N4 already factors a KKT matrix with FERAL to certify inertia (`conditioning.rs`).
The [solver scope packet](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-scope-execution.md#improvements-over-the-target-design)
adopts improvements I1–I4 in response.

## Scope

**Supersedes ADR-0107.** Its Outcomes 1, 4, 6 and 7 are restated below, so a reader need not
open ADR-0107. Outcomes 2, 3, 5 and 8 are replaced by items 5–10. ADR-0107's amendment of
Plan 20 §6 stands, and global identifiability stays outside the target.

This record delivers ADR-0109 item 4 (QP parametric sensitivity) through the same mechanism.
`pounce_convex::QpSensitivity` is excluded as a duplicate, and the rest of ADR-0109 stands.
Citations of ADR-0107 in ADR-0103 (item 6), ADR-0109 (item 4) and ADR-0110 (item 5, the
advanced step) now resolve to this record.

Governs blueprint §15.5 (the analysis and its mechanism), §19.4 (fit covariance and
intervals), §19.8 (propagation) and §25 (the fitting row). Implementation is Plan 22 S0, S1,
S3 and S4, with M2c for the commitment.

## Drivers

- **PS-12.** Every derived quantity states its validity conditions. It is withheld when they fail, and the reason is stored even though no data row exists.
- **PS-10.** The analysis runs only at a candidate that is qualified in original coordinates, whatever the backend reported.
- **AP-02 and DP-01.** One analysis, in one module, consumes a candidate in original coordinates. No backend type and no route-specific KKT reaches a consumer.
- **DP-13 and DP-16.** FERAL factors the matrix and reports inertia, and `pounce-sens-core` owns the sIPOPT algebra (Schur-complement step, reduced Hessian). Only the assembly is bespoke.
- **PS-07.** Second derivatives come from the model's own exact derivative programs, and their source is recorded.

## Options

| Option | Assessment | Selection |
|---|---|---|
| ADR-0107: POUNCE `SensSolve` plus an Ipopt barrier replica | Two mechanisms and two validity paths for one quantity. `pounce-sensitivity` is unavailable, and the replica duplicates Ipopt's internal system. Neither serves SCIP or QP routes, and presolve is restricted | Superseded |
| Backend-native sensitivity per route (sIPOPT, `pounce_convex::QpSensitivity`, HiGHS ranging) | One semantics per backend. sIPOPT has no C API | Rejected. sIPOPT (`ipopt_sens`) stays the parity oracle; `QpSensitivity` is excluded as a duplicate |
| Plan 20 §6: faer sparse LU on the KKT | No inertia, so second-order sufficiency cannot be checked | Rejected, as in ADR-0107 |
| **One active-set KKT at the candidate in original coordinates, assembled from the model's derivative programs and factored by FERAL, with `pounce-sens-core` over a FERAL `SensBacksolver`** | One mechanism and one validity record for every route. Independent of backend and presolve; its library parts are interface-checked | **Selected** |

**Library consideration (core §F).**
- *Capability:* local sensitivity at a KKT point, whichever backend found the point.
- *Candidates and fit:*
  - `pounce-sensitivity` is unavailable.
  - sIPOPT is C++ only, and Ipopt's C API does not expose its factorization.
  - `pounce_convex::QpSensitivity` covers QP only.
  - `pounce-sens-core` supplies the algebra over a caller-supplied backsolver, which fits.
  - FERAL supplies an LDLᵀ factorization with inertia and refined solves, which fits.
- *Decision:* no library assembles a KKT matrix from this model's derivative programs. That assembly is domain composition, bounded to one module (`pse-backend-native` `kkt.rs`).

## Outcome

### Retained from ADR-0107

1. **Admitted quantities** (ADR-0107 Outcome 1):
   - NLP parametric sensitivity of the primal and dual solutions;
   - the reduced Hessian;
   - parameter covariance and confidence intervals for steady and transient fits, as Wald and profile-likelihood intervals;
   - propagation of parameter covariance to outputs, Σ_y = J·Σ_θ·Jᵀ (the counterpart of IDAES `sens.py`).
2. **Typed validity record** (ADR-0107 Outcome 4, tightened by the review's F01 and F02).
   - Activity is classified against a declared threshold: strongly active, weakly active (ambiguous) or inactive.
   - Second-order sufficiency follows from the KKT inertia and the reduced-Hessian eigenvalues.
   - LICQ holds, and strict complementarity holds.
   - The candidate's original-coordinate KKT evidence qualifies it as `Stationary` or better, with a multiplier for every original row.
   - A fit additionally needs full response rank and the declared statistical model: weighted least squares with a declared standard deviation and unit importance for every included observation.

   A failed condition withholds the quantity and records which condition failed.
3. **Approximation labelling** (ADR-0107 Outcome 6). Gauss–Newton covariance is recorded with `approximation = gauss_newton`. It is valid under the declared statistical model with small residuals, and it neglects residual curvature. When the fit used exact second derivatives, including exact transient ones (ADR-0110), the exact covariance is used and recorded instead (item 8).
4. **Discrete problems** (ADR-0107 Outcome 7). Quantities are computed with the discrete assignment fixed, and are stated as conditional on it (ADR-0103). The assignment is the `Commitment` of item 9.

### Replaced and added

5. **One KKT-point analysis** (replaces Outcomes 2 and 3).
   - At a qualified candidate, `pse-backend-native` assembles one symmetric KKT matrix in original case coordinates. Its layout is `[x; active rows; active bound rows]`, and its blocks are the model's own Lagrangian-Hessian and Jacobian programs evaluated at the candidate's primal and dual values.
   - FERAL factors it (LDLᵀ with inertia).
   - The typed `KktPoint` records activity by original identity, LICQ from the inertia of `[I Aᵀ; A 0]`, curvature, inertia, a condition estimate and the backsolve residual.
   - The analysis serves Ipopt, POUNCE, the SCIP fixed-assignment re-solve and the QP routes (HiGHS, Clarabel, POUNCE-convex) alike.
   - There is no `pounce-sensitivity` route and no barrier replica.
6. **Library algebra.** `pounce-sens-core =0.12.0` runs over a FERAL-backed `SensBacksolver`, which solves by FERAL's refined solve:
   - `SensApplication` and `parametric_step` give the parametric step;
   - `IndexSchurData` over the parameter pin rows supplies the Schur data;
   - `compute_reduced_hessian_eigen` gives the reduced Hessian and its eigen-decomposition.

   The pin multipliers give df*/dp.
7. **Coordinates** (replaces Outcome 5).
   - The analysis works on original case columns, so presolve no longer restricts it, and presolve `Off` is no longer required.
   - It needs the postsolved multiplier of every original row. If postsolve does not recover one, or the recovered multipliers fail original-coordinate complementarity (the §25 recorded limit), the quantity is withheld with a typed reason. Presolve is never switched off as a fallback.
   - Results pass through the explicit normalization back-map, into physical units (parameter unit², output unit per parameter unit).
   - The reduced-Hessian sign convention is pinned by a unit test.
8. **Covariance by one rule** (refines Outcome 6).
   - When the fit used the exact Hessian, the covariance is exact: the B·K⁻¹·Bᵀ block of the fit's KKT analysis.
   - Otherwise it is Gauss–Newton, Σ = S·V·diag(s⁻²)·Vᵀ·S. It is computed from the existing response SVD with its right singular vectors, and never by forming JᵀWJ, which squares the condition number. The same SVD gives the identifiable subspace.
   - One fit has one covariance, labelled with its approximation.
   - Wald intervals use `statrs` quantiles.
   - Profile-likelihood intervals run as adaptive pin chains, two per parameter, with homotopy step control over the fit prepared once. Each point is seeded from its predecessor, and the seed is recorded as an input (PS-11).
9. **One pin transformation and one commitment.**
   - `Pinned` and `Unconstrained` move out of `execution/factorable.rs` into their own module, and serve the SCIP conditional re-solve, parameter pins (S1) and profile pins (S3). `Pinned::discrete` replaces `AlgebraicOracle::with_fixed_assignment`.
   - One `Commitment` records the fixed assignment behind conditional duals and sensitivities, for HiGHS `FixedLp` and the SCIP re-solve alike. `runtime.solve_runs` records it.
10. **Relations** (replaces Outcome 8).
    - A registry named structure `LocalValidity` lives in its own relation, `runtime.local_validity`, keyed by run, step and quantity. A withheld quantity has no data rows, so its reason needs this home.
    - Data relations: `runtime.parametric_sensitivities`, `runtime.reduced_hessians`, `runtime.parameter_covariances`, `runtime.parameter_intervals` and `runtime.propagated_covariances`.
    - Registry enums: `DerivedQuantity`, `WithheldReason`, `CovarianceApproximation`, `IntervalMethod` and `IntervalEnd`.
    - `DualQualification` gains `sensitivity_certified`.
11. **Propagation.** J comes from item 6 or from the fit's response derivatives. The validity of a propagated covariance is the conjunction of its upstream validity rows.
12. **Factor lifecycle.** The KKT factor never enters `SolveReport`. It is dropped after the step, or retained for the advanced step (Plan 22 Y5c) and charged to the job's allowance.

### Consequences

- `pounce-sens-core` becomes a direct dependency, beside the existing direct FERAL dependency (N4).
- `conditioning.rs` loses its two-pass second-order assembly (`second_order`, `Active`, `attach_second_order`), and `Evidence::second_order` becomes `Evidence::local`.
- Plan 22 S2 is deleted. ADR-0107's test `ipopt_route_sensitivity_matches_pounce_route` gives way to `sensitivity_backend_independent`.
- The active-set analysis differs from sIPOPT's barrier system by a term of the order of the barrier parameter at termination, so the parity tolerance must cover it.
- A weakly active constraint withholds the quantity; the analysis never picks a side for it.
- A fit with non-unit importance weights has no covariance.

### Compensating controls

- Withholding, with the reason stored in `runtime.local_validity`.
- Comparison against analytic NLPs, across backends, and against the `ipopt_sens` oracle.
- Negative controls: failed second-order sufficiency, failed LICQ, weak activity, an unidentifiable fit, and a fit without declared standard deviations.

### Confirmation

The tests named in `verification:`, run in Plan 22 S0, S1, S3 and S4. The
[change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0118)
interface-checked the library parts:
- `pounce-sens-core` 0.12.0 exports `SensBacksolver` (`dim`, `solve`), `SensApplication::parametric_step`, `compute_reduced_hessian_eigen` and `IndexSchurData`;
- FERAL 0.18.0 exports `Solver::solve_refined` and `inertia`;
- `pounce-sensitivity` appears in neither the registry cache nor `Cargo.lock`.

The decision itself is *Proposed*. Acceptance does not certify the implementation.

## Pros and cons

One mechanism and one validity record replace two routes that covered only part of the
backends. The cost is a bounded bespoke assembly, and the loss of sIPOPT's
barrier-consistent step, a difference of the order of the barrier parameter at termination.

## More information

- Solver scope packet: [improvements I1–I5](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/plans/22-solver-scope-execution.md#improvements-over-the-target-design) and packets S0, S1, S3, S4, M2c and N5, which own progress.
- [Change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0118), findings F01 and F02.
- Architecture companion [§6](../plans/22-solver-capabilities-architecture.md#6-sensitivity-covariance-and-uncertainty) (its two-route text is superseded here).
- Target review [T11](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t11); capability review L-N1 and S02.
- Related: ADR-0103, ADR-0105, ADR-0109, ADR-0110.

## Status history

- 2026-09-28 — proposed and accepted under the maintainer's authorization of Plan 22's full scope, and of the solver scope packet's improvements (2026-09-28), after the [change-tier review](../design_review/reviews/design_review_solver-scope-decisions_2026-09-28.md#adr-0118) returned Accept (author review, Proposed evidence level). Findings F01 and F02 were corrected in this record before acceptance. Supersedes ADR-0107: its Outcomes 1, 4, 6 and 7 are restated, and Outcomes 2, 3, 5 and 8 are replaced.
