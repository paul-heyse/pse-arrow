---
id: ADR-0100
title: Compose typed functions and indexed accounting as data
status: proposed
date: 2026-09-26
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-04, AP-05, AP-06, DP-13, PS-01]
blueprint: [§7.3, §6.15.1, §14.3]
review: docs/design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A scientific port needs model-specific Rust or a synthetic kernel contract fails.
verification: Wrong-basis rejection, demand/default controls, grouped dispatch, analytic partials, transfer pairing and independent closure tests.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/21-modeling-kernel.md]
---

# ADR-0100: Compose typed functions and indexed accounting as data

## Context

Indexed scientific laws require generic ownership, physical typing and independent closure, while libraries already own symbolic differentiation. This records Plan 21 decisions A6–A7; R4–R5.

## Scope

The Plan 21 target and its authorized K0–K8 implementation boundary, including authored
scientific functions, schemes and branch-scoped realizations. This amends the target
contract; qualification and finding dispositions belong to the execution packets and Plan 21.

## Drivers

Data-only extension, complete physical meaning, explicit dependencies and locally testable
semantic operations. Plan 21 owns execution and finding dispositions.

## Options

A Rust enum per correlation or scalar law repeats scientific knowledge. Custom differentiation duplicates Symbolica. Generic typed contributions and library-owned mathematics cover the variation axis.

## Outcome

Use quantity-polymorphic functions and Symbolica partial derivatives behind pse-math. Resolve interface defaults before lazy demand. Conservation and accounting share generic indexed contributions with explicit roles and transfer pairing; closure evaluates original contributions independently.

### Consequences

Through K8, numerical schemes are checked data and accelerators are explicit capability
references. Nested flash uses branch-scoped derivatives with verified admissibility;
unproved transitions and ties refuse rather than claiming selector smoothness. Caloric
defaults consume declared primitive functions and reference values, with derivative
conformance to heat capacity; general symbolic integration is not presumed.

Pure functions may reference immutable package entities, sets, enumerations and
tables through their admitted lexical/import closure. Runtime state remains an
explicit argument; definition-local captures are refused. Specialization retains
the referenced data and embeds selected values in the mathematical product, so
dataset edits invalidate affected compiled results. Structural interface parameters
are checked values resolved before guards and memberships, not solver coordinates.

K4–K7 implements grouped bodies through existing CasePlan assembly, shaped external
functions with explicit derivative sources, and symbolically verified piecewise joins.
Implicit realizations consume an injected inner solver; residual-verified roots receive
faer implicit derivatives. Regime transitions must meet the requested smoothness order.
Original additive terms and contributions remain independent diagnostic outputs.

Pulling a variable and defining equation together does not prove well-posedness. Structural validation remains mandatory; derivative domain obligations survive simplification.

### Compensating controls

Typed refusal, stable semantic identities, bounded expansion and targeted positive/negative
controls accompany each mechanism. Remove an old path only after its replacement and callers move.

### Confirmation

Wrong-basis rejection, demand/default controls, grouped dispatch, analytic partials, transfer pairing and independent closure tests. Target review supports architectural reasoning only; executed evidence belongs
to the owning packet, not a retrospective change to this decision's evidence.

## Pros and cons

The generic contract localizes future science changes to packages. Its initial language,
registry and physical-type migration costs are larger than adding another special case.

## More information

[Plan 21](../plans/21-modeling-kernel.md) and its companion documents own implementation
sequencing. [Target review](../design_review/reviews/design_review_modeling-kernel-k0-k3_2026-09-26.md).

## Status history

- 2026-09-26 — proposed; implementation authorized by the maintainer. Acceptance remains on the decision-PR route.
