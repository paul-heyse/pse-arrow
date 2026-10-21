---
id: ADR-0208
title: Bind the native IDAS band approximation to the exact coupled operator
status: proposed
date: 2026-10-20
deciders: [paul-heyse]
level: decision
principles: [AP-03, AP-07, PS-06, PS-07, PS-12]
blueprint: [§18.10]
review: docs/design_review/reviews/design_review_execution-performance-recommendations-audit_2026-10-20.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A supported band approximation is singular or materially slower on the actual coupled workload, or a qualified better ordering or approximate factor becomes available.
verification: Plan48e E05 checks actual native BBD shifted actions, coupled index-one trajectories, changed shift/parameters/mode, singular bands of invertible full operators, finite preallocation, cancellation/failure and actual setup/apply work against direct/None/Jacobi.
---

# ADR-0208: Bind the native IDAS band approximation to the exact coupled operator

## Context

The current IDAS Krylov route supports None/Jacobi. A full monolithic FERAL factor reduces
iterations but repeats a direct factorization and adds cost on the bounded small cases.
SUNDIALS7.1.1 already provides a library-owned band approximation and band factor/apply
through `IDABBDPrecInit`, including for the current serial vector owner.

## Scope

Add an explicit SPGMR/BBD choice within blueprint §18.10 and Plan48e E05. This is a native
preconditioner binding, not a new integrator, an alteration to the exact equations or a
physical block decomposition. Default direct KLU and existing None/Jacobi remain available.

## Drivers

A narrow band approximation can reduce factor work and storage relative to a complete
factor while reducing Krylov work relative to weak diagonal preconditioning. Prefer that
qualified mechanism on first principles without requiring a marginal short-run timing win.
Diagnose unexpected performance after deployment; preserve exact original assessment.

## Options

Continue None/Jacobi only; install a full factor as a preconditioner; or bind the native
band approximation. The latter supplies an existing approximate algorithm without bespoke
factor or iteration code. Symmetric KKT Schur routines cannot receive the general unsymmetric
shifted DAE operator without changing its meaning.

## Outcome

`IdasLinear::SpgmrBbd` carries a positive Krylov dimension and explicit difference/retained
half-bandwidths (`IdasBand { upper, lower }`) in original state ordering. Validate extents,
native index fit and retained bandwidth within the differencing envelope before effects.
Zero bandwidth is legal but does not guarantee invertibility. Native default difference
increments remain library-owned; no automatic physical partition or ordering is invented.

The local residual adapter delegates to the existing checked panic-contained current
residual for `cj M − f_y`; the serial communication callback is absent. Native BBD owns
grouped difference quotients, band factorization and application. The exact Krylov action
retains every off-band differential/algebraic coupling and original domain/provider check.
Only preconditioner work is approximate. Prepare Value/exact-JVP demand where qualified;
do not force full numeric Jacobian preparation just for BBD.

### Consequences

Expose native BBD workspace/local-residual/setup/apply observations through existing
diagnostics. Precharge complete band matrix, pivots, native vectors/views/headers and Krylov
storage before constructor effects. Independent/reset replacement must cover both old and
new populations if the library allocates the replacement before releasing the old one.
Fresh storage is valid until complete retained reset is qualified; no factor reuse is inferred.

### Compensating controls

The band of an invertible full operator can be singular. Preserve native setup/factor
failure and current source/stop failure; do not silently retry a different preconditioner
after effects. Native band kernels keep before/after cancellation/deadline checks and checked
residual/action callbacks. Current coefficients, shift, mode and parameters must establish
each setup. Validate original trajectories and actual true operator actions independently.

### Confirmation

**Tested:** the native68-profile comparison and focused BBD shifted-action, band/failure,
current-parameter/reset and finite-work controls pass. The actual authored-runtime BBD
control also passes with Value+JVP demand and no full numeric Jacobian. Its correction
preserves empty exact-entry plans as valid JVP-only layouts while refusing malformed
nonempty plans. The generated binding includes the native BBD header. Plan48e E05 owns
the commands, native populations and original-output limits. These results establish
selected composed consumption, not a universal band choice or performance guarantee.

## Pros and cons

The binding adds a bounded library-owned approximation with explicit scientific meaning.
Performance and convergence depend on ordering/bandwidth; singular or weak bands remain
honest failures, and arbitrary operator shapes do not acquire an automatic winner.

## More information

- [Plan48e E05](../plans/48e-dynamics-fitting-and-preconditioning.md#e05--coupled-shifted-dae-preconditioner)
- Vendored IDAS `idas_bbdpre.c` and `idas_bbdpre.h`.

## Status history

- 2026-10-20 — Proposed before implementation under accepted RC05 and maintainer deployment-first selection guidance.
