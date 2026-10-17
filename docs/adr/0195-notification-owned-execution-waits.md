---
id: ADR-0195
title: Wake execution waits from their owning resources
status: proposed
date: 2026-10-17
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, DP-21, PS-10]
blueprint: [§14.3, §18.10]
review: docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md
evidence: Interface-checked — existing flight, kernel scope, math admission and native process-exclusion owners; pinned event-listener 5.4.2 contracts.
supersedes: []
superseded-by: null
revisit: A wait cannot observe its actual resource release or stop without renewed polling or another scheduler.
verification: Plan43f wake-before-wait, cancelled waiter, timeout/release race, rollback release and exact child completion tests; Uno exclusion controls.
---

# ADR-0195: Wake execution waits from their owning resources

## Context

Plan43f F02 identifies periodic retries in math admission/resume, private native-child I/O/reap and Uno process exclusion. The existing raw cancellation flag is sufficient for cooperative numerical callbacks but cannot wake an idle owner. The maintainer delegated concrete remaining execution decisions.

## Scope

Extend the existing execution-scope/kernel and Uno native binding contracts under blueprint §14.3/§18.10. Keep admission and process exclusion with their current owners.

## Drivers

Cancellation and absolute deadlines must reach blocked waits without busy periodic retry, native work escaping ownership or partial semaphore acquisition preventing smaller ready work. Low cancellation/race tests remain runtime-independent.

## Options

Awaiting a partially acquired CPU team changes current admission semantics. A second scheduler or global flag-to-waker registry duplicates authority. Select a shared cancellation owner with one atomic stop state and library Event notification, actual permit-release notification, Linux owned descriptor readiness and Uno condition-variable exclusion.

## Outcome

The generic cancellation owner lives with shared execution contracts in pse-diagnostics and uses pinned event-listener. Kernel scopes and completion-owned flights carry that same owner; native numerical callbacks receive its read flag. Cancellation publishes stop before notification. A registered Waker may only signal a safe independently owned waiter; it does not execute numerical work or own a native session. Registration-before-state-check prevents lost cancellation. Raw flag construction cannot substitute for the notifying execution owner.

Actual semaphore owners notify after permits are returned, including tentative rollback; no count or queue authority is duplicated. Admission continues to attempt complete resources and release tentative ownership before waiting. Uno retains process-wide exclusion with an owned permit, generation predicate and absolute deadline; `pse_uno_notify_waiters()` safely signals the process-wide gate without a session pointer. Every waiter rechecks its own typed stop. Private child readiness/reaping remains attached to its exact owned descriptors and Child, with cancellation wake and actual wait/drain.

### Consequences

Migrate all actual scope/flight/native execution constructors and cancellation writers. Remove replaced periodic retry paths and tests. Direct scientific scalar evaluation can still borrow a stop flag, but blocked execution owners require the notifying control. No new crate, background reaper, fairness promise or scheduling policy is introduced.

### Compensating controls

Retain original clock, cancellation-versus-deadline distinctions, memory/job/CPU admission, scientific work and native completion ownership. Notifications are hints: register before checking, recheck predicates after every wake and tolerate spurious/late signals. A stopped waiter releases only its own tentative resources. Tool cancellation never cancels science.

### Confirmation

**Interface-checked:** source inspection establishes the periodic paths and pinned event-listener registration/Future behavior. Plan43f records implementation and revealing tests. This proposed record does not certify them or alter formal ADR acceptance.

## Pros and cons

A single stop state plus native-safe notification reaches synchronous and async owners without another runtime. It requires constructor migration and exact release ordering, which the old raw flag concealed.

## More information

[Plan43f](../plans/43f-execution-resources-and-platform-profiles.md), [Plan43c](../plans/43c-native-methods-reuse-and-matrix-boundaries.md).

## Status history

- 2026-10-17 — proposed before binding/control changes under delegated execution authority.
