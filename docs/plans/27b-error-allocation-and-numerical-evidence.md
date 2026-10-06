---
title: "27b: Error allocation and numerical evidence"
status: in-progress
date: 2026-10-05
adrs: [ADR-0163]
review_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#scenarios-that-distinguish-the-alternatives"]
---

# 27b: Error allocation and numerical evidence

## Purpose and foundation assessment

Derive numerical work from its consuming engineering requirement and report what the work
actually establishes. This companion owns B1–B4 and the producer-support boundary for CA-F03;
[25k](25k-integrated-qualification-and-closure.md#contextual-accuracy-review-dispositions) owns
finding disposition. Consume working goals and frozen context from
[27a](27a-accuracy-intent-and-contextual-policy.md), then deliver evidence and production demands
to [27c](27c-selective-refinement-and-completion.md).

The baseline already has `AccuracyDemand`/`AccuracyEvidence`, product/source identities,
`ErrorAmplification`, composite supplier uncertainty and certified reconstruction refinement.
These are reusable foundations. `ErrorAmplification::inner_allowance` maps consumed forward
accuracy through inverse/remainder evidence. Its generic propagation arithmetic must additionally
preserve outward rounding when claiming Certified evidence; current ordinary binary64 arithmetic
alone does not establish that property. Reconstruction's interval-to-error conversion already
widens primitive operations. Preserve that implementation rather than adding another proof engine.

`square_response::SparseFactor::action` accepts a physical right-hand side, reuses sparse LU
and checks the actual linear backward error. It does not certify nonlinear forward error.
Current reconstruction point demand uses the smallest normalized variable bound budget and
its action demand uses stationarity. `TrialHints::resolve` similarly borrows stationarity for
derivative backward error. These automatic associations are replaced where a consumed output
or action demand supplies the actual meaning.

Without a declared analysis goal, a required numerical supplier still has operational consumers.
Resolve its ordinary point allowance from the contextual resolution of the actual reconstructed
output, not its bound-violation allowance. Derivative/action demands come from the consuming
residual, linear action or native operation's admitted accuracy contract, not an unrelated KKT
field. Preserve those operational demands and all existing certification/selection obligations;
no-goal does not mean unchecked supplier products. Such existing required work does not create
an analysis-level output guarantee or trigger optional goal-estimation work.

## Evidence and influence contract

Keep `AccuracyEvidence` as the producer-facing strength/error product. Engineering assessment
also needs its physical target/location, actual point and context, method, branch/regime,
validity witness and typed unavailable reason. Declare that retained interpretation once in the
registry; do not pack meaning into strings or use NaN as absence.

An evidence product distinguishes output-error estimate/bound from residual error, linear
backward error, local integration error, empirical output variation and objective interval.
Only an admitted output-error interpretation can resolve an output goal. Exact symbolic
derivatives alone do not supply zero numerical error. A condition estimate remains an estimate,
not an inverse-norm upper bound. A deterministic uncertainty does not acquire statistical
confidence or a probability distribution.

Influence belongs to the consuming mathematical operation. It records input/output products,
coordinate transport, nonnegative amplification, any separately established remainder,
evidence class, dependencies and validity region. Reuse `ErrorAmplification` for its supported
inverse/remainder case; compose existing supplier and physical-expression operations around it.
No global graph guesses gains from quantity names or evaluates a second version of the science.

Certified composition requires conservative component bounds and valid influence over the
whole admitted region. Widen products, sums, denominator subtraction and divisions in the
appropriate direction. Overflow, invalid denominator or unrepresentable demands become typed
unavailable/precision outcomes. Estimated composition remains Estimated. A chain containing
unavailable required contribution is unavailable, not the sum of its known subset.

Combine dependent deterministic contributions by conservative summation unless the owning
operation establishes a tighter joint treatment. Do not use root-sum-square. Account for common
upstream errors once at their actual source before propagating them through multiple paths;
combining their correlated paths uses a valid joint influence or conservative absolute path sum.
Resource charging likewise follows actual operations, not each consuming goal.

### Allocation from actual contributions

Start with the contextual/native baseline, assess actual evidence, then allocate only when
the goal needs improvement. For output allowance `T`, numerical contributions `c_i = G_i*e_i`
and fixed admitted remainder `R`, require `sum(c_i)+R <= T`. A certified influence may use
the existing inverse/remainder formula instead of this additive form.

Reserve all nonreducible contributions first: `A = T-R-sum(c_fixed)`. If `A <= 0`,
refinement cannot meet the goal unless the remaining demands are already met.
When `A > 0` and positive reducible contributions exist, distribute the remaining output
allowance in their observed proportions:

`c_i_requested = A * c_i / sum(c_j_reducible)`

`e_i_requested = c_i_requested / G_i`

Use downward-rounded demands for Certified allocation. Take the tightest compatible demand
for a product shared by several goals after unit/normalization transport. Zero gain does not
request needless accuracy; unknown gain does not mean zero gain. Fixed contributions already
exceeding the allowance, invalid influence or unsupported producer capability produce a typed
unresolved outcome. Allocate only to actual reducible numerical products; do not attempt to
refine model discrepancy or input uncertainty.

This policy preserves observed contribution proportions without a cost optimizer or fixed
per-stage safety factor. It is not claimed to minimize runtime. Reassess after production:
changing validity, gains or output point can invalidate the previous allocation. Initial
unavailable evidence is obtained only through a declared supported evidence operation; absent
support does not justify blind successively tighter solves.

Native controls use the producer's mathematical interpretation of these demands. A tighter
work-control product is separate from immutable physical acceptance and carries its own key.
For each comparable control it may tighten, never loosen, the original requirement. Do not
take a raw minimum across unlike physical quantities or identify derivative accuracy with KKT
stationarity. Adapters retain reserved-option checks and class-specific limits; an infeasible
native setting is reported as unavailable/precision, not silently clamped into a claimed bound.

## Producer support and validity

An asymmetric optimum interval assesses value resolution about its reported representative
`q`: `error=max(abs(L-q),abs(U-q))`. Half-gap is lawful only if the reported value is the
midpoint; criterion-only classification consumes the interval directly.

The matrix defines initial evidence support. Every current class participates in admission,
assessment and completion; it need not fabricate the same estimator or certificate.

| Producer/product | Implemented target evidence operation | Strength and limits |
|---|---|---|
| Regular square selected solution | Evaluate original residual `r`, prepare/reuse admitted physical Jacobian factor, compute `d = -J^-1 r`, and evaluate protected-output derivatives. Use `sum(abs(q_xi)*abs(d_i))` plus admitted supplier/evaluation estimates as a local estimate. Retain the correction residual/backward error and validity; do not form a dense inverse. | Estimated only. Require regular square scope and an established local interior/branch. No small-step or residual-only forward certificate. Exact affine/certified interval evidence may resolve Certified goals through its separate valid operation. |
| Smooth NLP selected local solution | Reuse qualified KKT analysis on an isolated regular active set; apply the KKT residual correction to protected outputs, with actual derivative/source and original qualification. | Estimated, local to the admitted stationary branch. Rank loss, weak/ambiguous active set or unavailable Hessian/support withholds it. No global optimum or stable discrete choice claim. |
| Certified implicit/reconstruction output/action | Consume existing selected-root enclosures, uniform inverse bounds, interval derivative actions, branch proof and composite supplier uncertainty. Request only the output/action needed by the goal. | Certified only for the actual verified region/product. Preserve competitive-root exclusions and connected-path obligations; required certainty cannot be downgraded. |
| Other implicit supplier | Use admitted regular-square response and actual supplier residual/linear observations, propagating them into the consuming output. | Estimated if all required local validity exists; otherwise unavailable. Do not copy outer stationarity into the derivative demand. |
| LP/MIP/conic/global objective | Use the qualified original feasible objective and actual original-convention dual/global bound to form the appropriate optimum-value interval. Include numerical evaluation evidence under its retained interpretation. | Strength follows the actual bound/certificate, never a native solved label. Incumbent gap is not error of an arbitrary variable; wrong-side, unavailable or relaxed-only evidence cannot certify that value. |
| LP/conic variable or discrete/phase outcome without valid uniqueness/selection evidence | Retain point/selection and existing feasibility/integrality/branch facts. | Output/decision error unavailable. Degeneracy, branch transitions and categorical outcomes cannot borrow smooth sensitivity or objective-gap evidence. |
| Dynamic sampled/endpoint/integrated observable | Run the bounded paired output-stability operation below using existing native integrators and declared output samples/accumulations. Existing certified producer enclosures may contribute where applicable. | Explicitly Estimated empirical output-error interpretation, not a global trajectory enclosure. Invalid event/path correspondence or unsupported observation yields unavailable. |

For a regular square goal, the correction above is an estimator, not a new nonlinear iteration
controller. Its first-order approximation can fail outside a local region. Preserve the existing
rank/neighborhood checks, actual point and branch evidence; mark the method and limitations in
completion. Certified requests need valid region/remainder or interval evidence. Never promote
an estimate by applying a generic conditioning multiplier.

An Estimated correction of zero does not establish exact output error. Retain actual admitted
evaluation/supplier uncertainty and at least the output's physical representational spacing
as an explicit estimate floor when those products permit this interpretation. That spacing
only prevents fabricated exactness; it is not a rounding-error upper bound. Missing material
evaluation uncertainty remains unavailable. Certified zero requires an actual zero-error
product, not a zero residual or unchanged floating-point output. These numerical representation
checks do not change the shared engineering allowance.

Selected-observable derivatives use the same authored program and library mathematical owner.
Request only goal-relevant actions; reuse one factor across compatible outputs. Base no-goal
solves do not acquire these analyses. Estimated linear solves retain their implementation-error
controls: rank cutoff and binary64 backward-error checks are not engineering resolution knobs.
Where a producer accepts a consumer-derived action budget, pass it explicitly through that
owner and test the actual demanded product.

### Dynamic output assessment and refinement

Native state, residual, sensitivity and quadrature tolerances retain their separate roles.
SUNDIALS/IDA local weighted error and Diffsol local controls do not bound endpoint or integral
error. Initial conservation quadrature controls retain their current consuming inventory
allowance; independent original cumulative closure remains required.

Use the existing repeated integration operation to obtain an additional trajectory only for
compatible requested dynamic goals lacking sufficient evidence. One comparator integration
serves all compatible goals sharing controls, binding and branch/event context; retain only
the consumed scalar samples/accumulations. Compatible static goals likewise share one
residual/Jacobian/factor pass and sparse output actions. Existing study dispatch bounds live
case workers; grouping does not authorize unbounded concurrency. Compare each selected output at
the same admitted physical sample/actual endpoint, or its declared final integrated value.
Preserve parameter schedules, boundary conditions, branch and event correspondence. Do not
compare different event counts, modes or terminal meanings as if only numerical accuracy changed.

The first comparator tightens the contributing native local controls by one binary halving
at a single shared method owner. This is a declared contrast for estimating output variation,
not an engineering accuracy requirement or an inferred method-order factor. It changes no
goal, physical check or user tolerance; it is never tuned per model/fixture. Use the observed
physical discrepancy as an empirical estimate with its method explicitly retained. Include
available supplier/evaluation contributions, and mark the result Estimated even when those
contributions individually have certificates. A zero/sub-resolution discrepancy without other
support is unavailable comparison evidence, not fabricated exactness.

For later refinement, map the goal's residual output allowance and observed discrepancy into
a new requested control ratio; no continuing unconditional halving ladder. Native integration
controllers choose their steps/order and event handling. A request cannot claim Certified
trajectory error from this method. Agreement can preserve a shared model or branch defect;
the completion qualifier and independent scientific checks must retain that limitation.

Do not retain extra full trajectories when only scalar samples/accumulations are consumed.
Use existing bounded result/export owners and release unsuccessful comparator workers before
the next attempt. Charge both runs and assessment; all runs consume the parent deadline,
memory and attempt/work allowance. With insufficient allowance, return unresolved immediately.

## Packages and local acceptance

| Package | Required input | Delivery, owner and deletion | Progress |
|---|---|---|---|
| B1 — Evidence interpretation | Settled A0 goals and C0 boundary route | Math/model owner adds validity-bearing physical output evidence and pure classification inputs using existing accuracy products. Retire adapters that treat a residual/native success as output evidence. Targeted `engineering_accuracy_evidence` controls | done |
| B2 — Influence/allocation | Working A2/A3 and B1 | Math owner implements compatible physical transports, conservative/estimated composition, outward Certified arithmetic and observed-contribution allocation. Reuse ErrorAmplification and composite ownership. Targeted `goal_error_allocation` controls | done |
| B3 — Static/nested producers | Working B2 and complete selected goal dependencies | Native/math owners add square/KKT output actions, qualified objective intervals and consuming demands for implicit/reconstruction chains. Delete automatic point/action mappings from bound budgets/stationarity once migrated; preserve independent selection proof. Targeted `goal_accuracy_static` controls | done |
| B4 — Dynamics/integrals | Working B2 and dynamic location binding from A3 | Dynamic owner adds bounded comparator/consumer allocation, actual observation correspondence and uncertainty retention. Retire any interpretation of local tolerance or closure residual as global output error. Targeted `goal_accuracy_dynamic` controls | done |

The shared math/registry owners integrate B1/B2 before producer branches. B3 and B4 can be
implemented independently after their working contracts exist, with coordinated shared files.
Native capability substitutions remain inside integration owners; no hardcoded per-workflow
solver or tolerance policy is added.

## Verification and completion boundary

**Proposed controls:** the analytic relation `alpha*y-p=0` exposes residual-to-output
amplification independently of the allocator; rescaling it preserves physical output demand.
Affine and composite chains test summed/correlated contributions, unavailable sources, shared
upstream identity and outward rounding near binary64 representational boundaries. Strength
mixing cannot create certification. Changes to branch, point, parameters, goal or scale prevent
invalid evidence/factor reuse. Zero sensitivity does not cause needless work; unknown sensitivity
does not suppress a contribution. Unrepresentable/capability-limited demands stop truthfully.

Square and KKT controls verify actual derivative/action conditions, source validity and original
acceptance; they do not test empirical reproduction of a theoretical convergence order. Use
independently known output values and branch changes to challenge estimated/certified claims.
Objective controls distinguish optimal objective intervals from incumbent variable accuracy,
including degenerate and changed discrete outcomes.

Dynamic controls include a trajectory with independently known endpoint and accumulated output,
long accumulation, event correspondence, zero comparator variation, exhausted resources and
unchanged original conservation. Verify that local tolerance alone cannot resolve a Certified
output goal. A no-goal operation creates no comparator. Native tests use the existing capped
recipes and force-validation; ordinary interpretation/allocation tests need no linked runtime.

**Implemented / Interface-checked:** the inspected source foundations above and the review's
native/literature distinctions. **Proposed:** producer methods and all B package acceptance.
No new native execution or measurement occurred during authoring. Local B completion establishes
its stated products; end-to-end usefulness and performance require C/25k evidence.

Execution checkpoint, 2026-10-05: B1/B2 interpretation and allocation are implemented
with focused controls. B3 regular-square, KKT and objective producers consume
original-program interval arithmetic. The producer review exposed that output
representational spacing alone cannot account for residual/Jacobian evaluation error.
The correction reuses bounded IBEX point evaluation of the existing authored program,
including the required derivative order, and propagates its actual uncertainty through
the existing factor actions. Selected computed outputs use the same authored evaluation
route and original column order. Complete certified reconstruction now issues a sealed
receipt for its actual original coordinate enclosure; authored outputs consume interval
values over that whole box. Partial reconstruction does not certify retained free
coordinates. B4's shared comparator and observation correspondence are implemented.
Its focused review identified missing authored evaluation uncertainty and shared attempt
accounting, plus optional failure and comparator retention corrections. Those repairs are implemented with focused positive and refusal controls. The native original-square refinement reaches its
requested resolution with frozen acceptance and adapter-owned work projection. Dynamic
state-dependent evidence remains unavailable when the generated affine-rate supplier has no
arithmetic enclosure; provider-free time/parameter outputs are the bounded positive comparator
scope. Real dynamic and selected-root journeys pass separately; their combined selection precedes the stable handoff. No universal
certificate or performance claim follows from these implementations.

## Outcome (recorded after implementation)

Record implementation, a mistake corrected, deliberate deviations, source-specific support
limits and local evidence here; link the single CA-F03 disposition at 25k.
