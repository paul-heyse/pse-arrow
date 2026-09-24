---
id: ADR-0079
title: Use containment intervals for scope membership
status: proposed
date: 2026-09-23
deciders: [paul-heyse]
level: decision
principles: [DM-01, DM-13, DM-28, DM-31, DM-43, DM-60]
blueprint: [§6.7, §14.3]
review: docs/design_review/reviews/design_review_rust-computation-target-design_2026-09-23.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported scope query cannot express its containment meaning against complete forest intervals.
verification: just unit-rust-computation; selected rule planning units; just codegen-contracts-check; just adr-lint.
---

# ADR-0079: Use containment intervals for scope membership

## Context

Plan 13 W07 replaces recursive ancestor closure as an internal lookup substrate.
P5 currently computes all ancestor pairs before any descendant selector can run.

## Scope

Declare inferred.containment_intervals with one row per admitted instance, including
isolates. Use the W06 complete forest projection for parent, depth and DFS bounds.
Scope selectors join their requested targets and entity owners to these intervals.

## Drivers

Linear containment storage, exact strict-descendant semantics, finite resource
admission, deterministic witnesses, and preserved supported inspection products.

## Options

Keep recursive closure, add a custom predicate UDF, or expose typed intervals to
ordinary DataFusion joins and comparisons. Select the latter using the existing
library forest implementation and native expression semantics.

## Outcome

Build intervals once from the actual instance inventory before P5 inference. Remove
closure-producing rules and all closure consumers. Existing instance_reachability,
instance_tree and scope_reachability remain bounded inspection/publication projections
emitted after inference; no semantic query depends on them.

### Consequences

The registry gains one native derived relation. Internal read declarations and derived
rule strata change. Original actual source rows remain the provenance witnesses.

### Compensating controls

Complete forest admission refuses cycles, missing parents and duplicate identities.
Compare interval membership with independent library reachability; check strict self
exclusion for instances, direct-owner inclusion for ports, isolates and multiple roots.
Every emitted pair reserves memory and checks cancellation.

### Confirmation

Isolated units and native rule binding establish their named contracts. Whole compiler
cold/reused equivalence remains W19, and runtime/RSS qualification remains W20.

## Pros and cons

Internal membership no longer requires a quadratic relation. Explicit all-pairs
inspection still costs its output size and is bounded by the ordinary memory budget.

## More information

Plan 13 W07, ADR-0076 and the W07–W11 execution packet. The corresponding decision/design
PR carries a blueprint revision; this proposed record does not claim formal acceptance.

## Status history

- 2026-09-23 — proposed before interval relation and consumer implementation.
