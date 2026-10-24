# Oracles, acceptance and assertions

Findings about where expected results and acceptance allowances come from, and whether
assertions distinguish the behavior they name. Baseline and limits are in the
[collection README](README.md). Citations are `path:line` at source commit `cbda2efc6`.
Companion references are to the [test-assessment companion](../design_principles/core/test-assessment.md).

## <a id="dta-001"></a>DTA-001 — Acceptance allowances come from the production numerical policy and from budgets the system under test publishes, so oracle comparisons widen with what they check

**Principles:** AP-02, AP-04, DP-23, PS-13; companion §1, §3, §7.

**Claim at risk.** Results meet an accuracy justified independently of the system being checked.
In these checks the expected values are often genuinely independent (analytic answers, IDAES,
teqp, FeOS, parmest, sIPOPT). The acceptance allowance is not.

**Diagnosis.** Two sources supply most allowances.

*Shared typed constants.*
- `packages/reference/domain/models/numerical-policy.pse:14-29` declares typed constants, such as
  `pressure = 100 Pa`, `mole_fraction = 0.001` and `dimensionless = 0.001`.
- Lines 34-49 annotate the same constants as `engineering_rule` floors. The comment at lines
  8-9 says they "own the shared physical output-check resolutions and production engineering
  floors".
- Most fixture `expect` tolerances in `packages/reference/seed-data/models/*.pse` use them.
  About 305 of 481 are written as `tolerance numerical_policy.*`.

*Budgets published by the runtime.*
- Parity tests take their allowance from the `runtime.resolved_numerics` budget the product
  publishes. They check only that the budget is positive or self-consistent
  (`python/pse/parity/tests/test_sensitivity_parity.py:153-200`;
  `test_covariance_parity.py:182-185`).
- The PID parity allowance is `NumericalPolicy().engineering_relative_fraction × 1.0`
  (`python/pse/parity/tests/support.py:24-37`). That fraction is the generated projection of
  `DEFAULT_ENGINEERING_ACCURACY = 1e-3` (`crates/pse-model/src/numerics.rs:33`), and the
  fixture requests the same value as its integrator tolerance
  (`test_pid_parity.py:50-53`).
- Python component tests compare analytic values within the `tolerance` column of the result
  under test (`python/pse/tests/test_canonical_results.py:251-252`, `test_studies.py:121`,
  `test_native_workflow.py:1178-1182`, `test_plan14_acceptance.py:271-447`).
- The Rust helper `resolved_allowance` (`tests/support/plan14.rs:308-320`) asserts only that
  the resolved allowance is finite and positive. Global-certification and
  dynamic-optimization acceptance use it seven times
  (`tests/conformance/src/acceptance/global_certification.rs:162,227,470`;
  `dynamic_optimization.rs:128,142,341,428`).

**Consequence (conceals defects).**
- Changing a policy constant changes solver stopping and the oracle comparisons that would
  detect lost accuracy in the same step.
- A wiring defect that inflates one route's resolved nominal or budget widens acceptance in
  exactly the integration and parity tests positioned to expose it.
- Quantities with tighter attainable agreement are accepted at engineering resolution. One
  example: sIPOPT sensitivities of an exact QP are accepted at about 1e-3, where agreement
  near round-off is attainable.

**Counterevidence and bounds.**
- The resolver's unit tests pin budget magnitudes independently
  (`crates/pse-math/src/numerics.rs:1135-1519`).
- `engineering_target` (`crates/pse-runtime/src/workflow/tests.rs:87-104`) asserts that budgets
  are finite and positive. Several consumers add non-vacuity guards
  (`crates/pse-runtime/src/workflow/fitting/p09_tests.rs:177`).
- `resolved_physical_allowance` (`tests/support/plan14.rs:323-434`) checks the rule identity,
  the absence of a canonical fallback, and unit conversion.
- KKT unit tests compare analytic sensitivities at 1e-6
  (`crates/pse-backend-native/src/kkt/sensitivity/tests.rs:217-220`).
