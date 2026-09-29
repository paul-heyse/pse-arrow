---
id: ADR-0103
title: Declare a variable's domain as a typed facet with per-analysis-mode semantics
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [DP-01, DP-02, DP-03, PS-01, PS-04, PS-11]
blueprint: [§6.8, §18.1]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t15
evidence: Proposed
supersedes: []
superseded-by: null
revisit: An authored model needs a discrete domain the enum cannot express (a finite set of non-integer values, or an integer lattice other than a bounded range), or a backend requires a domain distinction the registry enum does not carry.
verification: Architecture scenario S10 and review scenario S06, settled by the Plan 22 M1/M2 tests var_domain_parses_and_renders, integer_requires_finite_bounds, binary_implies_unit_box, integer_requires_count_or_indicator_type, authored_milp_routes_to_highs, root_refuses_free_integer, initialization_fixes_and_restores_integers, fit_refuses_free_integer and duals_conditional_on_assignment.
standard: core-3.0/process-simulator-1.1
scenarios: [docs/plans/22-solver-capabilities-architecture.md#s10, docs/design_review/reviews/design_review_solver-capabilities_2026-09-27.md#s06]
---

# ADR-0103: Declare a variable's domain as a typed facet with per-analysis-mode semantics

## Context

`pse_math::binding::VariableDomain` already distinguishes continuous, integer, binary and
semi domains, and HiGHS consumes them, but the compiler hard-codes `Continuous`
(`grouped.rs`), so an authored model can never state a discrete decision
([F08](../design_review/reviews/design_review_solver-capabilities_2026-09-27.md#f08)a).
ADR-0102 admits discrete decisions into the target. Under ADR-0101 (proposed) annotations
carry numerical knowledge that may be omitted without changing a model's meaning; a domain
changes the problem class and therefore the meaning.

## Scope

A kernel contract within ADR-0099 (registry-defined language) and ADR-0101 (facts versus
annotations); neither is superseded. Binds the declaration syntax, the registry authority,
physical typing, bound admission and the semantics of a discrete variable in each analysis
mode. Constraint forms over discrete variables are ADR-0104.

## Drivers

- **DP-02.** A domain changes interpretation, so it is a type on the declaration, not a hint.
- **DP-01.** One enum owns the domain vocabulary from authoring to the native boundary.
- **PS-01.** A count or an on/off decision is dimensionless and must be typed as such; a
  semi-continuous flow keeps its physical quantity.
- **PS-04.** A discrete decision is a degree of freedom, and the role of every variable is
  explicit before a solver runs.
- **PS-11.** The same model serves optimization, simulation, initialization, dynamics and
  fitting; each mode states what a discrete variable means for it.
- **DP-03.** Finite bounds are an enforced admission invariant, not a documented expectation.

## Options

| Option | Assessment | Selection |
|---|---|---|
| `annotation domain` on a continuous variable | An annotation may be dropped without changing meaning (ADR-0101); a domain cannot | Rejected |
| A separate discrete-variable declaration kind | Duplicates the variable machinery; domain is orthogonal to role and quantity | Rejected |
| A case-level integrality switch | A case would change the model's meaning (D13) | Rejected |
| A declared domain facet on `var`, one registry enum, per-mode semantics | One authority; typing and admission at the declaration | **Selected** |

## Outcome

1. **Syntax.** `var <name>[<index>] : <QuantityKind> in <domain>;` with domain one of
   `continuous` (the default, which may be omitted), `integer`, `binary`, `semicontinuous`,
   `semiinteger`. Examples: `var units_on[u in units] : Indicator in binary;`,
   `var trays : Count in integer;`, `var steam : MassFlow in semicontinuous;`.
2. **One authority.** A registry enum `ModelingVariableDomain` with those five values is
   generated into `pse-model`. `pse_math::binding::VariableDomain` is deleted and `pse-math`
   consumes the generated type directly; `pse-model` already sits below `pse-math`, so no
   mapping layer exists. `runtime.solve_variables` gains a `domain` column, and
   `ProblemFacts` carries domains into routing.
3. **Physical typing.** `integer` and `binary` require a dimensionless quantity kind whose
   registry category is count or indicator. The semi domains keep the variable's physical
   quantity; the zero branch is exact and the active branch is bounded.
4. **Bounds.** `binary` implies [0, 1] and refuses conflicting bounds. `integer` and the semi
   domains require finite case bounds from `annotation bounds` or case values; admission
   refuses otherwise with a typed `Unsupported` cause naming the variable. Non-integral
   bounds on an integer variable are tightened inward exactly (ceiling of the lower,
   floor of the upper), and the transformation is recorded. A semi domain's active interval
   is [lb, ub] with 0 < lb.
5. **Integrality is never relaxed implicitly.** No route, retry or diagnostic replaces a
   discrete domain by a continuous one without a named, recorded transformation.
6. **Semantics per analysis mode.**

   | Mode | Discrete variables |
   |---|---|
   | Steady optimization | Decisions: MILP, MIQP or MINLP routes (ADR-0102) |
   | Steady square or root | Admitted only when the case fixes every discrete variable; the solve is then continuous |
   | Initialization | A declared stage fixes discrete variables at start or declared values as a scoped overlay, restored on every exit (PS-08, D13) |
   | Integrated dynamics | Fixed per segment (piecewise-constant inputs); free discrete decisions over time use the simultaneous route |
   | Simultaneous dynamic optimization | Admitted; lowers to a mixed-integer program |
   | Fitting | Refused unless fixed |
   | Duals, sensitivities, covariance | Computed on the continuous problem with the discrete assignment fixed; the validity record states that assignment (PS-12, ADR-0107) |
   | Structural and DoF analysis | Discrete variables count as degrees of freedom; lowered disjuncts are analysed under their realization (ADR-0104) |

7. **Backends without native semi domains.** HiGHS consumes semi domains natively. For SCIP
   a semi domain lowers through the named `semi(indicator)` transformation of ADR-0104
   (binary plus linking bounds); the lowering is recorded.

### Consequences

Grammar, registry and codegen change together (`just codegen`). The fitting route's ad-hoc
integrality check becomes a typed rule. Authors must supply finite bounds for every discrete
variable. F08a closes when M1 lands.

### Compensating controls

Typed refusal naming the variable; generated enum with no hand-written mapping; the per-mode
table is enforced at admission (Plan 22 M2), not documented only.

### Confirmation

The tests in `verification:` (Plan 22 M1, M2). The review's slot 3 physical-semantics table
covers the typing rule.

## Pros and cons

A declared facet costs a grammar and registry change, but it keeps domain meaning at its
single authority and removes the hard-coded `Continuous`. A mapping layer would have been
cheaper to introduce and would have left two enums to drift.

## More information

- Architecture companion [§2.1–§2.2](../plans/22-solver-capabilities-architecture.md#21-domain-facet).
- Target review [T15](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t15) (domain-enum authority settled here).
- Related: ADR-0099, ADR-0101 (proposed), ADR-0104, ADR-0107. Plan 22 packets M1, M2.

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T15 were corrected in this record before acceptance.
