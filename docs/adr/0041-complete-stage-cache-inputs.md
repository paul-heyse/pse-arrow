---
id: ADR-0041
title: Reuse complete stage inputs before introducing finer memoization
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-31, DM-32, DM-33]
blueprint: [§D14, §6.11, §14.3, §14.4]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0017]
superseded-by: null
revisit: Representative cold/warm measurements show whole-stage recomputation dominates total preparation cost and a complete finer dependency bundle passes differential checks.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0041: Reuse complete stage inputs before introducing finer memoization

## Context

R4-07 changes a bound package species set without changing the published P7 tuple. Fixed values also occupy a different segment from initial guesses.

## Scope

Amends cache dependency completeness and the initial granularity, preserving the artifact-hash memo and model/case distinction.

## Drivers

Prevent reuse of old equations after a domain, package, provider or bound-value change. Distinguish fixed numerical bindings from free guesses and retain the lineage that explains the output.

## Options

Keep the instance/template/features tuple: rejected as incomplete. Automatically track arbitrary reads: deferred by ADR-0042. Complete whole-stage keys: selected.

## Outcome

Initially memoize complete immutable stage inputs, including absence, registry/package/domain/binding artifacts, provider candidates, policies and relevant binary digests. Recompute at stage granularity and reuse equal complete input keys. Equal relation content may share storage; changed semantic lineage does not justify skipping a dependent stage. Refresh fixed/parameter bindings separately from free initial guesses; finer reuse needs complete dependency fixtures and measured benefit.

### Consequences

Some changes recompute more than the minimum. P7 is not initially cached per instance, and the invalidation table promises no unimplemented row-level reuse.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

Plan 02 requires incremental_vs_clean_binding_matrix and compares full values, membership and lineage, including provider absence and newly eligible candidates. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Complete keys simplify the correctness argument and support restart reuse. They may cause extra stage recomputation; equal relation storage is still shareable, and finer skipping requires a separate complete dependency contract.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0017 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
