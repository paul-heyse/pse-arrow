---
id: ADR-0200
title: Retain KLU symbolic analysis within private fixed-pattern applications
status: proposed
date: 2026-10-18
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, PS-06, PS-08]
blueprint: [§18, §23.1]
review: docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md
evidence: Measured
supersedes: []
superseded-by: null
revisit: A native library upgrade, new analysis setting or changed structure-ownership boundary invalidates the private fixed-pattern lifetime or its fresh-numeric contract.
verification: E05 fixed-pattern complete-operation contrast, mutation refusal and singular recovery, original-validation failure reset, recovered trial, real sparse implicit parameter reuse, accepted starts, original residuals, fresh numeric counts and cache release; F05 owns combined acceptance.
---

# ADR-0200: Retain KLU symbolic analysis within private fixed-pattern applications

## Context

The bundled SUNDIALS KLU adapter frees and reconstructs symbolic analysis when a fresh KINSOL application initializes it. Plan45 F13 identifies repeated structure work; fresh numerical setup and original residual assessment remain mandatory (blueprint §18).

## Scope

The accepted Plan45 RC08 route permits a narrow native linear realization within KINSOL, conditional on sound lifecycle and useful complete-operation benefit. This proposed record preceded E01, whose generic exact-key candidate was not adopted. The reopened E05 qualification selects a different private fixed-pattern owner with the same existing KLU method selection. The earlier experiment and current follow-up have distinct outcomes below; no formal ADR status or global solver selection changes.

### Reopened private-owner target (accepted RC08)

The maintainer authorized a second, materially different route after the bounded non-adoption below. First screen representative sparse shapes under short complete operations. Only a useful adoption case enables a dedicated fixed-pattern native constructor whose sole Rust Session owner retains canonical CSC structure and frozen analysis settings. In this mode, settings/reinitialization mutation and mutable native-state accessors refuse; scalar accounting remains available. Generic upstream KLU behavior is unchanged.

Begin every application by invalidating/freeing numeric factors, before even accepted-initial-point or error returns. Native initialization repeats that transition idempotently. First demanded setup retains only compatible completed symbolic analysis and full-factors current coefficients; within-call refactor/condition monitoring stays library-owned. Setup failure leaves numerical state unusable and the next setup performs full factorization. Preserve canonical CSC restoration after KINSOL clears native matrix indices. No copied CSC compatibility key or per-substitution structure scan is introduced.

Qualify original residuals, changed coefficients/pattern/settings, accepted starts, singular failure/recovery, cancellation/actual drain, retained accounting and the real nested implicit Session cache. A drained reset private owner may return after cancellation; uncertain drain or integrity cannot be cached. Adopt internally for eligible existing KLU Sessions only after useful complete-operation benefit; otherwise remove the candidate and retain F13 unresolved. No public solver variant, global method winner, old numerical validity or new native-internal cancellation capability is selected.

The [bounded prerequisite review](../design_review/reviews/design_review_plan45-physical-premises-and-fixed-pattern-klu_2026-10-18.md) assesses this target. Plan45e/E05 owns its implementation decision; E01's measurements and non-adoption remain historical.

## Drivers

Repeated coefficient changes at fixed sparse structure should reuse established symbolic structure without treating earlier numerical factors as current. Ownership and compatibility belong to the actual linear solver, with local failure and independently testable fresh-application transitions.

## Options

Stock KLU has the simplest upstream lifecycle but repeats symbolic work. Skipping initialization or retaining old numerical validity is unsound. The candidate retains only qualified symbolic state, performs a fresh factorization on each application's first setup, and leaves within-application setup/refactor policy with SUNDIALS. Existing Krylov/preconditioned routes remain eligible complete-operation alternatives; no universal solver winner is selected.

## Current follow-up outcome

**Implemented / Tested / Measured:** select the dedicated fixed-pattern constructor internally for eligible existing KLU Sessions. The ten targeted follow-up controls passed at `17b2179a0`, including fresh factors, accepted starts, failure/recovery, original validation, frozen state, actual implicit cache use, release and the complete short contrast. The independent implementation review's original-validation cleanup finding was corrected and closed. This updates the earlier candidate's non-adoption without changing its historical result or this ADR's formal proposed status.

