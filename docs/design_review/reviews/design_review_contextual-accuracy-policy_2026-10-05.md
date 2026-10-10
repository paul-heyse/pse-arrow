---
title: Context-aware engineering accuracy
date: 2026-10-05
tier: design
purpose: target
standard: core-3.3
profile: process-simulator-1.4
baseline: ad665a0222551196b1160e426f5242361215a6a0 plus current uncommitted tree
evidence: Implemented
architectural-fitness: Revise
behavioral-adequacy: Not Accept for the contextual-accuracy target
decision: Revise
disposition-owner: git:4c24721e691187e1a5b28398b29722fbde671da8:docs/plans/25k-integrated-qualification-and-closure.md
---

# Context-aware engineering accuracy

The current numerical-policy machinery provides a useful foundation: one sourced interpretation,
physical budgets associated with model identities, library-specific lowering, independent
original-space checking, and truthful distinctions among feasibility, optimality, closure and
permission to use a candidate. Those responsibilities should be preserved.

The agreed target needs a further contract connecting numerical work to engineering outputs and
decisions. Current tolerances principally govern residuals, bound violations, steps and native
optimality conditions. They do not establish how accurately an output is known, whether its
numerical uncertainty could change a declared decision, or when the automatic pipeline should
refine that uncertainty. Changing shared constants alone cannot supply that meaning.

**Architectural fitness: Revise for the agreed target**, principally under AP-04/G9.
**Behavioral adequacy: Not Accept a claim of automatic context-rationalized engineering accuracy.**
This is not a rejection of the narrower, truthful physical-feasibility support already implemented.
An analysis without declared decisions should still produce an honestly scoped ordinary engineering
result under shared defaults and explicit overrides.

## Scope, drivers and evidence

This independent, read-only design-tier review applies Core/template 3.3 and Process Simulator 1.4.
It examines accuracy selection, interpretation, consumption and assessment across square simulation,
nonlinear optimization, nested implicit evaluation, reconstruction, dynamics and representative
authored checks. Studies, recycles, branch-sensitive cases and discrete decisions supply variation
scenarios; their complete implementations are not qualified here. A fresh design reviewer supplied
the principal judgment; the coordinator inspected decisive sources and reconciled bounded native
library/literature research with it.

The target is useful accuracy throughout development and later use. Future work does not inherently
demand smaller tolerances: a tighter requirement needs an output or decision that benefits from it.
Robustness, bounded resource use, interactive performance and truthful evidence constrain the design.

The inspected baseline is HEAD `ad665a0222551196b1160e426f5242361215a6a0` with current uncommitted
changes. Source-backed paths below are **Implemented**; inspected interfaces are **Interface-checked**;
corrective directions are **Proposed**. No new execution or performance evidence was produced.

Relevant owners are blueprint §§16.1, 16.2, 16.5 and 16.6, §§13.2–13.6 and §19.2. The
[earlier convergence-criteria review](design_review_convergence-criteria_2026-10-05.md) retains its
narrower historical scope and does not supply this verdict.

