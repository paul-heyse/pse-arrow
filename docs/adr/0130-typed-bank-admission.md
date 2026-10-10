---
id: ADR-0130
title: Admit sourced records and bounded binary data banks
status: proposed
date: 2026-09-30
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-05, AP-06, DP-03, DP-09, DP-20, DP-21]
blueprint: [§6.1, §6.2, §6.15.1, §9.10, §22.1]
review: docs/design_review/reviews/design_review_typed-domain-model_2026-09-29.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A supported bank cannot use the common typed admission path, or document conversion bypasses resource accounting.
verification: Plan 23 E1 entity provenance, abstract-kind, keyed-document, binary worker round-trip, incremental-versus-clean and decoded-memory tests; final AUD assesses the changed boundary.
scenarios: [https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md#scenarios]
---

# ADR-0130: Admit sourced records and bounded binary data banks

## Context

Plan 23's remaining bank and catalogue scenarios expose missing entity provenance,
abstract-kind enforcement, keyed-kind documents and a binary Python input boundary.
ADR-0123 and ADR-0125 already own typed admission and data documents. This record
extends those mechanisms; the existing review identifies their motivating boundaries.

## Scope

Extend generic record and document contracts within the accepted typed-package target.
Science remains authored package knowledge. Keep this record proposed until the
scheduled Plan 23 AUD review accepts the changed boundary; implementation evidence
belongs in the owning plan rather than in this decision's initial evidence label.

## Drivers

SM1 requires attributable catalogue facts. DM1 requires keyed Parquet rows and binary
transport. DM4 requires transitive test-only taint. Large banks require bounded decoded
memory and reusable admission independent of the format adapter.

## Options

Per-bank Rust loaders duplicate typing and provenance. Putting a query engine in pure
modeling ties local admission to unrelated infrastructure. Extend the existing Arrow-free
row path, retaining Arrow/Parquet in runtime and Salsa as the admission reuse owner.

## Outcome

Entities and their supplied attributes may carry the existing typed provenance structure.
An entity supplies the default origin; specific attribute provenance cannot clear inherited
test-only dependencies. Sources may remain roots without their own provenance. Abstract
kinds cannot be instantiated; concrete refinements do not inherit the abstract marker.

Keyed-kind documents follow identity, reference and value-dependency admission phases.
Units and identifier schemes come from declarations. Text and binary documents cross
Python through one owned byte representation, preserving byte-exact durable transport.
Decode and admission charge transient and retained allocations before owned growth,
with explicit work limits and cancellation. No compatibility admission path is retained.

### Consequences

The modeling relation evolves and generated consumers migrate together. Provenance is
content, not entity identity. Existing numeric tolerances and source/role authority remain.

### Compensating controls

Synthetic pure tests cover invalid records, document boundaries, canonical identity,
transitive taint, resource refusals and incremental-versus-clean equivalence. Runtime
format adapters do not perform a second scientific interpretation.

### Confirmation

The Plan 23 E1 packet owns implementation and targeted acceptance. Final AUD and Q own
architectural assessment and comprehensive qualification. This record claims neither yet.

## Pros and cons

One admission contract serves inline and bulk data. The Python and registry consumers
must migrate together, and decoded-memory accounting has additional intermediate owners.

## More information

[Plan 23](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md), ADR-0123 and ADR-0125.
Enduring contract changes will be incorporated through the Plan 23 design amendment.

## Status history

- 2026-09-30 — proposed before E1 implementation under the maintainer-approved execution plan.
