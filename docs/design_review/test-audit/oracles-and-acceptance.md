# Oracles, acceptance and assertions

Current source baseline `05e6a2b8a`, 2026-10-23. The holistic follow-up
revalidates each primary finding below; diagnoses are *Interface-checked*, not new test results.
See the [integrated architectural account](architecture-and-assurance.md),
[baseline and revalidation outcomes](README.md#revalidation-outcomes), and the
[test-assessment companion](../design_principles/core/test-assessment.md).

## <a id="dta-001"></a>DTA-001 — Result-relative accuracy checks do not independently establish the allowance projected by that route

**Principles:** AP-02, AP-04, DP-23, PS-13; companion §1, §7.

**Follow-up disposition:** Unresolved evidence boundary, source baseline `05e6a2b8a`, 2026-10-23. Assessment is *Interface-checked*; no test was run.

`python/pse/parity/tests/test_sensitivity_parity.py` derives its normalized response allowance
from `runtime.resolved_numerics`. `test_covariance_parity.py::pse_estimate` similarly accepts
parameter estimates using published engineering budgets. `tests/support/plan14.rs::resolved_allowance`
checks finite positive budgets, and callers compare independent analytic answers within them.
These comparisons alone cannot distinguish a wrong route-level allowance projection from a
result meeting that wrong allowance. This is a boundary-specific discrimination limit, not
evidence that the numerical policy or current answers are wrong.

**Contrary evidence and corrected scope.** The policy is deliberately authoritative for physical
acceptance: `packages/reference/domain/models/numerical-policy.pse` declares shared output
resolutions and engineering floors. Reusing that declared contract is legitimate. Resolver tests
in `crates/pse-math/src/numerics.rs`, including
`engineering_accuracy_uses_frozen_scale_and_equivalent_unit_magnitudes`, independently pin
scales, precedence, unit magnitudes and budgets. `resolved_physical_allowance` also verifies
quantity/rule identity and conversion. Covariance parity compares the covariance itself against
the independent inverse-information matrix at `rel=1e-6`; it does not use parameter allowances
as covariance error budgets. Other cases, such as canonical close/reopen, primarily protect
lifecycle behavior and need not independently reverify numerical-policy correctness.

The earlier indictment of sharing production policy was too broad. Remaining inquiry concerns
whether each consequential projection/normalization boundary has complementary independent
expectations. No particular incorrect projection escaping the complementary controls was
established, so a route-level defect remains unproven. No suite-wide absence of those controls
or budget-inflation experiment is claimed.
Existing independent values and the selected engineering criterion must survive any correction.

## <a id="dta-002"></a>DTA-002 — Historical phase-reference comparisons do not distinguish every phase regime

**Principles:** AP-04, DP-23, PS-06, PS-13; companion §4, §7.

**Follow-up disposition:** Reframed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

The `complementarity_vapor_*` fixtures in `packages/reference/seed-data/models/bt-ideal.pse` compare `beta == 1`
with `relative numerical_policy.historical_reference_relative`, whose value is 0.1. Thus this
assertion admits a beta of 0.9 as well as 1. The paired material-balance expectation does not
distinguish those regimes. `tests/conformance/src/acceptance/complementarity_flash.rs::flash_phase_disappearance_agrees_across_realizations`
compares equilibrium temperatures, leaving beta to the authored comparisons. These checks
therefore do not independently establish the intended phase regime within each declared
realization/approximation.
This is a categorical discrimination gap when interpreting that journey as phase-regime evidence,
not an observed wrong phase result.

**Correction and counterevidence.** The maintainer explicitly selected ±10% for historical final
results; `docs/dev/validation-assessment.md` separates that comparison from stopping budgets,
feasibility and conservation. More precise reference values, or the possibility of tighter
agreement, do not on their own create a stronger acceptance obligation. The previous claim
that all point-evaluation comparisons must regain their old precision is withdrawn. Analytic
identities retain separate tight checks, and complementarity smoothing can legitimately alter
values. The same file's `nested_vapor` checks beta against one using
`numerical_policy.dimensionless` and checks both component balances: tighter complementary
phase evidence does exist. The beta assertion alone does not establish that 0.9 could pass
the complete validated journey. A regime-specific check would need its own justified classification contract while
preserving the selected historical-comparison policy. Its necessity is not a demand to tighten
every numerical tolerance or launch a new scientific campaign.

## <a id="dta-003"></a>DTA-003 — Three computed references lack a committed regeneration path

**Principles:** DP-21, DP-22, PS-13; companion §4, §7.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`packages/reference/data/references/models/references.pse` declares `pfr_reference`,
`cstr_controller_reference` and `decay_fit_reference` as independent computations, without
generator locators. The PFR outlet, decay-fit parameters and campaign CSTR trajectory consume
their frozen values in `unit-fixtures.pse`, `regression-fixtures.pse` and `cstr-dynamics.pse`.
Tracked-source searches for those identities, DOP853 and solve_ivp found method descriptions
and consumers, not the generating programs. This obstructs independent regeneration and
extension of those cases; it does not establish wrong frozen values.

Campaign/process source notes describe the numerical methods and tolerances. Other references
have committed generators, including `scripts/plan14_reference.py` and
`scripts/pr_stability_reference.py`. A generator may exist outside the repository; that
possibility remains unresolved. No historical search or external environment was refreshed.

## <a id="dta-004"></a>DTA-004 — The HiGHS mapping test does not detect loss of a declared status arm

**Principles:** PS-10, AP-02, DP-23; companion §4.

**Follow-up disposition:** Confirmed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`crates/pse-backend-native/src/highs.rs::termination` now names an undeclared fallback
`kHighsModelStatusUndeclared`. Its `full_status_mapping_preserves_limits_and_ambiguity` test
still rejects names starting with `Unknown(`. Dropping a declared mapping arm can therefore
fall through to `Inconclusive` without failing that supposed inventory assertion. The test
independently checks the category of code 9, but not the several time/iteration/solution/memory
limit categories. Its assertion that code 14 has `Assurance::None` remains a valid narrow
expectation; the constant implementation does not make all future assurance changes untestable.

The current production mapping has explicit correct-looking arms; no actual status error is
diagnosed. Uno, SCIP, KINSOL and Ipopt provide contrasting category controls. Clarabel's closed
enum gives compile-time arm completeness but not semantic category correctness. The previous
suspected Clarabel/POUNCE footprint remains unestablished, not another confirmed finding.

## <a id="dta-005"></a>DTA-005 — The integrated PR jet check shares primitive derivatives with the path it checks

**Principles:** PS-07, DP-23; companion §1, §7.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`crates/pse-runtime/src/workflow/modeling/pr_jacobian_tests.rs::root_conditions` checks the
implicit-function equations using the production primitive Jacobian and Hessians and the same
upper-triangle assembly convention. The material-response check independently evaluates one
finite change along one composition direction for root zero, with physical production allowances.
It checks non-vacuity for the fraction response, but does not independently identify all
second-order entries. Errors shared by this primitive/composition check can therefore escape it.

**Portfolio boundary.** `crates/pse-math/src/implicit_property_tests.rs::oracle` independently
derives PR first/second derivatives analytically and asserts shape and tight agreement. External
property references and conformance's first-order derivative diagnostic supply other challenges.
Consequently this is a limit of the integrated composition witness, not a claim that PR has no
independent Hessian evidence anywhere. Which cross-binding errors escape the complementary
controls remains case-specific and was not established by a mutation probe.

## <a id="dta-006"></a>DTA-006 — Opaque-input rejection checks can accept the wrong refusal

**Principles:** AP-05, DP-23, PS-01; companion §4.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`crates/pse-admission/tests/quantity_relations.rs` rejects four origin-sensitive quotient
inputs with bare `is_err()` assertions. The semantic IDs are opaque; an unknown quantity or
missing context would also satisfy the assertions. The IDs currently resolve in the physical
reference package, and one has an admitted control. Source therefore establishes weak
reason discrimination, not an actual wrong refusal or failure of all bare-error checks.

The domain-refusal corpus and typed relation-admission controls demonstrate stronger contrasting
patterns. Single-mutation admitted controls often sufficiently distinguish a refusal without
requiring every test to repeat a diagnostic enum. The previous broad counts are historical
search leads, not a freshly established footprint. Free-text modeling refusals have their own
cause in [DTA-028](doubles-and-coupling.md#dta-028).

## <a id="dta-007"></a>DTA-007 — Some numerical helpers lack shape and non-finite discrimination

**Principles:** AP-05, DP-23; companion §3, §4.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

Relative helpers in `kkt/sensitivity/tests.rs::close`, `factorable_tests.rs::close` and
`dynamics/process_tests.rs::close` can accept finite-versus-infinite comparisons because their
allowance becomes infinite. The dynamics comparator also zips chained state/output/sensitivity
vectors without checking each extent. These helpers alone cannot reject those malformed cases.
The finite `pse_quantity::numeric::approx_eq` contract has no external consumer in the inspected
Rust tree, but its different scalar tolerance contract does not make it a mandatory replacement.

**Bounds and counterevidence.** Existing manufactured references are finite. Some other helpers
check lengths/finiteness explicitly. The Clarabel comparison checks feasible quality, no validation
failure and original row identity before its zips; original Lagrangian validation in
`quality.rs::lagrangian_gradient` checks primal/dual dimensions and finiteness. Missing local
length assertions therefore do not establish that malformed solver results can pass that entire
journey. Retain a scoped helper robustness concern, with the exposed consumer footprint unresolved;
do not infer a thousand faulty assertions or prescribe one blanket comparator/tolerance.

## <a id="dta-020"></a>DTA-020 — Published KKT-derived results lack a nonlinear-constraint reference case in the inspected portfolio

**Principles:** PS-07, PS-12, PS-13, DP-23; companion §1, §7.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

The inspected sensitivity/reduced-Hessian tests in `pse-backend-native/src/kkt/sensitivity/tests.rs`,
runtime `workflow/modeling/sensitivity_tests.rs`, `workflow/uncertainty_tests.rs`, fitting
`covariance_tests.rs` and the sIPOPT/parmest parity fixtures use quadratic or linear models with
linear constraints. They independently challenge many values, signs, scaling and withholding
conditions, but a multiplier-weighted nonlinear constraint Hessian is zero in those cases.
The shared conformance derivative diagnostic is first-order. No authored sensitivity/covariance
reference fixture with a nonlinear active constraint was found in the inspected reference tree.

**Consequence.** An error confined to constraint-curvature contribution or evaluation at the
wrong nonlinear point can escape these derived-result comparisons. This is a portfolio-specific
discrimination gap, not an observed wrong published covariance or certification decision.
The earlier assertion that every objective-factor error would disappear on these examples was
incorrect: their objective Hessians are nonzero and have independent expected results.

Primitive/implicit Hessians and nonlinear dynamic adjoints have independent checks elsewhere.
`kkt.rs` calls the original oracle's Hessian at the candidate with its multipliers; runtime
derived adapters delegate Hessian evaluation. This offers shared-path evidence through solving,
but convergence alone does not independently establish each published nonlinear derived quantity.
No nonlinear solve or finite-difference probe was run here.

## <a id="dta-021"></a>DTA-021 — Some default model variants lack discriminating reference scenarios

**Principles:** PS-06, PS-13, DP-23; companion §4, §7.

**Follow-up disposition:** Narrowed, source baseline `05e6a2b8a`, 2026-10-23. Diagnosis is *Interface-checked*; no test was run.

`modeling/conformance.rs::coverage` accurately reports whether a definition was instantiated;
its message does not claim branch coverage. The reference portfolio nevertheless leaves concrete
semantic variants unchallenged: `HeatExchanger` defaults to countercurrent, while both solved
reference usages explicitly select cocurrent; the mixer minimum-pressure example has equal
inlet pressures, so min/max/mean agree.

Thus per-definition inventory cannot stand for evidence distinguishing all supported variants.
The gap belongs to the selected scenarios and interpretation of their assurance, not a dishonest
coverage implementation. A compiler transfer test instantiates the default exchanger but does
not establish its solved reference behavior. Analytic temperature-difference controls, other
covered separator/pressure-changer variants, and failing coverage controls are strengths.
Although PI parity disables derivative action, `packages/reference/campaign/models/cstr-dynamics.pse`
instantiates PID's default `Td=0.1s` and checks sampled trajectories. That is a distinct
derivative-action challenge; its missing committed generator concerns reproducibility (DTA-003),
not absence of discrimination.
No arbitrary branch quota, new universal coverage schema or actual wrong equation is claimed.
