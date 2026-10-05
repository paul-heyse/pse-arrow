# Design review template

**Version 3.4 · 2026-10-05** · Core layer: repository- and domain-agnostic.
The [principles](design-principles.md) define foundations AP-01–AP-07, refinements DP-nn
and gates G1–G9. Profile additions follow this template's versioned slots.

Use the [principles](design-principles.md) together with
[Heuristics for Efficient Architecture](efficient-architecture-heuristics.md) for consequential
choices. The companion supplies conditional execution patterns for existing AP-07/G9; retain
the current principles, twelve lenses and overlapping guidance.

Output and synthesis guidance revised 2026-10-01; assessment rules and slot identifiers unchanged.

## Part 1 — The review contract

### Tier and purpose

| Choice | Meaning |
|---|---|
| Change tier | A bounded change within an accepted design. Compress to slots 1, 4, 6, 7, 12; other slots only where material. |
| Design tier | A system/subsystem design, boundary change or new mechanism. Use the slots below with detail proportional to uncertainty. |
| Conformance purpose | Assess against current authorities; identify conflicts without silently overriding them. |
| Target purpose | Assess the best design for the functional target; route obstructing authority changes through slot 11. |

State the system/subsystem/change boundary separately from tier and purpose. Include the
suppliers and consumers needed to understand that boundary. A narrow code sample cannot
establish whole-system architecture. Drop irrelevant detail with a scope reason. A change
review does not require a whole-system census; it does address the affected change scenario.
Use the least investigation sufficient for the scoped judgments. Flow tracing is optional when
it resolves a concrete uncertainty; neither these slots nor AP-04/G9 require a complete flow trace.

### Evidence and acceptance

A document review establishes a proposed architecture and its reasoning, not runtime behavior.
An inspected interface is Interface-checked. Code demonstrates the path that exists; only
named executed tests/measurements support Tested/Measured claims. State current implementation
separately from the proposed correction, and record which guarantees were not examined.

Specifications express intended contracts; implementations realize them. Disagreement is a
finding or a stated pending change. Their coexistence is not itself duplicated authority.
Historical reviews retain their standard version, scope and observations.

Each applicable foundation has a verdict: **satisfied**, **violated** or **unresolved**.
Identify the scenario and mechanism, the concrete consequence, or the settling uncertainty.
Use **not applicable** only with a scope reason. Applicable DP/profile requirements may be
grouped by the argument they support; listing every rule is not evidence.

### Finding standard

| Field | Requirement |
|---|---|
| ID | Stable within the review; give the finding a linkable anchor, such as `f01`. |
| Finding | One falsifiable structural cause, grouping its concrete instances. |
| Principles / gates / scenario | Only identifiers that support the argument. |
| Evidence or gap | Read source/interface expression or document section; identify uncertainty honestly. |
| Consequence | A trigger leads to incorrect behavior, amplified change, leaked knowledge, repeated policy, unnecessary coordination or inability to test/reason locally, avoidable scans/crossings/materialization, excessive live state or queues, broad invalidation, or effect-mismatched resource/recovery lifetimes. |
| Correction | A direction, expected responsibility boundary and approximate affected surface. |
| Verification | How to distinguish a landed correction from renamed or relocated complexity; reasoning may suffice. |

An architectural finding can be material while all current functional tests pass. Conversely,
a large module or many changed files alone is not a finding. Show the responsibility crossed
and why that crossing is unnecessary for the scenario. Evidence labels apply to the actual
claim; an extension traced in source has not been executed merely because it is plausible.

Prioritize correctness/fidelity failures and choices that make supported workloads infeasible,
unstable or operationally disproportionate. Qualitative reasoning about operations and growth or
failure scenarios may establish an execution-fit defect. Assess change barriers, repeated semantic
ownership, testability and other
material complexity in the same functional context; priority follows consequence, not whether
evidence is a benchmark. Compare the simplest viable conforming realization, including native
composition, and explain the concrete benefit retained by a more expensive choice.

### Decision rules

Settle **behavioral/semantic adequacy** (G1–G8 and applicable domain gates) and
**architectural fitness** (G9, supported by the seven foundation verdicts) separately.
Neither substitutes for the other. G8 retains its established library-use meaning.
Core §1's domain-model MUST is assessed through AP-04 and G9: both model adequacy and
authoritative realization must hold. Domain-named output records and correct current outputs
cannot compensate for that gap; deferral does not waive it for supported behavior.

