---
title: "Plan 53a: Scientific assurance and references"
status: in-progress
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md, docs/plan_improvement_reviews/test-architecture-remaining-2026-10-24.md]
scenario_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
doc_role: plan
doc_owner: docs/plans/53-test-architecture-target.md
doc_topics: [current-work, design, validation, numerical-execution]
---

# Plan 53a: Scientific assurance and references

## Target and boundary

**Proposed:** close the named scientific discriminator gaps in [R01](../design_review/reviews/design_review_test-architecture-target_2026-10-23.md#r01), keeping independent primitive tests, shared model checks, actual native journeys, durable readback and selected parity. The [coordinator](53-test-architecture-target.md#package-state-and-execution-route) owns progress and DTA dispositions. This companion owns expected scientific meaning, not scheduling, preparation policy or a universal oracle. The package designs below retain their acceptance obligations; the execution evidence section distinguishes implemented, tested and remaining acceptance.

Policy reuse is legitimate when another responsibility is under test. DTA-001 supplies no established escaped fault in the inspected resolver/conversion/direct-export contracts: retain `engineering_accuracy_uses_frozen_scale_and_equivalent_unit_magnitudes`, `plan14::resolved_physical_allowance`, runtime direct projection and parity scale cancellation. Do not tighten allowances or add redundant tests merely to make a repair. DTA-008's complementary static/contextual readiness remains withdrawn. Preserve physical units, basis, reference state, signs, structural admission, original-space feasibility and exact/approximate labels across new cases (blueprint §14, §15–§18, §23.2, §24.1).

## Packages and independent expectations

<a id="a01"></a>
### A01 — Truthful local status and comparison controls

In the HiGHS adapter's actual test owner, replace the obsolete fallback-spelling exhaustiveness guard with an independently declared code/name/category/assurance expectation for every declared adapter status, including time, iteration, solution and memory limits, ambiguity and unknown fallback. A closed enum helps arm coverage but does not establish correct categories. A removed/misclassified declared arm must fail a meaningful expectation.

Repair the three local scientific comparators identified by DTA-007 with explicit finite actual/expected values and equal extent or sample-time correspondence before comparison. Keep their distinct absolute/relative/quantity policies; do not introduce a universal tolerance helper. Targeted tests include NaN/infinity, missing/trailing elements and a physically meaningful wrong result, alongside valid cases. Delete replaced weak assertions in the same change. This package is independent of invocation work.

<a id="a02"></a>
### A02 — Regime and model-variant witnesses

Settle [I01](53-test-architecture-target.md#inquiry-outcomes-and-remaining-questions) by deriving a concrete absent-phase bound for a fixture away from transition under the authored smooth-min realization. The existing dimensionless resolution is 0.001 and `epsilon_phase=1e-4`. For positive complementary quantities the zero relation gives `a*b=epsilon_phase²/4`; an independent positive lower bound on slack bounds the absent fraction by `epsilon_phase²/(4*slack_lower_bound)` plus propagated feasibility allowance. Apply that approximation envelope with the existing absolute fraction resolution. Record the fixture's slack and feasibility premises. Historical ±10% final-value acceptance stays separate; smooth/disjunctive/penalty realizations need not share exact endpoints. An arbitrary 0.99 cutoff or universal exact-zero assertion is not selected.

Add a deliberately interior wrong-regime control that fails the regime claim while preserving the existing `nested_vapor` tight fraction/material-balance witness. Add actual solved countercurrent-exchanger evidence and an unequal-pressure mixer case that separates minimum-pressure behavior from maximum/mean alternatives. Keep per-definition coverage honest: a declaration inventory is not solved-variant coverage. The PID journey already has nonzero default `Td=0.1` dynamic evidence; do not duplicate it to manufacture a gap. Its independent generator belongs to A04.

Acceptance uses targeted authored conformance/runtime/native cases, independent regime/variant properties and normal original-space closure checks. B04 may later change input construction; it must preserve these properties. A02 does not wait for minimal scientific closures.

<a id="a03"></a>
### A03 — Integrated second-order, sensitivity and covariance evidence

Keep the analytic PR primitive Hessians in `implicit_property_tests.rs`. Add a bounded integrated witness through the actual modeling/binding owner that challenges root identity, keyed coordinates, mixed second-order terms and the production sparse lower-triangle assembly. The review's upper-triangle shorthand does not change the actual production storage convention. Use a nontrivial independently analytic composition or directional second-order relation. Reusing the same questionable primitive as expected output would not close the integration gap. A swapped root/binding, missing cross term or coordinate permutation must change the expected assertion.

Use the concrete nonlinear constrained problem `min 0.5(x-a)²+0.5y²`, `y-x²=0`, with `L=f+lambda*g`. At `a=3`, `(x,y,lambda)=(1,1,-1)`, derivatives with respect to `a` are `(1/7,2/7)` and value Hessian `6/7`. Omitting constraint curvature produces denominator 5. At `a=18`, `(2,4,-4)`, derivatives `(1/25,4/25)` and value Hessian `24/25` discriminate stale nonlinear-point evaluation. Carry unequal coordinate/row/objective scales and keyed permutations through actual publication and reopening, rather than stopping at a low-level matrix assertion.

For `workflow/fitting/covariance.rs`, use free parameter `x`, state `y` constrained to `x²`, observations `[x~3,y~0]`, absolute sigma and importance one. Under the existing half-weighted-RSS convention and `B K^-1 Bᵀ` without a residual-variance multiplier, exact parameter variance is `1/7`; Gauss–Newton is `1/5`. With observations `[18,0]`, exact is `1/25`, Gauss–Newton `1/17`. Retain the nonlinear state constraint, unbounded/full-rank parameter and qualified KKT/curvature premises; an unconstrained polynomial fit would not expose missing constraint curvature. Assert exact/approximate labels and variance through the authored fit/result boundary and durable readback. Sensitivity, covariance and lifecycle remain distinct assertions even when the science is shared.

These independent analytic expectations were checked against the actual sign/scaling/statistical convention during implementation; native acceptance remains recorded below. Check the algebra against the actual sign/scaling/statistical convention before coding the witness; changing the production convention to fit the expectation is not authorized. Preserve existing linear, singularity, active-bound and independent primitive controls. Targeted native/runtime tests must reject the named wrong curvature/point/layout alternatives; no mandatory mutation campaign is added.

<a id="a04"></a>
### A04 — Reproducible independent reference generation

Resolve I02 for the three missing committed entrypoints named in `references.pse`: `pfr_reference`, `cstr_controller_reference`, `decay_fit_reference`. Recover a verifiable independent generator or commit a reduced-equation generator with exact inputs, equations/method, numerical tolerances, exact software environment, output locator and correspondence to the consumed PFR/controller/decay artifacts. Use established numerical routines suitable for the declared method; do not build a parallel ODE solver. Respect the clean-room IDAES boundary: inspect behavior, never copy reference code/docstrings/comments.

Reproduce the frozen reference claim in that declared environment. If a discrepancy appears, inspect derivation, inputs, rounding and method independently; do not overwrite goldens from current production outputs. Publish the reason and reviewed reference change before migrating consumers. Retain frozen inputs and independent derivation with the generator; remove stale unreproducible locator claims once replaced. Installed parity keeps its exact IDAES environment and fails rather than skips missing prerequisites.

<a id="a04-consumer-integration"></a>
### Remaining A04 consumer integration

**Proposed — [PI08](../plan_improvement_reviews/test-architecture-remaining-2026-10-24.md#PI08):**
retain the successful reference reproduction recorded below and resolved I02/DTA-003. The remaining
obligation is acceptance through the real selected consumers, not another generator rewrite.
Coordinator inquiry I10 owns the bounded decision; A04 retains implementation/acceptance ownership.

| Selected consumer | Evidence and next discriminating inquiry | Repair boundary and acceptance |
|---|---|---|
| Integrated controller | Retained report reaches a missing guard/numerical-input layout invariant in `region_composition`; inspect actual authored consumed inputs versus complete declared layout. | Correct the selected fixture/layout integration and demonstrate all consumed inputs correspond before its actual native/reference assertions. Do not weaken complete-layout admission. |
| Simultaneous controller | Retained report reaches `Workspace::admit_live_root` compiler retention admission; inspect the live preparation/retention set and necessary peak lifetime. | Remove avoidable retained state or establish sufficient explicit receiving premises for this selected case. The limit is not an OS-memory/functional-slot diagnosis; justify any budget change from required live work, never merely a green report. |
| PFR | Retained report has closure-validity refusals and 24 suspicious entries in 1,877 derivative comparisons, without missing sparsity. Examine sampling/step/scaling, cancellation, domain and actual derivative premises separately. | Distinguish production derivative error, invalid evaluation/envelope and diagnostic numerical effects using independent selected evidence. Preserve original-space physical/fresh numerical checks; neither harmless noise nor a production defect is established in advance. |

Before rerunning, check whether the relevant premise changed and select the smallest run that
settles it. Targeted wrong-case controls must expose the corrected contract, followed by the actual
selected native/reference consumer journey. The independent PFR/controller/decay inputs, equations,
frozen values and existing tolerances remain unchanged; do not loosen them or increase budgets
simply to obtain passing output. Preparation shared by D02/D03 never certifies a new numerical state.

If a selected failure requires a broader solver/formulation redesign, record its exact cause,
needed decision and constrained A04/S01 evidence at this owner. Seek separate selection for that
broader remedy; keep acceptance incomplete or explicitly dispositioned rather than moving it into
repository health or counting generator passes as consumer success. No full solver-fleet,
reference-regeneration, benchmark or environment-capacity campaign follows from this inquiry.

## Integration and completion

A01/A02/A03 may proceed independently once their scientific assumptions are settled. A04's reference provenance may inform A02's PID preservation, but does not block unrelated status/composition work. B02 owns the typed origin-refusal change and admitted single-change control for DTA-006; this companion contributes the independent scientific expectation. B04/B05 can replace repeated preparation only after the same witnesses still exercise fresh numerical states.

Completion requires the targeted wrong-case controls, independent reference reproduction and assembled scientific/readback journeys selected by [53g](53g-qualification-and-publication.md). It does not imply all formulations, solvers, platforms or parity routes were qualified. No benchmark or new tolerance policy is required. Retire replaced assertions/reference locator machinery alongside migrated consumers, preserving retained policy and primitive evidence.


## Execution evidence — 2026-10-24

The [coordinator's current acceptance boundary](53-test-architecture-target.md#current-acceptance-boundary--2026-10-24)
owns package progress, finding dispositions, exact modes, commands and current results.

**Implemented/Tested:** A01's independent 19-status inventory/unknown fallbacks and three
quantity-specific finite/extent/time comparators pass their meaningful controls. A02's nine-feed
phase journey and wrong-regime classifier controls pass; actual countercurrent exchanger and
unequal-pressure minimum mixer pass their independent expectations and original-space physical
checks. Supported water-only specialization isolates these two new variants. Native component
reports are assessed before complete original-space reconstruction, which can have a Constant
outer outcome. The tight nested-vapor witness and existing PID declarations remain present;
current assembled PID qualification is explicitly limited below.

**Implemented/Tested:** A03's two independently analytic PR roots pass integrated value,
Jacobian and all three lower-triangle weighted-Hessian cells in both caller coordinate layouts.
Expectations and row multipliers use the returned semantic identities, because selected rows
retain source order. The independently known liquid root seeds a declared-point assembly test;
actual runtime isolation still proves root selection. This does not claim default root-convergence
accuracy, direct Hessian-action preparation or inner numerical-requirement propagation. Both
nonlinear sensitivity/value-Hessian points and exact-versus-Gauss–Newton covariance points pass
actual publication and durable reopening. Reopened rows are compared in their declared primary-key
order, preserving full row content, identity and extent rather than assuming query ordering.

**Implemented/Tested:** all three independent generators reproduce frozen consumed values in
locked CPython 3.12.3 / NumPy 2.5.3 / SciPy 1.18.1: PFR 7/7, controlled CSTR 19/19, decay 25/25,
with four comparison/provenance controls passing. PFR/controller generation reports were refreshed
after final authored source changes. The PFR concentration difference is about `2.9e-9 mol/m³`;
per maintainer guidance standalone reproduction allows `1e-7 mol/m³`, far below the production
`1 mol/m³` acceptance, with separate tight DOP853/Radau agreement. No frozen golden changed.
I02 is resolved; reproduction is numerical evidence, not a certified global error bound.

**Tested — acceptance incomplete:** decay's actual native reference consumer passed. The final
selected PFR and integrated/simultaneous controller run reports 1,208 checks, one failed fixture,
two inconclusive fixtures and `complete=False`. Integrated controller reports a missing-layout
invariant; simultaneous controller reaches a compiler resource limit; PFR retains closure-validity
refusals despite supported interior temperature starts, plus 24 suspicious derivative entries
among 1,877 comparisons (zero missing sparsity entries). CSTR initial density now accesses the
same frozen source record through the declared interface; neither PID equations nor goldens changed.
The scientific runtime paths behind these remaining failures were not expanded into a
repository-health repair effort. A04 remains in progress until its selected consumer acceptance is settled;
provenance repair and independent reference reproduction are already established separately.

The maintainer-authorized permanent three functional store slots retain full memory charging,
unknown-owner protection, the original admission timeout and one timing slot. Four focused controls
pass after committed-main integration. The isolated canonical service was used without stopping
or reconfiguring the existing main/Plan52 services. Final receipts and source correspondence are
retained with the coordinator's artifacts; historical interrupted/refused preparations establish
no scientific result. No full parity, benchmark or repository-wide health campaign was run.
