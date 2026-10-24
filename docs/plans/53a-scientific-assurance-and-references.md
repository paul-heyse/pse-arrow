---
title: "Plan 53a: Scientific assurance and references"
status: in-progress
date: 2026-10-23
adrs: []
review_sources: [docs/design_review/reviews/design_review_test-architecture-target_2026-10-23.md]
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

## Integration and completion

A01/A02/A03 may proceed independently once their scientific assumptions are settled. A04's reference provenance may inform A02's PID preservation, but does not block unrelated status/composition work. B02 owns the typed origin-refusal change and admitted single-change control for DTA-006; this companion contributes the independent scientific expectation. B04/B05 can replace repeated preparation only after the same witnesses still exercise fresh numerical states.

Completion requires the targeted wrong-case controls, independent reference reproduction and assembled scientific/readback journeys selected by [53g](53g-qualification-and-publication.md). It does not imply all formulations, solvers, platforms or parity routes were qualified. No benchmark or new tolerance policy is required. Retire replaced assertions/reference locator machinery alongside migrated consumers, preserving retained policy and primitive evidence.


## Execution evidence — 2026-10-23

**Implemented:** A01 independently declares all 19 HiGHS status expectations plus unknown
fallbacks, and retains three quantity-specific comparators with finite/extent/time checks and
meaningful wrong-result controls. A02 derives a conservative phase envelope from independently
evaluated frozen Wagner inputs and directly joined resolved row/variable budgets. Perturbed
Rachford–Rice brackets at 363/373 K separate the 358/378 K feeds; the temperature-binding
budget and scalar feasibility envelope reduce the 5 K slack premise before applying
`epsilon_phase²/(4*(slack_lower-residual))`. Both wrong interior-regime controls exercise the
actual classifier. The authored unequal-pressure mixer separates minimum from maximum/mean;
the default countercurrent exchanger uses independent constant-capacity epsilon–NTU values.
A03 uses two independently analytic PR roots with keyed layout permutations and mixed terms,
and two nonlinear constrained points with publication and durable reopening. It preserves the
actual lower-triangle assembly rather than changing production to match the earlier shorthand.

**Tested:** A01's nine selected native-local status/comparator and retained projection/dynamics
controls pass with zero failures. The coordinator retains the selected command and mode.
A02/A03 compile under the force-validated native acceptance feature closure; their scientific
assertions have not yet run because the copied main service's receipt is not admitted in this
worktree. An isolated current-context service is the remaining prerequisite, not a reason to
weaken service admission or alter Plan52 state.

**Tested:** `just plan53a-reference` uses the locked CPython 3.12.3 / NumPy 2.5.3 / SciPy 1.18.1
environment and passes PFR 7/7, controlled CSTR 19/19 and decay-fit 25/25 checks, baseline zero.
`build/plan53a-reference/bin/python -m unittest scripts.tests.test_scientific_references -v`
passes four controls. Focused Ruff and Pyrefly checks of the five new Python files pass.
The entrypoints emit exact input/method/tolerance/environment/source identities and actual
numeric deltas in `build/plan53a-reference-results/`; the frozen authored values stay unchanged.
The PFR concentration difference is about `2.9e-9 mol/m³`; per maintainer guidance its standalone
reproduction allowance is `1e-7 mol/m³`, still far below the production `1 mol/m³` acceptance.
DOP853/Radau agreement remains a separate tight check. This numerical reproduction is not a
certified global error bound or product qualification. I02 is resolved without a golden migration.

**Remaining acceptance:** execute the A02/A03 native scientific/readback cases and the selected
actual PFR, integrated/simultaneous controller and decay-fit reference consumers. Keep A02–A04
in progress until these consumers pass. DTA-006's typed origin-sensitive refusal remains B02;
this companion does not close all of R01 or the entire Plan53 series. No unrelated repo-health,
benchmark or full-parity campaign is selected.
