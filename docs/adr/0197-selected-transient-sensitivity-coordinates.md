---
id: ADR-0197
title: Separate complete transient values from consumed sensitivity coordinates
status: proposed
date: 2026-10-17
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-07, PS-07, PS-11]
blueprint: [§13.6, §19.4]
review: docs/design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A consumer needs non-coordinate seed combinations or a backend cannot preserve selected physical-coordinate derivatives.
verification: Both native backends preserve analytic initial, scheduled and endpoint responses with empty, selected and full axes; fixed inputs remain physical values and tied scaled contributions add correctly.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: []
---

# ADR-0197: Separate complete transient values from consumed sensitivity coordinates

## Context

Plan44/F01 identifies transient fitting work over unconsumed fixed-input sensitivity axes. Complete integration values are necessary to evaluate physics; a consumer need not differentiate every value. Current sample indexing and admission conflate those layouts.

## Scope

Amend the internal forward-integration kernel contract under blueprint §13.6/§19.4. Ordinary public Forward still requests every admitted axis. This does not expose a user selection API, add hybrid sensitivity support, tighten accuracy or choose quadrature sensitivities.

## Drivers

An unconsumed fixed input retains its physical effect without another native sensitivity vector. Fitting/shooting select consumed bindings, native libraries integrate, and original physical assessment governs results. Scales, schedules, initial conditions and tied contributions remain explicit.

## Options

Full width repeats unused work. Truncating values loses physics; reducing width alone violates callback/builder contracts. Select unique ordered physical-coordinate axes while retaining complete values in the existing equation/session. Diffsol supports equation-owned parameters; IDAS supports an explicit sensitivity callback. No vendor extension or dependency change is selected.

## Outcome

Resolve one explicit forward-axis layout per prepared request/report. Values remain complete. Diffsol exposes selected coordinates and lifts them into complete values; IDAS allocates exactly the selected vectors with explicit derivatives. Initial and active scheduled-segment mappings use that same layout. Empty selection performs value-only integration; ordinary Forward resolves the full layout. Reports retain axis identity once, and all gathering/admission consumers use it rather than infer width from values. Published response rows include a required parameter-column ordinal: the physical integration axis for simulations, or public free-parameter order for fitting. This distinguishes scheduled intervals of one physical parameter without merging distinct derivatives; parameter identity and units remain explicit. Dynamic profile identities include the selected layout in `pse.dynamic.profile.v10`; the old framing remains frozen. Internal replay preserves one explicit semantic-parameter or physical-axis request instead of silently restoring full Forward.

### Consequences

Native request/report, initial derivatives, schedules, tolerance/workspace admission and fitting/shooting response gathering migrate together. Tied consumer contributions still add after physical conversion. Missing axes are unavailable, never zero-filled. Existing adjoint/exact-curvature layouts and hybrid refusals remain.

### Compensating controls

Analytic coupled/scaled/tied/scheduled controls cover both backends, empty/selected/full axes, initial and endpoint products, ordinary Forward publication and admission refusal. Native sensitivity error control and original scientific checks remain. Changed native step counts are not themselves a failure when the requested sensitivity population changes.

### Confirmation

**Implemented:** native selected layouts, preparation/admission, physical response publication, dynamic-profile v10 framing and typed replay selection are integrated. **Tested — 2026-10-17:** the coordinator’s 52-test native-local selection includes IDAS/Diffsol selected initial/scheduled/full/empty controls, fitting/shooting response assembly, selected preparation/replay and identity framing. Public Python response-column controls passed in the three-test projection selection. Commands, modes and zero-baseline results belong to [Plan44’s Outcome](../plans/44-pipeline-performance-optimization.md#outcome-specified-changes-and-test-setup). These are targeted functional checks, not measured benefit or whole-plan numerical qualification. Formal ADR acceptance remains separate.

## Pros and cons

Selected axes remove unused native work without a second integrator. Explicit layout propagation adds integration work across existing owners and must avoid inconsistent retained-array indexing.

## More information

[Plan44](../plans/44-pipeline-performance-optimization.md), [44b](../plans/44b-selected-sensitivities-and-workflow-products.md), blueprint §13.6/§19.4.

## Status history

- 2026-10-17 — proposed before implementation under the authorized Plan44 specified-change tranche; formal ADR acceptance remains separate.