| Situation in the declared scope | Decision |
|---|---|
| No MUST gap; all applicable gates pass; all applicable foundations satisfied | Accept at the stated document/implementation evidence level |
| Only SHOULD deviations with concrete exception records | Accept scoped, identifying deviations |
| Failed gate or violated MUST, including G9 | Revise; a materially different supported scope may be assessed explicitly |
| Material unresolved gate or foundation | Not Accept; name the missing decision or evidence |
| Competing authority, silent semantic loss or unbacked core capability | Revise or Reject according to the consequence |

Scope cannot be narrowed only in the verdict while the same broad capability remains claimed.
A legitimate new core concept may require coordinated edits; judge the declared variation
axis and architectural necessity. Tradeoffs do not waive a MUST. Acceptance of a target design
never closes its implementation work or certifies the whole product.

### Organize the review around its argument

The slots below identify content and stable references for profile additions. They are not a
required investigation sequence or a demand for one heading and table per slot. Group or
distribute explanations to suit the subject, while keeping the applicable assessment obligations,
foundation and gate judgments, decision and evidence limits discoverable. If a slot's content
appears elsewhere, link it where readers or profile references need a route. Presentation choices
do not change the selected standard's acceptance rules.

Lead with a concise integrated assessment: what the system intends to accomplish, which
responsibilities and choices shape its behavior, what supports the target, and which material
causes explain the problems. Develop the argument through coherent topics and concrete findings.
Several independent concerns may remain; no single root cause or overarching redesign is required.
Use tables for comparisons and navigation, and prose where causal reasoning needs explanation.
A compact finding index can link to detailed arguments instead of compressing them into wide cells.

A formal review has one accountable principal document. Substantial bounded analysis or evidence
may live in supporting documents when independently useful. The principal review owns the combined
scope, architectural explanation, relationships, judgment and evidence limits. Supporting material
names its narrower role, inspected baseline and relevant context and links back to the principal
review. Local judgments do not establish unexamined interactions or create separate overall
decisions. Current finding status remains at the binding-designated disposition owner.

Document boundaries should serve reader understanding. Keep tightly coupled reasoning together;
length, subsystem count and reviewer assignments do not dictate a split. Small reviews can keep
their argument compact in one document. Supporting analysis is optional and does not require a
new evidence folder, artifact hierarchy or registration process.

## Part 2 — Review slots

### 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | Documents/code; system, subsystem or change; relevant neighbors |
| Standard | Core/profile versions and binding |
| Tier / purpose | Change or design; conformance or target |
| Reviewer / date | Accountable reviewer; distinguish author review from independent review |
| Decisions | Behavioral adequacy; architectural fitness; overall decision, from slot 12 |
| Disposition owner | Plan/packet or other binding-designated owner of follow-up status |

State the functional target, qualities driving this change, baseline, supported scope and
non-goals. Include representative operations, relevant size/skew/growth, concurrency/deployment
constraints and resource envelope where material. Assess execution fit qualitatively and explain
material tradeoffs in plain language. This consideration requires no numerical estimates, cost
model, estimator, runtime cost accounting, execution-planning machinery, instrumentation, formal
cost proof or additional proof artifact; such mechanisms need a separate concrete functional or
operational requirement. Quantitative performance/capacity claims still require measurements,
and existing semantic correctness obligations remain. No invented capacity SLA is required.
Explain what was inspected, asserted, not examined or unavailable. Identify likely
variation axes and constraints before naming implementation mechanisms.

### 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State/effect owner | Local test setup |
|---|---|---|---|---|---|

Show the consequential dependencies, with a small diagram if helpful. Explain the composition
root and which decisions may vary independently. Distinguish an intentional shared semantic
contract from an incidental backend type. Name the context needed for a safe local change;
module/file count is not a proxy. Derive observed dependencies from source/manifests rather
than creating a competing hand-maintained dependency graph.
Assess whether the model captures consequential distinctions and behavior uses its definitions
within the review scope. Use relevant contracts and source; choose further investigation
according to what remains uncertain.

### 3. Contracts, authority and constraints

| Phenomenon, concept or operation / contract | Semantic scope, authoritative owner and update path | Consumer obligations / invariant | Enforcement and failure | Implementation, derived representations / evolution |
|---|---|---|---|---|

