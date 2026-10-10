---
id: ADR-0104
title: Lower indicator, SOS, cardinality, piecewise, logic, disjunction and complementarity declarations through named realizations
status: accepted
date: 2026-09-27
deciders: [paul-heyse]
level: decision
principles: [DP-06, DP-08, DP-15, PS-04, PS-06, PS-09]
blueprint: [§14.1, §19.7]
review: git:f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc:docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t15
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A lowering's declared equivalence fails its negative control (bigm(derived) cuts a feasible point, hull and big-M optima disagree on a bounded fixture, or a native-only realization is executed on a backend without its handler), or SCIP drops a constraint handler a native realization relies on.
verification: Architecture scenario S11, settled by the Plan 22 M3–M5 tests indicator_linear_lowering_matches_native, piecewise_sos2_matches_incremental, logic_propositions_lower_exactly, native_only_realization_refused_on_highs, gdp_hull_and_bigm_same_optimum, bigm_derived_from_bounds, hull_requires_finite_bounds, indicator_realization_requires_native_backend and flash_phase_disappearance_agrees_across_realizations.
standard: core-3.0/process-simulator-1.1
scenarios: [https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#s11]
---

# ADR-0104: Lower indicator, SOS, cardinality, piecewise, logic, disjunction and complementarity declarations through named realizations

## Context

§19.7 records that alternative sets and disjunctions have no declaration or lowering. The
kernel already has the evaluation-time counterpart: an `implicit … regime … eligible(...)`
block selects one equation set during evaluation, and `realize … using …` declares how it is
realized (ADR-0100, proposed). ADR-0102 admits decision-time alternatives, and ADR-0103 gives
variables discrete domains. PS-06 requires phase appearance and discontinuities to be handled
by a declared formulation policy with its approximation stated.

## Scope

A kernel transformation contract within ADR-0099 and ADR-0100; neither is superseded. Binds
the declarations, their lowerings, their equivalence claims and how routing treats native
realizations. The SCIP handlers are ADR-0105's binding; the ℓ1 route is ADR-0109.

## Drivers

- **DP-08.** Every lowering is a named transformation with declared inputs, preconditions,
  equivalence and failure modes; obligations survive it.
- **PS-06.** Smoothing, complementarity and discrete modes each state their approximation.
- **DP-15 and PS-09.** A realization that needs a native handler is refused on a backend
  without it; there is no silent conversion.
- **DP-06.** A disjunction reuses the regime structure instead of a second alternative syntax.
- **PS-04.** Structural and DoF analysis runs on what the solver will see, and its
  diagnostics still name the authored alternatives.

## Options

| Option | Assessment | Selection |
|---|---|---|
| Leave lowering to SCIP only (indicator and SOS handlers everywhere) | HiGHS could not serve linear GDP or piecewise models; one backend would own model meaning | Rejected |
| Library-owned GDP reformulation | No Rust library reformulates GDP over this IR; Pyomo.GDP is excluded from production (D12) | Not available |
| Kernel declarations with named realizations lowered at preparation, native or linear | One authored form, two execution paths, declared equivalence | **Selected** |
| Lowering during the solve (adding equations mid-run) | Violates D13 and hides structure from analysis | Rejected |

## Outcome

1. **Declarations.**
   - Indicator constraints: `eq name when y: lhs == rhs;` (and `when not y`, and inequalities).
   - SOS1 and SOS2 sets ordered by declared weights.
   - Cardinality: `atmost k of (...)`, `atleast`, `exactly`.
   - Piecewise-linear functions with breakpoints as data, lowered by SOS2 or incremental form.
   - Logic propositions over binary or Boolean variables: `and`, `or`, `xor`, `implies`, `exactly(k)`.
   - Disjunctions: `disjunction name { alternative a { eq …; annotation bounds …; } … }` with
     `realize name using bigm(M) | bigm(derived) | hull | indicator;`. An alternative reuses the
     regime structure (named alternatives, local equations and annotations). Logic between
     alternatives lowers as above; nesting is admitted and lowered inner-first in a recorded order.
   - Complementarity: `complements(a >= 0, b >= 0)` realized by `smooth(epsilon)`,
     `penalty(l1)` or `disjunctive`.
2. **Named transformations at preparation.** Each lowering is a compiler transformation that
   produces derived rows and variables. The authored revision is unchanged (D13); the
   realization, its parameters and its equivalence class enter preparation identity and the
   result record. Because lowering happens at preparation and never during a solve, the
   ADR-0016 revisit trigger (a native GDP lowering that adds equations during a solve) does
   not fire.
3. **Declared equivalence.**

   | Realization | Equivalence | Preconditions and refusal |
   |---|---|---|
   | `bigm(M)` | Exact if the authored M is valid; the validity is the author's assertion and is recorded as such | Finite M |
   | `bigm(derived)` | Exact for bounded disjuncts | M from `pounce-presolve` FBBT intervals over the declared box, widened outward by the declared relative margin; every row of the disjunct must be FBBT-complete with a finite interval, else typed refusal naming the row (never a default M) |
   | `hull` | Exact for linear disjuncts; nonlinear disjuncts use the ε-perspective `(λ+ε)·g(x/(λ+ε))` with declared ε and O(ε) approximation stated | Finite bounds on every disjunct variable |
   | `indicator` | Exact | Native handler (SCIP) only |
   | `smooth(epsilon)` | O(ε), continuation over ε through the existing continuation policy | — |
   | `penalty(l1)` | Exact at a nondegenerate solution with a sufficient penalty; a labelled least-infeasible point otherwise | Routes to POUNCE `L1ExactPenalty` (ADR-0109) |
   | `disjunctive` | Exact, discrete | SOS1 or indicator; SCIP |
   | `semi(indicator)` | Exact | For backends without native semi domains (ADR-0103) |

4. **Two execution paths.** A native path where the backend has the constraint handler (SCIP
   indicator, SOS1/SOS2, and/or/xor, logicor, cardinality, bound disjunction), and a linear
   path over finite bounds (HiGHS or SCIP). Routing refuses a native-only realization on a
   backend without the handler.
5. **An authored `penalty(l1)` is an explicit selection.** It is a declared requirement of the
   formulation, so only POUNCE with `L1ExactPenalty` is eligible and the selection is recorded;
   this does not contradict ADR-0109's "never automatic", because nothing is chosen for the
   author. Mixing `penalty(l1)` with other realizations in one solve is admitted; the ℓ1
   penalty then applies to all constraints, and the result record says so.
6. **Analysis.** Structural and DoF analysis run on the lowered problem; diagnostics map
   derived rows and variables back to the authored disjunction and alternative.
7. **Phase appearance** is authored as package data in any of the three forms, each with its
   approximation stated (Plan 22 M5).

### Consequences

The compiler gains transformations and the registry gains declaration families
(`just codegen`). Hull reformulations enlarge problems. Authors must declare finite boxes for
derived big-M and hull. A native-only realization pins the backend.

### Compensating controls

Equivalence classes in identity; refusal of incomplete FBBT and of native-only realizations on
the wrong backend; negative controls comparing realizations on the same fixture.

### Confirmation

The tests in `verification:` (Plan 22 M3–M5), including cross-realization agreement on bounded
fixtures.

## Pros and cons

Declarations with selectable realizations keep one authored model for HiGHS and SCIP and make
approximation explicit. The cost is bespoke reformulation code in the compiler, justified
because no library reformulates this IR and the transformations are domain-model lowering,
not solver machinery.

## More information

- Architecture companion [§2.3–§2.5](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/22-solver-capabilities-architecture.md#23-constraint-forms).
- Target review [T14](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t14) (ℓ1 realization composition) and [T15](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#t15) (lowering equivalence).
- Related: ADR-0016 (superseded by ADR-0112, which restates D13), ADR-0100, ADR-0103, ADR-0105, ADR-0109. Plan 22 packets M3–M5.

## Status history

- 2026-09-27 — proposed (Plan 22 D0).
- 2026-09-27 — accepted under the maintainer's authorization of the full Plan 22 scope (2026-09-27), after the [Plan 22 target review](https://github.com/paul-heyse/pse-arrow/blob/f57b71d56f6eb2c319c4340d6f26abc6a1dc5abc/docs/design_review/reviews/design_review_plan22-target_2026-09-27.md#decision) returned Accept (author review, Proposed evidence level). Findings T14, T15 were corrected in this record before acceptance.
