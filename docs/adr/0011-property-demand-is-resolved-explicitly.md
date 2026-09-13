---
id: ADR-0011
title: Adopt D8: a pass computes the property closure; inspection never constructs physics
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-20, DM-19, DM-26]
blueprint: [§D8, §9.6]
review: not-required: neither review raised a finding against D8
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A property provider needs to be selected at evaluation time rather than at P6, for example because a method's applicability depends on a solved value
verification: `python / test` `test_preflight_capability_backend` (raises before `ConcreteModel` construction); P6 golden tests in `tests/conformance`

---

# ADR-0011: Adopt D8: a pass computes the property closure; inspection never constructs physics

## Context

Blueprint D8 removes lazy attribute construction: a pass computes the closure of required properties from equations, selects providers by type and configured preference, persists the selection, and generates the property equations or kernel bindings.

## Scope

Binds when property demand is resolved and where the selection is stored. Provider implementations are §9.8 and ADR-0022.

## Drivers

IDAES builds properties on attribute access, which makes inspection mutating (charter DM-20) and makes "what does this model need" unanswerable without running it.

## Options

Lazy construction on access — rejected: inspection would construct physics and the model would not be a value. Require authors to list properties — rejected: the closure is derivable (D2).

## Outcome

P6 resolves property demand explicitly and persists the selection; inspection is semantically non-mutating.

### Consequences

A property that cannot be provided is a typed refusal at compile time naming the capability, not a runtime attribute error.

### Compensating controls

`test_preflight_capability_backend` asserts the refusal happens before any Pyomo object is built; the closure report's `method` category records unresolved demand.

### Confirmation

P6 golden relations pin the selection for each reference package; `python / parity` compares against IDAES's chosen methods.

## Pros and cons

Resolving eagerly costs compile time for properties a case may never evaluate; the benefit is that the model is inspectable without being executed.

## More information

Blueprint §D8, §9.6 (property demand resolution), §9.3 (method registry).

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
