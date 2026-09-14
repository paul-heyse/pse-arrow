---
id: ADR-0039
title: Enforce semantic admission and complete physical quantity operations
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-06, DM-07, DM-24, DM-41, DM-42, DM-43]
blueprint: [§D5, §4.1, §4.3, §4.4, §6.2, §8.3, §14.2]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0008]
superseded-by: null
revisit: A new quantity operation cannot be expressed by the registered composition policies, or a supported query bypasses generated admission.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0039: Enforce semantic admission and complete physical quantity operations

## Context

R4-01, R4-05 and R4-08 show that registration is passive, successful casts can lose values, and dimensional arithmetic alone does not determine a physical quantity type.

## Scope

Amends semantic admission, metadata interpretation and quantity-operation contracts. Registry declarations remain the sole type authority; Arrow and DataFusion execute their mechanical checks.

## Drivers

Reject malformed or incompatible meaning before a consumer assumes validity. Preserve basis, reference, scale, shape and subject through composed expressions without introducing another unit authority.

## Options

Keep passive registration and castability as admission: rejected by E1/E3. Add a second physical type library: rejected because it would duplicate the registry. Use generated validation and explicit operation rules: selected.

## Outcome

Generate a recursive validator and invoke it at bundle/provider admission, before optimization, after relevant plan transformations and at output admission. Match complete quantity contracts before attaching destination metadata; numeric and physical conversions require a declared, checked conversion. Declare composite quantity-operation rules and a WeightedMean operator in the registry.

### Consequences

Validation must cover nested fields, versioned metadata, cross-field references and row-level constraints. Unsupported physical combinations are typed errors; dimensional equality never silently chooses a basis or reference state.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

E1 and E3 characterize the pinned libraries; future boundary and quantity-composition fixtures in plan 02 must exercise supported import/query/output paths. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

One generated validator and quantity contract serves every boundary. The cost is explicit operation/conversion declarations and admission work; extension metadata alone would be cheaper but cannot enforce the required meaning.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0008 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
