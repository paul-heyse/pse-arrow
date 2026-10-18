---
id: ADR-0198
title: Reuse forward quadrature sensitivities for combined shooting products
status: proposed
date: 2026-10-17
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-07, DP-13, PS-07, PS-09]
blueprint: [§13.6, §19.4]
review: docs/design_review/reviews/design_review_pipeline-performance-optimization_2026-10-17.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A supported integral-only or hybrid consumer establishes a different qualified product crossover.
verification: Exact direct-parameter and initial/anchor quadrature gradients, scheduled reinitialization, selected axes, one/two windows, cancellation and retained-adjoint controls at existing accuracy; matched integration-purpose counts.
standard: {core: "3.4", process-simulator: "1.5"}
scenarios: []
---

# ADR-0198: Reuse forward quadrature sensitivities for combined shooting products

## Context

Plan44/O02 and accepted RC02 permit a conditional forward quadrature route. Combined terminal/path and integral shooting currently obtains forward responses and then separately integrates a checkpointed adjoint for its integral gradient. Pinned SUNDIALS 7.1.1 supplies exact user quadrature-sensitivity callbacks and explicit reinitialization/error control.

## Scope

Amend blueprint §13.6/§19.4 with one narrow IDAS binding consumed by first-order combined shooting requests already requiring forward responses. Integral-only requests retain adjoints; existing event/reset derivative refusals remain. No general public integral-sensitivity profile, scientific formula or accuracy policy changes.

## Drivers

Consume one complete useful native product instead of another forward/backward operation, while preserving physical derivative identities, original assessment, same-point product upgrades and native-owned iteration.

## Options

Retaining adjoints everywhere repeats work for combined requests. A universal forward route overfetches directions for integral-only requests. Select forward quadrature sensitivities only for the existing combined consumer and retain the alternative for its distinct demand.

## Outcome

The native binding accepts an explicit endpoint quadrature-derivative demand in the same selected physical-axis layout as the trajectory report. It returns trajectory and a dimension-checked optional quadrature-major/selected-axis endpoint product; incomplete or failed work cannot present that derivative product as complete. A completed empty-axis request is a valid empty derivative layout. The callback computes the complete chain rule, including direct parameter and initial/anchor dependence, with exact supplied derivatives rather than native difference quotients. Scheduled segments accumulate physical offsets and reinitialize native quadrature-sensitivity state. Existing normalization and derivative error policy determine units and tolerances; error control is explicit. Admission includes actual native workspace and retained K-by-Nq storage before effect. No unrelated allowance is widened.

### Consequences

Shooting's complete same-point demand/cache distinguishes the integral derivative product. Successful combined evaluations reuse it; failed upgrades preserve previously completed weaker products. Integral-only and other supported adjoint paths remain. Cancellation forbids new fallback work and all issued native work drains.

### Compensating controls

Analytic direct/state/anchor/scheduled derivatives, one/two windows, zero/few/more selected directions and independent checks protect the mappings. An adverse integral-only control preserves adjoints. Native counts and integration purposes establish avoided operations without a universal elapsed claim.

### Confirmation

Interface-checked against the pinned IDAS headers and existing selected forward/scheduled adapters before dependent implementation. Implemented and Tested: exact direct/scaled/initial/scheduled/empty/cancelled derivative controls, one/two-window shooting, retained-adjoint and admission-refusal/failed-upgrade controls in the native-local Plan44 selections. The assembled selection ran 86 tests with zero failures at ordinary accuracy and explicit force-validation. Actual operation counts establish removed adjoint requests for qualified combined demand; no universal elapsed benefit is claimed. Plan44 owns the complete qualification boundary.

## Pros and cons

One exact native product can remove a complete repeated integration; it adds a narrow derivative buffer/admission and scheduled-reinitialization obligation at the existing adapter.

## More information

[Plan44](../plans/44-pipeline-performance-optimization.md), accepted RC02 and blueprint §13.6/§19.4.

## Status history

- 2026-10-17 — proposed before implementation during authorized remaining Plan44 execution. Formal acceptance remains separate.
