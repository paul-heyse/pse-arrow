---
name: design-review
description: Develop and assess design corrections from the refreshed audit within the operator-selected library and principle focus. Compare alternatives against the layered standard and functional target, preserving independent judgment and evidence limits.
allowed-tools: Read, Glob, Grep, Bash, Write, Edit, Agent
user-invocable: true
model-baseline: claude-5 (2026-08)
---

# Design review

Assess the architecture's ability to support expected changes and its behavioral contracts.
Use the form requested by the user; a formal review has one accountable principal document in
the binding's review location, with supporting analysis where useful. Advice about the process
does not itself request a product audit or edits.
Apply domain-model assessment during a requested review or one due at the repository's declared
cadence. Ordinary implementation work does not itself initiate a domain-model review.
[REFERENCE.md](REFERENCE.md) supplies optional lenses and calibrated examples.

## Start from the refreshed audit and operator focus

Before a new engagement, refresh the diagnostic account applicable to its subject, including
direct invocation. Use [design-principle-audit](../design-principle-audit/SKILL.md) for broad
production scope and [design-principle-audit-tests](../design-principle-audit-tests/SKILL.md)
for a test-only engagement. Mixed engagements identify the relevant accounts under the binding;
a test-only review does not initiate a broad production audit.
Preparation and review share that baseline while it remains applicable; reconcile intervening
source/standard changes and cite the audit publication revision and finding anchors. The audit
supplies a challengeable diagnosis, not the selected correction. Concentrate review effort on
solutions, alternatives, preservation and opportunities within the operator's chosen focus.

The operator selects holistic/unconstrained, library-targeted, principle-focused or combined
focus. Resolve this from the request/conversation; clarify an unspecified choice rather than
choosing for the operator. Tier and depth remain separate choices. A library hypothesis directs
investigation, not adoption; principle focus directs depth while relevant guarantees constrain
remedies. Record encountered out-of-focus concerns without automatically broadening the assignment.

Loading assessment criteria for plan authoring/execution does not initiate a formal review or
broad audit. A requested [recommendation audit](../design-review-audit/SKILL.md) normally
continues this same engagement and integrates its revised judgment without a second broad audit.

## Find the standard

When supplied, the [library-leverage account](../../../docs/library_leverage/README.md)
offers codebase opportunities, focused capability applicability and comparative recommendations
to challenge. Preserve source `CBO` identities, consumed revisions, concrete mechanisms and
evidence limits. A leverage inquiry does not itself initiate this formal review or satisfy its
diagnostic-audit prerequisite; follow the binding's cadence and use one coherent engagement
where the applicable assessments can be satisfied together.

Locate the repository's `standard.toml` through its agent instructions. Read the selected
core principles, the `[core].heuristics` companion and template, the binding, and each applicable
profile and companion skill. Use the [design principles](../../../docs/design_review/design_principles/core/design-principles.md)
together with [Heuristics for Efficient Architecture](../../../docs/design_review/design_principles/core/efficient-architecture-heuristics.md).
When test code or test evidence is in scope, also load `[core].test_assessment`; that companion
owns the test-specific interpretation without relaxing the core or domain profiles.
If no declaration exists, apply the available core and say so. If the core is missing, report
that limitation. Profiles refine or tighten the core; they never waive a MUST. Follow the
binding's conflict route and state any effect on the verdict.

The core defines seven architectural foundations, operational refinements, independent gates,
evidence levels and the review contract. Its selected version is authoritative; do not infer
current requirements from an older review or duplicate the standard in this skill.

## Grounding

Judge the subject only against the loaded standard (principles, companion heuristics, template and
declared profiles) and the system's functional target, located through the binding. Every other
repository rule describes the current design, not the criteria for judging it: accepted decisions
and ADRs, blueprint binding decisions and product rulings, binding policies, agent instructions
and working agreements, operator decisions recorded in status or memory, global working rules and
dependency pins. Read them to understand the subject. Do not judge the design by its agreement
with them, narrow a recommendation to fit them, or review them for their own sake. Where the
target statement names a chosen mechanism, such as a store, language or framework, that choice is
a rule.

Rules for conducting and reporting the review still apply: its location, the reviewer's
independence, the evidence vocabulary and outcome reporting. Once the judgment is settled, the
review lists the rules its recommendations would change (see Identify rule impacts).

