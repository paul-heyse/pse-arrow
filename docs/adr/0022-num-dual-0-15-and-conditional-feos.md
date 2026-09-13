---
id: ADR-0022
title: Write every kernel against num-dual 0.15; FeOs stays conditional
status: accepted
date: 2026-09-13
deciders: [paul-heyse]
level: decision
principles: [DM-25, DM-43, DM-40, DM-31]
blueprint: [§3.1, §3.3, §9.8]
review: docs/design_review/reviews/design_review_arrow-native-idaes-core-blueprint-rev2_2026-09-13.md#7-principle-findings
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: `feos-core` releases a version that depends on `num-dual` 0.15 (checked monthly with `cargo info`)
verification: `tests/conformance` kernel derivative tests against hand derivations; `rust / family-check`; register row R-05

---

# ADR-0022: Write every kernel against num-dual 0.15; FeOs stays conditional

## Context

Blueprint §3.1 pins `num-dual` at 0.15.0 and every kernel body is written against its 0.15 trait shape (`DualNum<Primitive = f64>`). `feos-core` 0.10.1 requires `num-dual ^0.14`, whose trait shape differs, so the two cannot coexist in one kernel dialect.

## Scope

Binds the AD library and trait shape for kernels, and the condition under which the FeOs provider is enabled. Implicit-kernel derivatives are included: `implicit_derivative*` is library capability, not platform code.

## Drivers

One trait shape per kernel body or every kernel is written twice; a provider that forces a second AD generation into the graph would also force a second `num-dual` version past the family rule (ADR-0018).

## Options

Pin `num-dual` 0.14 to keep FeOs — rejected: 0.15 is the shape the operator catalog and the hyper-dual Hessian path are written against. Vendor a shim — rejected: two trait shapes in one workspace is the problem, not the fix.

## Outcome

`num-dual` 0.15.0; every kernel body is generic over `DualNum<Primitive = f64>`; implicit kernels use `num-dual`'s `implicit_derivative*` / `ImplicitDerivative`. `feos-core`/`feos` enter `pse-kernels-ext` only when their upstream tracks 0.15, which is a phase-4 gate.

### Consequences

SAFT providers are unavailable until the gate opens; Helmholtz reference fluids are expression templates and a platform kernel regardless (§9.8), so the gap is bounded to SAFT-family equations of state.

### Compensating controls

Register row R-05 carries the monthly `cargo info feos-core` check; `feos` never appears in `[workspace.dependencies]` until it does.

### Confirmation

Kernel derivative tests compare `num-dual` results against hand derivations in the kernel suites, which is what makes the implicit-function-theorem use auditable.

## Pros and cons

Waiting costs a provider; adopting early costs two AD generations in one graph.

## More information

Blueprint §3.1, §3.3 (num-dual and feos rows), §9.8 (provider kernels), §18.2; review finding R2-5; register row R-05.

## Status history

- 2026-09-13 — accepted with the repository-seeding pull request (backfilled from blueprint revision 3).
