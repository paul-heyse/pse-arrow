---
title: Contextual engineering accuracy contracts
date: 2026-10-05
tier: design
purpose: target
standard: core-3.4
profile: process-simulator-1.5
baseline: ad665a0222551196b1160e426f5242361215a6a0 plus current uncommitted tree
evidence: Proposed
architectural-fitness: Accept for the proposed contract scope
behavioral-adequacy: Accept for proposed semantics; implementation unqualified
decision: Accept
disposition-owner: docs/plans/25k-integrated-qualification-and-closure.md
---

# Contextual engineering accuracy contracts

**Accept at the Proposed contract level.** The revised Plan 27 target supplies the missing
engineering meaning without replacing numerical-product contracts, weakening root-selection
assurance or moving iteration into workflows. Its useful boundary is selective: ordinary
no-goal analyses retain physical qualification; declared goals request value resolution,
decision resolution or both; actual evidence and finite execution determine their outcome.

This acceptance is scoped to the proposal reviewed here, including the revised allocation and
shared-work rules. It does not establish implemented behavior, close CA-F01–CA-F04, accept
the existing unqualified PFR/native changes, or qualify all scientific workloads. No SHOULD
exception or supported MUST gap is accepted. The earlier source review retains its original
standard and judgment.

## Scope, baseline and evidence (slots 1 and 10)

This independent A0/C0 design-tier target review applies the current Core/template 3.4,
Process Simulator 1.5 and pse-arrow binding selected by `standard.toml`, following the
design-review and process-simulator skills. It examines these proposed documents:

- [Plan 27](../../plans/27-contextual-engineering-accuracy.md), shared target and sequence.
- [Plan 27a](../../plans/27a-accuracy-intent-and-contextual-policy.md), admission and context.
- [Plan 27b](../../plans/27b-error-allocation-and-numerical-evidence.md), evidence and allocation.
- [Plan 27c](../../plans/27c-selective-refinement-and-completion.md), execution and completion.
- [Proposed ADR-0163](../../adr/0163-contextual-engineering-accuracy.md), contract decision.

The baseline is HEAD `ad665a0222551196b1160e426f5242361215a6a0` plus the current
uncommitted tree on 2026-10-05. Relevant existing sources were inspected directly:
`pse-model/src/numerics.rs` and `strategy.rs`, `pse-math/src/numerics.rs` and
`derived.rs::ErrorAmplification`, `pse-backend-native/src/square_response/actions.rs`,
`pse-runtime/src/math/solves.rs::automatic_reconstruction_accuracy`, and
`pse-runtime/src/workflow/numerics.rs` and `workflow/modeling/results.rs`.
The preceding focused investigation also inspected selected implicit reconstruction and
trial-hint ownership. These are source/interface observations, not fresh execution receipts.

The boundary includes selected steady outputs, local optimization outputs, qualified optimum
objective intervals, implicit/reconstruction products, dynamic samples/endpoints/integrals,
study dispatch and the authored/request/result consumers they require. Relevant workload axes
are sparse problem size, goal fan-out, shared suppliers, long/stiff trajectories, case count,
concurrent requests and interruption. No fixed capacity SLA is invented. The supported evidence
scope is Plan 27b's matrix, including explicit unavailable outcomes; it is not a universal
error certificate. Whole-model physical fidelity, all derivative chains, concrete codec migration,
loaded native builds and integrated execution remain implementation/qualification obligations.

No probes or tests were warranted to settle the proposed boundary. Commands for targeted,
assembled and measured evidence remain owned by the implementation packets and Plan 25k.

## Responsibilities, contracts and physical meaning (slots 2 and 3)

| Responsibility | Owned decision and contract | Consumer and effect boundary |
|---|---|---|
| Registry/authoring/model | Goal meaning, typed defaults/scales, target/location and declaration precedence | Generated Rust/Python forms and compiler binding; no second expression evaluator |
| Model/math resolution | Engineering scale distinct from conditioning; frozen physical default and explicit overrides | Preparation and original acceptance consume the same immutable interpretation |
| Mathematical producer | Product-specific error meaning, influence, validity, branch and compatible tighter demand | Strategy consumes capabilities; local estimates cannot become certificates |
| Native integration | Actual sparse actions, qualified bounds, library controls and library state | Libraries retain iteration/globalization; tighter work products do not mutate original acceptance |
| Strategy | Selective evidence/refinement, progress and one finite task grant | Owns effectful attempts, charging and drain; not mathematical acceptance or publication |
| Completion | Resolution/criterion outcomes plus all independent candidate obligations | Retains immutable use permission; workflows/exporters project it without reclassification |