## Reading context

Use the repository's architecture entrypoint and relevant contract/source owners; consult current
work only when it affects the scoped question. Delegated reviewers use the brief and relevant owners
without repeating the root's general startup tour, following additional dependencies as needed. Use Markdown directly. Module/entrypoint pointers bound reading;
a source-proof manifest or exhaustive symbol inventory is not a prerequisite. Mechanical
document checks cannot establish architectural conformance. Ordinary implementation edits
need documentation updates only when an enduring contract, explanation or workflow changes.

## Scope the assessment

Infer `tier` (change/design) and system/subsystem/change boundary from the
request and binding. The `purpose` is always `target`, the best architecture serving the
functional outcome; the tier sets breadth only. The template's `conformance` purpose is not used:
whether an implementation matches its accepted design belongs to implementation review. Clarify only material ambiguity. Include the
suppliers and consumers needed to reason about the boundary; state exclusions. Focus changes
depth, not the obligation to report a material defect found outside the focus.

For design tier, start with the target, architectural drivers and credible variation axes.
Establish the domain phenomena, consequential distinctions and owned operations, then their
responsibility boundaries, hidden decisions, dependency direction and consumed contracts.
Assess whether the model is adequate and governs behavior using relevant contracts and source.
Domain-named output records alone do not establish alignment. Consider representative changes
and material test dependencies. Use the least investigation sufficient for the scoped judgment;
follow a flow only to resolve a concrete uncertainty, without a prescribed tracing sequence.
A change review can compress that reasoning to its affected scenario and foundations.

## Shared review roles

Use the [shared roles](../../../.agents/roles/README.md) for bounded code mapping and library
research when useful. Give a fresh design reviewer the target, requirements and source evidence
for an independent judgment. Inspect decisive evidence yourself; the coordinator owns
integration and finding disposition. Focused design advice for plan creation can be incorporated
in that plan and does not replace a formal review due under the binding.

## What to establish

Use relevant companion heuristics to recognize material execution patterns before committing to
physical organization, interfaces, preparation, assurance and lifecycles. Address material
mismatches while the design remains easy to change. Assess them qualitatively within AP-07/G9;
preserve the existing principles and lenses without an exhaustive checklist, cost models or
new proof machinery.

- **Architecture:** verdicts for the applicable foundations, grounded in responsibilities,
  contracts, composition and change scenarios. Name the context and dependencies needed to
  modify or test a component. AP-04 requires both model adequacy and behavior governed by its
  semantic authorities; a centralized but inadequate definition is insufficient. AP-07 qualitatively assesses
  the complete physical operation: necessary versus repeated work, access paths, crossings, live
  state, reuse, resource/transaction lifetime and failure recovery under relevant growth/skew
  and concurrency. Explain material tradeoffs in plain language. This consideration requires no
  numerical estimates, cost models, estimators, runtime cost accounting, execution-planning machinery,
  instrumentation, formal cost proofs or additional proof artifacts. Those mechanisms need a
  separate concrete functional or operational requirement; existing correctness obligations remain.
  Material unknown premises remain unresolved. Static structural evidence can
  establish unfit execution; quantitative benefits need measurement. G9 follows
  these verdicts without averaging, even when current outputs are correct. Deferral cannot
  waive a domain-model MUST for supported behavior.
- **Behavior:** settle each applicable core/profile gate on its own evidence. Preserve physical
  and numerical requirements when the subject touches them. A missing mechanism is unresolved,
  not a pass. Successful tests do not establish unexamined architectural qualities.
- **Findings:** concrete causes and consequences under the template's finding standard.
  Change amplification, repeated scans/crossings/materialization, excessive live state/queues,
  broad invalidation, mismatched recovery lifetimes, policy duplication, leaked implementation
  knowledge and unnecessary test dependencies are material even if current outputs are correct.
- **Alternatives:** compare the proposed design with the simplest viable and library-owned
  alternatives. Justify seams by their variation axis, including roadmap or exploration needs.
