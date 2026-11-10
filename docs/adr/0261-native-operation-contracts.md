---
id: ADR-0261
title: Declare native operation contracts at the source owner
status: proposed
date: 2026-11-09
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-06, AP-07, DP-24, PS-13]
blueprint: [§4.1, §20.5]
review: "docs/functional_reviews/surrealdb-functional-improvements-2026-11-09.md#O01"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A consumer requires explicit database rank or a new stored presence meaning.
verification: Plan72a declaration-to-field/function/readback, mutation-presence and native unsigned boundary controls; independent schema coverage controls.
---

# ADR-0261: Declare native operation contracts at the source owner

## Context

Plan72a improves the registry-owned native schema and codecs under blueprint §4.1/§20.5.
Literal vocabularies repeat membership; read rows conflate operation input shapes.
The maintainer accepted RC01 and breaking target designs on November 9.

## Scope

Amend native projection and mutation input contracts. Registry declarations remain authoritative.
Scientific hashing, frames, identity and operation authority retain existing meaning.

## Drivers

A vocabulary or field policy change should update database validation, generated inputs and
inspection together. Full unsigned values must survive storage/query/wire/read boundaries.
Independent malformed-value controls must detect shared generator omissions.

## Options

Repeated unions and row-shaped writes retain duplication. A separate editable schema introduces
competing meaning. Owner-derived constraints and operation-shaped conversion keep changes local
while preserving native engine execution and independent admission.

## Outcome

Generate named constrained strings for existing vocabularies, retaining stored spellings and
string ordering. Native ordered unions require explicit rank meaning. Create supply/defaulting,
merge unchanged/set/removal and complete replacement remain distinct. Readonly permits initial
supply where declared and forbids later mutation; it does not mean server-derived. Input types
grant no write authority.

### Consequences

Move canonical/private producers and consumers together. Native U64 storage uses exact numeric
coercion and checked Int/Decimal Proto widening. Timestamps and IEEE bits retain existing meaning.
Changed native interpretation advances explicitly; scientific frames do not move incidentally.

### Compensating controls

Retain transition, cross-field, issuer and lease checks. Native type/extension witnesses cover
SurrealKit v2 gaps; tool diagnostics/provenance remain separate from its digest. Ordinary open
never migrates or resets state.

### Confirmation

Pinned named-type, numeric coercion and Proto interfaces were inspected. Plan72a owns targeted
implementation evidence; Plan72i owns assembled receiving. No executed acceptance is claimed here.

## Pros and cons

One source removes repeated mappings; delivery requires coordinated generated outputs, callable
inputs, installed witnesses and consumers.

## More information

[Plan72a](../plans/72a-native-contracts-and-schema-interpretation.md) owns execution and handoffs.
[Functional assessment](../functional_reviews/surrealdb-functional-improvements-2026-11-09/contracts-and-schema-evolution.md)
records alternatives and supplier limits.

## Status history

- 2026-11-09: Proposed before dependent native contract implementation under authorized RC01.
