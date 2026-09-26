---
id: ADR-0091
title: Immutable publication and retention contract
status: proposed
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [DP-05, DP-19, DP-24]
blueprint: [§20.2, §20.4]
review: docs/design_review/reviews/design_review_plan16-foundations_2026-09-25.md#12-decision
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A new execution or storage path cannot preserve these distinctions.
verification: P12 publication interruption, settlement and exact read-only reopen tests
---

# ADR-0091: Immutable publication and retention contract

## Context

Plan 16 consolidates the comprehensive and data-model follow-up reviews. Its target
requires explicit contracts before replacing the current mechanisms. This decision
amends the active blueprint scope without rewriting accepted historical ADR bodies.

## Scope

The contract below and its existing owners; implementation sequencing is in
[the execution packet](../plans/16-p00-p04-execution.md). Acceptance through the
repository decision PR route remains pending; implementation is user-authorized.

## Drivers

Preserve authored meaning, make invalid selected requests observable, and use pinned
library mechanisms without a second semantic authority.

## Options

Keep the existing implicit conventions: rejected because callers can disagree.
Introduce a new universal registry/compiler: rejected because existing owners suffice.
Extend existing typed owners and derive consumer projections: selected.

## Outcome

P12 will publish immutable member paths under an explicit expected-parent precondition and persist an attempt identity. A coherent manifest is the visibility boundary. Conflict, uncertain completion and confirmed completion remain distinct; settlement inspects the attempt and exact members rather than retrying shared-table overwrite. Read-only reopening names exact versions and compatibility contracts. Retention preserves the closure required to reopen retained publications; no deletion is implied by this decision.

P00 records this downstream contract only. P00–P04 introduce no publication protocol migration, retention implementation or claims of recoverable publication under the new protocol.

### Consequences

P12 exposes a serializable ticket before effects begin. Each attempt owns immutable
member destinations; the conditional control manifest is committed last under its
explicit parent precondition. Read-only settlement uses exact manifest and
transaction witnesses and returns committed, proved noncommit/conflict or unresolved.
Uncertain history is never treated as permission to rematerialize an attempt.

Compatibility compares the complete versioned semantic closure against independently
compiled consumer expectations. Prose and implementation encodings are separate from
semantic identity. Unsupported historical formats return typed migration-required
errors; opening does not migrate or select a legacy runtime. Writers initialize
retention coordination, readers only acquire existing shared ownership, and maintenance
advances the generation while exclusively owning the retained closure.

Existing callers and fixtures move with their replacement. No compatibility execution
path survives merely to preserve old assumptions.

### Compensating controls

Targeted negative and equivalence tests enforce the touched contracts. Unsupported
selected behavior is refused explicitly; downstream packets retain their own gates.

### Confirmation

P12 publication interruption, settlement and exact read-only reopen tests. Whole-system acceptance is not inferred from these targeted checks.

## Pros and cons

One authority reduces semantic drift; explicit contracts require coordinated changes
across consumers and versioned identities.

## More information

[Plan 16](../plans/16-data-model-architecture.md),
[target review](../design_review/reviews/design_review_plan16-foundations_2026-09-25.md),
and blueprint sections named in the front matter.

## Status history

- 2026-09-25 — proposed before affected implementation; decision PR acceptance pending.
