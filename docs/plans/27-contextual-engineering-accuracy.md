---
title: "27: Contextual engineering accuracy"
status: in-progress
date: 2026-10-05
adrs: [ADR-0163]
review_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#scenarios-that-distinguish-the-alternatives"]
---

# 27: Contextual engineering accuracy

## Purpose, baseline and ownership

Connect numerical work to useful engineering accuracy. Ordinary analyses use shared,
contextual physical defaults and explicit overrides. A declared output or decision can
request additional evidence and bounded refinement where it affects that analysis.
Development stage does not itself justify different accuracy. Neither a native success code
nor a small residual establishes an output-error bound.

The maintainer confirmed the full CA-F01–CA-F04 scope of the
[contextual-accuracy review](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md).
The baseline is `ad665a0222551196b1160e426f5242361215a6a0` plus the current uncommitted
tree inspected on 2026-10-05. Preserve that tree, including its incomplete PFR and native
alternative corrections. Earlier receipts do not qualify those changes or this target.
Core 3.4 and ProcessSimulator 1.5 govern execution. The maintainer authorized full
implementation on 2026-10-05; source reviews retain their historical standard snapshots.

This coordinator owns the combined target, shared decisions and dependency sequence.
Companions own package progress and local evidence. [Plan 25k](25k-integrated-qualification-and-closure.md#contextual-accuracy-review-dispositions)
remains the sole CA finding-disposition owner and full campaign/measurement/closure owner.
Architecture sections and ADRs retain their authority. This series neither starts another
campaign nor duplicates 25k's status.

| Companion | Responsibility and product |
|---|---|
| [27a: Accuracy intent and contextual policy](27a-accuracy-intent-and-contextual-policy.md) | Admitted goals, separate engineering scales, contextual defaults and frozen provenance |
| [27b: Error allocation and numerical evidence](27b-error-allocation-and-numerical-evidence.md) | Actual output evidence, valid influence/allocation and producer demands |
| [27c: Selective refinement and completion](27c-selective-refinement-and-completion.md) | Bounded execution, immutable goal assessment, consumer migration and qualification handoff |

## Combined target and foundation assessment

The target composes existing semantic and execution owners. It introduces no new crate,
general workflow language, mathematical IR or universal estimator service.

| Foundation | Inspected baseline and selected change |
|---|---|
| Numerical policy | `pse-model::numerics` has sourced requirements and distinct KKT, gap, integrality, closure and incumbent controls. Extend its engineering context and goal meaning; retain those distinctions. |
| Resolution | `pse-math::numerics::resolve` freezes ID-bound physical budgets and provenance. Separate engineering characteristic scale from normalization nominal; keep explicit requirement interpretation. |
| Accuracy products | `AccuracyDemand`, `AccuracyEvidence`, `ProductionDemand` and `SemanticProductKey` already distinguish product, normalization, point, branch and strength. Add engineering consumers and validity-bearing influence; do not replace these contracts. |
| Reconstruction | `SelectedImplicitReconstruction` already supplies certified intervals, inverse amplification and bounded refinement; composite suppliers propagate actual predecessor errors. Derive requests from consuming outputs instead of stationarity or the tightest unrelated coordinate. |
| Response | `square_response::SparseFactor::action` solves arbitrary physical right-hand sides with a fresh backward-error observation. Reuse it for local output estimates; its backward error is not a forward certificate. |
| Dynamics | State, residual, quadrature and original conservation checks exist. Add output/accumulation evidence without claiming that local error weights bound a final trajectory. |
| Strategy and completion | Finite strategy grants, task scopes, charging ownership and immutable candidate-use composition exist. Execution refines; assessment classifies; completion and publication consume the final product. |
| Authored/generated boundaries | Registry declarations own rows and generated Rust/Python contracts. Extend their owners once, version changed semantics, and migrate consumers together. |

### Shared decisions

**D27-1 — Separate four meanings.** Coordinate conditioning, physical feasibility/closure,
engineering output resolution and decision stability remain separate. Bounds on original
constraint violations do not become bounds on solution error. Numerical error is also distinct
from input uncertainty, model discrepancy and statistical confidence.

**D27-2 — Contextual defaults.** The ordinary engineering allowance is
`B = max(F, r × S)`: applicable shared physical allowance `F`, shared relative fraction `r`,
and admitted engineering characteristic magnitude `S`. It is lowered into the existing
default absolute-budget slot, with default relative zero. Existing explicit numerical
requirements retain `absolute + relative × nominal`; this does not silently reinterpret
authored overrides. `S` is independently meaningful and never inferred from the current iterate,
normalization scale or gross cancellation terms. [27a](27a-accuracy-intent-and-contextual-policy.md#contextual-default-resolution)
owns its selection and lifecycle.

**D27-3 — Goals express what matters.** A goal declares value resolution, a quantitative
decision, or both. Decision-only goals do not inherit an additional output-digit requirement.
They may resolve at valid coarse error when far from a boundary. A specification band is an
engineering requirement; numerical resolution never widens it. Default evidence is
`Estimated`; a declared `Certified` requirement needs actual certification. Existing certified
root-selection obligations remain mandatory even for an Estimated output goal.

**D27-4 — Goal result policy.** Default `Assess` requires resolution of the requested goal,
but a resolved specification violation remains a valid analysis result. `RequireSatisfied`
additionally requires the specification to pass. Unresolved goals retain the candidate and
diagnostic evidence while refusing the requested goal-qualified result. No goals preserves
ordinary candidate use without an output-accuracy or decision-invariance claim.

**D27-5 — Selective work.** Obtain extra error evidence only for declared goal dependencies.
Refine contributors only where admitted evidence misses a requested resolution or overlaps a
decision boundary. Stop on resolved violation, unavailable capability, nonprogress, precision
limits or exhausted permission. Never loosen a goal or original acceptance to make it pass.

**D27-6 — Truthful support.** All existing numerical classes migrate to the same request,
assessment and completion contract. Evidence capabilities differ by class and product. A
missing conservative bound is explicitly unavailable; repeated stable digits do not supply it.
[27b's support matrix](27b-error-allocation-and-numerical-evidence.md#producer-support-and-validity)
is the initial supported scope, rather than a promise of universal global guarantees.

**D27-7 — Library ownership and cost.** Native solvers own iteration, globalization, local
error controllers and factors. Reuse admitted factors/derivatives, request only relevant output
actions and charge assessment work. Ordinary no-goal analyses do not start adjoints, full SVDs,
proof searches or paired solves solely for this feature.

### Alternatives and evidence

Shared static physical defaults are a cheap useful fallback, but cannot settle scale variation
or CA-F03/CA-F04. Pure relative tolerances fail at zero and near cancellation. Using a
conditioning nominal as the engineering scale obscures the actual accuracy rationale.
The selected maximum rule makes the dominating physical or relative allowance explicit;
addition would count both allowances even when one already expresses the useful resolution.
Explicit authored addition retains its existing deliberate meaning.

Goal-oriented influence directs work to consumed outputs. It adds evaluation, factors and
possibly reruns, so a speed improvement is a hypothesis, not a prerequisite or established
result. Do not build a numerical cost optimizer: proportional contribution allocation and
existing resource grants are sufficient for this bounded first implementation.

The [review's research assessment](../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#research-applicability-and-solver-specific-meaning)
explains the limits of conditioning estimates, goal-oriented literature and uncertainty decision
rules. Current official SUNDIALS documentation was also retrieved through Context7 during
authoring: [KINSOL stopping and forcing terms](https://github.com/llnl/sundials/blob/main/doc/kinsol/guide/source/Mathematics.rst)
and [IDA local error control](https://github.com/llnl/sundials/blob/main/doc/ida/guide/source/Mathematics.rst).
These support library-owned iteration and separate local/output errors. They do not establish
the loaded ABI or transfer new options into the captured SUNDIALS 7.1.1 baseline.

## Dependency sequence and review boundary

| Order | Contract available before dependent work | Execution/evidence owner |
|---|---|---|
| P0 | Proposed goal, context, evidence and permission contracts reviewed through the decision route | Coordinator; packages A0/C0 |
| P1 | Working admitted goals, frozen contextual policy and dependency identities | 27a A1–A3 |
| P2 | Working evidence interpreter and independently tested physical influence/allocation | 27b B1–B2 |
| P3 | Producer implementations consuming those demands, with validity and unavailable outcomes | 27b B3–B4 |
| P4 | Execution consumes actual evidence and grants; completion consumes final assessment | 27c C1–C3 |
| P5 | Migrated workflows, generated/durable consumers and end-to-end controls | 27c C4 |
| P6 | Stable assembled implementation, then one campaign and required measurements | 27c C5 hands off to 25k K3/K4/K5 |

A0 settles the semantic proposal; C0 inventories its boundary consequences and authors the
required ADR/design amendments before production changes. New relation meaning warrants an
ADR; changed hashing and Python boundary contracts require the ADR/design-review route.
Record the decision IDs in these front matters when they exist. Amend blueprint §§16.1,
16.2, 16.5, 16.6, §§13.3/13.6 and §19.2 through their owning architecture files and a revision
row. Do not edit accepted ADR rationale in place.

An agreed interface permits dependent design; it does not establish a working prerequisite.
B1 and C0 may be designed alongside A1, but execution consumers require tested A contracts.
Producer work can split by static/nested and dynamic responsibilities once B2 works. Shared
registry, numerics, strategy and completion edits each have one integration owner; logical
independence is not permission for concurrent writes to those surfaces.

No full campaign runs after each package. During functional work use touched-package compile
checks and targeted tests with explicit force-validation. Delete replaced mappings and callers
as soon as the replacement passes its targeted controls. Run regeneration as part of changed
registry declarations. Static hygiene and assembled testing wait for the functional handoff.

## Verification and completion

**Proposed verification:** companions define controls for review S01–S06. Independent expected
results include analytic affine amplification, explicitly specified interval classifications,
physical unit/datum conversions and trajectories with independently known accumulated outputs.
An expectation computed by the same allocator is not an independent accuracy oracle.

Plan 25k selects the full integration/native/Python/reference/parity and manual static scopes
for the final tree, once. The accuracy supplement measures no-goal overhead, clearly resolved
decision overhead and a genuinely refinement-requiring case, including total evaluations,
iterations/factors/proof work where observed, wall time and accounted memory. Preserve bounded
unknown counters. Pair identical model/binding/branch/requirements and run serially under the
memory-cap recipes; a coarse result that changes the required decision is not a speed win.
Do not make excluded host/CI scope or an unsupported universal bound a hidden completion gate.

Completion requires all packages, their immediate retirement obligations, generated/durable
consumer migration, S01–S06 acceptance within the stated evidence scope, and the 25k assembled
qualification/required measurements and final review. CA findings close only at their 25k owner
with linked corrective evidence; scheduling or accepting an ADR does not resolve them.

**Implemented / Interface-checked:** the reusable foundation assessment above is based on
source inspection and documentation. **Proposed:** all new contracts, algorithms and packages.
No product execution or performance measurements were performed while authoring this series.

## Current checkpoint

The maintainer's subsequent clean-pivot decision routes assembled target qualification to
[28e](28e-rebuild-retirement-and-qualification.md#qualification-handoff-from-existing-plans).
This plan retains its scientific contracts and functional evidence; CA finding dispositions
remain at 25k. The existing C5 handoff does not require completing old PG/Delta qualification
before the pivot, and its focused passes do not qualify the new substrate.

The ADR-0163 proposal, independently reviewed contracts and consumed-version inventory
are in place. Registry-owned declarations, contextual resolution, physical goal classification,
static evidence operations, bounded strategy refinement, dynamic comparison and retained
completion consumers are implemented in the current tree. Focused controls exercise these
owners; they do not establish the assembled campaign or performance.

Functional integration and the combined accuracy selection are complete. C5 carries the scientific work forward to 28e, retaining the finding dispositions at 25k. Square and KKT consumers now include actual same-point
value, derivative and matrix arithmetic uncertainty from the existing IBEX/FILIB adapter.
Active KKT output actions include the multiplier coordinates. Complete certified selected-root
reconstruction retains a point-bound enclosure that authored outputs can consume through
interval evaluation; partial reconstruction cannot issue that receipt. Explicit output goals
select their typed observation independently of reporting annotations. Optional numerical
estimator failure retains its typed cause without replacing the original solve conclusion.
Direct original-equation promotion retains its existing eligibility limits: the positive
selected-supplier control uses compiler-proven affine uniqueness. Branch-restricted suppliers
are not promoted by dropping their selection predicate.
Refinement permits changed work precision while freezing physical acceptance and consumes
the existing parent task grant rather than substituting a child catalogue limit.

The exact optimal-objective route now checks actual original affine coefficients, domains,
row sides, objective sense and offsets against the native exact optimum receipt, then includes
independent original-objective arithmetic. Rounded-source mismatch refuses correspondence;
native exact mode alone certifies only its uploaded problem. Its supported scope is bounded
affine problems without opaque providers or auxiliary/implicit coordinates.

Native refinement now keeps KINSOL residual scales anchored to frozen physical acceptance;
its adapter projects each requested row allowance through those actual scales. Tightening the
work tolerance no longer cancels itself or loses a demand on a row with a different allowance.
The supported original-square journey meets its declared output resolution.

Dynamic integration has corrected scheduled/endpoint input ownership, terminal failure precedence
and pre-operation arithmetic/work admission. Generated affine-rate providers currently expose
floating-point values without an arithmetic enclosure; outputs that consume those states cannot
borrow an interval claim from a provider-free output expression. A time/parameter-only observable
is the tested positive comparator scope alongside these explicit supplier refusals.
Integral goals remain unavailable without accumulated evaluator uncertainty; endpoint flux arithmetic
is not a substitute. Actual selected-root goal promotion and the dynamic journeys pass their focused controls. Supported zero/subresolution comparison retains actual evaluator uncertainty; spacing alone cannot resolve a goal. Temporary dynamic diagnostics are removed before the combined rerun.

Native refinement, dynamic and transport/readmission controls pass separately. The combined accuracy selection passes after the scope-end repairs; the remaining assembled scientific campaign,
required dev-profile measurements and final review transfer to 28e. None of
those enclosing obligations is complete yet. Preserve the existing PFR/native repairs and
historical receipts; qualify the final combined tree rather than inferring acceptance from them.

**Tested, 2026-10-05, zero-failure baseline:** the final `just unit-native-selected` rerun with the named accuracy/refinement/point-arithmetic/dynamic/original-objective/selected-root filters, default Nextest profile, linked native-solvers, explicit `pse-relations/force-validate` and one test thread passed all 77 selected tests in 11.746 seconds. The schema generator selection passed 5 tests, linked Python transport passed 7, native boundary contracts passed 22 and native version admission passed 1. These are focused functional controls; remaining assembled qualification and performance transfer to 28e, with the static and parity limits retained at 25k.

## Outcome (recorded after implementation)

Record what was built, a mistake made and corrected, deliberate deviations, scoped Tested and
Measured evidence, remaining limits and the 25k closure decision here when execution completes.
