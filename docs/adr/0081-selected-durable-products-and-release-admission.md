---
id: ADR-0081
title: Publish selected products and admit exact release updates
status: proposed
date: 2026-09-23
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-13, DM-26, DM-28, DM-38, DM-43]
blueprint: [§14.3, §14.4, §20.2, §20.4]
review: docs/design_review/reviews/design_review_rust-computation-target-design_2026-09-23.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A required product cannot be reconstructed from its exact descriptor, or retained resources exceed the admitted ownership envelope.
verification: just unit-rust-durability-resources; 30 isolated tests passed, zero failed, nextest default profile, explicit pse-relations/force-validate; just adr-lint. W19-W20 acceptance deferred.

---

# ADR-0081: Publish selected products and admit exact release updates

## Context

Plan 13 replaces automatic persistence of all compiler stages with selected complete
products. Existing native publication settlement and exact providers remain authority.

## Scope

Amend blueprint §14.3, §14.4, §20.2 and §20.4 with registry-declared product profiles,
artifact descriptors and whole-release checkpoints. Extend ADR-0046, ADR-0073 and
ADR-0076 ownership without creating another memory pool. Public wiring remains W15.

## Drivers

Complete durable meaning, exact reconstruction, semantic no-op updates, bounded
retained work and truthful cancellation of asynchronous and foreign computation.

## Options

Persisting every intermediate retains unnecessary work and bytes. CDF-only updates
cannot recover unavailable history. A separate manifest duplicates the Delta control
protocol. Selected products, endpoint fallback and shared owners avoid these costs.

## Outcome

Registry profiles select products independently of execution dependencies. Generated
descriptors identify exact members, semantic/build/algorithm/target contracts and
value assumptions. Existing control rows select actual output versions. Incompatible
descriptors refuse; no serialized Salsa handles or plan migration exists.

Validated edits and exact endpoint comparison share one atomic update path. Qualified
CDF is tried automatically, with endpoint fallback. Checkpoints advance only after
complete admission and do not certify a restarted in-memory database.

### Consequences

Required empty relations, validation, source support, native commit evidence and
current authorization remain mandatory. Reconstruction is exact and pure; effectful
results retain actual evidence. Uncertain effects reconcile before retry. A cancelled
running worker stays charged until it ends.

### Compensating controls

Versioned descriptor admission; exact provider capabilities; whole-update barriers;
finite keys/generations/jobs; reserve-before-allocation; last-reader ownership and
typed refusals. Pool accounting is not a process-RSS or hard-cancellation guarantee.

### Confirmation

Isolated units and static checks establish the implementation boundary. Actual Delta,
solver and FFI journeys remain W19; resource/performance measurement remains W20.

## Pros and cons

Selected publication reduces retained intermediates while requiring an explicit
reconstruction/support contract. Conservative admission can refuse work before a
measured implementation would exhaust physical memory.

## More information

[W12-W14 execution packet](../plans/13-w12-w14-execution.md), ADR-0076 and target-design
§4.6/§4.7. This proposed decision is not formal design acceptance.

## Status history

- 2026-09-23 — proposed before descriptor/profile/checkpoint implementation.
