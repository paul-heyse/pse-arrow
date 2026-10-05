---
title: Decision-relevant convergence and accuracy criteria
date: 2026-10-05
tier: change
purpose: target
standard: core-3.3
profile: process-simulator-1.4
baseline: ad665a0222551196b1160e426f5242361215a6a0 plus current uncommitted tree
evidence: Implemented
decision: Not Accept
---

# Decision-relevant convergence and accuracy criteria

The inspected criteria are not yet justified as an ordinary engineering-design accuracy
policy. The shared physical fallback demands `1e-8` of a nominal scale, independent native
optimality defaults also demand `1e-8`, and several authored campaign checks are much
tighter than the differences relevant to their reported outputs. The steady CSTR rejection
is a concrete example of excessive reference precision. This does not establish that all
remaining campaign failures have the same cause.

**Proposed:** start ordinary design accuracy at `1e-3` of a meaningful characteristic scale
(0.1%), then choose physical output allowances according to the analysis decision. Tighter
requirements need a named quantitative reason. The existing numerical requirement,
resolution and original-space checking contracts can express this policy; a new solver or
parallel acceptance mechanism is unnecessary.

## Scope, scenario and evidence

This bounded change-tier, target-purpose review applies Core/template 3.3 and Process
Simulator 1.4. Scenario **S01** is changing the accuracy of an ordinary design analysis
without changing its physical laws, supported domains, root-selection meaning or result
truthfulness. Adjacent consumers are native stopping, implicit/derived reconstruction,
dynamics and campaign reference checks. Blueprint §16 owns numerical resolution; PS-03,
PS-07, PS-08, PS-10 and PS-13 supply the relevant scientific obligations.

Source/interface findings below are **Implemented** or **Interface-checked**. Remedies are
**Proposed**. No new solver run, probe, build or benchmark was performed for this review.
The existing Plan 25k campaign observations are historical evidence, not qualification of
a revised policy. Independent read-only reviewer advice agrees with the scale distinction,
central ownership and separation of engineering accuracy from verification accuracy.
Whole-system qualification, unrelated solver defects and a comprehensive audit of every
scientific fixture are excluded. [Plan 25k](../../plans/25k-integrated-qualification-and-closure.md)
owns adoption, disposition and subsequent execution.

## Findings and causes

<a id="f01"></a>**F01 — Shared defaults have no decision-relevant accuracy rationale.**
`pse-math/src/numerics.rs::resolve` supplies `absolute = 1e-8 * nominal` and `relative = 0`
when no stronger requirement exists. The nominal can come from a quantity declaration or
canonical-unit fallback. `pse-model/src/numerics.rs` separately defaults KKT stationarity,
complementarity and continuous gaps to `1e-8`; MIP gaps and integrality have distinct fields.
`pse-backend-native/src/solve.rs::ResolvedAccuracy` lowers the resolved physical and KKT
budgets into native controls. These are explicit, coherent computational contracts, but
the inspected defaults do not explain why such precision could change a design choice.
Changing physical fallback alone leaves stricter independent optimality work active.

Correct the default policy in those existing owners, with meaningful nominals and explicit
overrides for consumers requiring more accuracy. Optimality should be justified against
objective improvement and constraint decisions, rather than assuming that every normalized
KKT number is an output-error bound. Integrality remains an integer-feasibility obligation;
it must not be relaxed mechanically with continuous engineering accuracy. Verify resolution
and native lowering locally before exercising affected scientific journeys. This finding
concerns DP-11 and PS-07/PS-10, not a demonstrated flaw in a native library algorithm.

<a id="f02"></a>**F02 — Authored design checks can require precision unrelated to their purpose.**
Authored reconstruction and conservation tolerances become explicit numerical requirements
through `workflow/modeling/cases.rs`; they outrank fallback. Reference expectations and
integration settings are separately authored. The following examples therefore need review
alongside the central defaults:

| Owner | Inspected criterion | Meaning and consequence |
|---|---|---|
| `campaign/models/cstr-dynamics.pse` | Steady temperature `1e-6 K`; original concentration comparison `1e-6 mol/m³` | Original concentration threshold is `1e-9 mol/L`. The recorded deviation `1.9592909978882744e-6 mol/m³` is about `9.6e-8` of the approximately `20.316 mol/m³` outlet concentration. Rejecting that difference is disproportionate for the inspected design purpose. |
| Same owner, dynamic route | Integration relative `1e-10`, normalized absolute `1e-12`; reported controlled trajectory checks `0.002 K` and `0.01 mol/m³` | Much tighter integration settings than output comparisons, without an inspected error-allocation rationale explaining the disparity. Local integration error and final trajectory error are different quantities. |
| `process/models/control-volumes.pse`, `thermodynamics/models/aqueous.pse` | Component flows `1e-7 mol/s`; solvent reconstruction `1e-7 mol/m³` | Explicit physical allowances survive a shared-default change. Balance/reconstruction budgets must be related to throughput and their effect on reported quantities. |
| `campaign/models/recycle-flash.pse` | Component transport `1e-7 mol/s`; composition references `1e-5`; flow references absolute `1e-6 mol/s` plus relative `5e-4` | The flow comparison already includes an engineering-scale relative allowance. The tiny absolute literal alone does not describe its effective acceptance. Overall `accounting` is not a required closure check. |
| `campaign/models/pfr-studies.pse` | Pressure and temperature references absolute `1e-12` plus relative `1e-5` | Effective allowances are about `1 Pa` and `0.003 K`, not `1e-12` in those units. Review the combined allowance and purpose, not the smallest literal. |
| `campaign/models/exchanger-costing.pse` | Area `0.003 m²`; capital cost `85 USD_CE500` | The authored commentary relates temperature/area uncertainty to about 0.13% of capital cost. This is a useful example of an explicit output-impact rationale. Its separate `1e-8 USD_CE500` aggregation identity tests arithmetic consistency. |

