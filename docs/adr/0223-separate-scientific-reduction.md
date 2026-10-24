---
id: ADR-0223
title: Separate scientific reduction from original reproduction
status: proposed
date: 2026-10-23
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-04, AP-06, DP-09, PS-10]
blueprint: [§24.1]
review: "not-required: short runtime document extension within existing diagnostic execution authority, explicitly selected by Plan52 RC05"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A concrete reduction family needs different branch, validity or provider correspondence.
verification: Reduce the authored negative-log diagnostic while refusing parser errors, unrelated singularities and different domain violations; retain originals, candidates and source mapping.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: [docs/plans/52c-behavioral-and-scientific-verification.md#C06]
---

# ADR-0223: Separate scientific reduction from original reproduction

## Context

Strict reproduction compares an original typed diagnostic, sources and conditions. Source edits
necessarily change identities, so weakening that predicate would make historical reproduction
untruthful. Plan52 C06 instead needs smaller examples retaining a selected numerical issue.

## Scope

Add separate reduction request/result documents and a source-aware consumer of existing diagnostic
execution (blueprint §24.1). Original reproduction, scientific hashes and execution ownership stay
with their current owners.

## Drivers

Retain an interpretable authored domain failure after deleting irrelevant work. Distinguish a
preserved issue from parser refusal, changed failure, incomplete execution and unavailable evidence.
Use existing runtime admission and cancellation rather than a new execution engine.

## Options

Manual subtraction remains sufficient for small cases. Weakening the original reproducer loses
its exactness. A generic scientific-equivalence framework adds unsupported meaning. Select a
bounded source-aware search with explicit predicates and correspondence for one supported family.

## Outcome

Reduction has separate identity and evidence. Every accepted candidate preserves the selected
code, stage, rule, domain premise and source correspondence; different failures never count.

### Consequences

First support authored numerical evaluation-domain failures. Retain original and reduced text,
explicit source correspondence, candidate classifications and the selected search bounds.

### Compensating controls

Refuse unsupported frozen/provider families and cancellation, resource or dependency failures.
Report local irreducibility only under the selected moves and exhausted bounds.

### Confirmation

The owning Plan52c C06 records actual functional checks; this proposed decision and its declaration
alone establish no executed reduction or assembled qualification.

## Pros and cons

Separate documents preserve exact reproduction and make reduction limits explicit. Each later
failure family must supply its own meaningful correspondence instead of inheriting a generic match.

## More information

[Plan52 C06](../plans/52c-behavioral-and-scientific-verification.md#C06) owns implementation and
[the coordinator](../plans/52-agent-effectiveness-and-engineering-workflows.md) owns RC05 status.

## Status history

- 2026-10-23 — proposed; maintainer authorized implementation of Plan52c/52d.
