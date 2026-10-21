---
id: ADR-0206
title: Initialize eligible ODE adjoint rates from the current exact action
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-07, PS-06, PS-10]
blueprint: [§18.10]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: An eligible current-point action cannot establish consistent adjoint rates, or actual short controls show no useful reduction in initialization work.
verification: Plan48e E06 compares first-order all-differential ODE adjoints with native consistent initialization, analytic and finite-difference gradients across observation jumps and schedules; DAE and second-order controls retain native IC.
---

# ADR-0206: Initialize eligible ODE adjoint rates from the current exact action

## Context

The backward residual is `M λ′ + f_yᵀ λ`. The adapter supplies zero rates at every
observation or segment restart and asks IDAS to compute consistent values. For a
first-order ODE with identity mass, the exact current action directly determines
`λ′ = −f_yᵀ λ`; no algebraic unknown or nonlinear initialization solve is needed.

## Scope

Narrow blueprint §18.10 within Plan48e E06. Only first-order backward problems whose
actual contract marks every state differential are eligible. This introduces neither
a new integration method nor a change to physical or local integration accuracy.

## Drivers

Avoid an unnecessary native consistent-initialization solve while preserving current
source/provider checks, observation impulses, scheduled segment history and truthful
native work. Numerical integration and globalization remain library responsibilities.

## Options

Retaining native IC everywhere is correct. Skipping IC with zero rates is incorrect.
Direct current-point rate initialization is available for identity-mass ODEs only;
algebraic and second-order backward problems keep native IC.

## Outcome

Before an eligible backward restart, evaluate the actual current state Jacobian through
the existing guarded callback owner and form the negative transpose action on the
post-jump adjoint. Validate every rate and current execution before publication. Supply
these rates to the unchanged `IDAReInitB` and quadrature reinitialization. Omit only the
now-redundant `IDACalcICB` call. Every observation and scheduled boundary still restarts
the backward problem; no checkpoint, replay, impulse, direct parameter contribution or
original assessment is removed. DAE and second-order problems keep their existing IC.

### Consequences

The eligibility test belongs to the actual native contract and backward demand.
Current action failure fails the operation; it cannot reuse a prior Jacobian or silently
replace an invalid derivative. No user option or automatic method selector is added.

### Compensating controls

Compare against an explicitly forced native-IC reference in test-only session state,
including nonlinear scheduled ODEs, multiple observations, zero/nonzero cotangents,
selected axes, cancellation and derivative failure. Retain DAE/second-order regression
controls. Independent analytic or finite-difference checks remain required.

### Confirmation

**Tested:** the Plan48e E06 native-IC reference pairs cover smooth/stiff ODEs and
scheduled samples at unchanged local0.001, with analytic output/gradient-effect assessment.
All pairs preserve original accuracy classification, including an original-invalid stiff
forward trajectory; gradients agree within about1.5e-8. Each observed pair removes four
backward nonlinear iterations with unchanged accepted steps. The independent nonsymmetric,
state/time/parameter-dependent transpose-action and failure/stop/DAE/Second eligibility
controls pass. Existing nonlinear adjoint, DAE and second-order regressions also pass in
the selected native-solvers correctness closure. Vendored IDAS7.1.1 and Context7 establish
that IC is optional for conditions already satisfying the backward system. These focused
controls do not qualify all Plan48 dynamics or establish a universal speedup.

## Pros and cons

An exact local action removes repeated nonlinear IC work for an identifiable subset.
The adapter must calculate and check the rates and preserve the conservative fallback
for algebraic or higher-order demands.

## More information

- [Plan48e E06](../plans/48e-dynamics-fitting-and-preconditioning.md#e06--corrector-initial-step-statistics-and-backward-reset)
- SUNDIALS IDAS adjoint usage documentation and vendored `idas/idaa.c`.

## Status history

- 2026-10-20 — Proposed before dependent production changes under authorized Plan48 execution.
