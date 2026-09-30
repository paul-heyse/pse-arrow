---
id: ADR-0128
title: Require explicit domain models to govern implementation and review
status: accepted
date: 2026-09-29
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-02, AP-03, DP-02, DP-03, DP-05, DP-06, DP-08]
blueprint: [§24.4]
review: docs/design_review/reviews/design_review_semantic-model-first-standard_2026-09-29.md#12-decision
evidence: Implemented
supersedes: []
superseded-by: null
revisit: Reviews accept output-only or inadequate domain models, miss independent semantic interpretations, or prescribe modeling machinery without a consumer.
verification: Author review scenarios S01-S03 trace output-only models, ordinary domain functions and inadequate single authorities through AP-04/G9; source inspection and generated-role synchronization qualify guidance only.
standard: Core 3.2; process-simulator 1.2
scenarios: [docs/design_review/reviews/design_review_semantic-model-first-standard_2026-09-29.md#s01, docs/design_review/reviews/design_review_semantic-model-first-standard_2026-09-29.md#s02, docs/design_review/reviews/design_review_semantic-model-first-standard_2026-09-29.md#s03]
---

# ADR-0128: Require explicit domain models to govern implementation and review

## Context

The maintainer approved equivalent adoption of the explicit domain-model requirement on
2026-09-29. Existing authority, typing and fidelity rules can otherwise be read as satisfied
by consistent output records while procedures independently define their meaning. This
extends ADR-0094's standard without changing its review cadence or scientific obligations.

## Scope

Adopt core/template 3.2 and process-simulator guidance 1.2, retaining AP/DP/G/PS IDs. Amend
blueprint §24.4, the binding, reviewer and process guidance. This is the authorized governance
scope; product code, scientific contracts, library exploration and capability skills are
unchanged. Historical reviews retain their original version and verdict.

## Drivers

Make the domain's phenomena and behavior explicit enough to localize semantic changes and
mechanism substitution. Preserve the other five foundations and flexible implementation.
The linked author review owns S01-S03 and its specification-level acceptance.

## Options

A single new paragraph leaves reviewer instructions incomplete. A new foundation, gate or
modeling framework duplicates assessment or prescribes machinery. Strengthening the existing
AP-04/G9 and review slots is selected: one authority, no new gate or artifact type.

## Outcome

Core §1 makes semantic-model-first domain design a MUST wherever implementation establishes
or interprets domain meaning. AP-04/G9 require both an adequate model and behavior governed
by its owned definitions and operation contracts. A deferral cannot waive that obligation
for supported behavior, even when current outputs are correct. Ordinary types, relations and
domain functions suffice; no universal ontology, DSL, registry or instruction serialization
is required. Existing rules clarify invariant ownership, contextual bindings and the semantics
preserved by each representation contract. Reviews trace phenomena into authorities,
implementation, consumers and expected changes.

### Consequences

The review template, profile, skills, canonical role and generated role align. Independent
scientific gates remain unchanged. Substitution preserves the domain contract or exposes an
incompatibility; locality does not guarantee inexpensive replacement.

### Compensating controls

Keep scope proportional: a bounded review retains its relevant scenario; generic mechanisms
need no invented domain entities. Independent tests challenge the domain contracts. Do not
reinterpret historical acceptance as product conformance to the new edition.

### Confirmation

The cited author review accepts the policy at source-inspected specification strength; no
independent review or effectiveness pilot is claimed. Role generation follows its existing
source. Checks establish document integrity, not architecture or scientific qualification.
Improved change locality remains Proposed until supported by actual changes.

## Pros and cons

Meaning becomes an implementation obligation with a concrete review consequence. Reviewers
must still exercise judgment about sufficient concepts and credible changes; more schemas
or abstractions do not demonstrate alignment.

## More information

[Current workflow](../authoritative_design/sections/design-change-workflow.md#section-24-4),
[standard](../design_review/design_principles/standard.toml), and the linked governance review.
The blueprint edit uses `PSE_DESIGN_EDIT=1` because this governance change is the maintainer's
explicit request; revision 82 records it. This local adoption does not publish a remote PR.

## Status history

- 2026-09-29 — maintainer requested adoption of the approved domain-model-first wording;
  implemented and accepted as documentation policy after the bounded author review.
