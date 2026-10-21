---
id: ADR-0209
title: Qualified fitting residual methods and neutral scratch reuse
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-07, PS-07, PS-12]
blueprint: [§5.3, §14.3, §19.4]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A qualified residual workload exposes unsupported constraints, objective interpretation, library work control or a significant complete-operation regression.
verification: Plan48e E07 exercises actual weighted FitOracle residuals and responses, independent original objective/gradient/quality, native curvature alternatives, domain/cancel/work refusal, and neutral callback-drop/reset/rebind ownership across final assessment and profile maps.
---

# ADR-0209: Qualified fitting residual methods and neutral scratch reuse

## Context

Fitting already owns weighted residual meaning, authored constraints and exact responses.
Its native NLP and final independent assessment reconstruct compatible mathematical scratch.
A residual-native optimizer can use the existing residual/response authority directly for
eligible fits, while neutral allocation reuse can remove repeated preparation without
substituting a cached producer result for final original checks (blueprint §19.4).

## Scope

Amend the fitting binding for a library-owned residual method and finite operation/chain-local
neutral scratch handoff. This record does not authorize a handwritten optimizer, changed loss,
relaxed physical accuracy, native success as scientific qualification or inexact integration.
Initial residual-method eligibility is unconstrained fitting with no finite parameter boxes;
box/general-constraint requests retain their existing native NLP route.

## Drivers

Use the consumed residual/Jacobian products directly, preserve source weights and coordinate
meaning, keep native iteration/globalization in the library, and perform fresh original checks.
Remove redundant compatible allocation/preparation while preserving provider effects, actual
execution ownership, changed profile/free-coordinate maps and failure isolation. A sound
first-principles reduction in work can justify adoption without a small noisy timing advantage;
a significant greater-than-twofold slowdown requires explanation before retaining that route.

## Options

Ipopt remains the class-correct constrained route, with its existing exact, Gauss–Newton and
limited-memory curvature choices. A library-owned LM residual method avoids expressing an
eligible residual problem solely through scalar objective/Hessian callbacks. The production binding pins levenberg-marquardt at full revision
`e67066cf1351e91e054c30fcb2ff49d005db1e50`, using the existing nalgebra 0.35 family.
The source comparison against released 0.15.0 (`ab5e16ffaf3b60fae8a73725d0886cda858a56e5`)
changes only the normal and development nalgebra dependency from 0.34 to 0.35; algorithm,
termination, traits and test sources are unchanged. Actual oracle controls still qualify the binding.

Keeping a complete FitOracle would retain numerical products, providers and controls. Existing
CaseWorker recycle_on_drop, CaseScratch reset and worker_from_scratch instead expose neutral
storage: provider destruction and control detachment precede publication, reset poisons numeric
slots, and current providers/scope bind again. Use this mechanism in bounded local slots rather
than adding a generic pool or treating invalidate_trial as a provider reset.

## Outcome

The fitting-owned `FitOptimizer::{Automatic, NativeNlp, ResidualLm}` is included in the
source document and complete profile identity. `Automatic` selects residual LM only after
actual no-box/no-general-constraint eligibility and exact supported weighted residual/response
meaning are established. `NativeNlp` preserves explicit NLP selection and its portfolio;
`ResidualLm` refuses an unsupported problem before source or native effects. Native solver
selection remains the policy of the NLP alternative, rather than an invented LM backend.
Explicit native backend, non-response/curvature, required presolve, reconstruction/block,
declared composition, connected branch, recovery/advice and native-option contracts exclude
automatic LM or cause explicit LM refusal before effects. Shared physical requirements,
deadlines, cancellation, actual foreign storage and finite work remain common. No required
native-only request is silently dropped because its parameter bounds happen to be absent.
Native iteration, factor and proof caps exclude LM while those counters are unavailable;
actual evaluation/attempt caps remain supported. The shared native-scaling permission maps
to the library's diagonal-scaling switch after physical coordinate normalization.

`ProfileRef` uses `NumericalMethod::{Backend(Backend), ResidualLm}` and the complete effective
key. This changes consumed strategy wire/hash interpretation: the actual numerical-strategy
document becomes version 4 with `NumericalStrategyV4` framing; all current producers/consumers
migrate together without an old-field compatibility shim. Fitting profile identity becomes
`FitProfileV5` because requested optimizer participates in its meaning. Replay constructors
preserve that request explicitly and advance their current artifact contract.

The common callable strategy names the actual residual method. Completion, profile chains,
result artifacts and Python documents retain the real LM termination and residual/Jacobian work;
an absent NLP report does not imply all-fixed evaluation or a fabricated Ipopt status. Both
alternatives enter the existing single independent final assessment and scientific completion.
Physical and global numerical controls keep their original meaning. Unboxed residual covariance
uses the proven empty active-bound set without fabricating native multipliers; native/constrained
covariance keeps its existing multiplier checks. Pin failures retain the literal native residual
record and first latched source cause before any independent assessment is attempted.