- **Library fit:** assess semantic fit, integration owner, exposed types, lifecycle, testing,
  upgrade/replacement cost and total machinery removed or introduced. Compare interacting native
  capabilities and optimizer visibility, locality, preparation/reuse and failure scope; a
  neutral wrapper must not discard consumed bulk or planning opportunities. Full capability eligibility remains;
  integration cost is assessed independently of whether a consumer already exists.

  Use optional [library overviews](../../../docs/libraries/README.md) for capability awareness,
  then Context7 and primary sources for consequential details. The selected
  native solver skill remains active. Capability maps retain their evidence scope; old
  utilization reports are historical references, not a live catalog or required lookup.

  **Preserve enabling mechanisms.** In any review focus, when a recommendation materially
  depends on library functionality, name the concrete capabilities and recognizable API names,
  explain how they compose to realize the intended responsibility or guarantee, and preserve why
  that choice matters. Include material conditions and limitations,
  evidence strength, and precise references to the relevant overview section, primary
  documentation or tagged source. Overviews orient discovery; qualify consequential contracts
  through the repository's research routes. Distinguish a selected mechanism from a candidate,
  illustrative example or unresolved feasibility question. Keep this account with the
  recommendation so planning can carry it forward without repeating broad capability discovery.
  Scale detail to the dependency; ordinary API usage needs no inventory or fixed format.

- **Decision and follow-up:** distinguish architectural fitness from behavioral adequacy, apply
  the template's decision rules, and link findings to one disposition owner. Do not implement
  recommendations merely because a review identified them.

## Calibrate judgment

Reason from the source, contracts and designs actually inspected. Use a probe only where
material doubt remains. Label proposed benefits and source-traced paths honestly; Tested and
Measured name executed checks and their conditions. Secondhand evidence is a lead until read.

Attack the relevant guarantee: an ordinary extension propagating into unrelated owners; an
implementation detail leaking into consumers; a pure responsibility requiring unrelated
infrastructure to test; two authorities diverging; a rewrite, cache or retry changing meaning.
For execution fit, challenge growth/skew, contention or interruption where material; reason
about examined effort separately from result size and preserve mandatory exactness/partiality
contracts. A bounded refusal is honest but insufficient if ordinary supported work is infeasible.
The reference and domain skills give conditional mechanisms to investigate. Do not force every
review into a cache, graph, registry or publication analysis.

Classify changes as instances, bindings, compositions, policies, domain concepts or mechanisms.
In a substantial architecture review, consider a relevant domain extension and mechanism
substitution where credible; give a scope reason when either does not apply. A bounded review
keeps its relevant scenario. Ordinary domain functions can supply operation contracts; a registry
or serializable instruction model is not required. Multiple files or a substantial module do
not establish entanglement. A specification and its implementation serve different
roles; reconcile disagreement instead of treating their coexistence as duplication. Independent
oracles may deliberately use a different representation.

Assess workloads required by the stated target even when the current plan excludes them; record
an obstructing rule as a rule impact (see Identify rule impacts). Do not silently expand
a bounded requested scope into whole-system qualification.

## Synthesize the assessment

Make the review a coherent argument about the system and the decision it should inform. Explain
the functional intent, the consequential responsibilities and design choices, what supports the
intended capabilities, and where the design falls short. Connect evidence to causes, consequences
and recommendations so the reader can understand the judgment without reconstructing it across
tables. Several independent concerns may remain; do not force them into one root cause or redesign.

Where findings interact, distinguish shared causes, prerequisite contracts, independent defects,
alternative remedies and conflicting recommendations. Group manifestations of one cause while
preserving distinct obligations needed for closure. Sharing a principle ID or component does not
make two findings the same defect. Assess whether proposed corrections fit together: individually
reasonable remedies can create duplicate owners, incompatible meanings or new consumer burdens.
Explain the relevant relationships in prose or a small diagram when useful; no finding graph or
whole-system remedy assessment is required for a bounded review.

In delegated work, the coordinator reconciles shared assumptions, overlapping findings, boundary
gaps and incompatible recommendations. Preserve the source and scope of each judgment; resolve
disagreements through decisive evidence or retain their effect on the decision. Local positive
assessments do not establish the adequacy of their interactions. Investigation assignments need
not become the published document structure.

## Make recommendations useful to subsequent design

For consequential remedies, describe the intended behavior and responsibility boundary: what
the corrected operation consumes, decides and produces, and what its consumers can stop
interpreting independently. State preserved guarantees, intentional behavior changes and support
limits where they constrain the correction. Inspected strengths can be preservation constraints,
such as a useful lifecycle separation, an independent check or an existing semantic distinction.
A narrow defect may need only a sentence; conceptual products do not mandate new types or services.

