---
id: ADR-0110
title: Extend the dynamics profile with IDAS schedules and events, Diffsol methods, adjoint and second-order sensitivities and shooting
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [DP-13, PS-07, PS-08, PS-10, PS-11]
blueprint: [§13.6, §18.9, §19.4]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A dynamic case needs a differential index above one, a variable-layout DAE, or reset sensitivities on the IDAS route.
verification: Review scenario S09 and architecture scenario S14, settled by the Plan 22 Y1–Y5 tests (the PID fixture with piecewise inputs against the IDAES PETSc example), idas_events_without_sensitivities, idas_constraints_keep_positivity, idas_staggered_matches_simultaneous, sdirk_matches_bdf_on_vessel, tsit45_refuses_mass_matrix, diffsol_klu_matches_faer_lu, adjoint_gradient_equals_forward_on_transient_fit, checkpoint_memory_bounded, gauss_newton_hessian_matches_jtwj, second_order_adjoint_matches_finite_difference, nmpc_closed_loop_on_antiwindup and shooting_matches_simultaneous_optimum.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s09, https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s14]
---

# ADR-0110: Extend the dynamics profile with IDAS schedules and events, Diffsol methods, adjoint and second-order sensitivities and shooting

## Context

ADR-0084 limits dynamics *initially* to the native ODE/index-1 fixed-mass profile; ADR-0093
adds a narrow IDAS route with recoverable trials and refuses hybrid IDAS sensitivities, with the
revisit trigger "a selected request needs hybrid sensitivities with recoverable residual trials".
§13.6 records that IDAS has no events or input changes, and that adjoint sensitivities are
unsupported. The IDAES PETSc PID parity case needs scheduled input changes with recoverable
trials (review [S09](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s09),
[L-D1](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#l-d1)); transient
fits with many parameters need adjoints and better Hessians (L-D2, L-D3); NMPC and MHE need
shooting routes and rolling horizons (L-D4).

## Scope

Adds within ADR-0084 and ADR-0093; supersedes neither. Their decisions stay true: provider
validity stays separate from derivative and dynamic eligibility, and the profile stays
ODE/semi-explicit index-1 with a fixed mass matrix. ADR-0084 called the profile *initial* and
ADR-0093's revisit trigger fires. ADR-0093 itself extended ADR-0084 in exactly this way,
without supersession. ADR-0084's "uncertainty claims remain outside this profile" is lifted for
fits by ADR-0107 under PS-12 validity, not by this record. KINSOL extensions (Plan 22 Y6) are
bindings within ADR-0083 and need no decision.

## Drivers

- **DP-13 and PS-07.** Adjoint, second-order and consistent-initialization machinery comes from
  SUNDIALS and Diffsol, not from bespoke code; derivative sources are recorded.
- **PS-08.** Recoverable trials at input-change times are a declared strategy.
- **PS-10.** IDA flags map to typed terminations ([F06](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f06)).
- **PS-11.** Dynamic modes declare index, consistent initialization, events and integration
  policy; rolling horizons reuse prepared structure with warm starts recorded.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep the ADR-0093 IDAS refusals | PETSc PID parity and many-parameter transient fits stay out of reach | Rejected |
| Bespoke reset-sensitivity saltation on IDAS | No library saltation; bespoke numerical machinery | Rejected (events with sensitivities stay on Diffsol) |
| CVODES or ARKODE | No capability beyond Diffsol and IDAS in the reviewed routes | Rejected (trigger: an authored IMEX or multirate need) |
| Extend IDAS and Diffsol within the fixed-mass index-1 profile, library-owned | Covers S09 and S14 with no new profile class | **Selected** |

## Outcome

1. **IDAS.** Scheduled input changes with recoverable trials at change times (`IDAReInit`,
   `IDASensReInit`, `IDACalcIC`, `IDAGetSensConsistentIC`), so forward sensitivities cross a
   scheduled change; events and roots without sensitivities (`IDARootInit`,
   `IDASetRootDirection`); `IDASetConstraints` for sign constraints; staggered sensitivities;
   SPGMR and SPFGMR with preconditioner hooks built from compiled Jacobian blocks; `IDA_Y_INIT`.
   IDA return flags map to typed terminations. Events with sensitivities stay on Diffsol, which
   owns reset sensitivities.
2. **Diffsol.** SDIRK methods `tr_bdf2` and `esdirk34`; explicit `tsit45` only for mass-free
   ODEs (refused with a mass matrix); the KLU backend through the `suitesparse` feature; the
   method is a typed profile field.
3. **Adjoint sensitivities.** Diffsol adjoint with checkpointing is primary (operator adjoint
   traits over faer). The IDAS adjoint (`IDAAdjInit` through `IDASolveB`, `IDAQuadInitB`)
   serves the recoverable-trial profile. A gradient-only fit mode consumes either; forward
   sensitivity remains the default wherever the full response Jacobian is needed (rank,
   covariance).
4. **Transient Hessians.** `HessianMode::GaussNewton` (JᵀWJ from forward sensitivities) and
   exact second-order sensitivities through IDAS forward-over-adjoint (`IDAInitBS`,
   `IDAQuadInitBS`); limited memory remains available.
5. **Dynamic optimization.** Simultaneous collocation stays primary. Single and multiple
   shooting are explicit routes over Diffsol forward or adjoint sensitivities; the NLP over
   shooting nodes runs through the one NLP runner (Plan 22 A2). NMPC and MHE are compositions
   over the staged-sequence primitive (A6) with warm restarts (N2), advanced-step sensitivity
   (ADR-0107) and per-step records in the operational store (ADR-0112).
6. **Limits kept.** Higher-index and general implicit DAEs, variable-layout modes and IDAS reset
   sensitivities remain outside the profile and are refused before native work.

### Consequences

§13.6's limits narrow when Y1–Y5 land. The dynamics profile gains typed method and Hessian
modes, which enter the profile identity. Adjoint checkpoints need a memory reservation.

### Compensating controls

Refusal before native work for anything outside the profile; typed IDA terminations;
cross-checks between forward, adjoint and finite-difference derivatives.

### Confirmation

The tests named in `verification:` (Plan 22 Y1–Y5). *Interface-checked*: the IDAS and Diffsol
entry points were read in the `native-solver-libraries` corpus by the capability review (L-D1–L-D5);
IDAS sensitivities are not runtime-qualified by the skill.

## Pros and cons

Every extension is library-owned and stays in one profile class. The cost is a wider typed
profile and two adjoint implementations, justified by their different trial-recovery
behaviour.

## More information

- Architecture companion [§7](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#7-dynamics-and-root-finding-extensions).
- Capability review F06, F12, L-D1–L-D8, S09.
- Related: ADR-0083, ADR-0084, ADR-0093, ADR-0107, ADR-0112. Plan 22 packets Y1–Y5.

## Status history

- 2026-09-27 — proposed (Plan 22 D0). Extends ADR-0084 and ADR-0093 within their decisions;
  ADR-0093's revisit trigger fired.
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level).