An eligible fitting residual route consumes the same FitOracle weighted residuals and exact
response Jacobian in the current physical coordinates. Native LM owns iteration and trust-region
updates. Explicit native termination and the first typed source refusal are retained separately;
a callback returning None cannot turn a domain, cancellation or task-work limit into convergence.
The existing finite iteration control conservatively caps actual parameter-update callbacks,
including rejected/restored trials; it is reported as parameter updates rather than an invented
native iteration counter. The library's finite residual evaluation cap is also enforced; typed
work refusal dominates callback None. Native iteration/factor counts remain unavailable.
All conversions and library workspace are admitted before allocation and included in complete
operation work. The known dense workspace extent must also fit the effective foreign allowance
when selecting the method and its preparation demand. An oversized dense route is ineligible
for Automatic, which retains the current NLP route; explicit ResidualLm refuses before effects.
The execution boundary checks its actual current allowance again. This is a known workspace
eligibility condition, not an empirical dimension or timing cutoff. No finite bound or authored
general constraint is silently ignored.

Final original assessment constructs a fresh numerical trial and checks the original objective,
constraints and quality independently of the optimizer's product cache. Existing fitting results,
rank/covariance assumptions and profile-likelihood interpretation remain authoritative. A new
binding is adopted only for its qualified eligibility; constrained fits keep the existing NLP.

### Consequences

Neutral scratch has one operation/chain-local owner, tagged by the actual experiment role and
exact immutable CaseAssembly consumed by that role. Outer FitProblem, free/rank maps and prior
execution-scope identities are not compatibility keys for buffers that contain none of them. It becomes available only after actual callback Drop and native
return. It contains no Point, result, provider bank, applicability witness, execution, completed
work ledger or active compute permission. Reset overlap and slot metadata are prepaid before
effects. Fresh current providers and the current execution scope are rebound before any new evaluation.
Dynamic consumers reuse qualified neutral CaseScratch for actual immutable program roles on
the owning thread; current frames, wrapper layouts, caches and native workspaces are rebuilt; absent a complete child-return/reset contract, that subset stays fresh.
Owner-affine native workspaces remain governed independently by ADR-0205.

### Compensating controls

An unknown/incompatible map, failed reset, surviving callback owner, source/provider refusal,
cancellation or native failure discards the candidate rather than replaying an old numerical
result. Equal vector lengths do not establish compatibility. Profile pins and changed free/rank
maps retain their existing mathematical transformations and independent final evaluations.

### Confirmation

**Tested, focused:** canonical-runtime controls
`fitting_residual_production_matches_native_complete_operations_at_original_accuracy`,
`fitting_owned_residual_lm_uses_real_method_and_fresh_original_assessment`,
`fitting_residual_lm_public_result_reopens_actual_method_termination_and_profile` and
`fitting_residual_dense_workspace_eligibility_precedes_numerical_preparation` pass actual
weighted residual/response meaning, independent original quality/covariance/profile products,
honest persisted/reopened method/status and known dense-workspace refusal/fallback before
preparation. Callback domain/cancel/work refusal and neutral scratch Drop/reset/current-map
controls pass their focused selections. Plan48e records modes, commands, zero-baseline counts
and the separately corrected actual-child transient-curvature refusal/output-atomicity
control, whose exact canonical rerun passes 1/1 with zero compiler warnings/errors against zero.

**Measured, bounded complete operations:** the matched production control exercises ordinary
and extreme scales, near-collinearity and positive-domain nonlinear response in both route
orders at unchanged 0.001, including preparation, solve and fresh quality/covariance/prediction.
LM complete times are 2.058–3.690ms across these fixtures; residual/Jacobian counts are literal,
not inferred iterations. This is not a universal performance or objective-gap guarantee.
Independent analytic/scaled-gradient/quadratic-gap checks preserve the distinction between
scaled stationarity and a physical objective-gap target.

Owner-affine native storage remains independently governed by ADR-0205; parallel experiment
neutral child return stays fresh without a complete return/reset contract. Bounds, general
constraints and required native-only contracts retain NLP or explicit unsupported refusal.
Formal decision status remains proposed; focused controls do not replace Plan48h assembled
qualification.

## Pros and cons

Residual-native iteration and neutral allocation handoff remove distinct forms of redundant work.
They add an explicit class eligibility boundary and reset/ownership obligations; constrained and
unsupported ownership populations continue through their existing qualified native paths.

## More information

- [Plan48e](../plans/48e-dynamics-fitting-and-preconditioning.md)
- [ADR-0205](0205-fixed-profile-idas-storage.md)
- [Fitting owner](../authoritative_design/sections/workflows-and-results.md#section-19-4)

## Status history

- 2026-10-20 — Proposed before production residual-method adoption and neutral fit scratch handoff; private inquiry qualification remains pending.

- 2026-10-20 — Updated focused tested evidence and qualification limits from Plan48e; formal status remains proposed.
