---
id: ADR-0200
title: Retain stock KLU after bounded symbolic-owner qualification
status: proposed
date: 2026-10-18
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, PS-06, PS-08]
blueprint: [§18, §23.1]
review: docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md
evidence: "Tested and Measured: Plan45e/E01 exact-settings/fresh-numeric controls and short 128/512-variable comparisons; the candidate did not establish material complete-operation benefit and was removed. Stock KLU and explicit Krylov routes remain."
supersedes: []
superseded-by: null
revisit: A representative repeated workload is materially dominated by symbolic analysis, making a complete-operation benefit plausible despite additional state and adapter maintenance.
verification: E01 fixed-pattern changed-coefficient solves, changed settings/pattern, singular recovery, accepted starts, current residuals, fresh numeric counts, pressure and last-owner release.
---

# ADR-0200: Retain stock KLU after bounded symbolic-owner qualification

## Context

The bundled SUNDIALS KLU adapter frees and reconstructs symbolic analysis when a fresh KINSOL application initializes it. Plan45 F13 identifies repeated structure work; fresh numerical setup and original residual assessment remain mandatory (blueprint §18).

## Scope

The accepted Plan45 RC08 route allowed a narrow opt-in native linear candidate within KINSOL, conditional on sound lifecycle and useful complete-operation benefit. This proposed record preceded that experiment. Qualification now selects **non-adoption**: the production target remains stock KLU and the existing explicit Krylov/preconditioned alternatives. This is not a formal ADR status transition, a default solver change or a claim that repeated symbolic analysis has been eliminated.

## Drivers

Repeated coefficient changes at fixed sparse structure should reuse established symbolic structure without treating earlier numerical factors as current. Ownership and compatibility belong to the actual linear solver, with local failure and independently testable fresh-application transitions.

## Options

Stock KLU has the simplest upstream lifecycle but repeats symbolic work. Skipping initialization or retaining old numerical validity is unsound. The candidate retains only qualified symbolic state, performs a fresh factorization on each application's first setup, and leaves within-application setup/refactor policy with SUNDIALS. Existing Krylov/preconditioned routes remain eligible complete-operation alternatives; no universal solver winner is selected.

## Outcome

Retain stock KLU and the existing explicit SPGMR/Jacobi route. The experimental symbolic-retaining profile, binding and lifecycle-only tests are removed from production; the experiment and removal remain in Git history. The generalized banded-system comparison remains useful for actual supported routes. KINSOL owns nonlinear iteration and within-call numerical setup; fresh application setup and original residual assessment remain mandatory.

The tested candidate compared exact occupied CSC structure and all KLU analysis-affecting settings, admitted known additional storage and required fresh numerical factors per application. Independent inspection caught KLU's normalization of four floating inputs; rejecting noncanonical ranges corrected false substitution refusal after a successful setup. Lifecycle soundness alone did not establish sufficient benefit to retain the additional owner.

### Consequences

**Measured:** three changed-coefficient applications at fixed 512-variable sparse structure took stock KLU 2.658/2.628/2.360 ms and the candidate 2.527/2.608/2.275 ms, with retained allowances of 589,768 and 647,216 bytes respectively. The existing SPGMR/Jacobi contrast took 1.690/1.642/1.755 ms with 348,944 bytes. These are one short local observation of each changed application, not repeated statistical samples or a universal backend ranking. Removing repeated symbolic calls did not establish a material complete-operation gain against extra retained state, per-substitution CSC comparisons and bundled-adapter maintenance. The 128-variable contrast likewise supplied no compelling adoption case.

### Compensating controls

Keep stock numerical initialization and failure/recovery semantics. Retained native allocations do not authorize old factors. Explicit Krylov selection retains its existing problem eligibility and original physical residual checks; this observation does not make it the global default. Plan45 F13 remains unresolved under the accepted bounded RC08 disposition.

### Confirmation

**Tested:** the experimental E01 controls covered exact settings/pattern changes, fresh factors, accepted starts, numerical failure/recovery, cancellation, pressure and release. The final five-control native repair selection also exercised the retained stock/Krylov contrast with zero failures. Plan45e owns the bounded inquiry outcome; Plan45f owns assembled qualification and its limits.

## Pros and cons

The retained upstream owner avoids a second maintained binding and extra state. Repeated symbolic analysis remains a known cost. Revisit only when a representative symbolic-dominant workload supplies a materially different complete-operation case.

## More information

[Plan45 rule decisions](../plans/45-pipeline-execution-optimization.md#rule-decisions), [E01](../plans/45e-sparse-native-and-provider-effectiveness.md#e01--repeated-sparse-preparation-and-qualified-alternatives), and [native execution](../authoritative_design/sections/numerical-execution.md#section-18).

## Status history

- 2026-10-18 — proposed under accepted Plan45 RC08 before candidate implementation; stock default retained pending qualification.
- 2026-10-18 — independent implementation inspection identified KLU's normalization of four floating inputs; candidate eligibility explicitly rejects those noncanonical ranges before setup rather than reporting successful setup followed by false substitution refusal.
- 2026-10-18 — bounded E01 qualification selected non-adoption under accepted RC08; removed the experimental production binding and retained stock/Krylov comparisons. Formal status remains proposed.
