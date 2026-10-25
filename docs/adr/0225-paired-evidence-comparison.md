---
id: ADR-0225
title: Compare explicit evidence populations through their native owner
status: proposed
date: 2026-10-24
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-04, AP-07, DP-09, DP-21, DP-23]
blueprint: [§23.1, §24.1]
review: "not-required: short native evidence kernel contract within existing Rust-owned diagnostic boundaries; maintainer authorized Plan52 F01 execution"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required comparison cannot preserve exact correspondence and source custody through the finite native evidence operation.
verification: Native paired-evidence tests independently specify pairs, ambiguity, denominators and measurement incompatibility; canonical and offline consumers agree on the same supplied cuts.
---

# ADR-0225: Compare explicit evidence populations through their native owner

## Context

Plan52 F01 needs baseline/candidate comparison through the existing evidence owner. Its finite
questions and selected section readers already preserve original sources and population meaning;
interface-local heuristics would create another authority for pairing and completeness.

## Scope

Add a finite native comparison contract within blueprint §23.1/§24.1. Public scientific inputs,
execution outcomes and source identities remain owned by their existing producers.

## Drivers

Comparisons must distinguish real changes from missing execution, ambiguous identity and changed
selection. Offline and canonical consumers need the same interpretation under bounded detail.

## Options

Retain single-population queries alone, compare in host middleware, or extend the existing native
analysis. Select the native extension: sources parse once, existing native analysis owns correspondence and relational
denominators, and consumer adapters need no duplicate semantic model.

## Outcome

Compare explicit source populations using exact compatible owner identity or a supplied mapping.
Never invent correspondence from names or nearby timestamps. Preserve unmatched/ambiguous subjects,
full denominators, independent section completeness and incompatible measurement conditions.

### Consequences

Native/offline and MCP consumers migrate together. Bounded display does not narrow semantic
populations; original scientific and qualification outcomes are never rewritten by comparison.

### Compensating controls

Independent small fixtures include failure→pass, persistent failure, unexecuted/ambiguous subjects,
selection changes and noisy incompatible measurement populations. Existing protected readers retain custody.

### Confirmation

Proposed; implementation and exact command/evidence limits belong to Plan52 F01/H02 Outcomes.

## Pros and cons

One native owner avoids divergent interpretation, while requiring receiver/schema and actual
consumer migration rather than an isolated presentation helper.

## More information

[Plan52 F01](../plans/52f-evidence-analysis-and-trace-correlation.md#F01) owns delivery;
[the coordinator](../plans/52-agent-effectiveness-and-engineering-workflows.md) owns progress.

## Status history

- 2026-10-24 — proposed before dependent implementation.
