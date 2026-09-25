---
id: ADR-0090
title: Shared execution vocabulary and structured boundaries
status: proposed
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [DP-02, DP-17, DP-21, PS-10, PS-12]
blueprint: [§4.2, §18.6, §21.1, §23.2]
review: docs/design_review/reviews/design_review_plan16-foundations_2026-09-25.md#12-decision
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A new execution or storage path cannot preserve these distinctions.
verification: Vocabulary codec round trips and source-attributed boundary refusal unit tests
---

# ADR-0090: Shared execution vocabulary and structured boundaries

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

The schema registry owns durable semantic tags projected into library-neutral pse-model values, Arrow codecs and Python contracts. Backend identity, native termination, candidate kind, qualification, unavailable evidence reasons and metric alternatives stay distinct. Heat input/output are distinct from work; absolute time origin and elapsed duration are distinct.

Adapters own backend-native behavior and translate statuses into shared tags. Static capability inventory is distinct from contextual eligibility and selected execution (P07). Boundary diagnostics classify invalid model, unsupported capability, resource exhaustion, trial rejection, nonfinite evaluation, infrastructure failure, cancellation, conflict, incompatibility and internal failure, retaining source identities, stage and observations. Missing evidence uses typed reasons, never invented NaN observations. P10 owns complete result/storage projection migration.

### Consequences

P05–P06 extends the existing vocabulary with resolved numerical requirements,
source-attributed presolve evidence, exact/numerical/inconclusive convexity evidence
and immutable candidate assessment. Native termination, numerical acceptance,
physical closure and final usability remain separate facts. Public serialization
consumes the completed assessment and does not perform another physical evaluation.
The [P05–P06 execution packet](../plans/16-p05-p06-execution.md) and its
[implementation review](../design_review/reviews/design_review_plan16-numerics-facts_2026-09-25.md)
record the implemented extensions and their scoped qualification.

The pure authored scalar-function vocabulary and exhaustive handler tags live in
`pse-quantity::functions`, alongside physical operation contracts. `pse-math`
re-exports and executes them; schema reflection reads the same declarations directly.
This avoids a generator → math → provider → generated-model dependency cycle without
weakening the existing generator dependency ceiling or copying function names.

Existing callers and fixtures move with their replacement. No compatibility execution
path survives merely to preserve old assumptions.

### Compensating controls

Targeted negative and equivalence tests enforce the touched contracts. Unsupported
selected behavior is refused explicitly; downstream packets retain their own gates.

### Confirmation

Vocabulary codec round trips and source-attributed boundary refusal unit tests are
recorded in the execution packet. P02 implements shared tags, capability inventory
and structured boundaries; complete result migration remains P10. Whole-system
acceptance is not inferred here.

## Pros and cons

One authority reduces semantic drift; explicit contracts require coordinated changes
across consumers and versioned identities.

## More information

[Plan 16](../plans/16-data-model-architecture.md),
[target review](../design_review/reviews/design_review_plan16-foundations_2026-09-25.md),
and blueprint sections named in the front matter.

## Status history

- 2026-09-25 — proposed before affected implementation; decision PR acceptance pending.