Include identities, absence/outcome states and equivalence only where interpretation depends
on them. Explain substitution for the capabilities actually consumed, including unsupported
work and effects. Identify intended specification versus observed implementation explicitly.
Assess verbs alongside nouns: applicability, inputs, outcomes, state changes, effects and the
invariants owned by concepts, relationships and operations. Multiple enforcement points may
share one invariant definition, but each extra check addresses a distinct failure, trust transition
or recovery benefit. Established immutable validity is reused while its premises hold. Distinguish
semantic scope/completeness, examined effort and output limits. Ordinary domain functions can
supply these contracts; semantic ownership does not prescribe physical layout or placement.

### 4. Change scenarios and composition

Give scenarios stable local IDs (`S01`, `S02`, …) so findings and work can cite them. Reference
an existing scenario definition instead of re-authoring it when its meaning is unchanged.

| Scenario / stimulus, kind of change and conditions | Expected response and change boundary | Edit/composition path | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|

Choose the small set that distinguishes the design alternatives. Useful scenarios include an
ordinary extension, implementation/library replacement, new workflow, representation change,
independent test and meaningful contract evolution. For each, identify genuinely new meaning,
reused primitives, repeated decisions, affected owners and needed context. Add failure/recovery
journeys when lifecycle is material. No mandatory number of scenarios or numeric change quota.
Where material, include growth, skew/high-degree, concurrency and interruption scenarios beside
extension scenarios. Ask what scales with necessary input versus incidental declarations or
boundaries, what stable work survives change, and which work retries repeat. Global work can
be required even for small output; a bounded refusal cannot establish supported-scale fitness.
Distinguish instances, bindings, compositions, policies, domain concepts and mechanisms; explain
why edits belong to their semantic owners. In a substantial architecture review, consider both
a relevant domain extension and mechanism substitution where credible, with a scope reason
when either does not apply. A bounded review keeps its relevant scenario. Locality does not
promise inexpensive replacement when the substitute has incompatible capabilities.

### 5. Mechanisms and execution, where material

Deepen only the mechanisms needed to settle the architecture or its correctness obligations.

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|

For computations, include structural/value inputs, termination, precision and determinism
where relevant. For boundaries, include ownership, representation, batching and loss. A
noncomputational subsystem need not invent cache, graph or publication requirements.
For a material complete operation, assess access paths, fan-out, passes, sorting/hashing, serial
crossings, movement/re-encoding, live copies/intermediates, and resource/transaction lifetimes.
Explain justified materialization, spill/backpressure, preparation reuse and late hydration,
including cancellation/drain and retry scope where relevant. Semantic/module boundaries need
not become I/O or materialization boundaries. Local passing slices do not establish assembled
fitness; interactions may multiply scans, pools or retained state.

### 6. Architectural assessment and gates

Consider relevant [execution heuristics](efficient-architecture-heuristics.md) within the
scoped assessment, addressing material mismatches before physical organization, interfaces,
preparation, assurance and lifecycles become hard to change. Use qualitative reasoning without
an exhaustive checklist, cost models or new proof machinery; do not restart settled reviews.

| Foundation | Scenario and evidence / scope reason | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | | | |
| AP-02 Stable contracts | | | |
| AP-03 Composition | | | |
| AP-04 Domain model and semantic authority | | | |
| AP-05 Explicit structure | | | |
| AP-06 Local reasoning/testability | | | |
| AP-07 Execution fits workload | | | |

| Gate | Pass / fail / unresolved / not applicable | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | | | |
| G2 Semantic fidelity | | | |
| G3 Validity | | | |
| G4 Hidden behavior | | | |
| G5 Consistency and recovery | | | |
| G6 Transformation and reuse | | | |
| G7 Truthful capability claims | | | |
| G8 Library leverage | | | |
| G9 Architectural fitness | | | |
| Applicable profile gates | | | |

G9 follows the individual foundation verdicts, without averaging. For a change review, omit
unaffected foundation rows with a scope explanation. Gate judgments must identify evidence;
passing code tests do not establish unexamined architecture. AP-07 qualitatively assesses execution
fit through G9, without another gate. Material unknown workload premises remain unresolved;
known work amplification cannot be excused merely as unmeasured performance.

### 7. Findings

| ID | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | | | | | | |

Record applicable refinements and strengths that affect the argument. Separate current defects
from proposed improvements with no demonstrated defect. Status belongs to the disposition
owner in slot 11, not to a second independently edited copy in this review.

