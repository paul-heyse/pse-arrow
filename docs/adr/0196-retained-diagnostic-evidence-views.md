---
id: ADR-0196
title: Declare immutable diagnostic evidence views
status: proposed
date: 2026-10-17
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, PS-10]
blueprint: [§23.1]
review: docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A selected evidence query cannot preserve event membership, explicit unknowns or finite ownership under this declaration.
verification: Plan43e immutable evidence-view controls for exact source event keys, repeated and missing records, nonfinite/availability, admitted buffers and independent export failure.
---

# ADR-0196: Declare immutable diagnostic evidence views

## Context

Plan43e requires queryable retained evidence without mixing values from different native events. Existing source JSONL and diagnostic capture own raw evidence and retention; a derived Arrow view supplies analysis, not scientific publication.

## Scope

Add derived diagnostic field, invocation and capture relations within blueprint §23.1. The maintainer delegated complete Plan43 choices; formal ADR acceptance remains separate.

## Drivers

Queries need exact operation/producer/generation/event membership, field ordinals, units, availability and coverage. Live observation must remain independent from analytical export and scientific storage.

## Options

Inferring schemas or flattening only numeric values loses meaning. A second persistent diagnostic store duplicates ownership. Select one registry-declared Arrow projection over existing immutable source pages, using existing allocation leases and DataFusion table capabilities.

## Outcome

`runtime.diagnostic_evidence_fields` declares the field projection. `runtime.diagnostic_invocations` and `runtime.diagnostic_captures` preserve the actual joined invocation and optional capture documents, their endpoint/coverage meaning and exact operation links. Event and field ordinals preserve membership; zero field ordinal preserves an empty event. Typed values and explicit unavailable/nonfinite categories remain distinct. Source pages remain the raw authority, and repeated event identities refuse this projection rather than silently deduplicate them.

### Production-completeness correction — 2026-10-17

**Interface-checked:** Launch captures extend the existing capture projection with an explicit process subject and optional operation link; the derived key identifies the actual operation or launched process without inventing a diagnostic operation before execution publishes one. Numerical diagnostic exports retain optional originating invocation/attempt/failure context. These references preserve interpretation only and do not grant scientific use, replay, cancellation or publication permission. Changed view/document versions migrate all actual consumers through the registry and existing generators.

### Consequences

Rust/Python readers select finite pages and finite analytical memory. Escaped Arrow buffers retain their admitted lease; DataFusion consumes the same schema and buffers. Views preserve partial coverage and cannot upgrade scientific permission or imply a complete history.

### Compensating controls

No new scheduler, store writes, SQL parser or charge ledger. Exact times remain seconds/nanoseconds. Export failure leaves raw evidence and target science intact; queries choose compatible families, products and outcomes explicitly.

### Confirmation

**Tested, 2026-10-17:** actual registry-derived Arrow/DataFusion views preserve source keys, nullable unavailable fields, exact unsigned sequences, leased lifetime, repeated-key refusal and memory/independent projection failure. Actual Python and Trace Processor consumers passed. Plan43e/43g Outcomes own selected/composite conditions; derived views remain observational and do not establish causal certainty or scientific validity.

## Pros and cons

The projection enables ordinary Arrow/SQL analysis with one authority. Field rows require explicit event-key joins, which preserve the original coherent boundary instead of guessing simultaneity.

## More information

[Plan43e](../plans/43e-diagnostic-analysis-artifacts-and-profiling.md), [Plan43](../plans/43-solver-diagnostics-and-performance.md).

## Status history

- 2026-10-17 — proposed before dependent relation implementation under delegated execution authority.
