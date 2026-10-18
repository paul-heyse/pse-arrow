---
id: ADR-0200
title: Retain qualified KLU symbolic state across fresh numerical applications
status: proposed
date: 2026-10-18
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-07, PS-06, PS-08]
blueprint: [§18, §23.1]
review: docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md
evidence: "Interface-checked: bundled SUNDIALS 7.1.1 KLU lifecycle inspected; Plan45 RC08 accepted; adoption remains conditional on 45e/E01 qualification."
supersedes: []
superseded-by: null
revisit: A CSC or analysis setting cannot be represented by complete compatibility, or representative sparse measurements show no complete-operation benefit.
verification: E01 fixed-pattern changed-coefficient solves, changed settings/pattern, singular recovery, accepted starts, current residuals, fresh numeric counts, pressure and last-owner release.
---

# ADR-0200: Retain qualified KLU symbolic state across fresh numerical applications

## Context

The bundled SUNDIALS KLU adapter frees and reconstructs symbolic analysis when a fresh KINSOL application initializes it. Plan45 F13 identifies repeated structure work; fresh numerical setup and original residual assessment remain mandatory (blueprint §18).

## Scope

This publishes a narrow opt-in native linear binding within the existing KINSOL design, before implementation. It amends blueprint §18/§23.1. Stock KLU remains the default and comparison route. Neither nonlinear iteration nor the separate implicit-function derivative factorization moves to this binding.

## Drivers

Repeated coefficient changes at fixed sparse structure should reuse established symbolic structure without treating earlier numerical factors as current. Ownership and compatibility belong to the actual linear solver, with local failure and independently testable fresh-application transitions.

## Options

Stock KLU has the simplest upstream lifecycle but repeats symbolic work. Skipping initialization or retaining old numerical validity is unsound. The candidate retains only qualified symbolic state, performs a fresh factorization on each application's first setup, and leaves within-application setup/refactor policy with SUNDIALS. Existing Krylov/preconditioned routes remain eligible complete-operation alternatives; no universal solver winner is selected.

## Outcome

An explicit KluRetainedSymbolic profile selects the candidate. Exact compatibility includes matrix dimensions/format, occupied CSC pointers/indices and every KLU analysis-affecting input: tol, memgrow, initmem_amd, initmem, maxwork, btf, ordering, scale and halt_if_singular. Floating settings compare exact bits. Eligibility requires stock AMD/COLAMD ordering, absent user_order/user_data callbacks and halt_if_singular enabled; unknown callback semantics refuse. Output statistics are not input settings.

Application entry and initialization invalidate numerical state, including applications that accept their initial point without setup. The first setup of each new application performs fresh numerical factorization; later setup follows stock current-application refactor/condition policy. Solve requires a successful setup for the current application. Pattern/settings changes discard incompatible symbolic state. Failed factorization or cancellation never leaves numerical validity available to a later application.

### Consequences

The candidate adds bounded native state, exact structure/settings comparison, explicit selection and maintenance of a small bundled adapter extension. Known key/native metadata storage is admitted in addition to existing opaque factor allowances. Instrumentation distinguishes actual symbolic analysis, fresh numeric factorization, refactor and substitutions. Adoption requires a useful complete repeated sparse operation, not only fewer symbolic calls.

### Compensating controls

Retain stock defaults and typed refusal for unsupported candidate configurations. Test changed structure/settings, numerical failure then recovery, accepted starts, cancellation, pressure and last-owner release. Current physical residuals independently determine scientific permission; symbolic reuse grants no permission.

### Confirmation

Plan45e/E01 owns implementation, exact-settings qualification, stock/Krylov contrasts and adoption. Plan45f owns assembled qualification. This proposed record establishes the bounded target, not tested implementation or a performance claim.

## Pros and cons

The candidate removes repeated library structure work while keeping fresh numerical factors and KINSOL ownership. Its cost is a small bundled binding extension whose benefit and complete lifecycle must be established for an explicit route.

## More information

[Plan45 rule decisions](../plans/45-pipeline-execution-optimization.md#rule-decisions), [E01](../plans/45e-sparse-native-and-provider-effectiveness.md#e01--repeated-sparse-preparation-and-qualified-alternatives), and [native execution](../authoritative_design/sections/numerical-execution.md#section-18).

## Status history

- 2026-10-18 — proposed under accepted Plan45 RC08 before candidate implementation; stock default retained pending qualification.