Dependency direction is authored intent → admitted/frozen context → consumed mathematical
demands → actual producer evidence → pure goal assessment → composed completion. Strategy
connects these operations and owns additional execution. Registry ownership determines meaning;
it does not require a separate table scan, factor, worker or transport for every goal kind.
Admission, resolution, allocation and classification can be tested with their semantic inputs
without native startup or an operational store. Native/dynamic tests remain necessary for
producer claims.

| Physical concept | Convention and invariant | Proposed authority |
|---|---|---|
| Engineering scale and allowance | Full quantity/basis; magnitudes or admitted differences; affine point datum excluded from scale | 27a contextual resolver and typed shared rule |
| Specification | Inclusive upper/lower limit or closed band; affine limits convert as points | Admitted goal criterion |
| Output numerical resolution | Optional positive error magnitude, independent of the specification band | Admitted goal resolution |
| Numerical evidence | Exact target/location/point/context/branch, physical transport and actual strength | Producer product plus retained interpretation |
| Feasibility/closure/optimality | Residual and bound violation, cumulative conservation, stationarity and objective gaps remain distinct | Existing policy/quality/completion owners |

Value-only, criterion-only and combined goals are meaningfully different. A criterion-only
goal can resolve far from a boundary without demanding extra output digits. Approximate
equality uses its authored physical band, not the numerical resolution. A deliberate singleton
band can remain unresolved. For an admitted envelope, containment means Satisfied, disjointness
means Violated, and overlap means Unresolved; strength and validity remain visible.

`Assess` can permit a resolved violation because discovering a failed specification is a valid
analysis result. `RequireSatisfied` adds the requirement to pass it. Neither bypasses original
checks or terminal failure. Arbitrary Boolean checks keep their current contract; they are not
reverse-engineered into goals. NoGoals means NotRequested, not implicitly satisfied or certified.

**Well-posedness boundary:** this proposal does not change variable roles, square-structure
admission, degrees of freedom or supported differential index. Existing structural/model
admission remains upstream. Error assessment additionally requires the local regularity,
selection/branch and derivative support stated by its producer; missing premises withhold
that assessment rather than inventing them. This review does not requalify current structural
analysis or physical models.

## Change scenarios and execution fit (slots 4 and 5)

