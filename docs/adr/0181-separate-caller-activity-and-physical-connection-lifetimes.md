---
id: ADR-0181
title: Separate caller release from activity and physical connection lifetimes
status: proposed
date: 2026-10-12
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, AP-07, DP-09, PS-10]
blueprint: [§20.5, §20.6, §21]
review: docs/design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f04
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported remote deployment or independently restartable transport requires a different authentic physical shutdown owner.
verification: Plan37 B1 caller-release/admission races, durable activity and unknown-effect handoff, protected-reader refusal, and interrupted two-driver shutdown; Q1 owns actual assembled native/Python acceptance.
---

# ADR-0181: Separate caller release from activity and physical connection lifetimes

## Context

The reviewed Python runtime close clears shared caches and disconnects shared canonical clients. Result-reader protection covers some live dependencies but does not model all admitted work. Plan37 RC04 confirms distinct caller, activity and physical connection owners.

## Scope

Amend blueprint §20.5/§20.6/§21 for the existing local-first integration. This changes caller close semantics, not numerical cancellation, durable completion or remote support. RC08 moves pure study policy into the existing model owner; RC09 adds beta tooling under existing operational authority. Neither requires a new production crate or SDK universe.

## Drivers

One runtime must be releasable while another renews or publishes. Actual dependency lifetimes govern shutdown; reference counts and cached aliases do not. Admission and shutdown must conflict atomically without a long transaction or lock across science or transport teardown.

## Options

Select a shared physical admission gate, independent logical caller admission, and explicit terminal activity ownership. Keeping caller-triggered disconnect allows shared teardown. Counting Arc aliases confuses idle references and active work. Serializing all work behind a global runtime mutex repeats an unnecessary execution boundary. Existing SDK drivers remain the physical transport owner beneath this small canonical composition contract.

## Outcome

A logical caller close is idempotent and refuses new work through that caller. Clones share its closed state; a separately constructed runtime has independent caller authority. Admitted work retains an activity through heartbeat, publication, cancellation, drain and exact settlement or transfer to an explicitly owned recovery activity. An idle alias is not an activity and cannot bypass admission to issue work.

The shared physical owner atomically refuses shutdown while actual activities or protected readers remain, preserving their usable admission; otherwise it moves Open to Closing before excluding new admission and draining both clients. Refusal is not close. Cancellation or partial driver failure leaves Closing and authentic drain/recovery ownership, never false success or automatic reopening. Reopening requires a newly validated owner. Shutdown does not infer that submitted remote effects or native workers have stopped.

Ordinary Rust/Python callers receive typed closed/unavailable diagnostics. The Python close spelling remains but its declaration is regenerated with logical-release meaning. Explicit cache clearing stays separate. Physical shutdown remains an explicit deployment/fixture/admin operation; remove ordinary caller wiring to it. The common gate covers direct canonical operations and activation as well as runtime entrypoints.

### Consequences

Migrate all affected owners together, including Operations, heartbeat, readers, analyses, study/fit/horizon flows, Python entrypoints and fixture administration. Keep short RPC and long activity lifetimes distinct. Recovery must retain the actual capability it needs rather than release an activity merely because the submitting future was cancelled.

### Compensating controls

Local peer and policy controls distinguish refused admission, before-dispatch interruption, unknown effects and physical drain. Test cancellation between the two driver closes. No compatibility API retains caller-triggered physical close.

### Confirmation

The operator confirmed RC04 and authorized this tranche on 2026-10-12. The source review motivates the change; the [bounded target review](../design_review/reviews/design_review_b1-connection-lifetimes_2026-10-12.md) accepts the proposed contract before implementation. Targeted tests and assembled Q1 remain different claims; proposed ADR status is unchanged until the repository's decision-PR route.

## Pros and cons

The selected contract permits independent caller release and explicit finite shutdown. It adds activity ownership at real admission/settlement boundaries; the cost is justified by dependencies that already exist, not a universal lifecycle framework.

## More information

[Plan37](../plans/37-surrealdb-lifecycle-and-integration.md) owns status and findings; [37b](../plans/37b-connection-and-transport-lifetimes.md) owns detailed behavior. [Source review F04](../design_review/reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md#f04) records the baseline.

## Status history

- 2026-10-12: Proposed before implementation under operator-confirmed RC04; no ADR status transition or assembled qualification claimed.