[Plan 25k](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/25k-integrated-qualification-and-closure.md#contextual-accuracy-review-dispositions)
owns current disposition. Proposed corrections are not automatically scheduled.

## Existing responsibilities and guarantees

| Owner | Responsibility and consumed contract |
|---|---|
| `pse-model::numerics` | Shared numerical policy, explicit overrides and independent KKT, integrality, gap, closure and incumbent controls |
| `pse-math::numerics` | Physical magnitude conversion, precedence, frozen target budgets and provenance |
| Modeling preparation | Bind authored requirements and derive term-based nominals at the selected nominal point |
| Native adapters and runners | Translate resolved budgets into class-specific controls; libraries own iteration and globalization |
| Implicit/reconstruction producers | Produce and verify numerical points/actions under explicit consumed demands |
| Dynamic preparation/integrators | Consume state, residual and quadrature controls; retain original conservation obligations |
| Workflow assessment/completion | Combine original checks and native evidence into one immutable candidate-use decision |

The resolver chooses each field by source precedence, rejects conflicts and invalid magnitudes,
records canonical fallback, and supports strict nominal completeness. Magnitude conversion excludes
affine offsets. Integer coordinates preserve their lattice.

Top-level term scaling uses original additive terms evaluated at a nominal point: free variables
take resolved nominals; fixed variables and sensitivity parameters retain their values. Nested
`Configuration::Hints` instead resolves its declared configuration at each enclosing trial, including
input-dependent term observations. These are different lifecycles, not evidence that every scale is
static or that current iterates arbitrarily redefine outer acceptance.

Native lowering also contains important distinctions. NLP row transport accommodates heterogeneous
physical budgets. Ipopt retains separate component criteria; POUNCE uses its distinct conservative
overall criterion. KINSOL derives per-coordinate residual and step scaling and classifies
`KIN_STEP_LT_STPTOL` as a limit, not success. Fresh original-space checks remain authoritative.

| Physical meaning | Representation and authority |
|---|---|
| Temperature accuracy | A magnitude in K or an equivalent interval representation; distinct from an absolute temperature point |
| Material flow/inventory | Typed molar, component, element or volumetric quantities; bases remain distinct |
| Power/energy transfer | Distinct physical kinds despite sharing W |
| Integrality | Original integer-lattice violation; not continuous engineering resolution |
| Stationarity/complementarity | Normalized optimality conditions; not output-error bounds |
| Dynamic closure | Inventory minus integrated signed flux and permitted transfers; independently assessed |

Structural admission remains upstream of solving. KINSOL checks square structure; dynamic lowering
requires the declared supported index-1 partition. This review does not re-establish whole-model
well-posedness or derivative fidelity.

## Scenarios that distinguish the alternatives

| ID | Scenario and expected boundary |
|---|---|
| **S01** | Ordinary design without declared decisions: resolve shared contextual defaults, retain overrides and provenance, return ordinary numerical/physical qualification without promising decision invariance |
| **S02** | Change throughput, units, affine datum, or a nearly cancelling net quantity: preserve physical meaning; explain the characteristic scale and absolute floor rather than treating raw magnitude as universal accuracy |
| **S03** | A declared specification lies near the computed output: assess quantitative margin and numerical error evidence; refine within budget or report an unresolved decision |
| **S04** | An ill-conditioned nested relation, reconstruction chain or long trajectory amplifies numerical error: allocate internal work from the consuming output requirement and retain independent original checks |
| **S05** | Replace a root/NLP/integration backend: retain semantic requirements and class-specific outcomes; change lowering inside its integration owner |
| **S06** | A phase, active-set or discrete alternative changes: withhold invalid local sensitivity claims; preserve branch, integrality, objective-gap and certification distinctions |

These include policy changes, contextual bindings, composition and mechanism substitution. A new
unit using established quantities should inherit the policy mechanism; genuinely new engineering
meanings may require a deliberate contract extension.

## Findings

### <a id="ca-f01"></a>CA-F01 — The policy lacks a contract for engineering output resolution and decision stability

**Principles:** AP-04, AP-05, DP-02, DP-11; G2/G9; S01, S03, S06.

**Implemented evidence:** `NumericalPolicy` and `ResolvedTarget` describe coordinate magnitudes and
numerical budgets. `quality::observed` interprets variable budgets as **bound-violation** allowances
and row budgets as **constraint-violation** allowances. Existing `AccuracyDemand`/`AccuracyEvidence`
describe identified numerical products. None of these inspected contracts expresses the complete
analysis-level meaning of a protected output, a material decision margin and the evidence required
to resolve it.

A bound-violation allowance is not a bound on the distance to the true solution. A feasible,
stationary point does not necessarily identify a stable engineering choice. Existing statistical
covariance propagation in blueprint §19.8 has another meaning and cannot substitute for numerical
error assessment.

**Proposed correction:** extend the existing analysis/policy boundary with the smallest sufficient
output or decision contract: protected quantity, physical resolution, relevant specification or
comparison, required evidence strength and bounded refinement permission. Keep “no declared decision”
explicit and lawful.

**Verification:** exercise goal admission and interpretation independently of native startup.
Demonstrate that numerically feasible results can retain an unresolved engineering decision, while
ordinary no-goal results remain available and honestly scoped.

### <a id="ca-f02"></a>CA-F02 — Default acceptance can inherit a conditioning scale without an engineering rationale

**Principles:** AP-04, DP-11, PS-07; S01, S02.

**Implemented evidence:** `numerics::resolve` defaults to `absolute = 1e-3 * nominal`, with relative
zero. Continuous coordinate scale also derives from nominal. An absent quantity nominal falls back
to one canonical unit; `term_scale` returns one when all observed terms are zero. The shared reference
policy supplies fixed physical allowances.

These choices are explicit and inspectable, but normalization, characteristic physical magnitude
and material engineering resolution are different decisions. A scale that assists conditioning can
be unsuitable for a trace quantity, cancellation-sensitive net duty, temperature difference or
quantity with an arbitrary datum. Conversely, a small fixed absolute allowance can demand unnecessary
work at large scale.

The provisional `1e-3`, 0.1 K, 1 W and 1 kg examples are not established optima. Correct affine
magnitude conversion already exists; this finding concerns scale **meaning**, not an offset-conversion
defect.

**Proposed correction:** retain shared default parameters and explicit individual overrides, but
define their contextual rationale and scale lifecycle. Evaluate static physical allowances,
range/capacity/nominal-based allowances and mixed absolute-relative rules against S02. Magnitude
scaling is a candidate, not a mandated formula. A recorded canonical fallback may remain available;
strict completeness stays an explicit choice.

**Verification:** equivalent-unit and datum scenarios, near-zero/trace cases, cancellation and scale
changes must retain the intended physical allowance. No per-fixture adjustment to observed errors
constitutes closure.

### <a id="ca-f03"></a>CA-F03 — Internal demands reuse outer numerical budgets without completing output-error allocation

**Principles:** AP-02, AP-04, DP-11, PS-07/PS-10; G2; S04.

**Implemented evidence:** `TrialHints::resolve` supplies physical unknown/residual budgets directly
and takes derivative linear-system backward-error tolerance from KKT stationarity.
`automatic_reconstruction_accuracy` derives point demand from normalized variable budgets and action
demand from stationarity. Dynamic quadratures initially inherit the tightest consuming inventory
allowance, while state integration has separate local controls.

These operations preserve distinct fields, but the automatic mappings do not establish how their
error affects the requested engineering output. For example, in the dimensionless relation
`αy − p = 0`, residual error `ε` permits state error `ε/|α|`. This mathematical counterexample concerns
the contract; it is not an observed solver failure.

Reconstruction already contains substantial budget-dependent refinement, including root/residual
tightening and directional derivative allocation. The defect is therefore incomplete
**goal-derived allocation**, not absence of refinement. Likewise, final dynamic closure can truthfully
refuse a trajectory without establishing that its initial local allocations were sufficient.

**Proposed correction:** derive internal demands from actual consuming output/action requirements
and valid propagation information. Keep residual error, forward point error, derivative backward
error, trajectory error and cumulative closure distinct. Use conservative combination where
conservative bounds exist; do not impose an arbitrary tightening factor or statistical combination
of deterministic errors.

**Verification:** independently known amplification cases, supplier chains and accumulated quantities
should demonstrate required output accuracy, truthful unavailable evidence and finite refusal when
the allowance cannot be established.

### <a id="ca-f04"></a>CA-F04 — Point checks do not supply the quantitative evidence needed for boundary refinement

**Principles:** AP-03–AP-05, DP-12, PS-10; S03, S04.

**Implemented evidence:** generic `ModelingHint::Check` assessment retains a Boolean-like value,
`tolerance: None`, and `satisfied = value == 1`. Objective-bound checks have a specialized retained
comparison/bound interpretation. General checks do not provide a quantitative margin and numerical
uncertainty that the automatic strategy can consume to distinguish “clearly satisfied”, “clearly
violated” and “numerically unresolved”.

A failed check may therefore trigger a truthful refusal without establishing whether more numerical
accuracy could resolve it. A passed point predicate similarly establishes its current predicate
value, not stability under numerical error. This is consistent with the current check contract but
insufficient for S03.

**Proposed correction:** preserve generic Boolean checks and add quantitative assessment for declared
accuracy goals/specifications. Producers supply physical margins and estimates/bounds with validity;
strategy owns bounded refinement; completion retains the decision and its evidence. Arbitrary Boolean
expressions need not be reverse-engineered into automatic goals.

**Verification:** cases on both sides of a threshold, overlapping error evidence, nonprogress and
exhausted budgets must produce distinguishable outcomes. Refinement must preserve original physical
acceptance and terminal cancellation/resource/domain failures.

## Recommended design boundary and alternatives

**Proposed:** compose the existing owners into a selective accuracy operation:

1. Resolve shared defaults and explicit overrides, retaining the chosen physical scale, provenance
   and lifecycle.
2. Solve with class-specific native controls and assess original feasibility, domains and closure.
3. Where a goal requires it, assess the output’s numerical error and decision margin using available
   valid evidence.
4. Refine the contributing numerical work only when that evidence overlaps the declared requirement,
   within the enclosing time, memory, work and attempt allowances.
5. Return a resolved decision or an explicit unresolved outcome. Publication consumes the completed
   assessment and never starts refinement.

The first obligation is the semantic contract in CA-F01. CA-F03 concerns its producer demands;
CA-F04 concerns its strategy/completion consumers. CA-F02 can improve ordinary fallback independently.
A new service, crate or universal estimator framework is not implied.

Without declared goals, use the contextual engineering fallback. Bounded output-stability comparisons
may be useful where conditioning or other observations justify them, but every ordinary solve need
not compute an adjoint, full SVD or proof. Stable digits from repeated solves are empirical evidence;
they can preserve the same wrong branch or model error.

| Alternative | Assessment |
|---|---|
| Shared static physical defaults plus overrides | Simplest viable ordinary fallback; cheap and explainable, but insufficient alone near decisions or across large scale variation |
| Magnitude-aware defaults | Useful contextual candidate; requires meaningful magnitudes and floors; does not establish output propagation or decision invariance |
| Sensitivity/goal-aware selective refinement | Best fit for the agreed target; additional derivative/assessment cost only where justified; local validity and evidence strength must remain explicit |
| Universal proof or blanket tighter tolerances | Adds cost without an output-driven rationale; not selected |

### Research applicability and solver-specific meaning

**Interface-checked:** native accuracy controls establish different properties by problem class.
Shared engineering intent should lower through those distinctions, rather than becoming one
universal stopping number.

| Class | Established numerical meaning | Required interpretation |
|---|---|---|
| Square roots | KINSOL tests scaled residuals; a small-step stop can indicate stalling | Preserve residual acceptance and typed stall outcomes; neither establishes forward output accuracy. [KINSOL 7.1.1, §7.2.6](https://sundials.readthedocs.io/en/v7.1.1/kinsol/Mathematics_link.html). |
| Smooth NLP | Ipopt combines scaled NLP error with separate absolute dual infeasibility, constraint violation and complementarity criteria | Preserve independent feasibility/optimality budgets and acceptable termination. These do not certify a global optimum or output-error bound. [Ipopt termination options](https://coin-or.github.io/Ipopt/OPTIONS.html). |
| ODE/DAE | IDA controls estimated local error with absolute-relative state weights | Near-zero states need absolute allowances; endpoint, event-time and cumulative output accuracy require their own assessment. [IDA 7.1.1, §5.2](https://sundials.readthedocs.io/en/v7.1.1/ida/Mathematics_link.html). |
| Dynamic sensitivities/integrals | Diffsol exposes state, sensitivity and output tolerances, plus parameter scales | Use the actual consuming quantity; a state tolerance is not automatically an integral or sensitivity budget. [Diffsol tolerance guidance](https://github.com/martinjrobins/diffsol/blob/main/book/src/specify/tolerances.md). |
| LP/MIP | Feasibility, integrality and incumbent-bound objective gaps are separate properties | Objective-gap accuracy does not guarantee individual variable accuracy or unchanged discrete choices; retain original-objective and bound-source qualification. [HiGHS option definitions](https://github.com/ERGO-Code/HiGHS/blob/master/docs/src/options/definitions.md). |
| Conic | Clarabel requires feasibility criteria and either absolute **or** relative gap convergence; reduced accuracy uses separate tests/statuses | Preserve `AlmostSolved` distinctions rather than merging reduced/full criteria. [Clarabel 0.11.1 source](https://github.com/oxfordcontrol/Clarabel.rs/blob/v0.11.1/src/solver/implementations/default/info.rs). |

**Applicability:** `Cargo.lock` resolves Diffsol 0.16.2, Clarabel 0.11.1, sundials-sys 0.6.2 and
HiGHS Rust 2.4.0/highs-sys 1.15.0. Exact local Diffsol and Clarabel source supports the interface
distinctions above. The captured SUNDIALS baseline is 7.1.1; committed Ipopt bindings identify
3.14.20. These are source/build facts, not verification of currently loaded native libraries.
Current upstream guidance supplies conceptual context; defaults and additional options do not
transfer automatically to an unverified build.

**Proposed architectural inference:** goal-oriented error assessment can prioritize the residual
contributions affecting a requested output. Becker–Rannacher’s publisher abstract describes residual
weighting by approximate adjoint sensitivities in Galerkin finite elements. That motivates selective
output-oriented work here, but its numerical guarantees do not automatically transfer to nonlinear
process equations, implicit suppliers or branch selection. Only the publisher abstract was
inspected; no full-body theorem review is claimed.
[Becker–Rannacher, 2001](https://www.cambridge.org/core/journals/acta-numerica/article/abs/an-optimal-control-approach-to-a-posteriori-error-estimation-in-finite-element-methods/5C67A03F528C6FA69F37A97DF5C3BE19).

JCGM 106 explains why decisions near specification limits need an explicit decision rule and how
guard bands affect acceptance/rejection risks. Its probability statements concern measurement
uncertainty and assumed distributions. Deterministic numerical tolerances are not standard
uncertainties or confidence intervals. Numerical error, input uncertainty and model discrepancy
must retain separate meanings.
[JCGM 106:2012, §§8.2–8.3](https://www.bipm.org/documents/20126/50065304/JCGM_106_2012_E.pdf/fe9537d2-e7d7-e146-5abb-2649c3450b25).

**Preservation constraint:** engineering resolution is separate from assurance and root-selection
semantics. Automatic reconstruction currently requests `Certified` evidence. Coarser output
allowances do not authorize replacing certified selection with estimated selection, omitting
competitive-root exclusions or discarding branch obligations. Any assurance-policy change requires
its own explicit decision. Retained PR proof-deadline evidence identifies proof cost; excessive
engineering output precision is not established as its cause.

Limit goal-assessment overhead to relevant outputs, reuse admitted derivative/factorization products
where lawful, and leave native iteration controllers with their libraries. These are proposed cost
controls, not measured improvements.

Library-owned solvers should retain iteration, forcing terms and globalization. KINSOL explicitly
distinguishes residual success from small-step stalling and already supplies adaptive inexact linear
solves. [SUNDIALS 7.1.1, §§7.2.6 and 7.2.9](https://sundials.readthedocs.io/en/v7.1.1/kinsol/Mathematics_link.html).

Output influence and conditioning explain why backward error alone is insufficient. A local
correction such as `−q_x J⁻¹r` can inform an **estimate**, but a guarantee needs additional region,
remainder, evaluation and branch evidence. Condition estimates must not silently become conservative
upper bounds. [Higham, numerical stability](https://nhigham.com/2020/08/04/what-is-numerical-stability/);
[Higham 1990, §7](https://nhigham.com/wp-content/uploads/2023/08/high90g.pdf).

Avoiding oversolving can reduce unnecessary inner work, subject to the method’s assumptions. This
supports native adaptive controls, not indiscriminate coarse outer accuracy.
[Eisenstat–Walker 1996, §§1–3](https://users.wpi.edu/~walker/Papers/forcing_terms%2CSISC_17%2C1996%2C16-32.pdf).
No performance improvement is measured here.

## Foundations and gates

| Foundation | Verdict and basis |
|---|---|
| AP-01 | **Satisfied** within inspected separation of physics, policy, lowering and completion |
| AP-02 | **Unresolved for target**: consumed output/error contract and producer allocation need settlement; native contracts remain useful |
| AP-03 | **Unresolved for target**: compose goal refinement through existing strategy rather than per-workflow rules |
| AP-04 | **Violated for target**: current coordinate/product model does not adequately govern the agreed engineering decision meaning |
| AP-05 | **Violated for target**: required margins, evidence and boundary-refinement outcomes lack an explicit consumed contract |
| AP-06 | **Satisfied** for existing policy locality; proposed additions should retain independently testable admission and assessment |

| Gate | Judgment |
|---|---|
| G1 | **Pass** for inspected sourced policy and completion ownership |
| G2 | **Fail for target**: required engineering and error meanings remain incomplete |
| G3 | **Unresolved** for proposed goal/evidence validity; existing numerical rejection boundaries are preserved |
| G4 | **Pass** in inspected completion/publication boundary; inspection does not launch refinement |
| G5 | **Unresolved** for the proposed refinement lifecycle and exhausted-budget decision outcome |
| G6 | **Pass** for inspected physical projection/normalization contracts; new contextual reuse requires explicit identity |
| G7 | **Unresolved** for target behavioral claims; current source alone supplies no qualification |
| G8 | **Pass** for inspected library ownership; no replacement solver machinery recommended |
| G9 | **Fail** through AP-04/AP-05, without averaging other strengths |
| PS-G1 | **Unresolved** for changed-policy scientific scope; physical typing/closure separation is preserved |
| PS-G2 | **Not applicable to the proposed accuracy-policy change**: this review proposes no change to variable roles, structural admission or supported differential index. It does not qualify structural well-posedness of the current implementation. |
| PS-G3 | **Unresolved** for goal-derived accuracy and refreshed implementation execution; current truthful status/check contracts remain strengths |

## Verification, limits and disposition

| Claim | Evidence and remaining obligation |
|---|---|
| Existing resolution/lowering/checking paths | **Implemented / Interface-checked**, source inspection; no new runtime receipt |
| Goal-aware contract and selective refinement | **Proposed**; establish S01–S06 without weakening original acceptance |
| Current selected science | Existing bounded receipts only; not enclosing qualification |
| Performance/robustness benefit | **Proposed**, unmeasured; compare total work and output/decision outcomes on representative cases |

Coordinator inspection of retained receipts finds six additional shared-policy fixtures passed,
while the later two PFR fixtures failed after reaching workflow checks. Failed Boolean rows do not
establish physical deviation magnitudes or the sufficiency of a revised allowance. These observations
do not justify loosening checks. The pending PFR conservation migration and native-alternative
changes remain unvalidated. The inspected receipts are
`build/plan25k-20261005/global-policy-additional/seed.fixtures.arrow` and
`build/plan25k-20261005/pfr-pipeline-handoff/seed.fixtures.arrow`, with their adjacent check tables.
The former contains 1,830 check rows (1,823 passed, seven not applicable); the latter contains 7,695
(7,682 passed, ten failed, three not applicable). They are historical selected execution evidence
against the zero-failure baseline, not new tests or qualification of this proposed design.

Production tests should establish implemented premises and declared behavior: physical spacing and
stencil coefficients, assembly/boundaries, supported derivatives/domains, original acceptance, and
output stability at declared resolution. Empirical reproduction of theoretical process
convergence-order ratios is not required. Focused numerical verification may use explicitly
justified tighter budgets.

Blueprint §§16.1/16.2/16.6 and §§13.3/19.2 need the eventual contract through the decision/design
route. “A check never starts a solve” should remain true: refinement belongs to execution, with
completion remaining immutable. No SHOULD exception is proposed.

**Decision: Revise the architecture for the agreed contextual-accuracy target; do not accept its
behavioral claim yet.** Preserve truthful current feasibility support and ordinary no-goal operation.
Settle output/decision meaning first, then its allocation and bounded consumer behavior, with
stronger accuracy driven by analytical relevance rather than project stage. Plan 25k remains the
single disposition owner.
