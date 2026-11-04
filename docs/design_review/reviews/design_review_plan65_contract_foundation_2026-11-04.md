---
title: Plan65 pure contract foundation assessment
date: 2026-11-04
doc_role: review
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, libraries, validation]
---

# Plan65 pure contract foundation assessment

## Scope and evidence

Independent bounded advice from `creation_approach_challenge`, integrated by the coordinator
before A01 extraction. Input: ADR-0241, Plan65a/b, actual owner manifests and projection code,
foundation commit `297ae19b5`, and the host's passive original-report consumer. Criteria:
core 3.4, efficient-architecture heuristics 1.0 and process-simulator 1.5. This is the
execution foundation decision assessment, not a new broad diagnostic engagement or product
qualification. No builds or tests were run by the assessor.

## Judgment

**Accept-scoped architecture:** a pure shared contract vocabulary has an actual independent
host consumer without importing scientific execution. Behavior and extraction correctness
remain unverified until the implementation and Plan65 controls establish them. Formal ADR
acceptance remains separate.

Contract depends on no product owner crates. Diagnostics, IDs, vocabulary and quantity may
depend on contract, with their foreign projection implementations relocated to those type
owners to preserve Rust coherence. Standard-library implementations remain in contract.
Move Direction once, preserving spellings/order/inclusion semantics and changing schema and
registry sourcing together; leave no forwarding foundation.

Moving existing EffectFacts/EffectScope once is reasonable because they are declarative
metadata rather than execution machinery. Generic facet parameters or erased bags add
complexity without serving the identified consumers. Declared effect facts do not authorize
effects or establish automatic replay permission.

Contract errors retain direct thiserror/miette implementations, existing codes, paths,
categories and validation semantics. TypedDiagnostic integration belongs at diagnostics;
model's local DiagnosticProjection integration stays at model. Contract must not invoke the
existing domain diagnostic macro. External miette traits cannot be implemented for a foreign
contract error from another crate.

Borrowed observation access has no inverse constructor. Both general generated adapter entry
points must suppress native inverse adoption; reconstructible parents containing observation
owners refuse with source attribution. Passive snapshot data remains ordinarily decodable.
Migrate hardcoded model paths in foreign-adapter macros and generated bodies as well as imports.

## Implementation obligations

Plan65 A01/A05 own complete consumer migration, dependency-boundary compilation, observation
construction tests and removal of the old foundation. Source roles, error decoration and
generated publication must move together. Independent advice does not close those packages
or establish scientific support. R01/R02 disposition stays with the coordinator.
