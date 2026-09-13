---
id: ADR-0020
title: Defer salsa; the artifact-hash memo is the only memoization mechanism
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-58, DM-32, DM-28]
blueprint: [§14.3]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A measurement shows sub-pass granularity finer than per-instance is needed inside P7
verification: The §24.3 compile-time benchmark for P7 on a flowsheet-scale model; register row R-01

---

# ADR-0020: Defer salsa; the artifact-hash memo is the only memoization mechanism

## Context

Blueprint §14.3 defers `salsa` behind the artifact-hash memo with a stated trigger. The first review's F13 asked whether the two mechanisms duplicate each other; the supporting-library capability map measured that salsa's backdating and accumulators work as described, which does not change the question.

## Scope

Binds the memoization mechanism for phases 0-3. It does not reject `salsa` permanently; the trigger is a measurement, and the register row carries it.

## Drivers

`salsa`'s distinctive value is automatic dependency tracking, which duplicates `pass_specs`' declared inputs; an undeclared read is a contract violation, not something to track; its unwind-based cancellation cannot cross the Ipopt boundary.

## Options

Adopt `salsa` now — rejected: a second dependency system and a second cancellation model with no measured need (charter DM-58). Adopt it only for P7 — deferred to the same trigger.

## Outcome

`salsa` is not a dependency. Per-instance memoization inside P7 uses the same key shape over `(instance_id, template hash, resolved features)`.

### Consequences

If the trigger fires, adoption means reconciling two cancellation models at the Ipopt boundary, which is the part to design before adopting rather than after.

### Compensating controls

Register row R-01 carries the trigger, the owner and a phase-gated review date so the deferral does not become an omission.

### Confirmation

The §24.3 benchmark measures P7 compile time per instance; the register review reads it monthly.

## Pros and cons

Deferring costs a possible re-plumb later; adopting now costs a mechanism nothing has asked for.

## More information

Blueprint §14.3 (pass engine); review finding F13; `docs/adr/register.md` row R-01.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
