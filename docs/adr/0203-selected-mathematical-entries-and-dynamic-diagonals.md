---
id: ADR-0203
title: Prepare exact selected mathematical entries for dynamic diagonals
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-03, AP-07, PS-06, PS-09]
blueprint: [§14.3, §13.6]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A selected provider or formulation cannot establish the exact demanded entries or a new dynamic residual/mass convention changes their consumption.
verification: Plan48 E04 independently compares raw physical RHS diagonal, shifted Jacobi actions, actual derivative requests and full-fallback observations across ODE/DAE layouts, scaling, duplicate contributions and failed upgrades.
---

# ADR-0203: Prepare exact selected mathematical entries for dynamic diagonals

## Context

Plan48 N01 identifies full dynamic Jacobian preparation for a Jacobi consumer that needs only a diagonal. Existing selected partials support output/coordinate rectangles; projecting a full assembled Jacobian does not remove that work. The actual physical diagonal can cross local output/coordinate ordinals and include several scaled contributions.

## Scope

Extend the existing demanded mathematical preparation owner in blueprint §14.3 with exact semantic row/column entry selection, then consume it in the fixed-mass dynamic oracle of §13.6. This adds no numerical method, alternate calculus engine, implicit accuracy change or new solver-selection default.

## Drivers

Retain one support and derivative authority. Carry actual demand into the compiled library products, preserve ordered checks/provider effects and original source mapping, and let the native adapter own its shifted preconditioner formula.

## Options

Full exact partials remain a valid explicitly identified fallback for an oracle without selective capability. A local numeric-diagonal projection is insufficient because semantic rows and state columns need not share local ordinals. Prepare the exact contributed entries through existing body support and library First products, grouping only fitting demands and retaining their canonical scatter.

## Outcome

The target mathematical entry adapter consumes requested semantic row/column pairs and current source support. It prepares only the supported contributing output/formal demands, preserving duplicate accumulation, normalization, boundary inputs, guards and provider/effect/current-context obligations. Structural absence yields zero; unknown membership or a contribution outside admitted support remains a contract refusal. It does not silently substitute a full matrix or claim full derivative coverage from selected entries.

Repeated provider occurrences across distinct selected programs require a complete current
linearization association before atomic publication. Preparation enumerates actual provider
requirements from retained staged support. An immutable registration descriptor does not prove
support for an arbitrary SelectedColumns vector or a future worker's current linearization.
Built-in dynamics therefore choose ordinary full preparation before compilation/native effects
when those selected-provider premises cannot be established. Provider-free selected preparation
proceeds without a new provider capability platform. The actual immutable reconstruction
factory may also qualify its single-root nested implementation for First selected columns:
the admitted output/residual/selector orders suffice, all residual/selection/hint/term
programs and the registration graph are provider-free, and its concrete worker implements
SelectedColumns. This is owner-specific preparation eligibility, not a registration-wide
capability flag or permission to skip current effects. Dependent, regime and unknown factories
retain full preparation before numerical effects. Explicit low-level selected callers must
qualify exact worker demand and matching complete tokens for every repeated current source;
missing/changed tokens refuse the fill. Equal numerical results give no permission and no
after-effect full retry is allowed. Single-group sources acquire no gratuitous cross-call
requirement. A guard/Value-only provider may retain selection only where its actual existing
demand and context obligations are unchanged.

The dynamic oracle's `rhs_state_diagonal(mode,time,state,parameters,output)` means raw partial `df_i/dy_i` in its ordered physical state coordinates for `Function::Rhs`. It contains no native `cj`, mass or tolerance. Differential and algebraic rows use the same raw f contract. The IDAS adapter alone forms `cj*M_i - diagonal_i`, with `M_i` equal to one for differential states and zero for algebraic states, preserving the existing finite/tiny reciprocal fallback.

Runtime preparation maps semantic state identities and original row/coordinate scales into those pairs, including duplicate contributions. Anchored integral states append zero self-diagonal because their flux does not depend on the wrapper integral state. Selected, Value and full derivative coverage are distinct transactional products. Failed stronger work preserves an earlier complete compatible weaker product, and partially written caller output is never published as success.

