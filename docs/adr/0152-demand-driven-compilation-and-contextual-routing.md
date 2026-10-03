---
id: ADR-0152
title: Prepare selected mathematics through contextual execution requirements
status: proposed
date: 2026-10-02
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, AP-06, DP-01, DP-04, DP-08, DP-09, PS-07, PS-09, PS-10]
blueprint: [§5.3, §14.1, §14.3, §14.4, §18.7, §19.3, §23.2]
review: docs/design_review/reviews/design_review_demand-driven-compilation-and-contextual-routing_2026-10-02.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A supported case requires a demand or contextual admission condition that its existing mathematical or adapter owner cannot express.
verification: Plan 25l S01-S06 focused demand, contextual readiness, failure meaning and preserving boundary controls; Plan 25k owns scientific and performance qualification.
standard: core-3.3 / process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s01, docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s02, docs/design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md#s06]
---

# ADR-0152: Prepare selected mathematics through contextual execution requirements

## Context

The full-case review's F01/F02/F03 show that semantic selection, mathematical preparation, contextual routing and completion do not compose at their current boundaries. Plan 25l retains the physical model, native adapters and mathematical libraries while correcting when consumer demand and scientific evidence govern execution.

## Scope

Amend blueprint §14.1/§14.3/§14.4 and §18.7, with relevant identity and generated diagnostic contracts in §5.3/§19.3/§23.2. No new mathematical engine, crate, DSL, supported equation class or numerical fallback is selected. Historical recorded contracts and digest bytes keep their interpretation.

## Drivers

Topology and Value/First consumers must not require unrelated executable/Second work. Solver substitution must follow actual scientific, representation and runtime requirements; absent preparation is not mathematical incapability. Policy must be testable with explicit inputs independently of native execution and storage.

## Options

Larger budgets and explicit fixture backends retain the causal coupling. Preparing every candidate's maximum requirements forces expensive optional work. A generic IR or solver framework duplicates stable semantic/library owners. Select demand-indexed products and contextual assessment through the existing compiler, math, runtime and adapter seams.

## Outcome

Semantic specialization is independently usable by topology. Executable outputs are selected with their complete mandatory closure before admission. Immutable value/control/domain programs and demand-indexed support/artifacts have distinct readiness, identity and ownership. Structural incidence retains conservative all-branch dependencies without claiming evaluated provider derivatives. Value constructs no numerical derivative support; First never constructs Second. Stronger products cannot mutate weaker products, reset budgets or memoize transient refusal.

Candidate assessment consumes an immutable case/profile/build-runtime snapshot. Shared routing owns intent, classes and ranks; adapter/settings/representation owners supply contextual requirements and refusals. This replaces the narrower §18.7 assertion that static capability-record admission alone suffices. Missing class, structural or scientific evidence returns explicit finite demands; missing artifacts remain preparation requirements. Selection may reconsider only established scientific/representation incompatibility, deterministically against the same snapshot. Resource, cancellation, infrastructure and native failure never select another solver. Final preflight verifies readiness before an attempt.

Completion keeps no candidate, unavailable quality, evaluated infeasibility, native termination and use policy separate. One owned exhaustive projection preserves public failure meaning. New vocabularies derive from the registry; public operation documents derive from Rust. Current identity preimages that change meaning receive new versions. Existing recording, directional admission and preserving migration operations govern durable changes.

### Consequences

The producer and all consumers of a changed contract migrate together and delete the replaced production path. Additional immutable support/assessment products retain existing allocation leases, service cache and flight owners. Runtime observations enter only identities whose decisions consume them; they do not invalidate unrelated symbolic bodies.

### Compensating controls

Selected-output/order counters, independent physical/domain checks, complete incidence/alias controls, failed-upgrade and clean/incremental controls, deterministic routing and no-fallback controls, exhaustive completion projection and generated/historical boundary controls establish the distinctions. No SHOULD exception or MUST waiver is proposed.

### Confirmation

**Implemented/Tested, 2026-10-02:** the bounded target review preceded governed code. Plan 25l records 124 selected Rust tests, five Python codec tests, final default/linked compilation and generation under its named Linux/license conditions. Assembled scientific and dev-profile measurement evidence remains Plan 25k scope. ADR status remains proposed pending its decision PR.

## Pros and cons

Demand follows consumed meaning and avoids unrelated work, at the cost of explicit readiness dependencies and more immutable product keys. Existing libraries continue to own differentiation, sparse algebra, structural algorithms and native iteration.

## More information

[Execution owner](../plans/25l-flowsheet-compilation-and-solver-routing.md); [finding disposition owner](../plans/25-design-remediation.md#full-case-compilation-and-solver-routing-follow-up); [diagnosis](../design_review/reviews/design_review_flowsheet-compilation-and-solver-routing_2026-10-02.md).

## Status history

- 2026-10-02 — proposed for authorized Plan 25l implementation; no publication or acceptance-status change implied.
