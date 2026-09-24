---
id: ADR-NNNN
title: TITLE
status: proposed
date: YYYY-MM-DD
deciders: [paul-heyse]
level: decision
principles: [DP-00]
blueprint: [§0.1]
review: not-required: REASON
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An observable trigger, not a date.
verification: The test, lint or CI job that shows this holds.

---

# ADR-NNNN: TITLE

*Delete the guidance under each heading as you fill it in. A small deviation
fills each section in one line — design principles §H: "a short, concrete record
is enough". Cite `blueprint §N.M` rather than restating the design.*

## Context

Two or three sentences: the situation that forces a decision, the blueprint
section it governs, and the design-review finding that raised it, if any.

## Scope

What this record binds and what it deliberately leaves open. A `must-gap`
narrows the supported scope here; it never claims compliance. If the record
*amends* the blueprint rather than deviating from it, say so — that is why its
level is `decision`.

## Drivers

The forces, one line each: correctness, reproducibility, supply chain, parity,
cost. The principle IDs (`DP-nn`, `PS-nn`) go in the `principles:` field, not here.

## Options

Each option in one line, with the reason it was not taken. "Do nothing" counts.

## Outcome

The decision, stated as a rule someone can follow or break.

### Consequences

What this forces on the implementation, including the unpleasant parts.

### Compensating controls

What keeps the downside bounded: a lint, a pin, a register row, a fallback.

### Confirmation

How anyone can tell the decision still holds — the same thing named in
`verification:`, plus where its output lands.

## Pros and cons

A short list, or a two-row table, only when the options were genuinely close.

## More information

Links: blueprint sections, the design-review finding, the capability-map
evidence, the register row, the plan that implements it.

## Status history

- YYYY-MM-DD — proposed.