Distinguish the evidence establishing a diagnosis from the maturity of its remedy. A sound
finding remains reportable when the best replacement needs further design. Explain the proposed
direction, viable alternatives and material assumptions, and what decision or evidence would
settle or reopen the recommendation. A revised remedy need not invalidate the original diagnosis.
Apply the standard's evidence vocabulary to these distinct claims rather than adding confidence
scores or a second status system.

Challenge a consequential remedy with a revealing legitimate case: could it reject valid work,
erase a distinction, weaken a guarantee or transfer policy to the wrong owner? Reasoning may
settle the question; probes remain discretionary. Relate claimed benefits to the mechanism and
conditions that would produce them, including material integration and ownership costs.

Separate consequence-based priority from prerequisite order. A lower-priority contract correction
can enable a more urgent consumer fix. Explain the dependency through the capability or meaning
required, not just finding numbers. Consider preservation, external consumers and intermediate
states when transition feasibility affects a recommendation. Detailed work packages, staffing,
schedules and migration procedures belong to subsequent planning unless explicitly requested.

Connect uncertainty to the decision it affects. Distinguish unexamined breadth from a missing
premise that could change the verdict or selected remedy; name suitable settling evidence where
material. An unexamined area is not itself a defect. Keep the next consequential decision and
follow-up obligation clear without inventing implementation work merely to make the review actionable.

## Identify rule impacts

After the judgment and recommendations are settled, compare the recommended design with the
current rules named under Grounding. List each rule it would change, replace or retire, including
a rule it makes unnecessary, in the template's slot 11. Give each item a stable identifier
(`review#RCnn`), the rule and where it is recorded, the proposed change, the findings or
recommendations that depend on it, and what the recommendation becomes if the rule is kept.
State "none" when nothing is affected. The review does not edit, supersede or route these rules:
plan creation presents them to the operator, and a change takes effect only once confirmed there.

## Organize the output for its readers

The investigation structure, system decomposition and published argument need not coincide.
Group detailed discussion by coherent responsibility, question or architectural cause. Tables
support comparison and navigation; use prose for explanations that lose meaning in wide cells.
A compact finding index can link to richer arguments with stable finding IDs.

Start with enough synthesis to explain the conclusion and its scope, then develop the supporting
arguments at the depth needed. For a substantial topic, supporting documents can hold bounded
analysis or evidence. Split when the reasoning is useful to consult independently; keep tightly
coupled explanations together when separation would make readers repeatedly reconstruct them.
Length, one file per component or one document per reviewer is not sufficient reason to split.

The principal review owns the combined scope, architectural explanation, material relationships,
overall judgment and evidence limits. Supporting documents identify their narrower role, baseline
and consumed context, with a clear route back to that review. They may carry bounded judgments
but do not create competing overall verdicts or disposition ledgers. Small reviews can do this
in one compact document.

Use the selected template's slots as stable content references, with relevant profile additions
and proportional detail. Group or distribute the explanations where this improves understanding,
keeping applicable foundation, gate and decision judgments discoverable. Preserve the selected
standard's assessment obligations and acceptance rules; flexible presentation does not waive them.
The template guides placement and the [reference](REFERENCE.md#synthesis-and-corrective-reasoning)
offers calibrated examples. These are authoring considerations, not additional mandatory review stages.

## Deliver the review

A read-only delegated reviewer returns the complete review text and intended path to the
coordinator, who publishes it while preserving the reviewer’s judgment.
Keep finding IDs linkable and record standard version, inspected scope and disposition owner.
State architectural fitness, behavioral adequacy, overall decision and evidence limits. A target
accepted as a design remains unqualified implementation until supported by execution evidence.
Close with the decision, material findings, rule impacts, coverage and artifact path. If nothing
is wrong, say so with the coverage limits; do not manufacture findings or pilot results.

Requested recommendation audits use the companion and attributed integrated-revision convention
in [design-review-audit](../design-review-audit/SKILL.md). Preserve the exact input in Git before
replacement. The revised principal explains the resulting design; the companion owns judgments
of the first round. Editorial dispositions do not change implementation status.

## Failure modes

- Fitting a recommendation to an existing ADR, ruling or policy instead of the standard and the
  functional target.
- Auditing the rule corpus instead of the design, or treating a rule impact as applied before the
  operator confirms it.
