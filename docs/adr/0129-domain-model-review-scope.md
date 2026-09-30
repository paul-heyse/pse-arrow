---
id: ADR-0129
title: Assess domain modeling during bounded reviews without mandatory flow tracing
status: accepted
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-06]
blueprint: [§24.4]
review: docs/design_review/reviews/design_review_domain-model-review-scope_2026-09-30.md#12-decision
evidence: Implemented
supersedes: [ADR-0128]
superseded-by: null
revisit: Routine implementation again triggers mandatory modeling reviews or flow traces, or bounded reviews cannot establish model adequacy and authority.
verification: The bounded author review assesses ordinary implementation, sufficient contract evidence and unresolved semantic authority against the revised guidance.
standard: Core 3.3; process-simulator 1.3
scenarios: [docs/design_review/reviews/design_review_domain-model-review-scope_2026-09-30.md#4-change-scenarios-and-composition]
---

# ADR-0129: Assess domain modeling during bounded reviews without mandatory flow tracing

## Context

The maintainer reports that prescribed tracing is slowing agents and requests equivalent
changes across the repositories: remove the standing AGENTS modeling mandate, retain the
criterion in the principles and review skill, and apply it during confined review periods.
Blueprint §24.4 owns this policy.

## Scope

Adopt core/template 3.3 and process-simulator guidance 1.3. Amend the policy owner, binding,
reviewer and process guidance. Product contracts and scientific requirements retain their
owners; historical reviews retain their standards and verdicts. This amends governance rather
than waiving a design criterion.

## Drivers

Keep review work proportional to the scoped question. The linked author review considers
ordinary implementation, a review settled by contracts/source, and unclear semantic authority.

## Options

Deleting only the AGENTS bullet leaves mandatory tracing in the skill, template and role.
Removing the model criterion loses an assessment the maintainer wants to retain. Keep the
criterion within bounded reviews and let reviewers choose sufficient investigation.

## Outcome

Remove the standing AGENTS modeling mandate and prescribed tracing sequences from active
review guidance. Apply domain-model assessment during requested or scheduled design reviews;
ordinary implementation does not itself trigger it. Flow tracing is optional when a concrete
uncertainty warrants it. Preserve the existing cadence and all AP/DP/G/PS identifiers.

Retain core §1, AP-04 and G9: an adequate, scoped model represents consequential distinctions,
and domain behavior realizes owned definitions and operation contracts. Ordinary types,
relations and domain functions suffice; output schemas alone do not establish alignment.
In-scope MUST gaps still prevent acceptance. These carry forward ADR-0128's model criteria;
its standing mandate and prescribed investigation process are superseded.

### Consequences

The core, template, profile and skills retain review expectations with less prescribed procedure.
Reviewers remain responsible for adequate evidence. Reduced overhead is a Proposed benefit.

### Compensating controls

Keep architectural fitness separate from scientific adequacy. Scope and unresolved questions
remain explicit; review acceptance never certifies unexamined product behavior.

### Confirmation

The linked author review accepts the source guidance at documentation-policy strength.
Canonical skills and the reviewer generate their runtime copies through agent-config-sync.
Static and product qualification are not_run for this documentation-only scope, following the
repository's execution rhythm; no measured effectiveness or independent agent trial is claimed.

## Pros and cons

Ordinary work avoids a recurring modeling assessment. Review quality depends on judgment
about evidence depth, supported by the retained criteria and scoped acceptance rules.

## More information

[Current workflow](../authoritative_design/sections/design-change-workflow.md#section-24-4)
and [standard](../design_review/design_principles/standard.toml). The maintainer explicitly
authorized this policy edit; `PSE_DESIGN_EDIT=1` permits revision 83. This local adoption
publishes no remote PR, following the existing local adoption precedent.

## Status history

- 2026-09-30 — maintainer requested the equivalent policy change; implemented and accepted
  as documentation policy after the bounded author review.
