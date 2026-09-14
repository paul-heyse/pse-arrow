---
id: ADR-0048
title: Use standard relational operators and explicit-schema readers
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-18, DM-25, DM-38, DM-41, DM-53, DM-54, DM-56, DM-58]
blueprint: [§5.4, §6.10, §6.11, §14.2, §21.1, §24.1, §26]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A representative supported scan/import needs an operation outside the bounded algebra or measured optimization triggers in R-24/R-25 fire.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0048: Use standard relational operators and explicit-schema readers

## Context

L3/L4/L9 identify existing physical predicates, aggregation/unnesting and explicit-schema readers for current or declared consumers. R4-11 shows the exact-pushdown test cannot detect missing rows.

## Scope

Adds bounded rule-algebra and ingestion bindings inside D10. It does not add a crate family, new query engine or new modeling language.

## Drivers

Use existing operator implementations for actual scan, contribution and observation consumers. Keep exact filtering, physical meaning and import policy explicit while avoiding another general-purpose engine.

## Options

Reimplement predicate semantics or reject all Arrow readers because inference exists: rejected. Add a general query/ingestion platform: rejected. Bind existing pinned mechanisms to explicit consumers: selected.

## Outcome

Use DataFusion physical predicates and Arrow filters as the shared exact evaluator; compare pushed scans with complete unpruned results. Add typed aggregate/unnest operations for contribution grouping and membership projection. Use explicit-schema Arrow CSV/JSON readers only for declared tabular observation imports. Keep Parquet pruning, bounded coalescing and predicate preimages behind workload and equivalence checks where their benefit is unmeasured.

### Consequences

Aggregate null/empty/order behavior, provenance, reader schema/units and filter completeness remain platform contracts. Predicate preimages must be equivalent replacements; conservative ranges can only prune with a residual filter.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

Pinned interfaces were inspected in review L3/L4/L8/L9. Plan 02 names pushdown_vs_unpruned, relational_expansion_reference and explicit_schema_observations; register R-24/R-25 records deferred optimization decisions. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Shared predicates, aggregates, unnesting and declared-schema readers reduce custom mechanisms. The platform still owns null/empty/order/provenance and physical conversions; optional pruning and coalescing require separate consumer measurements.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