For consequential corrections, explain the intended operation and consumer behavior: inputs,
owned decisions, outputs or effects, preserved guarantees and intentional differences. Existing
strengths can constrain the remedy. Explain enough to distinguish a correction from relocated
complexity, while leaving routine implementation choices open. A narrow finding may need only
one sentence of correction.

Keep diagnosis and remedy maturity distinct. State what the evidence establishes, which direction
is proposed and what unresolved requirement or assumption affects that recommendation. A sound
finding remains actionable without a fully designed replacement. Challenge a consequential remedy
with a legitimate case it might wrongly reject, flatten or mishandle; reasoning may suffice.

Synthesize related findings through their causes and obligations. Identify shared causes,
independent defects, prerequisite contracts, alternative remedies and conflicting recommendations
where material. Group concrete manifestations without merging distinct closure obligations.
Explain whether the corrections fit together; neither shared terminology nor the same component
establishes a common cause. No mandatory dependency graph or additional analysis stage is implied.

### 8. Library fit and ownership cost

Relevant entries in docs/library-utilization.jsonl, where available, may offer particularly
useful leads on established capabilities and integration patterns. Consulting them is optional.

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade/replacement cost | Bespoke code removed / recommendation |
|---|---|---|---|---|---|

Apply principles §F to material choices. Full library eligibility remains. A prospective
capability may be explored before a consumer exists; assess any integration machinery and
its architectural role. Compare composed capabilities, optimizer visibility, physical access,
data locality/movement, intermediate state, lifecycle and failure/recovery behavior. Preserve
consumed semantics rather than forcing a wrong built-in or a neutral interface that discards
needed native operations. Do not force a wrapper, feature restriction or dependency upgrade.

### 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / qualitative operational tradeoffs | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | | | | | |
| Proposed design | | | | | |
| Library-owned alternative | | | | | |
| Simplest viable alternative | | | | | |

Compare complete preparation, execution, publication, recovery and change, including generated
and library-induced machinery. Separate necessary work from incidental scans/crossings; state
the retained benefit when choosing more work or obligations. Qualitative reasoning can settle
execution fit; numerical performance/capacity claims require measurements.

Rows may coincide; say so. Explain why a seam is justified by a credible variation axis and
why another abstraction would not help. Report performance claims at their evidence strength.

Relate expected benefits to their mechanism, relevant conditions and material ownership costs.
Explain which premise would change the recommendation and how it could be settled. Consider
external consumers, preserved meaning and intermediate states when transition feasibility affects
the choice. Detailed migration procedures and execution packages belong to subsequent planning
unless explicitly requested.

### 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|

Use the cheapest reliable evidence. Distinguish source tracing, proposed changes and executed
experiments. Preserve independent oracles. Dependency checks and contract tests support named
properties; they never synthesize architectural acceptance. Follow the repository's rhythm for
implementation checks and final qualification.

Distinguish unexamined breadth from a missing premise that could change the verdict or corrective
direction. Connect consequential uncertainty to the decision it affects and suitable settling
evidence. An unexamined area is not itself a defect. Evidence for a diagnosis does not automatically
qualify its remedy, and a refined remedy need not invalidate the diagnosis.

### 11. Authority changes, exceptions and disposition

Link blocking authority text, the required change and its route. SHOULD deviations use
principles §H; MUST gaps remain explicit.

Each actionable finding links to one disposition owner selected by the binding. Record there:
`finding reference | scenario reference | disposition | decision/work owner | evidence or
revisit trigger`. Suggested dispositions are open, scheduled, resolved, deferred, superseded
and disproved. Resolved requires evidence of the correction; disproved requires the evidence
that defeats the finding; deferred requires an accountable owner and observable trigger.
Scheduling or accepting an ADR does not establish implementation. A supersession names its
successor. Keep historical observations intact and link current status instead of copying it.

### 12. Decision

State behavioral/semantic adequacy, architectural fitness, then the overall decision under
Part 1. Name the strongest evidence, current uncertainty and the exact scope accepted.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|

Separate consequence-based priority from prerequisite order: a less urgent contract correction
may enable a more urgent consumer fix. Explain the capability or meaning required by a dependency.
Identify the next consequential decision and follow-up obligations without turning the review
into an implementation schedule. In delegated reviews, reconcile shared assumptions, boundary
gaps and conflicting recommendations before stating the combined judgment; preserve material
disagreement and its effect on the decision when the evidence does not settle it.

A successful review makes the next representative change understandable and identifies what
would falsify the design's claims. It need not add a framework, artifact or test where none
improves the decision.
