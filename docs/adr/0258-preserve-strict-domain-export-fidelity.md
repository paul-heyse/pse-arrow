---
id: ADR-0258
title: Preserve strict-domain fidelity in native export
status: proposed
date: 2026-11-09
deciders: [paul-heyse]
level: decision
principles: [DP-08, DP-15, PS-06, PS-10]
blueprint: [§18.10.1]
review: "not-required: scoped correction of an existing SCIP export rule; see Plan66 Symbolica improvement I06"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A selected native backend can faithfully encode an open domain without restricting the original feasible set.
verification: Below-margin feasible point, unattained infimum, reachable partial operator at an unsafe endpoint, and an unhoisted conditional guard; verify original candidate qualification and relaxed bound provenance in native execution.
---

# ADR-0258: Preserve strict-domain fidelity in native export

## Context

The [Plan66 Symbolica review I06](../plan_improvement_reviews/plan66-symbolica-leverage-2026-11-09.md#I06)
showed that SCIP's inward strict-range margin can remove original feasible points while the
export still reports exact fidelity. For `min x` on `0 < x ≤ 1`, it can produce a lower bound
above the original infimum, even if its candidate passes the original checks.

## Scope

Correct SCIP factorable export and its evidence under ADR-0105/0106. ADR-0249's native
predicate representation and IBEX semantics remain unchanged.

## Drivers

Native global evidence must cover every original feasible point. A candidate's original
feasibility cannot justify an export bound for a narrower model.

## Options

An inward margin is a restriction. An outward closed relaxation is sound only when the
exported expressions can be evaluated across its relevant closure; refusal preserves the
contract when that premise cannot be established.

## Outcome

At the original endpoint, export a sound closed superset as `Relaxed`, or refuse where
safety is unestablished. Use one rule for strict obligations and implicit bounds. A
conditional guard that is not exported or discharged reduces fidelity; it is not hoisted
into an unconditional constraint. An inward margin could only serve an explicitly
candidate-only operation; there is no current consumer, so remove it.

### Consequences

Some formerly exact claims are relaxed or unsupported. Native exact arithmetic does not
override semantic export fidelity or make an exact-only request eligible for a relaxation.

### Compensating controls

Check reachable partial operators using admitted closed bounds and the effective native
operator settings. Continue independent qualification in the original domain.

### Confirmation

The maintainer selected this direction on 2026-11-09. D01/D04 and L retain separate
targeted and assembled acceptance evidence.

## Pros and cons

Preserves global bound meaning without an arbitrary margin, at the cost of refusing
certification when the available bounds cannot establish safety.

## More information

[Plan66 decision](../plans/66-integrated-simulation-target.md#confirmed-strict-domain-export-direction).

## Status history

- 2026-11-09 — Proposed; maintainer selected the direction before implementation.
