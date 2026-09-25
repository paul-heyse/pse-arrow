---
id: ADR-0093
title: Qualify native strategies and residual dynamics
status: proposed
date: 2026-09-25
deciders: [paul-heyse]
level: decision
principles: [DP-02, DP-17, DP-21, PS-07, PS-10, PS-12]
blueprint: [§D9, §9, §13.6, §18.6]
review: docs/design_review/reviews/design_review_plan16-native-strategies_2026-09-25.md#12-decision
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A selected request needs hybrid sensitivities with recoverable residual trials or a variable-layout DAE.
verification: P07-P09 execution packet records force-validated default/native Rust suites, rebuilt Python journeys and required static checks.
---

# ADR-0093: Qualify native strategies and residual dynamics

## Context

Plan 16 P07-P09 extends the initially admitted ADR-0083/ADR-0084 native profile.
The maintainer selected Diffsol plus a narrowly qualified IDAS route. The approved
execution contract is recorded in [the packet](../plans/16-p07-p09-execution.md).

## Scope

Extend contextual routing, start and stage provenance, time interpretation and the
fixed-mass ODE/index-1 profile. IDAS implements the same authored model through a
residual adapter; it introduces no second compiler or general index reduction.
This proposed decision extends the initial supported profile, not historical M22
qualification. Blueprint reconciliation follows the decision PR route before acceptance.

## Drivers

Keep native algorithm ownership, preserve immutable model meaning, and make every
qualification and unsupported combination observable.

## Options

Diffsol alone cannot recover typed residual trial failures. A bespoke retry loop is
rejected. Retain its qualified hybrid helpers and add the pinned IDAS binding for
recovery-dependent smooth dynamics. No dependency upgrade is necessary.

## Outcome

Preparation owns contextual eligibility and effective thread policy. Native stops,
original numerical qualification, physical usability and seed availability are
separate. Allocation reuse never implicitly selects a numerical start. Stage
bindings are overlays, not updates to the original case. Time is converted once.

### Consequences

Public requests and result records change through the registry and existing typed
owners. Diffsol numerical time partials are labelled. The initial IDAS route refuses
hybrid fitting sensitivities; its smooth sensitivity and consistent initialization
contracts require consumer tests, not reference-skill receipts.

### Compensating controls

Explicit capability refusals, bounded native lifetime and history, independent
original-model checks, analytic derivative controls and one final scoped qualification.

### Confirmation

The execution packet records targeted acceptance and final qualification separately.
The scoped implementation self-review accepts the tested Linux profile. Native presolve
or QP regularization can produce a feasible candidate without original stationarity/gap
qualification; those outcomes and explicit stronger controls have consumer regressions.
Decision-PR acceptance of this record remains pending.

## Pros and cons

Library algorithms remain authoritative and model meaning stays shared. Two narrow
dynamic adapters require explicit capability composition and separate qualification.

## More information

[Plan 16](../plans/16-data-model-architecture.md), ADR-0088, ADR-0089 and ADR-0090.

## Status history

- 2026-09-25 — proposed before affected implementation; user authorized execution.
