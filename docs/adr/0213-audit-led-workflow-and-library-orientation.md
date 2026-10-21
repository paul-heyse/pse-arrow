---
id: ADR-0213
title: Adopt audit-led review and deliberate completion assessment
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-06, DP-13, DP-22, DP-23]
blueprint: [§24.4]
review: "not-required: operator approved the shared process design and its local rollout; no product design is assessed"
evidence: Implemented
supersedes: []
superseded-by: null
revisit: Actual audits expose unclear ownership or ineffective research guidance.
verification: Inspect skill discovery, artifact ownership, native-solver-only selection and catalog retirement; run agent, setup and documentation controls.
standard: "Core 3.4; ProcessSimulator 1.5"
scenarios: []
---

# ADR-0213: Adopt audit-led review and deliberate completion assessment

## Context

The operator approved separating broad diagnosis, focused solution review, independent
recommendation assessment and claimed-completion assessment. The existing workflow in
blueprint §24.4 has review/plan/execution pairs but no explicit persisted diagnostic account
or dedicated completion audit. Large shared library bundles and the catalog are being retired
as general research entrypoints.

## Scope

Amend §24.4's process guidance and local skill/research tooling. Preserve Core 3.4,
ProcessSimulator 1.5, scientific gates, native agent defaults and existing plan ownership.
This installation performs no audit, changes no product scope and rewrites no existing plan
or review. The local instructions are implemented under the operator's rollout authorization;
this record remains proposed under the repository's decision-record lifecycle. No publication
or remote decision-status action is part of this work.

## Drivers

Make diagnosis persistent without making it default production-editing context. Give solution
review a clear operator-selected focus, leave methods to reviewer judgment, and make independent
assessment useful without another implementation ledger. Improve first-pass library awareness
while preserving the actively used native solver capability skill.

## Options

Retaining the combined workflow would keep diagnosis, remedy selection and completion claims
too easy to conflate. Mandatory recurring audits would add burden to every implementation slice.
Use deliberate callable boundaries with clear owners and flexible reasoning guidance instead.

## Outcome

Refresh one living design-principle audit before each new solution-review engagement; preparation
and review can consume its applicable baseline. The operator selects holistic, library-targeted,
principle-focused or combined focus. A requested recommendation audit publishes net changes and
an attributed integrated principal review, preserving the exact input in Git. A requested
completion audit updates the same owning plan and separates implementation, qualification and
benefit claims; audit authorization does not authorize production repair.

Optional repository-agnostic library overviews orient research; Context7 then primary sources
resolve consequential current details. Only native-solver-libraries remains selected and
synchronized. Other central skills remain inactive references without local links. Remove the
library-catalog MCP, refresh/read helpers and resolver; keep independent capability maps,
storage inventory and historical reports consumed by older reviews/plans.

### Consequences

Three small skills, two audit output locations and the library overview/authoring scaffold
gain real consumers. No recurring hook, score, alignment lint or new agent role is introduced.
Historical skill/catalog references do not establish current API qualification.

### Compensating controls

Stable finding and rule-impact identities retain their source meaning. Plans remain the sole
owners of scheduled progress. Exact input recoverability and attribution distinguish a revised
review from its first author's judgment. Existing scientific acceptance obligations remain.

### Confirmation

Implemented guidance and tooling are inspected through the skill index and binding. Focused
agent/configuration, setup and documentation checks establish discovery/publication behavior,
not product or scientific qualification. The rollout coordinator reports their actual outcomes.

## Pros and cons

Independent reasoning gets a clear purpose and usable output; it still requires a deliberate
scope and cannot guarantee that a future audit will uncover every omission.

## More information

- [Current workflow owner](../authoritative_design/sections/design-change-workflow.md#section-24-4)
- [Local process skills](../../.codex/skills/README.md)
- [Repository binding](../design_review/design_principles/binding/pse-arrow.md)
- [Library overview guide](../libraries/authoring-guide.md)

## Status history

- 2026-10-20: Proposed record for the operator-authorized, locally implemented process rollout.