- Allowances are typed per quantity, not one blanket tolerance.
- The policy values are maintainer-selected (2026-10-05). This finding concerns the evidential
  coupling of acceptance to production policy and published budgets, not the chosen values.

**Uncertainty.** The residual risk is in route-level wiring and in policy evolution, not in the
resolver's arithmetic, which is pinned. A probe that forces a ×100 nominal fallback and reruns
the sensitivity, covariance and PID parity tests would show how much acceptance moves.

**Related.** [DTA-002](#dta-002) concerns which allowance class a comparison uses.

## <a id="dta-002"></a>DTA-002 — Oracle comparisons use the engineering or historical allowance class even where the reference supports, and the claim needs, a much tighter check

**Principles:** AP-04, DP-23, PS-07, PS-12, PS-13; companion §3, §4, §7.

**Claim at risk.** Two claims:
- Fixed-input property evaluations agree with high-precision references.
- Each phase-equilibrium formulation reproduces the reference phase regime.

**Diagnosis.** Tolerances are assigned by policy class rather than by what the comparison must
discriminate.

*Point evaluations at engineering resolution.*
- Commit `5260a3e9a` (2026-10-07) replaced oracle-specific relative tolerances, mostly
  `relative 2e-8`, with `relative numerical_policy.dimensionless` (0.001).
- One example is `packages/reference/seed-data/models/pcsaft-fixtures.pse:21`. It compares
  `ln_phi` against teqp 0.23.1 and was `relative 2e-8`.
- About 70 such expectations are in `pcsaft-fixtures.pse`. IDAES-printed PR values in
  `pr-oracle.pse:26-37` moved from `relative 1e-5` to 1e-3.
- These are `procedure check` point evaluations with no solver stopping involved
  (`crates/pse-runtime/src/workflow/modeling/conformance.rs:1386`).
- The reference notes say exact retrieval and identities keep their own precision, and
  historical observations and solved outputs take the shared allowances
  (`packages/reference/seed-data/sources.md:17-26`). Point checks against computed references
  fall into neither group.

*Phase fraction at ±10%.*
- `packages/reference/seed-data/models/bt-ideal.pse:145,150,160,165,175,180` accept
  complementarity `beta` against the IDAES BTIdeal reference with
  `relative numerical_policy.historical_reference_relative`, which is 0.1
  (`numerical-policy.pse:32`).
- For the vapor case this accepts β ≥ 0.9, a two-phase answer. For the two-phase case it
  accepts 0.356–0.436.
- The nested flash on the same feed still checks β = 0.3961 to 0.001 (`bt-ideal.pse:121`). The
  oracle therefore supports the tighter check.
- The companion expectation in each fixture is a material-balance identity, which holds for
  any consistent β.
- The Rust cross-realization test compares only equilibrium temperature. It leaves β to these
  source expectations (`tests/conformance/src/acceptance/complementarity_flash.rs:168-240`).

**Consequence (conceals defects).**
- A mistyped late digit in a PC-SAFT constant, or a small convention error, can shift
  `ln_phi`, `h_residual` or `cp_residual` by up to about 0.1% and pass. teqp and FeOS are the
  only independent PC-SAFT value-and-derivative references found.
- A complementarity sign or branch error that lands in the wrong regime near the dew point
  passes unless the temperature also moves by more than 0.1 K.

**Counterevidence.**
- Both policies are explicit maintainer selections:
  - engineering resolution, 2026-10-05 (`numerical-policy.pse:5`);
  - historical-reference comparison, 2026-10-08 (`numerical-policy.pse:6`;
    `docs/dev/validation-assessment.md`).
- Analytic identities kept their precision: `pr-oracle.pse:55-57` at 1e-12, and caloric
  identities at 3e-8.
- Smoothing (`smooth_min` at 1e-4) can legitimately shift β somewhat.
- The diagnosis does not challenge either policy. It concerns their application to comparisons
  whose references support, and whose claims need, finer discrimination.

**Uncertainty.** How much of each allowance current results actually use is not established.
Expectation tolerances add their parts: allowance = absolute + relative·|expected|. That rule
is asserted independently at `crates/pse-compiler/src/workspace/modeling/tests.rs:4056-4093`,
where 0.25 + 0.001·1000 gives 1.25.

Two probes would settle the remaining question: record β for the nine complementarity
fixtures, and perturb one PC-SAFT coefficient by about 1e-5 relative to confirm the fixtures
still pass.

## <a id="dta-003"></a>DTA-003 — Three computed references used as expectations have no committed generator

**Principles:** DP-21, DP-22, PS-13; companion §4, §7.

**Diagnosis.**
- `packages/reference/data/references/models/references.pse:52-54` declares three computation
  entities with tools but no `locator`:
  - `pfr_reference` ("DOP853 and Radau at rtol = atol = 1e-12");
  - `cstr_controller_reference`;
  - `decay_fit_reference`.
- In the same file, `plan14_thermo_reference` and `pr_stability_reference` (lines 50-51) carry
  locators to committed data and generators.
- The frozen values are used directly. Examples:
  - `outlet_temperature == 303.599299658173 K`
    (`packages/reference/seed-data/models/unit-fixtures.pse:200-202`);
  - `a == 2.012342`, `k == 0.506099` (`regression-fixtures.pse:16-19`);
  - the `cstr-dynamics.pse` campaign samples.
- No tracked file contains the integration or fitting script. Only `references.pse` and
  `packages/reference/campaign/sources.md` mention DOP853 or `solve_ivp`.

**Consequence (obstructs diagnosis).** When a fixture fails or the reduced model changes, nobody
can regenerate, audit or extend the expected values. This is the unreproducible-evidence
pattern the repository records as a past incident for rustdoc extraction.

**Counterevidence.** `packages/reference/campaign/sources.md:25-48` describes the method and
tolerances. The other computed references have isolated generators that import no product
code (`scripts/plan14_reference.py`, `scripts/pr_stability_reference.py`, and the IDAES and
FeOS reference scripts).

**Uncertainty.** A generator might exist outside the repository. The search covered tracked
files and the deleted-file history of `scripts/`.

## <a id="dta-004"></a>DTA-004 — Solver status-mapping tests do not state the intended meaning of limit and ambiguity codes, and the HiGHS test became vacuous

**Principles:** PS-10, AP-02; companion §4.

**Claim at risk.** Native termination codes map to truthful typed outcomes. Time, iteration,
solution and memory limits must never read as solved, and ambiguity must remain visible.

**Diagnosis.**
- `full_status_mapping_preserves_limits_and_ambiguity`
  (`crates/pse-backend-native/src/highs.rs:1799-1806`) asserts three things:
  - no code 0..=18 produces a name starting `Unknown(`;
  - code 9 maps to `InfeasibleOrUnbounded`;
  - code 14 has `Assurance::None`.
- The fallback arm now returns `"kHighsModelStatusUndeclared"` (`highs.rs:1783`; changed in
  `59249b060`), so the first assertion cannot fail.
- `Assurance::None` is hard-coded for every code (`highs.rs:1786`), so the third cannot fail
  either.
- The limit categories the test is named for (codes 13, 14, 16, 18) are not checked.

**Consequence (conceals defects).** A dropped or renumbered arm falls through to `Inconclusive`
and the test still passes. The truthful-outcome claim is protected only by downstream
behavior.

**Footprint.**
- *Established:* HiGHS.
- *Suspected:* no unit test stating independent expected categories was found for the Clarabel
  mapping (`crates/pse-backend-native/src/conic.rs:916-936`) or the POUNCE mapping
  (`pounce.rs:1108`).
  - Clarabel's mapping is an exhaustive match over the library's closed status enum, so a
    dropped arm cannot compile. Whether each status maps to the right category is still
    unasserted.
  - Real-solver outcome tests may cover some codes indirectly; that was not traced.

**Counterevidence.** Uno, SCIP, KINSOL and Ipopt do state independent expectations:
- `uno.rs:708+` lists named categories;
- `scip_tests.rs:651+` checks exhaustiveness and pessimism;
- `kinsol.rs:2187` and `ipopt.rs:212-221` match their actual fallbacks.

**Probe.** Delete the `kHighsModelStatusTimeLimit` arm locally and run the test. Source
reasoning predicts it still passes.

## <a id="dta-005"></a>DTA-005 — The Peng–Robinson implicit-derivative test checks jets against production's own primitive derivatives, with one loose independent directional response

**Principles:** PS-07, DP-23; companion §1, §7.

**Diagnosis.** In `crates/pse-runtime/src/workflow/modeling/pr_jacobian_tests.rs`:
- `backward_error` is "the same componentwise linear backward error used by the production
  implicit solver" (`:485-497`).
- `root_conditions` (`:652-799`) checks the first- and second-order implicit-function
  equations using production's primitive Jacobian and Hessians. It mirrors production's
  `a <= b` assembly (`:678-680`).
- The only check that does not pass through those primitives is one finite step: 10 × the
  input budget along one composition axis, for root 0 (`:1610-1700`). It is accepted within
  the sum of production variable tolerances.
- The ratio of response to allowance is not asserted, except that the fraction output must
  exceed its allowance.

**Consequence (conceals defects).**
- An error in the primitive Jacobian or Hessian propagates into both the jets and the checks.
- Second-order jets receive no independent challenge here.

**Counterevidence.** Three other checks reach the PR model:
- `crates/pse-math/src/implicit_property_tests.rs:174-201` checks PR implicit second
  derivatives against a hand-derived implicit-function oracle at 2e-11.
- FeOS and teqp oracles check `ln_phi`, pressure and residual caloric properties, at the
  allowance discussed in [DTA-002](#dta-002).
- Modeling conformance runs a shared central-difference derivative diagnostic with
  `Inconclusive` handling, a known truncation-bias control
  (`crates/pse-backend-native/src/derivative_diagnostics.rs:85-92,192-251,385`) and
  wrong-partial and missing-support controls (`derivative_dynamics.rs:323-329`).

**Uncertainty.** Moderate. The residual gap is narrower than the test file alone suggests. The
shared diagnostic is first-order only and samples at the start point. It compares entries
below 1 in magnitude absolutely, at 1e-4, in normalized coordinates
(`derivative_diagnostics.rs:334-345`). Second order is covered generally by
[DTA-020](#dta-020).

## <a id="dta-006"></a>DTA-006 — Rejection tests often assert only `is_err()`, so with opaque inputs they can pass for an unrelated reason

**Principles:** AP-05, PS-01; companion §4.

**Diagnosis.**
- `crates/pse-admission/tests/quantity_relations.rs:237-241` rejects four quotients of an
  origin-sensitive point quantity by a fraction. The quantity IDs are opaque hex values and
  the assertions are `.is_err()` only.
- An unknown or re-declared ID, or missing context, would satisfy them equally.
- All four IDs currently resolve in `packages/reference/physical/materials/physical.yaml`, and
  one has a positive control in the same test.

**Footprint.**
- *Established:* one manifestation.
- *Suspected:* about 1,476 single-line `assert!(… .is_err())` sites outside generated code, set
  against 619 `unwrap_err`/`expect_err` and 157 `matches!(…Err(…))` sites.
- A 15-site sample found most cases paired with single-mutation positive controls. The risk
  therefore concentrates where inputs are opaque and no adjacent admitted control exists.

**Counterevidence.** The strongest negative-case evidence names its reason and has an admitted
control:
- the domain-refusal corpus (`crates/pse-modeling/src/domain_schema.rs:229-270`);
- relation admission (`crates/pse-relations/tests/admission.rs:143-232,302-379`).

Where a refusal has a typed reason (domain, realization and objective refusals), tests assert
it. The dominant modeling refusal class has no typed reason, which is
[DTA-028](doubles-and-coupling.md#dta-028).

## <a id="dta-007"></a>DTA-007 — Floating comparisons are re-authored per file, several accept non-finite expectations or mismatched lengths, and the declared finite comparison contract is unused

**Principles:** AP-05, DP-23; companion §3.

**Diagnosis.**
- `pse_quantity::numeric::approx_eq` (`crates/pse-quantity/src/numeric.rs:30-45`) is documented
  as "the finite comparison contract of blueprint §7.3". It refuses non-finite inputs and
  invalid tolerances, and it has no callers in tests or production.
- Tests instead use about 1,000 inline `abs() <=` checks and local helpers.
- *Infinite expectations.* The relative form `(a-b).abs() <= r*(1+|e|)` accepts any finite
  actual against an infinite expectation. Examples:
  - `close` in `crates/pse-backend-native/src/kkt/sensitivity/tests.rs:217-221`;
  - `crates/pse-math/src/factorable_tests.rs:261-263`, which accepts if either side is
    infinite;
  - `crates/pse-backend-native/src/dynamics/process_tests.rs:148-168`;
  - `crates/pse-runtime/src/workflow/uncertainty_tests.rs:42-50`.
- *Mismatched lengths.*
  - `process_tests.rs` `close` zips chained state, output and sensitivity vectors after
    asserting only sample counts, so a length difference in one field silently misaligns the
    rest.
  - Cross-backend comparisons zip primal and dual vectors without comparing lengths
    (`crates/pse-backend-native/src/pounce_convex_tests.rs:102-113`,
    `clarabel_tests.rs:216-235`).

**Consequence.** These gaps can erase a difference: an infinite reference, or a missing
component. Severity is low. Fixture domains are mostly finite, and production result validation
probably enforces vector dimensions, though that was not verified.

**Counterevidence.**
- All helpers use `<=`, so NaN is rejected. No negated comparison was found.
- Some helpers check finiteness explicitly:
  - `assert_action` (`p09_tests.rs:18-24`);
  - `near` (`tests/support/plan14.rs:154-159`);
  - `close_scaled` (`kkt/sensitivity/tests.rs:636`).
- No blanket tolerance is shared across unrelated quantities.
- `approx` 0.5.1 is already in `Cargo.lock` (library lead). Its relative comparison returns
  false when exactly one side is infinite, and its slice comparisons require equal lengths.
  Those are the two gaps above.

## <a id="dta-020"></a>DTA-020 — Sensitivities, covariances and their validity verdicts are tested only on problems with constant curvature and linear constraints

**Principles:** PS-07, PS-12, PS-13, DP-11, DP-23; companion §1, §4, §7.

**Claim at risk.** Several published second-order results are correct for nonlinear models:
- parametric sensitivities and the inverse reduced Hessian;
- exact-Hessian and propagated covariances;
- the `second_order` and `certified` validity verdicts.

**Diagnosis.** Every KKT-level and end-to-end test of these quantities uses a constant
Lagrangian Hessian with zero constraint curvature (Σλᵢ∇²gᵢ = 0):
- `crates/pse-backend-native/src/kkt/sensitivity/tests.rs:21-28`: a quadratic objective with
  linear `g` and `h`.
- `kkt/tests.rs:8`: "f = ½xᵀQx with linear rows g = Ax".
- `crates/pse-runtime/src/workflow/modeling/sensitivity_tests.rs:30-48`: quadratic and linear
  models, reused by `uncertainty_tests.rs`.
- `fitting/covariance_tests.rs:22-30`: "On a linear model…".
- `fitting/p09_tests.rs:1194-1200`: "no residual curvature".
- The independent references were drawn from the same class: sIPOPT on a QP
  (`python/pse/parity/tests/test_sensitivity_parity.py:1-14`) and parmest on a model linear in
  its parameters (`test_covariance_parity.py:1-13`).
- No authored reference fixture requests sensitivities or covariances.
- The shared conformance derivative check is first-order only
  (`crates/pse-runtime/src/workflow/modeling/conformance.rs:2150-2156`).

**Consequence (conceals defects).** Each of these defects vanishes when curvature is constant
and constraints are linear, so all of them would pass:
- a wrong sign or weight on the multiplier-weighted constraint Hessian (solvers differ in their
  multiplier sign conventions);
- a Hessian evaluated at a stale or normalized point;
- missing row scaling of constraint Hessians;
- mishandling of the objective factor.

Sensitivities, exact covariances and confidence intervals for nonlinear fits would then be
published wrong. The second-order-sufficiency inertia check could flip, so a result could be
certified or withheld wrongly. NLP solutions themselves remain protected by first-order KKT
validation.

**Counterevidence.**
- Second derivatives do have independent expectations elsewhere:
  - primitive, authored-expression and PR implicit Hessians against hand-derived values
    (`crates/pse-math/src/derivative_tests.rs:978-1012`; `implicit_property_tests.rs:174-201`);
  - the IDAS second-order adjoint against finite differences on a nonlinear reactor
    (`crates/pse-backend-native/src/dynamics/adjoint_tests.rs:836-860`);
  - arclength curvature on nonlinear residuals.
- Withholding decisions are well controlled ([Strengths](README.md#strengths-to-preserve)).

**Uncertainty.** Whether the derived path shares Hessian assembly with the Ipopt callback, which
would give indirect evidence through convergence, was not traced.

**Probe.** Solve `min x²+y² s.t. x·y = p`, or an exponential-decay fit with an exact Hessian.
Compare dx/dp, the reduced Hessian and the covariance against hand-derived values or against
central differences of re-solves.

**Related.** [DA-010](../audit/structure-and-invariants.md#da-010) is a separate production
cause and was not reexamined.

## <a id="dta-021"></a>DTA-021 — Reference coverage counts a definition once any fixture instantiates it, so parameter-selected variants that nothing discriminates count as covered

**Principles:** PS-13, PS-06, DP-23; companion §4, §7.

**Claim at risk.** Each unit model is compared with a reference. The conformance coverage row
reads "definition instantiated by an authored fixture".

**Diagnosis.** `coverage` (`crates/pse-runtime/src/workflow/modeling/conformance.rs:356-400`)
counts instantiated definitions, not the `when` arms that parameters select. Some fixtures also
use conditions under which arms cannot be told apart.

*Heat exchanger: the default arm has no solved fixture.*
- `HeatExchanger` defaults to `cocurrent:Boolean=false`
  (`packages/reference/process/models/heat-exchanger.pse:7`), which selects the countercurrent
  arm (`:24-27`).
- Every solved fixture sets `cocurrent=true`
  (`packages/reference/seed-data/models/unit-fixtures.pse:130-133`; `exchanger-costing.pse`).

*Mixer: the default arm cannot be distinguished.*
- The default `minimize` pressure arm of `Mixer` is checked only by `mixer_saponification`.
- There both inlets are at 101,325 Pa and the outlet is accepted within ±100 Pa, so a min, max,
  mean or either-inlet relation all pass.
- The `equality` arm is never instantiated.

*PID: derivative action has no generated reference.*
- PID derivative action is on by default.
- The parity and control fixtures set Td = 0 (`python/pse/parity/tests/test_pid_parity.py:43-44`).
- Derivative action is compared only against `cstr_controller_reference`, which has no
  generator ([DTA-003](#dta-003)).

**Consequence (conceals defects).** Swapping the countercurrent temperature differences, or
substituting `smooth_max` for `smooth_min`, passes every reference check. Coverage still
reports the definition as covered. For the heat exchanger, the default arrangement is the
unchecked one.

**Suspected wider footprint.**
- Other parameter-selected arms, which were spot-checked only. Separator split bases, pressure
  changer variants, ControlVolume0D bases and both PFR schemes are covered.
- The logarithmic mean has `valid(a>0 and b>0 and a!=b)`, which refuses a balanced
  countercurrent exchanger whose end differences are equal. The only LMTD fixture uses 20 K and
  10 K.

**Counterevidence.** Per-definition coverage has a failing control. Temperature-difference forms
are checked analytically at 1e-10 K. `smooth_min` is exercised by the complementarity fixtures.

**Uncertainty.** The arms read as consistent today; the gap is missing evidence, not an
established defect.
