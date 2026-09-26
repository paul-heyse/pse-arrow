---
id: ADR-0094
title: Center design review on architecture and change scenarios
status: proposed
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, DP-22, DP-23]
blueprint: [§24.4]
standard: Core 3.0; process-simulator 1.1
scenarios: [docs/design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#s01, docs/design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#s03]
review: docs/design_review/reviews/design_review_architecture-first-standard_2026-09-25.md#12-decision
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A representative extension or subsystem review cannot distinguish architectural change cost from behavioral correctness.
verification: Plan 17 document consistency review, just adr-lint, just lint-agents and just docs; effectiveness pilots are maintainer-owned follow-up.
---

# ADR-0094: Center design review on architecture and change scenarios

## Context

The maintainer approved reorganizing design review around separation of concerns,
stable contracts, composition, authoritative meaning, explicit constraints and local
reasoning. Core 2.0 gives more operational weight to semantic correctness and library
use than to decomposition, testability and the cost of architectural change.

## Scope

Amend blueprint §24.4 and the review standard, skills, roles and decision/plan guidance.
This is a governance decision, not product architecture qualification. Implementation
is authorized by the maintainer's request; formal decision-PR acceptance remains pending.
Historical reviews and accepted ADR bodies retain their original meaning. This extends
the layered standard of ADR-0085 without replacing its scientific or evidence obligations.

## Drivers

Localize expected change, make contracts and ownership discoverable, support independent
testing, and extend through composition. Preserve scientific correctness, full library
eligibility, proportional evidence and the existing final qualification rhythm.

## Options

Keep Core 2.0 unchanged: leaves architecture subordinate to mechanism checks.
Add a separate architecture scoring framework: duplicates authority and encourages ceremony.
Reorganize the existing standard and review: selected; one process assesses architectural
fitness and behavioral correctness through scenarios and explicit tradeoffs.

## Outcome

Use the versions and paths declared in `standard.toml`. Core foundations AP-01–AP-06
organize the retained DP refinements; G9 makes architectural fitness consequential to
acceptance alongside G1–G8 and applicable scientific gates. A review starts with drivers,
responsibilities and representative changes before investigating mechanisms. No aggregate
score offsets a failed obligation. A design-stage acceptance does not certify implementation.

### Consequences

Skills, role instructions, review slots and profile additions move together. New reviews
record standard version, scope and evidence strength. Existing DP/PS identifiers remain
interpretable through the version migration table; historical verdicts are not relabelled.
Plan-owned finding dispositions link decisions, packets and evidence. Indexes link to the
status owner instead of copying live packet status.

### Compensating controls

Mechanism detail is conditional on scope. Ordinary changes use the compressed tier;
probes address material uncertainty. Full library eligibility remains, with integration
cost assessed independently. There is no automated architecture score or new verdict service.

### Confirmation

Plan 17 implements the documents and workflow and checks their consistency. The cited
governance review assesses this specification only. At the maintainer's request, effectiveness
pilots are separate follow-up and are not an implementation acceptance condition. Repository
checks establish document links, ADR shape and synchronized roles, not architectural quality
or product behavior.

## Pros and cons

Explicit architectural consequences improve decomposition and testability decisions.
Scenarios require judgment; a checklist of vocabulary would still miss real coupling.

## More information

[Plan 17](../plans/17-architecture-first-design-review.md),
[standard declaration](../design_review/design_principles/standard.toml),
[governance review](../design_review/reviews/design_review_architecture-first-standard_2026-09-25.md).
The blueprint edit uses `PSE_DESIGN_EDIT=1` because governance is the authorized work;
the decision/design PR must retain that explanation and the revision row.

## Status history

- 2026-09-25 — proposed before implementation; maintainer authorized the process revision
  and then reserved pilots for separate maintainer work.
