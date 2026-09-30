---
name: design-review
description: Review system architecture, a proposed design or an existing code scope for change locality, contracts, composition, explicit domain models and semantic authority, constraints and local reasoning. Apply the repository's layered standard, including scientific profiles where relevant. Use for architecture/design reviews, modularity or testability assessments, and library-integration tradeoffs; ordinary implementation does not itself request a review.
allowed-tools: Read, Glob, Grep, Bash, Write, Edit, Agent
user-invocable: true
model-baseline: claude-5 (2026-08)
---

# Design review

Assess the architecture's ability to support expected changes and its behavioral contracts.
Use the form requested by the user; a formal review produces one artifact in the binding's
review location. Advice about the process does not itself request a product audit or edits.
Apply domain-model assessment during a requested review or one due at the repository's declared
cadence. Ordinary implementation work does not itself initiate a domain-model review.
[REFERENCE.md](REFERENCE.md) supplies optional lenses and calibrated examples.

## Find the standard

Locate the repository's `standard.toml` through its agent instructions. Read the selected
core principles and template, the binding, and each applicable profile and companion skill.
If no declaration exists, apply the available core and say so. If the core is missing, report
that limitation. Profiles refine or tighten the core; they never waive a MUST. Follow the
binding's conflict route and state any effect on the verdict.

The core defines six architectural foundations, operational refinements, independent gates,
evidence levels and the review contract. Its selected version is authoritative; do not infer
current requirements from an older review or duplicate the standard in this skill.

## Reading context

Start with the repository's architecture and current-work entrypoints, then the affected
contract and source owners. Use Markdown directly. Module/entrypoint pointers bound reading;
a source-proof manifest or exhaustive symbol inventory is not a prerequisite. Mechanical
document checks cannot establish architectural conformance. Ordinary implementation edits
need documentation updates only when an enduring contract, explanation or workflow changes.

## Scope the assessment

Infer `tier` (change/design), `purpose` (conformance/target), system/subsystem/change boundary
and optional focus from the request and binding. Clarify only material ambiguity. Include the
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

- **Architecture:** verdicts for the applicable foundations, grounded in responsibilities,
  contracts, composition and change scenarios. Name the context and dependencies needed to
  modify or test a component. AP-04 requires both model adequacy and behavior governed by its
  semantic authorities; a centralized but inadequate definition is insufficient. G9 follows
  these verdicts without averaging, even when current outputs are correct. Deferral cannot
  waive a domain-model MUST for supported behavior.
- **Behavior:** settle each applicable core/profile gate on its own evidence. Preserve physical
  and numerical requirements when the subject touches them. A missing mechanism is unresolved,
  not a pass. Successful tests do not establish unexamined architectural qualities.
- **Findings:** concrete causes and consequences under the template's finding standard.
  Change amplification, policy duplication, leaked implementation knowledge and unnecessary
  test dependencies are material even if current outputs are correct.
- **Alternatives:** compare the proposed design with the simplest viable and library-owned
  alternatives. Justify seams by their variation axis, including roadmap or exploration needs.
- **Library fit:** assess semantic fit, integration owner, exposed types, lifecycle, testing,
  upgrade/replacement cost and bespoke machinery removed. Full capability eligibility remains;
  integration cost is assessed independently of whether a consumer already exists.

  **Optional context: existing library use.** A focused look at relevant entries in
  docs/library-utilization.jsonl, where available, can be especially helpful when considering
  implementation alternatives. The catalog highlights library capabilities and integration
  patterns already found useful in the codebase, and may reveal opportunities to reuse
  established mechanisms instead of introducing ad hoc equivalents. It remains useful as a
  source of ideas even when some entries lag the implementation. Consultation is optional;
  catalog entries inform the alternatives, while alignment with the design principles governs
  the judgment.

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

In target purpose, assess workloads required by the stated target even when the current plan
excludes them; record obstructing authority changes through the binding. Do not silently expand
a bounded requested scope into whole-system qualification.

## Output

Follow the selected template and relevant profile additions, with proportional detail.
A read-only delegated reviewer returns the complete review text and intended path to the
coordinator, who publishes it while preserving the reviewer’s judgment.
Keep finding IDs linkable and record standard version, inspected scope and disposition owner.
State architectural fitness, behavioral adequacy, overall decision and evidence limits. A target
accepted as a design remains unqualified implementation until supported by execution evidence.
Close with the decision, material findings, coverage and artifact path. If nothing is wrong,
say so with the coverage limits; do not manufacture findings or pilot results.
