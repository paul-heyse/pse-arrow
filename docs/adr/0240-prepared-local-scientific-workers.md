---
id: ADR-0240
title: Bind prepared local scientific workers to complete provider operations
status: proposed
date: 2026-11-03
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-07, DP-10, DP-15, DP-19, PS-01, PS-02, PS-07]
blueprint: [§9.11]
review: docs/plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required scientific implementation cannot preserve coherent state, derivative demand, exact context or physical process settlement through the local worker.
verification: Plan64k same-science in-process/worker comparison, independent physical and derivative controls, explicit unsupported demands, crash/cancel/drain and stale-context refusal.
---

# ADR-0240: Prepared local scientific workers

## Context and scope

The maintainer accepted Proto RC04 on 2026-11-03 and selected working local-worker delivery.
This proposes a provider binding under the existing physical/scientific authority, amending
blueprint §9.11. It does not select remote deployment, distribute a solve, or move science
into the transport. Existing in-process provider placements retain their distinct purpose.

## Decision and alternatives

Use an explicitly prepared authenticated local process, a synchronous owned proxy and typed
Buffa/ConnectRPC messages for whole coherent provider requests. Retain exact ProviderSpec,
data/implementation identity, physical ports, prepared demands and attempt/session scope.
The first worker invokes the same CoolProp WaterFactory as the in-process ADR-0238 control;
this establishes isolation/interchange rather than new scientific breadth. Preparation never
downloads native libraries or implicitly launches a missing provider during inspection.

Transport every supported demand as its real operation and refuse unavailable capabilities.
No scalar-output RPC, action-to-dense fallback or automatic ambiguous replay is selected.
The first water implementation supplies its actual CompleteJets capabilities; action products
and linearization tokens remain unavailable unless their native implementation is added and
independently qualified. No token supplied by a worker can become local sealed evidence.
Issuance requires actual support, current context re-establishment and expected-context checks.

## Consequences and confirmation

The proxy waits from owned numerical/native execution, not an arbitrary async executor task.
Cancellation/deadline preserves admission and custody until work settles or the owned child is
terminated and reaped, and parent-side I/O has joined. An unbounded original scope remains
unbounded; a finite scope carries its original remaining allowance and does not renew it
on evaluation. Initial unreleased protocol v1 uses absent `remaining_ns` for unbounded
scope; the discarded draft's fixed 600-second allowance is not a compatibility contract.
Reconnect/replacement invalidates preparation and derivative contexts.
64k owns the bounded preparation/evaluation/close protocol and complete process qualification;
64d/f retain the physical configurations and 64h the independent science/reference claims.
The existing in-process route remains the lower-crossing placement, not a compatibility shim.
Measure complete solves before asserting performance benefit. No current behavior is established
by this proposed record; actual binding implementation follows the decision/review route.

## Sources

[64k](../plans/64k-prepared-scientific-workers.md), the
[review I04](../plan_improvement_reviews/plan64-proto-integration-and-extensibility-2026-11-03.md#I04)
and [ADR-0238](0238-native-water-property-provider.md) retain the consumed contracts and limits.
