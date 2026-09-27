---
id: ADR-0111
title: Author multiple objectives by priority and weight; lexicographic natively on HiGHS, staged elsewhere
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [DP-11, DP-13, DP-14, PS-01, PS-04, PS-09]
blueprint: [§7.5, §18.7]
review: docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t12
evidence: Proposed
supersedes: []
superseded-by: null
revisit: HiGHS changes the semantics of its multi-objective interface, or a staged NLP level fails qualification because of its objective-bound constraint on a fixture where the degradation tolerance is declared.
verification: Plan 22 C3 tests lexicographic_milp_native and lexicographic_nlp_staged_matches_weighted_limit, plus the refusal controls for a dimensional weighted sum and for competing objectives without priority or weight.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s10]
---

# ADR-0111: Author multiple objectives by priority and weight; lexicographic natively on HiGHS, staged elsewhere

## Context

ADR-0101 (proposed) lets an analysis select one scalar physical member as its objective and
refuses competing objectives. Price-taker, design-and-operation and utility studies trade
objectives such as cost, emissions and throughput. HiGHS supports several linear objectives
with priorities, weights and per-objective degradation tolerances; no other linked backend
does (capability review [§8.4](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#84-coefficient-and-conic-highs-clarabel), L-C8).

## Scope

A kernel and routing contract. It refines ADR-0101's objective clause — competing objectives
without priority or weight are still refused — and records that refinement in ADR-0101's
status history; ADR-0101 is proposed, so nothing is superseded. Pareto-front generation is a
study over weights or bounds, not a solver feature, and is not decided here.

## Drivers

- **DP-13 and DP-14.** Use HiGHS's native lexicographic objectives where they exist; compose,
  rather than write, a multi-objective solver elsewhere.
- **PS-01.** A weighted sum adds quantities, so they must be dimensionless or explicitly
  normalized — the rule ADR-0101 already applies to elastic penalties.
- **PS-04 and DP-11.** A lexicographic stage bounds an earlier objective; with a zero tolerance
  the bound is active at the optimum and constraint qualification fails for interior-point
  NLP solvers. The degradation tolerance is part of the contract.
- **PS-09.** The route follows from the class.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Keep one objective only | Forces authors to hand-aggregate, hiding the trade-off | Rejected |
| A bespoke multi-objective solver | Solver machinery (G8) | Rejected |
| Weighted sums only | Cannot express strict priority | Rejected as the only form |
| Priority and weight members; native lexicographic on HiGHS; staged sequence elsewhere | Library where it exists, composition elsewhere, one authored form | **Selected** |

## Outcome

1. **Authoring.** `annotation objective` gains `priority` (an integer; a lower value is
   optimized first) and `weight` (a dimensionless real) members, and each priority level
   declares an absolute and a relative degradation tolerance. Several objectives are admitted
   only when every one declares a priority, or all of them share one priority with weights;
   otherwise they are refused as before.
2. **Within a level.** The level objective is Σ wᵢ·fᵢ. Each fᵢ must be dimensionless or carry an
   explicit declared normalization; mixing dimensions is refused.
3. **Across levels.** Lexicographic. Each later level is solved subject to
   fₖ ≤ fₖ* + max(absₖ, relₖ·|fₖ*|), sense-adjusted, for every earlier level k. A zero
   tolerance is admitted only on the native LP/MILP path; staged NLP, QP and MINLP levels
   require a positive tolerance.
4. **Routes.** LP and MILP use HiGHS's native lexicographic objectives
   (`Highs_passLinearObjectives`) with blending off and the declared tolerances. Everything else
   runs as staged-sequence steps (Plan 22 A6): each level is a full solve through the one NLP
   runner or SCIP, and each objective bound is added by a named transformation.
5. **Results.** Per-objective relations record the value, level, applied bound and the stage
   attempt. The final candidate is qualified in original coordinates like any other.

### Consequences

The annotation grammar and registry change (`just codegen`). Staged problems re-solve once per
level, so preparation must be reused (value-only rebind, A6).

### Compensating controls

Refusal of dimensional weighted sums and of zero tolerances on staged NLP levels; per-stage
records.

### Confirmation

The tests named in `verification:` (Plan 22 C3).

## Pros and cons

Priority, weight and tolerance cover the common engineering forms with one authored
declaration. A staged route costs one solve per level, which the value-only rebind keeps cheap.

## More information

- Architecture companion [§8](../plans/22-solver-capabilities-architecture.md#8-coefficient-and-conic-extensions).
- Target review [T12](../design_review/reviews/design_review_plan22-target_2026-09-27.md#t12).
- Related: ADR-0101 (objective clause refined), ADR-0105. Plan 22 packet C3.

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](../design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T12 were corrected in this record before acceptance.
