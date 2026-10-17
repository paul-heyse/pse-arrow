---
id: ADR-0194
title: Separate dynamic output coverage from trajectory advancement
status: proposed
date: 2026-10-17
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-04, AP-07, PS-06, PS-09, PS-10]
blueprint: [§13.6]
review: docs/design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: An admitted product cannot preserve source attribution, scientific validity or bounded execution under this contract.
verification: Plan43d covered-output exhausted-budget, root/reset/sample ordering, multiple segments, replay, independent backward directions and equivalent dense/sparse output-grid controls.
---

# ADR-0194: Separate dynamic output coverage from trajectory advancement

## Context

F08 identifies IDAS hard stops at requested samples. Native per-call step limits and restored checkpoint counters cannot alone establish cumulative effort. Pinned IDAS replay advances inside its library owner; Diffsol exposes accepted-step counters through owner wrappers.

## Scope

Amend blueprint §13.6. The maintainer authorized complete Plan43 execution and delegated the remaining concrete choices to the coordinator. This record precedes dependent implementation; formal ADR acceptance remains separate.

## Drivers

Interactive re-solve, studies and dynamics need demanded work, explicit semantic authority, independent original scientific checks and cancellation-safe ownership. Local tests must exercise the responsibility with its actual dependencies.

## Options

Keeping sample hard stops constrains the adaptive mesh unnecessarily. Removing stops alone changes budget/event behavior. Select one combined output/physical-boundary/effort contract, with a maintained IDAS replay guard at the actual library owner rather than conservative checkpoint-block reservations.

## Outcome

Profile Version2 interprets max_steps as accepted native advancement, independently for the complete forward/recording trajectory, all forward replay of one derivative operation, and each complete backward functional/direction trajectory. Physical segments, reinitializations, checkpoint restores and sample jumps do not renew permission. Covered output/interpolation and cotangent jumps consume zero advancement. Each backward direction has its own cap; this is not an aggregate cap across all directions.

### Consequences

Only genuine physical boundaries set hard stops. Root-aware native output service preserves event/reset ordering and consistent initialization. Forward remaining-step settings and maintained library-owner replay and per-backward-memory accepted-step guards enforce the contract at actual advancement. Retain IDAS NORMAL scheduling: ONE_STEP advances all active backward directions and can needlessly advance a direction whose output is already covered while another catches up. A backward guard is per underlying backward memory, survives checkpoint-call boundaries and preserves native output scheduling. Migrate public Rust/Python/nested requests to Version2; reject Version1 execution and regenerate design-phase inputs. Existing full-profile framed identity already consumes the changed version.

### Compensating controls

Keep failed native calls, rejected trials, callbacks, IC work and factors separate from accepted steps. Preserve outer cancellation, absolute deadline, existing WorkLimits, numerical accuracy, capability restrictions, trajectory prefixes and original qualification. Covered samples remain available at zero remaining steps; a further advance does not.

### Confirmation

**Tested, 2026-10-17:** linked native controls exercise covered output, cumulative forward/replay/backward effort, event/reset/sample and restored-producer behavior. Independent runtime nonlinear DAE/sensitivity/quadrature, action, fitting/shooting/covariance and managed interruption journeys passed. Plan43d/43g Outcomes own selected/composite conditions; no default latency winner or whole-model qualification is claimed.

## Pros and cons

The selected boundary removes incidental work and makes validity locally inspectable. It requires complete consumer migration and revealing failure controls rather than an additional compatibility path.

## More information

[Plan43](../plans/43-solver-diagnostics-and-performance.md), [source review](../design_review/reviews/design_review_solver-diagnostics-and-performance-target_2026-10-17.md).

## Status history

- 2026-10-17 — proposed before dependent implementation under delegated execution authority.
