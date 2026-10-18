---
id: ADR-0199
title: Carry consumed adjoint parameter coordinates to native work
status: proposed
date: 2026-10-18
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, PS-06, PS-10]
blueprint: [§13.6, §19.4]
review: docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md
evidence: "Proposed: Plan45 RC07 accepted; native and workflow source inspected; implementation and verification owned by 45d/D01."
supersedes: []
superseded-by: null
revisit: A consumer needs a derivative coordinate or second-order mapping absent from the explicit layout.
verification: D01 full/selected/empty Diffsol and IDAS adjoint controls, fitting/shooting mappings, schedule/anchor and second-order regression tests.
---

# ADR-0199: Carry consumed adjoint parameter coordinates to native work

## Context

Native adjoints currently construct complete parameter gradients before fitting and shooting project consumed coordinates. Plan45 F12 identifies unnecessary parameter products and quadratures; complete physical values and state adjoints remain necessary (blueprint §13.6).

## Scope

This adds an internal first-order adjoint parameter layout within the accepted native integration design. It amends blueprint §13.6/§19.4 without changing ordinary public full-adjoint meaning, forward-axis meaning, Hessian directions or scientific accuracy.

## Drivers

Interactive fitting and shooting should request the products they consume. One explicit ordered layout must compose through native admission, callbacks, initial/direct corrections and physical-to-consumer mappings, with independent backend tests.

## Options

Full-then-project preserves results but repeats unnecessary native work. Reusing forward-axis fields conflates different products. Select a separate All/Selected layout, with explicit empty demand, while native libraries continue owning integration and backward state evolution.

## Outcome

Carry unique ordered physical parameter coordinates through the complete first-order adjoint request, preparation identity, allocation admission, both native adapters and returned gradients. Dynamic profile identity uses a new v11 frame carrying the adjoint selection; historical v10 bytes retain their old meaning. Complete values and required state adjoints remain full. Ordinary public requests select All; Selected(empty) means no parameter-gradient coordinates, never fabricated zero derivatives. Tied/scaled, scheduled and anchor contributions map by the returned coordinate identity. Second-order callers explicitly retain the first-order demand they require rather than reinterpreting Hessian columns.

### Consequences

Every actual fitting/shooting consumer and common gradient caller must migrate with the native layout. Numerical width reduction alone does not permit reducing unknown native/factor allowances.

### Compensating controls

Reject duplicate/out-of-range coordinates before effects; retain original derivative capability checks, current cancellation, fresh trial IVP state and weaker completed products after failed upgrades.

### Confirmation

The accepted RC07 target supplies design authority. Plan45d/D01 owns implementation and targeted cross-backend full/selected/empty, mapping, failure and second-order acceptance. Plan45f owns assembled qualification; this proposed record claims neither.

## Pros and cons

Demand reaches actual library parameter products and quadratures without omitting physical effects. The tradeoff is an explicit layout migration across both adapters and workflow consumers.

## More information

[Plan45 rule decisions](../plans/45-pipeline-execution-optimization.md#rule-decisions), [D01](../plans/45d-demanded-derivatives-and-dynamics.md#d01--consumed-adjoint-parameter-gradient-coordinates) and [native trajectory contracts](../authoritative_design/sections/workflows-and-results.md#section-13-6).

## Status history

- 2026-10-18 — proposed before Plan45 D01 implementation; maintainer accepted RC07.
