---
id: ADR-0089
title: Versioned semantic identity projections and compatibility
status: proposed
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [DP-04, DP-09, DP-11, DP-24]
blueprint: [§5.1, §5.3, §14.4, §20.5]
review: docs/design_review/reviews/design_review_plan16-foundations_2026-09-25.md#12-decision
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A new execution or storage path cannot preserve these distinctions.
verification: Projection mutation and signed-zero incremental-versus-clean unit tests
---

# ADR-0089: Versioned semantic identity projections and compatibility

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

Use versioned, named projections rather than whole-object serialization. Body identity includes typed expressions, formals, guards, physical contracts and provider contracts, excluding diagnostic definition IDs, spans and documentation. Instance/revision includes bindings and topology; case includes roles, values, bounds, parameters and guesses; analysis includes outputs, mode, derivatives and policy. Prepared artifacts add consumed specialization constants, profile and actual library environment; native layout includes coordinate order, sparsity and compatibility, excluding mere coefficient values. Used starts, results and publications have separate scopes.

All floating framing and reuse equality use pse-ids canonical_f64_bits: preserve signed zero and canonicalize NaNs only in domains permitting NaN. Authored numerical inputs remain finite. Registry semantic contract identity excludes documentation and transport encoding; existing durable compatibility enforcement remains until P12 migrates it atomically. No persisted artifact is silently reinterpreted by introducing a new projection.

### Consequences

P11 audits preparation, executable program, native allocation compatibility and
attempt configuration separately. Storage invalidation cannot evict mathematical
programs. Explicit program clearing fences late insertion; retiring shared work
remains owned until completion and is distinct from a new caller's cancellation.
Immutable prepared products share allocation ownership while native worker state
remains private. Salsa rotation follows retained entries and bytes, with explicit
input durability, rather than the number of requests or publications.

P05–P06 numerical-policy identities include effective coordinates, budgets and source
provenance. Bound mathematical and convexity evidence includes the exact consumed
values, formulation, normalization and proof policy; free trial values do not establish
these facts. Structural block identity includes semantic scope and membership, not its
position in a chosen execution order. These extensions are Implemented in the P05–P06
execution packet; its targeted mutation/transport tests and final regressions record
the scope of qualification.

Existing callers and fixtures move with their replacement. No compatibility execution
path survives merely to preserve old assumptions.

### Compensating controls

Targeted negative and equivalence tests enforce the touched contracts. Unsupported
selected behavior is refused explicitly; downstream packets retain their own gates.

### Confirmation

Projection mutation and signed-zero incremental-versus-clean unit tests are recorded
in the execution packet. P01/P02 implement the semantic foundations; durable
compatibility migration remains P12. Whole-system acceptance is not inferred here.

## Pros and cons

One authority reduces semantic drift; explicit contracts require coordinated changes
across consumers and versioned identities.

The [P05–P06 execution packet](../plans/16-p05-p06-execution.md) and
[implementation review](../design_review/reviews/design_review_plan16-numerics-facts_2026-09-25.md)
record the numerical/fact extensions and their scoped qualification.

## More information

[Plan 16](../plans/16-data-model-architecture.md),
[target review](../design_review/reviews/design_review_plan16-foundations_2026-09-25.md),
and blueprint sections named in the front matter.

## Status history

- 2026-09-25 — proposed before affected implementation; decision PR acceptance pending.
