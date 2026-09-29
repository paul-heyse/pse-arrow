---
id: ADR-0107
title: Admit parametric sensitivity, covariance and uncertainty propagation under PS-12 validity
status: superseded
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [DP-11, DP-13, PS-07, PS-10, PS-12]
blueprint: [§15.5, §19.4, §19.8, §25]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t11
evidence: Proposed
supersedes: []
superseded-by: ADR-0118
revisit: The S1 API verification finds that neither pounce-sensitivity 0.12.0 nor pounce-sens-core composes with the FERAL factory and the presolve back-map, or sensitivity_agrees_with_ipopt_sens fails beyond its declared tolerance.
verification: Review scenario S02 and architecture scenario S13, settled by the Plan 22 S1–S4 tests sensitivity_matches_analytic_nlp, sensitivity_withheld_when_sosc_fails, sensitivity_agrees_with_ipopt_sens (parity-container oracle), ipopt_route_sensitivity_matches_pounce_route, linear_regression_covariance_analytic, unidentifiable_fit_withholds_covariance and uncertainty_propagation_linear_exact.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s02, docs/plans/22-solver-capabilities-architecture.md#s13]
---

# ADR-0107: Admit parametric sensitivity, covariance and uncertainty propagation under PS-12 validity

## Context

§25 refuses covariance, global identifiability and uncertainty claims; §19.8 records no
uncertainty propagation; §19.4 reports local response rank without a statistical claim. The
IDAES 2.13 target (ADR-0097) includes parmest covariance and confidence intervals and `sens.py`
propagation. Plan 20 §6 proposed a bespoke faer sparse LU on the KKT system, which cannot report
inertia on a symmetric indefinite matrix and so cannot check second-order sufficiency
([capability review L-N1](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#l-n1), G8).

## Scope

Amends the target in §19.4, §19.8, §25 and the §15.5 limit row, and Plan 20 §6. Binds which
derived quantities are admitted, their mechanism, validity record and coordinates. Global
identifiability remains outside the target. ADR-0083's statement that no covariance follows
"from a solve or local rank" stays true: covariance follows only from a validated reduced
Hessian or a declared Gauss–Newton approximation.

## Drivers

- **PS-12.** Sensitivities and covariances state their validity conditions and are withheld
  when those fail.
- **DP-13 and G8.** Library-owned sensitivity machinery (sIPOPT semantics in POUNCE) instead of
  a bespoke KKT solver.
- **PS-07.** Exact second derivatives are required; the derivative source is recorded.
- **DP-11.** Results are stated in original physical coordinates with the back-maps explicit.
- **PS-10.** Sensitivity is computed only at a qualified candidate.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Plan 20 §6: faer sparse LU on the KKT | No inertia, so SOSC cannot be checked; bespoke assembly | Rejected |
| POUNCE `sensitivity` on the POUNCE route and `pounce-sens-core` with a FERAL LDLᵀ backsolver on the Ipopt route | Library-owned semantics, inertia and PS-12 inputs on both routes | **Selected** |
| sIPOPT through Ipopt | C++ only, no C API; kept as the `ipopt_sens` parity oracle | Rejected for production |
| Gauss–Newton from the response SVD alone | Drops residual curvature; transient and limited-memory fits only | Complement, with its approximation stated |

## Outcome

1. **Admitted quantities.** NLP parametric sensitivity of primal and dual solutions; the
   reduced Hessian; parameter covariance and confidence intervals for steady and transient
   fits; profile-likelihood intervals as a study over fixed parameters; uncertainty
   propagation of parameter covariance to outputs (J Σ Jᵀ, the counterpart of `sens.py`).
2. **POUNCE route.** `pounce-rs` feature `sensitivity` (`SensSolve` with deltas and the reduced
   Hessian) beside `pounce::Session`. Plan 22 S1 first verifies the `pounce-sensitivity` 0.12.0
   API; if it does not fit, `pounce-sens-core` serves both routes. That choice is made once at
   S1 and recorded in the plan; it is never a run-time fallback.
3. **Ipopt route.** `pounce-sens-core` (`SensApplication`, `parametric_step`) over a
   barrier-replica KKT assembled from the `NlpOracle` derivative programs and factorized by a
   FERAL LDLᵀ backsolver that reports inertia. The KKT assembly is the only bespoke part and is
   justified: Ipopt's C API does not expose its factorization.
4. **Validity record (typed).** Activity classification with a declared threshold (strongly
   active, inactive, ambiguous); second-order sufficiency from the reduced-Hessian eigenvalues
   and the factorization inertia; LICQ from the Schur-step factorization; strict
   complementarity; for fits additionally a qualification of `Stationary` or better, full
   response rank and the declared statistical model (weighted least squares with declared
   standard deviations). A failed condition withholds the quantity and records why.
5. **Coordinates.** Results map back through presolve column maps (presolve `Off`, or parameter
   columns kept) and the normalization back-map H_phys = S_f·S⁻¹·H_norm·S⁻¹, with physical units
   (parameter unit², output unit per parameter unit). The reduced-Hessian sign convention is
   pinned by a unit test.
6. **Transient fits.** Covariance from the Gauss–Newton matrix JᵀWJ of the existing response
   SVD is recorded with `approximation = gauss_newton`: valid under the declared statistical
   model with small residuals, and neglecting residual curvature. When exact transient second
   derivatives are available (ADR-0110) the exact route is used and recorded instead.
7. **Discrete problems.** Computed with the discrete assignment fixed, and stated as
   conditional on it (ADR-0103).
8. **Relations.** New registry relations for sensitivities, reduced Hessians, covariances,
   intervals and propagated output covariance, each carrying its validity columns.

### Consequences

A direct FERAL dependency for inertia (Plan 22 N4). Presolve policy constrains sensitivity
(parameter columns must survive). Plan 20 §6 changes its mechanism.

### Compensating controls

Withholding on failed validity; comparison against analytic NLPs and the `ipopt_sens` oracle;
the unidentifiable-fit fixture must withhold covariance.

### Confirmation

The tests named in `verification:` (Plan 22 S1–S4). The `pounce-sensitivity` API is *Proposed*
until S1; `pounce-sens-core` was *Interface-checked* by the capability review.

## Pros and cons

The library route supplies sIPOPT semantics with inertia on both NLP routes. The Ipopt route
still needs a replica KKT assembly, and Gauss–Newton covariance for transient fits is an
approximation that must be labelled.

## More information

- Architecture companion [§6](../plans/22-solver-capabilities-architecture.md#6-sensitivity-covariance-and-uncertainty).
- Target review [T11](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t11).
- Capability review L-N1, S02; Plan 20 §6 (amended in D0).
- Related: ADR-0083, ADR-0097, ADR-0103, ADR-0110. Plan 22 packets S1–S4, N4.

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T11 were corrected in this record before acceptance.
- 2026-09-28 — superseded by ADR-0118.
