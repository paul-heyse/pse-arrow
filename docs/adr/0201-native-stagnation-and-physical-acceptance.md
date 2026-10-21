---
id: ADR-0201
title: Separate native stagnation controls from physical acceptance
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-07, PS-06, PS-08, PS-12]
blueprint: [§16.6, §18.10]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A native release changes the stopping formula, or a qualified shifted/scaled problem demonstrates premature stagnation or unbounded work under the selected policy.
verification: Plan48d D01 compares physical residuals, actual KINSOL status and work for near-zero/large/shifted coordinates, residual scaling, accepted starts and unsuccessful stagnation under finite limits.
---

# ADR-0201: Separate native stagnation controls from physical acceptance

## Context

Plan48 AF12a identified a false equivalence in blueprint §16.6. The selected SUNDIALS 7.1.1 source computes scaled stagnation as `max(abs(step)/(1/uscale + abs(current_iterate)))`; a coordinate shift changes that denominator. Passing the physical feasibility scalar therefore does not make this an absolute physical-step test.

## Scope

Amend the KINSOL control meaning in blueprint §16.6 and §18.10 under accepted Plan48 RC01. Preserve original scientific budgets, including practical 0.001 endpoints where selected. Cone controls and other adapters remain separate inquiries.

## Drivers

The adapter must own the actual library control meaning for roots, implicit solves and recycles. A large original coordinate must not cause a relative stagnation threshold to be interpreted as an absolute success criterion. Work remains bounded by the enclosing cancellation, deadline and effort limits.

## Options

Retaining the current scalar with truthful wording is a valid conservative comparison. The native roundoff-scaled default separates stagnation from physical residual acceptance without introducing project-owned polishing. A new physical step criterion or universal maximum-step setting would require another independent policy and is unnecessary for this correction. Compare the native default against explicit method controls in Plan48 I12 before closing adoption.

## Outcome

KINSOL residual scaling continues to express original row budgets. Its scaled-step control expresses native stagnation only; `KIN_STEP_LT_STPTOL` remains a Limit with no convergence assurance, followed by independent original-quality assessment. The target policy uses KINSOL's native roundoff default, represented by its documented zero sentinel, subject to Plan48d's bounded native qualification. Explicit positive adapter settings retain their native meaning.

### Consequences

Large coordinates can require additional native iterations instead of stopping prematurely. The existing maximum-step API remains in place; no new unconditional cap or retry loop is introduced. Zero is a supported default sentinel, never a zero physical tolerance.

### Compensating controls

Every attempt retains finite limits and fresh original checks. Unknown or failed native mutation cannot authorize retained history. A native success is distinct from original physical acceptance.

### Confirmation

**Tested:** Plan48d D01's ten selected native control/submitted-start tests pass in the
solver/KLU/isolation/Uno/PETSc correctness closure, including near-zero, shifted and large
coordinates, scaling, actual native stagnation and independent original residual checks.
The zero roundoff-default policy does not change the practical physical tolerance.
Vendored SUNDIALS7.1.1 and its documentation establish the relative denominator and
zero-default semantics. The owning Outcome retains commands, conditions and the corrected
maximum-step fixture assumption; this is not qualification of every scientific model.

## Pros and cons

Native ownership avoids a duplicate nonlinear method and a fictitious absolute-step guarantee. Its relative denominator still depends on native coordinates; the independent original check is therefore indispensable.

## More information

- [Plan48d D01](../plans/48d-native-controls-starts-and-recycle-policies.md#d01--kinsol-control-meaning-and-native-policy)
- [KINSOL documentation](https://sundials.readthedocs.io/en/latest/kinsol/Usage/index.html)

## Status history

- 2026-10-20 — Proposed before the Plan48 RC01 governed control correction; formal acceptance status unchanged.
