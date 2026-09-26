# Design review template

**Version 3.0 · 2026-09-25** · Core layer: repository- and domain-agnostic.
The [principles](design-principles.md) define foundations AP-01–AP-06, refinements DP-nn
and gates G1–G9. Profile additions follow this template's versioned slots.

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
| Consequence | A trigger leads to incorrect behavior, amplified change, leaked knowledge, repeated policy, unnecessary coordination or inability to test/reason locally. |
| Correction | A direction, expected responsibility boundary and approximate affected surface. |
| Verification | How to distinguish a landed correction from renamed or relocated complexity; reasoning may suffice. |

An architectural finding can be material while all current functional tests pass. Conversely,
a large module or many changed files alone is not a finding. Show the responsibility crossed
and why that crossing is unnecessary for the scenario. Evidence labels apply to the actual
claim; an extension traced in source has not been executed merely because it is plausible.

Priority follows consequences: correctness/authority violations first; then architectural
change cost, testability and library ownership; then measured cost. Do not discard a coupling
finding because it has no known wrong numerical output. Compare the least complex viable
alternative and assess over-construction from any integration, including library adoption.

### Decision rules

Settle **behavioral/semantic adequacy** (G1–G8 and applicable domain gates) and
**architectural fitness** (G9, supported by the six foundation verdicts) separately.
Neither substitutes for the other. G8 retains its established library-use meaning.

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
non-goals. Explain what was inspected, asserted, not examined or unavailable. Identify likely
variation axes and constraints before naming implementation mechanisms.

### 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State/effect owner | Local test setup |
|---|---|---|---|---|---|

Show the consequential dependencies, with a small diagram if helpful. Explain the composition
root and which decisions may vary independently. Distinguish an intentional shared semantic
contract from an incidental backend type. Name the context needed for a safe local change;
module/file count is not a proxy. Derive observed dependencies from source/manifests rather
than creating a competing hand-maintained dependency graph.

### 3. Contracts, authority and constraints

| Meaning / contract | Authoritative owner and update path | Consumer obligations / invariant | Enforcement and failure | Derived representations / evolution |
|---|---|---|---|---|

Include identities, absence/outcome states and equivalence only where interpretation depends
on them. Explain substitution for the capabilities actually consumed, including unsupported
work and effects. Identify intended specification versus observed implementation explicitly.

### 4. Change scenarios and composition

Give scenarios stable local IDs (`S01`, `S02`, …) so findings and work can cite them. Reference
an existing scenario definition instead of re-authoring it when its meaning is unchanged.

| Scenario / stimulus and conditions | Expected response and change boundary | Edit/composition path | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|

Choose the small set that distinguishes the design alternatives. Useful scenarios include an
ordinary extension, implementation/library replacement, new workflow, representation change,
independent test and meaningful contract evolution. For each, identify genuinely new meaning,
reused primitives, repeated decisions, affected owners and needed context. Add failure/recovery
journeys when lifecycle is material. No mandatory number of scenarios or numeric change quota.

### 5. Mechanisms and execution, where material

Deepen only the mechanisms needed to settle the architecture or its correctness obligations.

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|

For computations, include structural/value inputs, termination, precision and determinism
where relevant. For boundaries, include ownership, representation, batching and loss. A
noncomputational subsystem need not invent cache, graph or publication requirements.

### 6. Architectural assessment and gates

| Foundation | Scenario and evidence / scope reason | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | | | |
| AP-02 Stable contracts | | | |
| AP-03 Composition | | | |
| AP-04 Authoritative meaning | | | |
| AP-05 Explicit structure | | | |
| AP-06 Local reasoning/testability | | | |

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
passing code tests do not establish unexamined architecture.

### 7. Findings

| ID | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | | | | | | |

Record applicable refinements and strengths that affect the argument. Separate current defects
from proposed improvements with no demonstrated defect. Status belongs to the disposition
owner in slot 11, not to a second independently edited copy in this review.

### 8. Library fit and ownership cost

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade/replacement cost | Bespoke code removed / recommendation |
|---|---|---|---|---|---|

Apply principles §F to material choices. Full library eligibility remains. A prospective
capability may be explored before a consumer exists; assess any integration machinery and
its architectural role. Do not force a wrapper, feature restriction or dependency upgrade.

### 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | | | | | |
| Proposed design | | | | | |
| Library-owned alternative | | | | | |
| Simplest viable alternative | | | | | |

Rows may coincide; say so. Explain why a seam is justified by a credible variation axis and
why another abstraction would not help. Report performance claims at their evidence strength.

### 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|

Use the cheapest reliable evidence. Distinguish source tracing, proposed changes and executed
experiments. Preserve independent oracles. Dependency checks and contract tests support named
properties; they never synthesize architectural acceptance. Follow the repository's rhythm for
implementation checks and final qualification.

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

A successful review makes the next representative change understandable and identifies what
would falsify the design's claims. It need not add a framework, artifact or test where none
improves the decision.
