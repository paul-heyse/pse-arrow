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
verification: The scenario/property and named analysis, test, lint or measurement that settles it.
# Optional snapshot and links; remove if not relevant.
standard: null
scenarios: []

---

# ADR-NNNN: TITLE

*Delete the guidance under each heading as you fill it in. A small deviation
fills each section in one line — design principles §H: "a short, concrete record
is enough". Cite `blueprint §N.M` rather than restating the design.*

## Context

Two or three sentences: the situation that forces a decision, the architecture
section it governs (`blueprint §N.M`), and the design-review finding that raised it, if any.

## Scope

What this record binds and what it deliberately leaves open. A `must-gap`
narrows the supported scope here; it never claims compliance. If the record
*amends* the architecture sections rather than deviating from them, say so — that is why its
level is `decision`.

## Drivers

Name the functional and architectural drivers and link representative change scenarios.
Explain the required responsibility boundaries, composition and local test setup. Principle
IDs (`AP-nn`, `DP-nn`, `PS-nn`) go in `principles:`; scenario definitions retain one owner.

## Options

Compare the current baseline, proposed design and simplest viable/library-owned alternatives
as relevant. State change locality, contract and testing effects, integration cost and the
reason for selecting or rejecting each option. Rows may coincide.

## Outcome

The decision, stated as a rule someone can follow or break.

### Consequences

What this forces on the implementation, including the unpleasant parts.

### Compensating controls

What keeps the downside bounded: a lint, a pin, a register row, a fallback.

### Confirmation

How the scenario/property in `verification:` is established and where evidence lives.
Separate architectural reasoning, document checks and executed behavior. The owning plan
tracks current implementation; ADR acceptance does not certify that work.

## Pros and cons

A short list, or a two-row table, only when the options were genuinely close.

## More information

Links: architecture sections, the design-review finding (or its `git:` source once
retired), the capability-map
evidence, the register row, and the plan/disposition owner that implements it.

## Status history

- YYYY-MM-DD — proposed.
