---
id: ADR-0040
title: Resolve property demand from normalized seeds and name immutable stage outputs
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-18, DM-20, DM-21, DM-22, DM-31]
blueprint: [§D8, §4.1, §6.11, §9.6, §14.1]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0011]
superseded-by: null
revisit: A selected method requires demand that cannot be declared over the finite normalized scope before realization.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0040: Resolve property demand from normalized seeds and name immutable stage outputs

## Context

R4-06 identifies P6 reading P7/P8 equations and P15 initialization requirements despite preceding those stages. A relation-wide single producer also cannot describe successive math artifacts.

## Scope

Amends demand discovery and stage ownership within the existing three artifact levels. No fourth model authority or database is introduced.

## Drivers

Make the pass DAG decidable before realization, including law-only and initializer-only demand. Allow several stages to carry one schema without competing mutable producers.

## Options

Discover demand from realized equations through an implicit backward loop: rejected. Contract a larger mutually recursive compiler stage: viable but unnecessary for declared demands. Resolve normalized seeds and check realization completeness: selected.

## Outcome

Extract property-demand seeds from normalized template, law, port, display and initializer declarations. P6 closes selected method requirements over a finite bound scope before realization. Pass contracts name immutable input/output ports; successive stages may carry the same relation schema under distinct artifact handles.

### Consequences

P7 through P15 cannot invent new property demand. They reject undeclared requirements and incomplete providers; stage-output ownership and acyclicity become registry checks.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

Plan 02 specifies demand_seed_closure, stage_bundle_graph and realization-completeness fixtures, including law-only and initializer-only requirements. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Finite seeds and named ports make missing requirements and producer ambiguity observable. Authors must declare opaque requirements early; undeclared late demands are refused instead of triggering an invisible extra compile loop.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0011 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
