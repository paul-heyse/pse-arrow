---
id: ADR-0242
title: Separate scientific handle retirement from worker shutdown
status: proposed
date: 2026-11-04
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, DP-24, PS-13]
blueprint: [§9.11, §21.7]
review: docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A provider requires stateful cross-handle scientific composition that cannot preserve independently scoped cancellation and retirement.
verification: Two-handle real-worker controls; stale handle refusal; explicit shutdown cancellation, native/I/O drain and original child settlement.
---

# ADR-0242: Separate scientific handle retirement from worker shutdown

## Context

The prepared worker currently treats Stop(handle) and Close(handle) as process-wide session
clearing. Plan65 E04/D05 requires independent prepared handles and explicit shutdown custody.

## Scope

Amend the selected scientific-worker operation contract within ADR-0240's prepared local
placement. Stop targets original handle work; Close retires its settled state. Add an explicit
authenticated Shutdown request and response for retiring the whole session/process boundary.
Protocol changes are intentional design-phase breaks. This record precedes activation.

## Drivers

Closing one prepared provider must not cancel another. Transport disconnect and per-handle
retirement cannot establish process-wide native drain. A shutdown acknowledgement and actual
child/I/O settlement remain separate observations.

## Options

Keeping implicit global close prevents independent handles. Removing global clearing alone
loses the worker shutdown proof. Select per-handle cancellation and explicit shutdown through
the existing native session/process owner, without a new supervisor or lifecycle registry.

## Outcome

### Consequences

Bind cancellation, in-flight work, native state and stale-token refusal to original handle
generations. Shutdown closes admission, cancels and settles all remaining handles, then drains
transport and reaps the child. Clients and server migrate together through generated contracts.

### Compensating controls

Test two independent handles, stop during evaluation, stale handles, close after settlement,
disconnect without shutdown, and native failure combined with I/O cleanup failure. A response
or an empty stream cannot grant a drain proof.

### Confirmation

The operator authorized Plan65 execution, overlap and breaking target changes on 2026-11-04.
This record remains proposed; implementation, targeted tests, assembled acceptance and formal
ADR acceptance remain separate claims.

## Pros and cons

Explicit scopes support independent scientific operations and clear custody. They require
coordinated protocol/client/server changes and careful in-flight ordering.

## More information

[Plan65e](../plans/65e-scientific-portability-and-execution.md) owns E04/E05 execution;
[Plan65d](../plans/65d-local-operations-and-services.md) owns reusable custody behavior.

## Status history

- 2026-11-04: Proposed before per-handle lifecycle implementation.
