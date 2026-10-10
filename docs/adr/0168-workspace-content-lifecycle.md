---
id: ADR-0168
title: Interpret document lifecycle once and retain producer-owned evidence
status: proposed
date: 2026-10-09
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, AP-06, AP-07, DP-19, DP-22]
blueprint: [§24.4.1]
review: docs/design_review/reviews/design_review_workspace-content-lifecycle-target_2026-10-09.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A new document family, runtime or producer cannot preserve native meaning or protection through these owners.
verification: Scoped target review and Plan 32 metadata/body, concurrent-plan scope, publication, bootstrap and exact resource-lifecycle acceptance.
standard: Core 3.4
scenarios: [docs/design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md#8-revealing-scenarios-and-alternatives]
---

# ADR-0168: Interpret document lifecycle once and retain producer-owned evidence

## Context

Separate metadata parsers, implicit asset copying and filename-based report compaction
make relevance, delivery and retention disagree. The maintainer confirmed RC01–RC03,
authorized Plan 32 implementation, retired pre-28 plan documents without salvage, and
requires multiple concurrent plans with content-level clarity over aggregate open scope.

## Scope

Amend blueprint §24.4.1 for document metadata, discovery, publication and producer-owned
artifact lifecycle. This is a decision, not a deviation. Plan 28 and other plans retain
all their implementation/qualification work; Plan 32 supplies workspace mechanisms only.

## Drivers

Review scenarios S01–S07 cover relevant discovery, metadata migration, unknown ownership,
evidence delivery, concurrent consumers, interrupted cleanup and truthful public facts.
Preserve the carefully reviewed substantive plan-authoring guidance and local reasoning.

## Options

Keep manual interpretation and suffix cleanup: low initial effort but ambiguous ownership
and recurring retention failures. Adopt a central content service or editable backlog:
adds another authority and synchronization without a demonstrated need. Select shared
library-backed interpretation, sparse defaults and derived views over native owners,
explicit publication assets and the existing exact producer resource owner. This removes
repeated interpretation without imposing document templates or a universal sweeper.

## Outcome

Interpret metadata once using pinned tooling libraries, preserving document bodies and
native family fields. A closed path/key/raw-scalar-digest adapter normalizes declared
legacy immutable encodings in memory before the same strict parser; accepted source bytes
are unchanged and new/mutable documents remain strict. Defaults and controlled lifecycle metadata describe purpose,
provenance, owner and retention; they never grant deletion. Derive aggregate packet/finding
scope from explicitly bound native tables and canonical cross-plan references, without
copying editable statuses or restricting concurrent plans. Publication selects assets
independently of retention. Producer-declared artifact roles, finalized identities,
references, borrowers, truthful outcomes and actual drain govern exact disposal.

### Consequences

Parser consumers, docs-only bootstrap and direct launch paths migrate together. Native
status tables remain authoritative. New producer manifests are sealed before eligibility;
undeclared/legacy artifacts remain protected. Pre-28 plan documents are removed with only
necessary historical-reference repairs and no content transfer or scope import.

### Compensating controls

Strict metadata/reference validation, native-corpus and body-byte tests, isolated docs
provisioning, publication rollback, temporary read borrows and exact cleanup reservations
bound failure. No automatic authored-document deletion, routine housekeeping gate, new
end-of-turn hook or generic prune command is introduced.

### Confirmation

The maintainer selected the rules and execution scope on 2026-10-09. The independent
scoped target review establishes architectural judgment; executed acceptance and bounded
measurements belong to Plan 32. Proposed status is not ADR acceptance or product qualification.

## Pros and cons

Shared interpretation and native-derived views make future changes local and observable.
The cost is explicit table/asset bindings and a separate minimal docs environment; unknown
metadata or resource ownership remains visible instead of being guessed.

## More information

- [Plan 32](../plans/32-workspace-content-lifecycle.md) owns implementation and findings.
- [Source review](../design_review/reviews/design_review_workspace-content-lifecycle_2026-10-09.md) records the predecessor assessment.
- [Target review](../design_review/reviews/design_review_workspace-content-lifecycle-target_2026-10-09.md) assesses this target.
- [Workflow owner](../authoritative_design/sections/design-change-workflow.md#section-24-4-1) owns the enduring contract.

## Status history

- 2026-10-09 — proposed under maintainer-authorized Plan 32 execution; status acceptance remains on the decision-PR route.