Anchored wrappers prepare their conservative all-mode/function CSC and canonical scatter
through the existing finite `prepare_layout(allowance)` / `use_prepared_layout` route before
derivative evaluation. Production window preparation retains both ordinary and observed
wrapper layouts. Value-only wrappers remain usable without a derivative layout; derivative
evaluation without admitted preparation refuses before effects. Direct callers migrate to
the same route instead of retaining a callback-time triplet/sort path or hidden dense allowance.

### Consequences

The general oracle has an exact full fallback; its selected/full route is observable. Built-in modeling dynamics implement the selective route end to end. New provider limitations remain explicit at their own request boundary rather than weakening the scientific checks.

### Compensating controls

Check reordered state/local coordinates, nonuniform scales, overlapping contributions, structural zeros, changed mode/parameters/provider/branch and cancellation. Independently construct the shifted diagonal for ODE and index-1 DAE examples. Assert actual demanded work on a wide Jacobi-only model and retain full-matrix consumers unchanged.

### Confirmation

**Implemented:** compiler-issued exact entry plans, mathematical selected products,
semantic dynamic scatter and the native raw-diagonal consumer now form one preparation
and execution route. Static First support remains complete while numerical Value, selected
entries and full products retain distinct coverage. Opaque, dependent and otherwise
unqualified provider owners choose full preparation before numerical effects; the concrete
childless implicit owner qualifies only under the provider-free First premises above.

**Tested:** native-local `unit-native-capability-package pse-backend-native` with
`native-solvers` passed the focused 28-control selection with zero failures, including
`jacobi_selected_diagonal_preserves_shift_mass_fallback_and_no_full_requests`,
`rhs_diagonal_full_fallback_is_explicit_and_refuses_before_output_publish`,
`rhs_diagonal_full_fallback_keeps_prepared_support_refusal_without_support_replay` and
`anchored_raw_diagonal_forwards_without_full_wrapper_matrix_and_checks_current_flux`.
The authored runtime
`selected_jacobi_compiler_runtime_native_pipeline_keeps_exact_demand_and_coverage`
passed in the canonical six-control selection (run
`59360525-7375-404a-b7f7-fda825f6577f`: five passes and one separately owned population-control
failure against zero, zero compiler warnings/errors). Its actual producer recorded
ten selected setups matching all ten native setup evaluations, zero full setups and
positive selected-entry work under complete aggregate observation coverage. Numerical
workers retained no full Jacobian. The same-source/profile full-KLU comparison preserves
the original physical 0.001 classifications: one of 48 sample/coordinate points exceeded
that budget on both routes; native completion does not certify global accuracy.

**Tested:** the separate ordinary-run
`selected_jacobi_physical_goal_refuses_missing_rate_uncertainty_without_promoting_native_completion`
passed in the canonical 32-control selection (29 passes and three separately owned/control
failures, zero compiler warnings/errors). Its explicit Estimated/RequireSatisfied 0.001s
goal remains Unresolved/EvaluatorUncertainty with no accepted result or guessed refinement.
Generated implicit-rate First capability supplies no arithmetic uncertainty enclosure.
[Plan48e E04](../plans/48e-dynamics-fitting-and-preconditioning.md#e04--selected-jacobi-demand)
owns the focused evidence and limits; [48h](../plans/48h-qualification-and-design-publication.md)
still owns pending selected assembled qualification. Formal ADR status remains proposed.

## Pros and cons

This removes unconsumed derivative construction while reusing existing mathematics and native ownership. It requires explicit selected coverage and semantic scatter instead of treating equal dimensions as compatibility.

## More information

- [Plan48e E04](../plans/48e-dynamics-fitting-and-preconditioning.md#e04--selected-jacobi-demand)
- [Mathematical demand](../authoritative_design/sections/mathematics-and-compilation.md#section-14-3)
- [Dynamic execution](../authoritative_design/sections/workflows-and-results.md#section-13-6)

## Status history

- 2026-10-20 — Proposed during maintainer-authorized Plan48 execution, before dependent selected-entry implementation; formal acceptance remains separate.
