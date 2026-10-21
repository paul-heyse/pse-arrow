---
id: ADR-0204
title: Bound native cone refinement by original quality and remaining work
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-07, PS-08, PS-12]
blueprint: [§18.6, §18.10]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A cone/recovery case defeats the bounded correction, or a native release provides a directly original-budget-aware stopping facility.
verification: Plan48d I13 uses independent SOC distances and PSD eigenvalues across magnitudes/coefficients, finite failure and actual native status/work; production adoption retains original checks and tests no retry after cancellation, deadline or iteration exhaustion.
---

# ADR-0204: Bound native cone refinement by original quality and remaining work

## Context

Clarabel0.11.1 checks relative primal/dual feasibility. Its denominator depends on data and
iterate norms; the shared physical scalar cannot make that a per-row original-unit test.
Plan48 I13's million-scale SOC returned native Solved with original violation0.0626 against
0.001. Simply reducing the configured tolerance by that violation ratio repeated the same
trajectory. POUNCE-convex has a different absolute/gated-relative test and is assessed separately.

## Scope

Amend blueprint §18.6/§18.10 for one bounded Clarabel working refinement, within the existing
adapter and the original attempt's time/iteration permission. Physical policy, scientific
qualification, routing defaults, native algorithms and public settings remain unchanged.

## Drivers

A native-success candidate failing original quality can justify additional library work;
it cannot justify weakening acceptance. The adapter has the actual residual, scaling and
native session, while workflows should consume truthful outcomes without reproducing them.
The correction must not introduce a host Newton loop, unbounded retries or another solver path.

## Options

Always tightening a global scalar imposes unnecessary work and still cannot guarantee original
acceptance. A bound based only on RHS magnitude misses iterate/normalization effects. Native
original-budget stopping is not exposed by this binding. One observed-residual-based retry
is a bounded local correction with independent final assessment; retaining the original
failure remains correct whenever the correction is ineligible or unsuccessful.

## Outcome

Only after native success with finite failed original quality, positive finite native primal
residual and remaining permission may the adapter make one additional native solve. Its
working feasibility tolerance is the smaller of one eighth of the preceding native tolerance
and `observed_native_primal_residual/(8*original_quality_ratio)`. It must be positive, finite
and strictly smaller. This is a working heuristic, not an equivalence or convergence promise.
No retry follows native failure, a certificate, validation failure, cancellation, expired
clock or exhausted iterations. The retry receives remaining iterations and wall time, shares
the original Execution and uses Clarabel's own fresh-start solve. Each actual attempt retains
its native status, native working tolerance, original assessment summary and work; total native
iterations include both. Only the final actual native status and independent original-quality
assessment determine the returned candidate's meaning.

### Consequences

Some native-success/original-failure cases cost one extra cold solve and may remain failures.
At magnitude1e9 the tested retry returned NumericalError despite original feasibility; it
stays an error. No multiplier, warm state or external iterate capability is invented.

### Compensating controls

The original budgets remain unchanged, the attempt count is at most two, and retry work never
renews the enclosing clock or iteration allowance. A zero/nonfinite residual or underflowed
working target retains the original failure. Reporting summaries are finite and bounded;
full candidate/quality storage from the first attempt retires before constructing the second.

### Confirmation

**Tested:** private native inquiry `conic::control_tests::bounded_observed_cone_refinement_probe`
on Linux, selected native-solvers profile,100 total iterations/2seconds and original0.001,
reduced million-scale SOC violation to approximately5.3e-8 in4+6 iterations. Billion-scale
reported NumericalError in6+11; no success was inferred from its candidate. Independent
SOC/PSD magnitude/coefficient probes establish limits of scalar mappings. These inquiry results
support this bounded target, not production qualification or arbitrary cone guarantees.

**Tested:** the five `conic::refinement::tests` production controls pass within the
28-test native-local backend selection at unchanged original0.001 on2026-10-20
(100 total iterations/2seconds for the primary SOC/PSD cases). They cover both
attempts' total-work accounting, exhausted/unknown/failed retry refusal, a remaining
iteration limit, independent original SOC/PSD checks, and cancellation from report
publication on either return path. Existing conic controls pass in that selection.
This is focused adapter acceptance, not arbitrary-cone or assembled qualification.

## Pros and cons

The native adapter owns the correction and preserves all final scientific checks. It pays for
one new solve only after an observed original-quality failure. The observed ratio is not a
proof about future iterates, so failure remains an explicit supported outcome.

## More information

- [Plan48d](../plans/48d-native-controls-starts-and-recycle-policies.md#d02--per-adapter-cone-controls-and-bounded-refinement)
- [Clarabel settings](https://clarabel.org/stable/api_settings/)

## Status history

- 2026-10-20 — Proposed before bounded production refinement; formal ADR acceptance unchanged.
