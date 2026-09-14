---
id: ADR-0042
title: Defer finer memoization until whole-stage measurements justify it
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-32, DM-33, DM-56, DM-58]
blueprint: [§D14, §14.3, §24.3, §26]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0020]
superseded-by: null
revisit: A measured workload still needs finer dependency tracking after complete per-instance reuse is characterized.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0042: Defer finer memoization until whole-stage measurements justify it

## Context

The simpler compiler in review section 8 starts with complete stage keys. ADR-0020 assumed an incomplete per-instance cache and therefore needs a different deferral boundary.

## Scope

Amends the granularity and adoption trigger; salsa remains absent from runtime dependencies.

## Drivers

Keep one dependency mechanism until a measured workload needs finer tracking. Dependency correctness and cancellation behavior must precede claims of incremental speed.

## Options

Adopt salsa or a custom fine-grained dependency engine now: rejected without a measured consumer. Keep the old incomplete tuple: rejected. Refine one existing memo in measured steps: selected.

## Outcome

Use the artifact-hash memo at whole-stage granularity first. Evaluate a complete per-instance bundle only after the R-22 measurement trigger. Consider salsa under R-01 only if that measured case requires automatic finer dependency tracking and its cancellation model has a complete boundary contract.

### Consequences

Compile latency may initially be higher, but there is one dependency declaration and one cancellation boundary to validate.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

Register R-01 and R-22 track the decisions; plan 02 names the cold/warm and incremental-versus-clean experiments. No speedup is claimed. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Deferral avoids a second revision/cancellation model and premature per-instance keys. Whole-stage preparation can cost more; R-22 measures that cost before refinement, and R-01 separately evaluates salsa.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0020 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
