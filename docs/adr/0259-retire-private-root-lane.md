---
id: ADR-0259
title: Retire the private native root lane
status: proposed
date: 2026-11-09
deciders: [paul-heyse]
level: decision
principles: [AP-07, DP-10, DP-16, DP-19]
blueprint: [§18.8]
review: "not-required: retirement of an unconsumed opt-in private experiment within existing native execution contracts"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported root workflow requires independently placed execution and cannot use the existing performing owners.
verification: No live caller, launcher, generated protocol or fixture selector remains; retain ordinary KINSOL and native-session lifecycle evidence and supplier transfer controls.
---

# ADR-0259: Retire the private native root lane

## Context

The [Plan66 Symbolica review I07](../plan_improvement_reviews/plan66-symbolica-leverage-2026-11-09.md#I07)
found only acceptance callers for the provider-free root experiment. The maintainer selected
retirement if the deployment inventory stayed clear. Workspace runtime and deployment sources
contain no supported performing caller; the launcher is explicitly prepared for its own tests.

## Scope

Replace only the private root placement direction of proposed ADR-0215 and ADR-0246. Generic
native-session custody, the selected scientific-worker protocol and Symbolica's admitted numerical
transfer mechanisms are independent.

## Drivers

Remove an unused process boundary and its generated, launch and fixture surfaces without losing
working native root execution or symbolic authority.

## Options

Retain the opt-in process as a distinct supported operation, or retire it and keep existing native
performing owners. There is no consumer for the separate process, so retire it.

## Outcome

Delete the private root runtime route, launcher, generated protocol and explicit fixture. Retain
shared supplier transfer tests at their owning layer; they supply no canonical restore authority.

### Consequences

The private root executable and its fixture selector no longer exist. No compatibility surface is
provided.

### Compensating controls

Preserve ordinary root execution, parked service-owned native sessions, and provider worker
admission and settlement.

### Confirmation

Plan66 C04/B03 own source-to-consumer removal and focused verification.

## Pros and cons

Reduces the operational and generated surface; a future isolated consumer needs an explicit placement decision.

## More information

See the [current placement decision](../plans/66c-preparation-execution-and-service-lifecycles.md#scientific-wire-retirement).

## Status history

- 2026-11-09 — Proposed after maintainer selection and a fresh caller inventory.
