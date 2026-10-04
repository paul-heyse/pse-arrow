---
id: ADR-0157
title: Admit pinned Uno and PETSc profiles through scoped foreign bindings
status: proposed
date: 2026-10-03
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, DP-13, DP-19, PS-09, PS-10]
blueprint: [§3.2, §18.6, §18.8, §18.9]
review: docs/design_review/reviews/design_review_declared-solve-pipeline-decisions_2026-10-03.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: A source/build/profile change alters callback, ABI, bound or process-global assumptions.
verification: P6/P8/P9 linked ABI, recoverable/terminal callback, bounds/inert-option refusal, effective-setting, candidate rejection and destruction controls; Plan 25k K3/K4 qualify integrated behavior and resources.
standard: core-3.3 / process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s08, docs/design_review/reviews/design_review_solver-acceleration-and-globalization_2026-10-03.md#s12]
---

# ADR-0157: Admit pinned Uno and PETSc profiles through scoped foreign bindings

## Context

The current adapters do not supply all selected bounded trust-region and library pseudo-time/block profiles. Both reviews retain those established methods with corrected phase-specific callback and bound contracts.

## Scope

Add narrow pse-uno-sys and pse-petsc-sys foreign crates and safe adapters in pse-backend-native under the native-solvers capability graph. Extend the existing Ipopt sys bridge for admitted persistent C++ sequence use. Keep foreign objects private to integration.

## Drivers

Preserve original scientific permission, explicit applicability, bounded lifetime and change locality while integrating the complete Plan 25m target. Mechanism substitution and pure policy testing must stay within their responsible owners.

## Options

Custom trust-region/SER/NASM implementations duplicate mature numerical ownership. Blanket PETSc rejection ignores tagged trial handling. Select bounded library profiles and narrow foreign bridges; leave inadmissible exact-Hessian/bound combinations visibly unavailable.

## Outcome

Pin Uno commit 805a27ecf802eec33d1aea85ae73efa8a24f71be and PETSc v3.24.0 through the existing checksummed native acquisition/build owner. Uno starts with filtersqp/LBFGS/no inertia correction/HiGHS QP+LP and filterslp/HiGHS LP; never submit an indefinite exact Hessian to this QP route. Guard every foreign entrypoint/getter and Rust callback; recoverable trial returns and terminal latches use distinct Uno exception paths. PETSc uses real double, serial MPIUNI and an explicitly checked integer ABI with central initialization/lifetime and per-instance options. Admit SNESNEWTONTR only without variable bounds; distinguish invalid initial and recoverable trial points. Admit NASM only for domain-safe unbounded square blocks, preserving sub-SNES versus scatter ownership. Admit TSPSEUDO with explicit inner SNESNEWTONTR, candidate-domain rejection and library step reduction/growth; fixed mass stays unchanged through rejected retries and state-dependent mass contributes its derivative. Bound/domain-fragile composition uses admitted local solvers rather than overclaiming PETSc bounds. Native profiles declare effective controls, derivative/action demand, setup reuse and stop semantics. Installing a binding never changes explicit selection. Coordinate threads, job permits and cleanup with existing scopes.

### Consequences

The pinned Uno LBFGS model supplies a positive Hessian operator, while its HiGHS QP
binding requires an explicit triangular matrix. The scoped binding mechanically
materializes that library-owned operator through its public product action. It owns
no quasi-Newton update or curvature algorithm. Admit only a positive-definite operator,
check triangular capacity against the finite foreign allowance before allocation,
poll the original scope between columns, and record actual materialization work.
SLP consumes the native zero-Hessian form. An indefinite operator is not admitted.

The scoped SQP provider obtains a feasible point and basis through a HiGHS LP on
the current subproblem, then supplies those copied library results through
`setSolution` and `setBasis` with `qp_allow_hot_start` enabled. This avoids the
pinned QP initialization path's `1e-4` truncation of freshly computed small feasible
activities. Both calls consume the original required accuracy and finite
cancellation/deadline allowance. Keep objective and bounds unchanged at the QP
boundary; the feasibility LP uses its zero objective solely to construct the QP
start. Record actual additional LP calls and simplex iterations, effective native
settings and each native stop. QP completion and original-space assessment establish
the resulting solve outcome. This current-subproblem seed honors the explicit task
start and adds no inherited state permission.

Migrate producers and consumers together and remove replaced paths after targeted acceptance. The existing Plan 25m packet/finding table owns implementation progress; no parallel ledger is added.

### Compensating controls

Typed admission/refusal, finite work, actual native contract tests and independent original assessment constrain integration. No license restriction or hypothetical benchmark ranking excludes a selected library.

### Confirmation

**Proposed:** scoped decision review precedes changed-contract implementation. Prior source reviews establish the library/design basis, not acceptance or product qualification. Named checks are planned, not executed claims.

## Pros and cons

The target makes consequential distinctions explicit and permits isolated policy testing, at the cost of coordinated producer/consumer migration and foreign lifetime contracts.

## More information

[Implementation and disposition owner](../plans/25m-integrated-solve-pipeline.md); [source review](../design_review/reviews/design_review_integrated-solve-pipeline_2026-10-03.md).

## Status history

- 2026-10-03 — proposed for maintainer-authorized Plan 25m implementation; decision-PR acceptance and publication remain separate.
