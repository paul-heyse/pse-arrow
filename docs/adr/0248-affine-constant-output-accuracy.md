---
id: ADR-0248
title: Keep affine-constant output estimates in original coordinates
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, PS-05, PS-06, PS-13]
blueprint: [§15.5.1]
review: "not-required: bounded original-output accuracy contract within authorized Plan64h and Plan65 execution; focused independent source assessment performed before activation"
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A required original-output case cannot establish source-anchored constant provenance or evaluate full original First derivatives at its admitted point.
verification: Boundary unary anchor and binary chain; bound-only constants and unanchored cycles refuse; moving dependents refuse; original affine residual and uncertainty affect output error and row demands; stale assumptions and original guard failures refuse; all three original exchanger comparisons.
---

# ADR-0248: Keep affine-constant output estimates in original coordinates

## Context

Plan64h's original exchanger accuracy goals reach a whole-square inactive-bound refusal.
Some coordinates are constitutively fixed by source equalities at a physical bound.
The existing source-affine projection and library reduction distinguish useful constant
coordinates, but library `Constant` recovery alone can originate from equal variable bounds
or rounded elimination and does not prove source tangent constancy.

## Scope

Amend blueprint §15.5.1 with a private, fixed-parameter output-accuracy profile. This
profile produces Estimated original-equation correction evidence. Public parameter
response and predictor admission retain their existing physical-interior requirement.

## Drivers

Reuse original mathematical products, error propagation and execution ownership. Preserve
all original rows, coordinate identities, physical acceptance, guards, task budgets and
independent comparison allowances. Do not make a feasible-root or reconstruction certificate
from a numerical affine recovery map.

## Options

Retaining unconditional interior refusal cannot assess fixed boundary inputs. Accepting
every library constant or dropping rows loses provenance or error contributions. A fully
reduced evaluator can support a broader population but adds projection, lifting and
reconstruction-uncertainty work. Select a narrower source-provenance profile that retains
the full original factor and its existing uncertainty/adjoint machinery. It is algebraically
equivalent to affine block elimination for the admitted anchored constants; its numerical
conditioning can be more conservative.

## Outcome

The backend produces a sealed accuracy frame from matching current source facts, original
scope, consumed fixed/parameter values, point, normalization and accuracy identity. Source
unary affine equalities with nonzero coefficients anchor constant coordinates; source binary
affine equalities propagate constancy from those anchors. Intersect that closure with
`pounce_presolve::linear_eq_plan::build_plan` zero-threshold `VarRecovery::Constant`
results. Equal bounds, unanchored cycles, numerical Jacobian zeros and rounded cancellation
never seed provenance. Nonzero source coefficients or offsets that underflow to zero refuse
numeric coefficient projection.

Only those proven coordinates may lack a physical interior in this profile. Every moving
coordinate must be strictly inside its physical bounds at the original point. Keep the
separate tolerance-based physical acceptance check: an acceptance budget is not a required
derivative-neighborhood radius. Evaluate and factor the complete original
Jacobian at the actual original point, and preserve complete original residual, arithmetic
uncertainty, gradient uncertainty, output uncertainty, correction and original-row demands.
Do not snap the point, zero its affine defects or return the private factor as a public
parameter-response capability. Full original First guard/selector checks remain required.

### Consequences

The existing estimator includes normal correction off the affine manifold and its effects
on nonlinear rows without a second residual/gradient projection. The witness includes source
provenance and point dependencies. Missing source facts, arithmetic uncertainty or actual
First support withholds evidence. The independent original physical feasibility check remains
tolerance-based; no exact feasible-root existence, two-sided parameter neighborhood or
Certified reconstruction claim follows, including when a rounded recovered constant lies
at a bound. This is a pointwise first-order output-error estimate; it establishes no
nonlinear remainder bound, correction-endpoint feasibility or finite-neighborhood guarantee.
The estimator neither evaluates nor applies the numerical correction as a nonlinear step.
An interval correction tube would serve a stronger future assurance contract and is not
required by this pointwise operation.

### Compensating controls

The public square-response path cannot consume the private constant exemption. Keep
Estimated evidence classification and original row-work refinement. Retain typed refusal
with every original offending coordinate, value, bounds, tolerance and margins. If an
original guard cannot be evaluated, investigate the separately admitted reduced-evaluator
contract instead of weakening guard or bound policy.

### Confirmation

Focused independent source assessment supports the block-elimination argument and its
limits. Implementation and executed controls belong to Plan64h/Plan65h; this proposed record
does not establish scientific acceptance or measured benefit.

## Pros and cons

The selected profile reuses the existing complete estimator and original evidence. It
deliberately supports fewer affine patterns and can factor a larger, worse-conditioned
system than a fully reduced evaluator.

## More information

- [Original scientific acceptance owner](../plans/64h-reference-coverage-and-qualification.md#receiving-scientific-comparison-checkpoint--november-5)
- [Source-affine proof](../../crates/pse-math/src/class_evidence.rs), [root library reduction](../../crates/pse-backend-native/src/presolve/roots.rs)
- [Original-coordinate estimate and row demands](../../crates/pse-backend-native/src/engineering_accuracy.rs)

## Status history

- 2026-11-06 — proposed before activating source-anchored original-output accuracy admission.
- 2026-11-06 — clarify strict point-local moving-coordinate interior independently of
  physical acceptance budgets, following focused source reassessment before activation;
  no change to proposed status or public parameter-response admission.