The review uses [S01–S06](design_review_contextual-accuracy-policy_2026-10-05.md#scenarios-that-distinguish-the-alternatives)
and binding PSE-S01–PSE-S07. Their distinguishing consequences are:

| Scenario/variation | Proposed route and physical work |
|---|---|
| S01, ordinary no-goal edit/re-solve | Resolve context with existing preparation; no added adjoint, proof search or paired run solely for goals; reuse unchanged structure |
| S02, units/datum/capacity/cancellation | `max(F,r*S)` uses admitted engineering inputs; explicit numerical addition keeps its meaning; context changes invalidate only actual consumers |
| S03, many goals near/far from limits | Share compatible evidence passes; separated decisions stop; refinement requests actual contributing products and margins |
| S04, amplified supplier/long integral | Reserve fixed contributions, propagate actual influence, share upstream work and retain unavailable sources; no local-error-to-global-bound shortcut |
| S05, native substitution | Adapter/producer owns capability and control differences; the same assessment/completion contract remains consumed |
| S06, branch/discrete/active-set change | Invalidate incompatible local evidence; preserve selected-root proof, integrality and objective-bound distinctions |
| PSE-S07, case growth/interruption | Sparse dependency views, one compatible static factor/evidence pass, grouped comparator integrations, bounded existing study workers, parent grants and worker drain |

Static selected-output assessment reuses physical derivative/factor products: required Jacobian
work and sparse factorization can depend on the full admitted mathematical scope, while output
actions depend on protected outputs. The target does not require a dense inverse or dense
goal-by-model closure. Necessary full-scope work is distinct from rebuilding one factor per goal.
Point/parameter/branch changes invalidate numeric evidence while compatible structure can survive.

For compatible dynamic goals, one comparator integration supplies the requested scalar samples
or accumulations. Additional full trajectories are not retained just to assess scalar goals.
Long/stiff integration can remain expensive because it is necessary numerical work; multiplying
that integration by the number of compatible output goals would be incidental amplification.
Existing bounded study dispatch and native pool policy remain authoritative; grouping grants
no extra concurrent workers or independent maximizing thread pools. Superseded native workers
drain before replacement. Durable recovery uses the existing granted execution transition and
prior accounted work; reading completed evidence cannot restart integration.

| Numerical stage | Formulation/derivatives/scaling and outcome boundary |
|---|---|
| Context/default resolution | Original physical types and guards unchanged; conditioning separate; finite frozen magnitudes with strict/fallback behavior |
| Square/KKT evidence | Existing derivative owner and supported order; physical residual actions; local validity/rank required; Estimated, not nonlinear forward certification |
| Certified supplier evidence | Existing interval/root-selection owner; conservative operations over admitted region; required branch proof retained |
| Objective evidence | Original-convention primal/bound interval and actual bound strength; no variable-accuracy inference |
| Dynamic comparator | Existing integrators and event contracts; distinct local controls; empirical Estimated variation with original conservation assessed independently |
| Refinement/completion | Producer-derived tighter work demand, finite grant and actual reassessment; no moved threshold, loosened physical acceptance or publication effects |

Plan 27c's representable margin target is a work request, not evidence that a boundary is
resolved. Certified subtraction, transport and allocation require the stated directed arithmetic;
the final conservative envelope remains decisive. Nonprogress and exact-boundary refusal keep
finite behavior without substituting application iteration controllers for native algorithms.

## Review observations and required admissibility (slots 7 and 11)

### <a id="ca27-f01"></a>CA27-F01 — Reserve fixed contributions before reducible allocation

The first inspected 27b formula allocated `T-R` among reducible contributions without reserving
other fixed errors. With `T=1`, fixed error `0.4` and reducible error `0.8`, that request could
allocate `1` to the reducible product while total error remained `1.4`. This was a concrete
proposal defect under AP-04/AP-05, DP-11 and PS-10, relevant to S04.

The revised proposed rule now uses `A=T-R-sum(c_fixed)` and a reducible-only denominator.
It requires downward-rounded Certified demands and actual post-production reassessment. Direct
inspection of that amendment settles the proposal defect. Implementation acceptance must
exercise mixed fixed/reducible sources, exhausted/zero remaining allowance, correlated shared
sources and rounding boundaries. This does not resolve CA-F03's production obligation.

### <a id="ca27-f02"></a>CA27-F02 — Share compatible evidence operations before increasing workload

The initial dynamic wording admitted a comparator for each requested goal without explicitly
grouping compatible goals. That left AP-07 unsettled for goal fan-out: `g` outputs could cause
`g` complete integrations despite one integration serving all of them. The revised proposal
explicitly groups controls/binding/branch-compatible dynamic goals, shares static residual/
Jacobian/factor passes and sparse actions, retains scalar observations and preserves bounded
study workers. Proposed ADR-0163 preserves this physical route. This settles the proposal's
material uncertainty without a benchmark; it establishes no measured cost coefficient.

Production acceptance must distinguish actual sharing from merely shared charge labels. Observe
operation counts for several compatible goals, retain explicit incompatible groups, and challenge
large sparse models, long trajectories and interrupted workers in the assembled scope.

The accepted proposal also requires these admissibility conditions already implied by its
contracts; they are enforcement obligations, not permission to fill them with weaker evidence:

- A value-resolution assessment identifies the reported representative. For an asymmetric
  interval `[L,U]` and reported value `q`, its bound is at least `max(abs(L-q),abs(U-q))`.
  Half-gap cannot silently bound an incumbent objective presented as `q`. Criterion-only
  classification can consume the interval directly without choosing an optimum representative.
- No valid-looking allowance upgrades absent evaluation/supplier uncertainty, branch evidence
  or derivative support. An empirical comparator remains explicitly Estimated; zero variation
  alone does not establish exactness or certification.
- Context/goal changes enter admission identity; tighter work precision has its own identity.
  Compatible factor reuse is producer-owned and cannot reuse stale point evidence. Final
  goal refusal cannot manufacture otherwise forbidden seed permission.
- C0 must inventory actual codecs, generated requests, job/study payloads, operations storage
  and result consumers before production boundary edits. Supported historic no-goal readmission
  means NotRequested; old result bytes never gain a new accuracy guarantee.
- Final permission is composed once from all independent obligations. Cancellation/resource/
  domain terminal causes retain precedence; assessment, export and restart inspection are pure.

Plan 25k remains the sole CA finding-disposition/qualification owner. The review records the
draft observations and corrected proposal, not a duplicate implementation status ledger.
Plan 27 owns package execution; the architecture amendments belong to blueprint §§16.1/16.2/
16.5/16.6, §§13.3/13.6 and §19.2 through the existing decision/design route. Proposed ADR-0163
changes goal, identity and generated boundary meaning; its acceptance remains distinct from
implementation acceptance. No accepted ADR rationale was amended by this review.

## Alternatives and library fit (slots 8 and 9)

Shared static physical defaults are the simplest viable ordinary fallback. They do not settle
output propagation or decision margins. Pure relative defaults fail at zero; conditioning-only
scales leak unrelated numerical rationale into physical acceptance. Blanket tighter tolerances
or universal certification add mandatory work without a protected-output benefit.

The selected target uses shared contextual defaults plus selective existing numerical products.
Sparse factors/actions and native objective bounds preserve consumed capability instead of
hiding it behind one universal tolerance. Existing interval reconstruction supplies its own
certified products. Native integrations retain local controllers; the additional paired operation
is a bounded engineering evidence operation, not a reimplemented controller. A common method
owner prevents fixture/workflow-specific variants. This additional run has a concrete benefit
for requested dynamic empirical assessment and is excluded from ordinary no-goal work.

No new library API, dependency admission or loaded-ABI claim is accepted here. Native/math
capability skills were consulted for ownership boundaries; current local source establishes the
already available sparse response route. A later producer requiring unsupported derivatives,
transposed actions, observation locations or assurance must revisit its explicit capability,
not broaden this verdict. Extra estimator work may cost more than baseline execution; improved
latency or robustness is Proposed until the planned representative measurements.

## Foundations and gates (slot 6)

All verdicts below concern the revised Proposed contract scope, not implemented conformance.

| Foundation | Verdict and reason |
|---|---|
| AP-01 | Satisfied: science, engineering policy, producer evidence, effectful strategy and immutable completion have distinct owners |
| AP-02 | Satisfied: actual product/strength/validity and unavailable outcomes survive backend substitution |
| AP-03 | Satisfied: goals compose through shared math/strategy/completion rather than workflow-local rules |
| AP-04 | Satisfied: output resolution, criterion, engineering context, evidence and use policy have adequate separate meanings; corrected allocation respects them |
| AP-05 | Satisfied: admission, evidence compatibility, finite refinement and refusal have explicit enforcement boundaries |
| AP-06 | Satisfied: semantic operations admit local tests without unnecessary solver/store effects |
| AP-07 | Satisfied: revised shared sparse/comparator route and bounded existing execution lifetimes address material workload growth without universal no-goal work |

| Gate | Judgment in proposal scope |
|---|---|
| G1 | Pass: one declaration and consumed semantic owner per meaning |
| G2 | Pass: no-goal, unavailable, estimated/certified, optimum/output and independent obligations remain distinct |
| G3 | Pass: typed admission and validity enforcement are specified; no weak-evidence upgrade is allowed |
| G4 | Pass: only execution performs additional work; completion/publication inspection is immutable |
| G5 | Pass: finite grants, progress, terminal precedence and existing durable transitions specify lifecycle |
| G6 | Pass: actual bindings/context/point/branch/work demand determine consumed reuse identity; C0 implements concrete migration |
| G7 | Pass: matrix and unavailable outcomes limit the claim; no tests, certificates or speed measurements are invented |
| G8 | Pass: existing mathematical/native owners retain iteration, factors, local controls and certification |
| G9 | Pass: all seven foundations satisfied for the revised target boundary |
| PS-G1 | Pass for proposed physical semantics: differences/bases, original checks and cumulative closure retain separate authority; implementation science unqualified |
| PS-G2 | Not applicable to changed structural scope: no variable-role, differential-index or structural-admission change; enclosing well-posedness unqualified |
| PS-G3 | Pass for proposed numerical contract: actual evidence, finite stops and independent acceptance govern claims; actual producer execution unqualified |

## Verification and decision (slots 10 and 12)

**Interface-checked / Implemented:** inspected existing numerical-product, resolver, sparse
action and completion owners supply reusable paths. **Proposed:** all new goals, engineering
defaults, producer interpretation/allocation, refinement and boundary migrations. Proposal
reasoning and the allocator counterexample settle this review; no new Tested or Measured
claim is made. No production tests or probes were run, and no formal numerical theorem is claimed.

The packets' independent affine-amplification, physical-unit/datum, interval-classification,
fixed/reducible allocation, branch-change and accumulation controls are appropriate. Add actual
shared-operation counts and the representative-value condition above to their acceptance.
Implementation must challenge estimator limits rather than derive its oracle from the allocator.
The assembled Plan 25k campaign, no-goal/clearly-resolved/refining measurements and scheduled
integrated review establish the implementation's named scope; proposal acceptance is not that
receipt. No model-parameter or physical formulation change is proposed here, so existing unit/
property conformance evidence remains historical until affected execution is requalified.

**Behavioral/semantic adequacy: Accept for the proposed semantics. Architectural fitness:
Accept for the revised proposed target. Overall decision: Accept at Proposed evidence level,
scoped to the contracts, support matrix and admissibility conditions above.** Concrete codec
choices, numerical behavior, scientific adequacy, scalability coefficients and full product
qualification remain outside this acceptance and must receive their planned implementation
evidence. The earlier source review and current CA implementation dispositions are unchanged.
