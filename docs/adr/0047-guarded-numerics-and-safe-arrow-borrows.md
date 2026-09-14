---
id: ADR-0047
title: Preserve guarded numerical semantics and borrow through safe Arrow views
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-24, DM-28, DM-29, DM-36, DM-37, DM-40]
blueprint: [§D6, §D11, §6.11, §7.3, §7.4, §18.2, §21.2]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0014]
superseded-by: null
revisit: A backend cannot express an adopted numerical policy or measured buffer-transfer cost justifies a new layout.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0047: Preserve guarded numerical semantics and borrow through safe Arrow views

## Context

R4-02 and R4-09 expose eager tape evaluation and unguarded floating rewrites/FMA. L7 shows that safe primitive buffer views already account for array slice offsets.

## Scope

Amends numerical lowering, control flow and Arrow-to-native borrowing; the math IR and native workspace remain separate representations.

## Drivers

Preserve observable numerical values and failures under rewriting, branching and backend lowering. Use Arrow ownership and slice APIs where they meet the native consumer contract.

## Options

Unconditional real-algebra rewrites and FMA: rejected by E6. Reject all sliced arrays: safe but needlessly restrictive. Explicit numerical policy plus safe slice-aware views: selected.

## Outcome

Preserve ordered operations and guarded failure behavior by default. Persist any reassociation, FMA and comparison policy in the case-bound preparation contract; a backend that cannot honor it refuses it. Compile explicit conditional blocks/masks, scope shared values to the executed branch, and differentiate only executed paths. Borrow null-free contiguous primitive values through owning safe Arrow views, including eligible slices; otherwise perform checked, reserved copies.

### Consequences

Affine structure must be established over decision variables before choosing Pyomo LinearExpression. Lifetime and cancellation tests are required for borrows; no performance gain is claimed without measurements.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

E2/E6 are counterexamples. Plan 02 requires guarded_branch_execution, numerical_policy_conformance, nonlinear_affine_lowering and arrow_borrow_lifetime fixtures. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Ordered branch-aware execution makes failure semantics explicit and safe views permit eligible slices. Some algebraic optimizations and backend routes must be refused; copying remains the supported fallback, with performance decided by measurement.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0014 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