The provisional CSTR edit to `5e-6 mol/m³` remains similarly stringent and unqualified. It
is not the proposed engineering policy. Correct the affected authored design requirements
with a physical scale, decision purpose and effective allowance; do not widen a failed
expectation merely enough to pass the observed error. Reference digits alone do not justify
acceptance precision. Verification should preserve an independent oracle and show both an
acceptable engineering difference and rejection of a materially wrong result (PS-03/PS-13).

<a id="f03"></a>**F03 — Internal accuracy and assurance must retain separate meanings.**
`workflow/modeling/implicit.rs::TrialHints::resolve` consumes resolved unknown/residual
budgets, while derivative linear-system accuracy consumes KKT stationarity.
`math/solves.rs::automatic_reconstruction_accuracy` derives point allowance from normalized
variable budgets, action allowance from stationarity, and requests `Certified` evidence.
Neither an outer residual nor a successful native status establishes an output-error bound
for an ill-conditioned model. Inner error can be amplified by sensitivity, recycles or
integration; a tighter internal budget can be justified by the requested outer allowance.

Finite-difference diagnostics divide evaluation differences by a small perturbation. A
`1e-6` perturbation combined with evaluations accurate only to `1e-3` can invalidate a
derivative comparison. Keep a separately justified verification budget or suitable diagnostic
step/error allocation. Step size, continuation minimum step, smoothing parameters, exact
identities and interval proof controls are not ordinary output tolerances. They must not be
changed by a literal numerical floor. Quantify the allocation where material and preserve
the distinction between estimated and certified evidence (DP-11, PS-07, PS-10).

## A rationalizable engineering policy

**Proposed:** reuse the existing frozen physical budget
`epsilon = absolute + relative * nominal`. Choose meaningful characteristic scales before
solving; do not invent them from a trial value. The `1e-3` normalized design default is a
starting allowance, not a guarantee of decision invariance or a floor on every number.
For each affected output or analysis, record:

1. The decision or reported quantity the accuracy protects, and the smallest difference
   considered material under the model and data uncertainty.
2. The physical units, characteristic scale and effective absolute/relative allowance.
3. Any smaller internal allocation and how propagation/conditioning makes it necessary.
4. Any tighter exception: trace concentrations, proximity to a specification or phase
   boundary, sensitivity/optimization accuracy, or a correctness/verification obligation.

For example, `0.01 K` can be a declared design temperature allowance. Applying `1e-3` to
a `300 K` nominal instead gives `0.3 K`, so it is not equivalent. Likewise `0.001 mol/L`
equals `1 mol/m³`, about 4.9% of the CSTR outlet concentration: it is appropriate only if
that concentration difference cannot affect this analysis's decision. At a decision boundary
where the requested accuracy cannot resolve the choice, report the ambiguity or request a
tighter explicit analysis; never silently choose a side.

Keep physical original-space checks authoritative, including balances and bounds. Give
balance tolerances a throughput/output-impact rationale and account for cumulative drift
in dynamics. Align native stopping with the requested feasibility and objective precision;
do not bypass closure to accept a native success code. Library-owned iteration remains the
execution mechanism. A blanket replacement of every small constant loses these distinctions;
adjusting only reference checks leaves avoidable native work; adapter-specific defaults
would introduce competing policy. Existing requirements plus revised defaults are the
simplest fitting alternative.

## Assessment and implications for the campaign

For S01, AP-01/AP-02 are **satisfied** by separation of semantic policy, physical resolution
and native lowering; AP-03/AP-04 are **satisfied** by composition through existing explicit
requirements, units, nominals and sourced precedence. AP-05/AP-06 are **satisfied** within
the inspected boundary: budgets and provenance are explicit and resolution can be exercised
without whole-simulator qualification. The model can represent the proposed policy; the
defect is the rationale and chosen values, not a need for another accuracy type universe.
G9 therefore **passes** for this bounded policy-change scenario at source evidence strength.

G1/G2 **pass** within inspected resolution: independent physical, KKT, integration and proof
meanings are distinguished. G6/G7 and PS-G3 are **unresolved** for a revised engineering
policy until its allowances and executed acceptance are established. PS-G1 is **unresolved**
for the affected revised physical criteria; this review does not certify balances under new
allowances. G3–G5, G8 and PS-G2 are not reassessed: unrelated validity, persistence/recovery,
library implementation and structural well-posedness are outside S01. No whole-product
acceptance follows from the architectural finding.

The CSTR precision mismatch is a well-supported hypothesis. Recycle's positive failure needs
its actual original-check deviations before attributing it to tight tolerance; the negative
restore case has a separate `Unsupported` versus `TrialRejected` classification problem.
Nested PR deadline observations place active work in rigorous IBEX competitive root exclusion.
Larger engineering allowances are not established to remove that global-proof bottleneck.
Changing certified selection into estimated selection would be a separate assurance decision,
not convergence tuning. The earlier C++ contractor-order experiment has no passing or measured
evidence and is not endorsed by this accuracy review.

**Decision: Not Accept the current settings as a decision-justified design policy.** Adopt
the direction above through Plan 25k, settle the affected physical allowances and exceptions,
then implement centrally and verify the changed journeys. No further probes are needed to
establish the default/fixture precision mismatch. Revised-policy qualification and the
broader campaign closeout remain pending.