Three repeated 1,024-variable affine-grid replacement/solve observations were stock 2.606/2.823/2.855 ms versus fixed 2.287/2.256/2.337 ms, with the same 1,066,696 retained bytes and original residual checks. Corresponding nonlinear-grid observations also improved while retaining the same numeric/refactor/substitution counts. Only later symbolic analyses disappear. These are short scoped observations, not a universal solver ranking; Plan45e/E05 records conditions and Plan45f/F05 owns combined qualification. The earlier generic exact-key candidate stays deleted.

## Previous bounded outcome

Retain stock KLU and the existing explicit SPGMR/Jacobi route. The experimental symbolic-retaining profile, binding and lifecycle-only tests are removed from production; the experiment and removal remain in Git history. The generalized banded-system comparison remains useful for actual supported routes. KINSOL owns nonlinear iteration and within-call numerical setup; fresh application setup and original residual assessment remain mandatory.

The tested candidate compared exact occupied CSC structure and all KLU analysis-affecting settings, admitted known additional storage and required fresh numerical factors per application. Independent inspection caught KLU's normalization of four floating inputs; rejecting noncanonical ranges corrected false substitution refusal after a successful setup. Lifecycle soundness alone did not establish sufficient benefit to retain the additional owner.

### Consequences

**Measured:** three changed-coefficient applications at fixed 512-variable sparse structure took stock KLU 2.658/2.628/2.360 ms and the candidate 2.527/2.608/2.275 ms, with retained allowances of 589,768 and 647,216 bytes respectively. The existing SPGMR/Jacobi contrast took 1.690/1.642/1.755 ms with 348,944 bytes. These are one short local observation of each changed application, not repeated statistical samples or a universal backend ranking. Removing repeated symbolic calls did not establish a material complete-operation gain against extra retained state, per-substitution CSC comparisons and bundled-adapter maintenance. The 128-variable contrast likewise supplied no compelling adoption case.

### Compensating controls

Keep stock numerical initialization and failure/recovery semantics. Retained native allocations do not authorize old factors. Explicit Krylov selection retains its existing problem eligibility and original physical residual checks; this observation does not make it the global default. F13 remained unresolved at that earlier close; the current follow-up outcome above records its subsequent qualified correction.

### Confirmation

**Tested:** the experimental E01 controls covered exact settings/pattern changes, fresh factors, accepted starts, numerical failure/recovery, cancellation, pressure and release. The final five-control native repair selection also exercised the retained stock/Krylov contrast with zero failures. Plan45e owns the bounded inquiry outcome; Plan45f owns assembled qualification and its limits.

## Pros and cons

The fixed-pattern extension removes repeated symbolic analysis without another copied CSC key, per-substitution scans or extra measured retained payload. It adds a small maintained native lifecycle contract that must be rechecked on upgrades. Generic upstream behavior and existing explicit Krylov routes remain available. Full numeric factors remain fresh, and no benefit is claimed for accepted starts or the first analysis.

## More information

[Plan45 rule decisions](../plans/45-pipeline-execution-optimization.md#rule-decisions), [current E05](../plans/45e-sparse-native-and-provider-effectiveness.md#e05--reopened-fixed-pattern-sparse-qualification), [historical E01](../plans/45e-sparse-native-and-provider-effectiveness.md#e01--repeated-sparse-preparation-and-qualified-alternatives), and [native execution](../authoritative_design/sections/numerical-execution.md#section-18).

## Status history

- 2026-10-18 — proposed under accepted Plan45 RC08 before candidate implementation; stock default retained pending qualification.
- 2026-10-18 — independent implementation inspection identified KLU's normalization of four floating inputs; candidate eligibility explicitly rejects those noncanonical ranges before setup rather than reporting successful setup followed by false substitution refusal.
- 2026-10-18 — bounded E01 qualification selected non-adoption under accepted RC08; removed the experimental production binding and retained stock/Krylov comparisons. Formal status remains proposed.
- 2026-10-18 — maintainer reopened F13 under RC08 for representative short screening and a narrower private fixed-pattern lifecycle. Prior non-adoption remains valid for its candidate/cases; new adoption is conditional and formal status stays proposed.
- 2026-10-18 — narrower fixed-pattern owner selected after the representative grid/band comparison and ten targeted controls; original-validation cleanup finding corrected. Combined acceptance remains 45f/F05; formal status unchanged.
