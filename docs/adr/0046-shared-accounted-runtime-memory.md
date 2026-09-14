---
id: ADR-0046
title: Share runtime budgets and reserve platform allocations explicitly
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-30, DM-35, DM-39, DM-50, DM-59]
blueprint: [§14.3, §18.2, §18.8, §20.1, §24.3]
review: docs/design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md#6-acceptance-gates
evidence: Proposed
supersedes: [ADR-0029]
superseded-by: null
revisit: Measured pool/process peaks diverge materially or a supported consumer cannot reserve before allocation.
verification: just adr-lint; just docs; docs/plans/02-blueprint-revision-5-contracts.md Verification; bash docs/design_review/evidence/blueprint-rev4-2026-09-13/run.sh
---

# ADR-0046: Share runtime budgets and reserve platform allocations explicitly

## Context

R4-13 identifies Arrow/native allocations outside the query pool and separate snapshot sessions multiplying deployment budgets. Arrow tracking reservations are not fallible allocation limits.

## Scope

Amends resource ownership and the scope of typed exhaustion guarantees, preserving snapshot-local catalogs.

## Drivers

Prevent concurrent snapshot sessions from each consuming an entire deployment budget. Cover live platform buffers with reservations and state exactly which allocations the library pool cannot control.

## Options

Give every session its own full deployment budget: rejected. Treat Arrow tracking as a rejection mechanism: rejected. Share runtime accounting with explicit platform reservations and a supported size envelope: selected.

## Outcome

Share one configured RuntimeEnv and accounted budget across snapshot sessions in a process. Query consumers use FairSpillPool; platform-owned canonicalization, binding and result buffers reserve through MemoryConsumer/MemoryReservation::try_grow before allocating. Track consumers and peaks, and report process peak separately. Limit guarantees apply to accounted consumers, not arbitrary allocator/solver memory.

### Consequences

Reservations must cover temporary copies and coexistence peaks, release on error/cancellation, and avoid double counting shared buffers. External solvers retain separate declared process budgets.

### Compensating controls

The implementation gates are explicit in [plan 02](../plans/02-blueprint-revision-5-contracts.md). Document acceptance does not establish runtime behavior. Deferred optimizations remain disabled until their register trigger and conformance evidence are satisfied.

### Confirmation

Pinned memory API source is interface-checked in the review. Plan 02 names query/canonicalization/result budget tests and concurrent-snapshot cancellation checks; whole-process protection is not claimed. `just adr-lint` and `just docs` check documentation structure only; the plan distinguishes these checks from future behavioral acceptance.

## Pros and cons

Shared accounting exposes consumer limits and release behavior. It requires correct ownership and estimates for coexisting buffers; solver/global allocator memory remains outside the fallible guarantee and must be measured separately.

## More information

See the governed blueprint sections, the [revision-4 review](../design_review/reviews/design_review_blueprint-rev4-library-contracts_2026-09-13.md), its [reproducible library evidence](../design_review/evidence/blueprint-rev4-2026-09-13/README.md), the [revision-5 contract review](../design_review/reviews/design_review_blueprint-rev5-contracts_2026-09-13.md), and [plan 02](../plans/02-blueprint-revision-5-contracts.md).

## Status history

- 2026-09-13 — proposed before the revision-5 blueprint amendment; user requested reconciliation of all review findings and library opportunities.
- 2026-09-13 — proposes superseding ADR-0029 for the corrected contract above; accepted arguments remain immutable.
- 2026-09-13 — accepted (revision-5 review, Accept for bounded proposed scope; evidence remains Proposed until the wave-1 implementation review).
