---
id: ADR-0010
title: Adopt D7: balance, costing and utility laws are templates expanded over a contributions substrate
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-16, DM-17, DM-18, DM-25]
blueprint: [§D7, §10.1, §10.2]
review: not-required: neither review raised a finding against D7
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A law family cannot be expressed over contributions and needs a compiler rule of its own
verification: `tests/conformance` law-expansion golden tests per control-volume variant

---

# ADR-0010: Adopt D7: balance, costing and utility laws are templates expanded over a contributions substrate

## Context

Blueprint D7 makes material, energy, momentum, element and charge balances, and the costing and utility-minimization laws, instances of law templates expanded over a `contributions` substrate by the generic compiler. Control volumes are template compositions that declare which contributions exist.

## Scope

Binds how conservation laws are expressed. The contribution kinds themselves are §10.2.

## Drivers

IDAES writes one `build()` per balance variant; the variants differ only in which contributions exist; extension locality (charter §E) asks for one authoritative addition per new law.

## Options

One template per balance variant — rejected: the combinatorics of 0D/1D by phase/element/charge is exactly what IDAES pays for. A general equation authoring surface with no law concept — rejected: the balance structure is what diagnostics and scaling reason over.

## Outcome

Laws are templates; control volumes compose them by declaring their contributions; the generic compiler expands them (§10).

### Consequences

A new contribution kind is a registry addition; a new law family is usually a template, not code (§22.3).

### Compensating controls

Golden expansion tests per control-volume variant; the closure report's `balance` category is open while any law has unmatched contributions.

### Confirmation

`rust / test` runs the conformance layer; slice A and slice B acceptance (§24.2) exercise the lumped and distributed variants.

## Pros and cons

Templates are indirection; the alternative is the maintenance surface D2 exists to remove.

## More information

Blueprint §D7, §10.1 (conservation law template), §10.3-§10.4 (control volumes), §11 (unit model library).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
