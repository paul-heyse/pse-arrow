---
id: ADR-0241
title: Extract pure owner-derived contract and directional adoption interfaces
status: proposed
date: 2026-11-04
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-05, DP-01, DP-02, DP-24]
blueprint: [§4.7, §21.7, §23.2]
review: docs/design_review/reviews/design_review_plan65_contract_foundation_2026-11-04.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A selected consumer cannot express its structural projection without importing product behavior or recreating native authority.
verification: Dependency-boundary compilation; observation-only source and generated construction controls; independent option-preservation and semantic-evolution cases in Plan65 A/B.
---

# ADR-0241: Extract pure owner-derived contract and directional adoption interfaces

## Context

Plan65 extends source-owned contracts to independent host evidence and scientific operations.
The current generic graph lives in product-heavy `pse-model`; its output direction filters
descriptor participation but generic adapters still emit inverse construction. That is unsafe
for sealed verdict, permission and settlement owners (review I05).

## Scope

Propose Plan65 R01/R02: extract the pure structural declaration/access vocabulary into
`pse-contract`, leaving scientific/diagnostic adapters at their actual owners. Before extraction,
extend the existing vocabulary with explicit owner construction roles and one-way observation
access. This records the selected implementation direction; formal acceptance and architecture
publication follow the existing decision route. No accepted record is modified.

## Drivers

One native scientific operation and one original developer report must inform the foundation.
Independent readers need recorded facts without obtaining live permission or custody. Generic
tools must not import science solely to interpret structure. Native mathematical representations,
semantic identities and original effect owners retain their responsibilities.

## Options

Keeping two generic graphs repeats interpretation. Moving all scientific adapters into a
common crate preserves the unwanted coupling. Making all objects Proto-first duplicates native
scientific policy. Select a pure shared graph with owner-local executable semantics instead.
Retain reconstructible data access separately from observation access; an output filter or
encode-only convenience function alone cannot prevent another generic entry point generating
inverse construction.

## Outcome

### Consequences

Observation owners expose only version/layout and field access; they have no generic native
constructor. Codegen emits native-to-wire observation conversion only. Requests use explicit
contextual constructors; sealed control capabilities remain outside generic adoption. Typed
versioned options describe roles and effects but cannot grant native authority. One native
checker interprets those options across generation, import and inspection.

### Compensating controls

Test the general adapter entry points as well as convenience wrappers. A well-formed historical
observation cannot become a current verdict, readiness grant or drain capability. Keep native
report file/extent/digest/waited-status reconciliation and scientific admission. Preserve exact
custom options and effective feature settings across real descriptor acquisition routes.

### Confirmation

The operator authorized the Plan65 target, breaking changes and overlapping Plan64 execution
on 2026-11-04. Implementation/testing and independent architectural assessment remain separate
claims. The record remains proposed until the prescribed decision process accepts it.

## Pros and cons

This removes repeated mechanical projection while preserving domain ownership and independent
testability. It introduces a crate boundary and generator roles that require complete consumer
migration; no forwarding compatibility foundation will remain after extraction.

## More information

[Plan65](../plans/65-proto-integrated-target.md) owns execution and recommendation disposition.
[65a](../plans/65a-contract-foundations-and-generation.md) and
[65b](../plans/65b-representation-validation-and-interpretation.md) retain concrete library
mechanisms, sources, migration and acceptance. Buf adapters, runtime validation and new physical
models remain separately selected work, not consequences of this decision.

## Status history

- 2026-11-04: Proposed before affected foundation and construction-role implementation.
