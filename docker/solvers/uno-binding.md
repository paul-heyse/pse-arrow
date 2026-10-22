<!--
SPDX-License-Identifier: MIT OR Apache-2.0
Copyright (c) 2026 Paul Heyse
-->

# Scoped Uno numerical binding

This declaration accompanies [uno-pse.patch](uno-pse.patch) and the
[exception-shielded bridge](../../crates/pse-uno-sys/bridge/pse_uno.cpp).
ADR-0157 and ADR-0215 own the decision; blueprint §16.2 owns the execution contract.
The source revision and build options are declared in the workspace manifest.
The native receipt includes the patch identity; the Rust build identity includes that
receipt and the bridge sources, and every solve records that identity.

## Admitted methods and integration

The binding retains filter trust-region SQP with Uno's positive L-BFGS operator and
SLP with zero Hessian. Uno owns its update, curvature, globalization and restoration
algorithms. HiGHS owns the LP/QP solves. Original-space validation and explicit start
permissions remain with the Rust execution owners.

Scope callbacks, original deadlines forwarded to each HiGHS call, terminal memory and
cancellation latches, bounded observation/readback, and exception containment are
integration. `threads=1`, `parallel=off` and the serial shared HiGHS scheduler are also
determinism-bearing controls. They remain admitted controls, rather than evidence that
arbitrary parallel HiGHS execution has been qualified.

The `NoRelaxation` patch publishes the merit globalization already selected by Uno.
At the pinned revision, `NoRelaxation` owns that strategy independently of the constrained
preset; its option override is retained in `Result` and read by
`uno_get_solver_string_option`. It is not reapplied as a new optimizer selection.

## Four retained numerical changes

| Change | Meaning and reason | Observable controls |
|---|---|---|
| Outer accuracy forwarded into HiGHS | Set primal feasibility from the admitted outer primal tolerance, and dual feasibility/optimality from its dual tolerance. Refuse below the linked HiGHS floor. | Effective primal, dual and optimality readbacks; required outer accuracy remains independently checked. |
| Zero QP regularization | Set `qp_regularization_value=0` for the admitted positive-definite L-BFGS profile, avoiding an additional diagonal model perturbation. | Effective regularization readback. Indefinite operators remain refused. |
| Current-subproblem feasibility seed | For a constrained QP, solve an LP with the current linear constraints and zero objective; copy that LP's solution and basis into the current HiGHS QP with hot start enabled. This avoids the pinned QP initialization's small-activity truncation. | Actual additional LP calls, available simplex iterations/status, hot-start and LP tolerance readbacks. It imports no previous task state. |
| Explicit L-BFGS operator materialization | Apply Uno's operator to coordinate vectors and retain the lower triangle for HiGHS. This preserves the supplied action while introducing quadratic representation/storage and corresponding rounding/order effects. | Actual columns, completed materializations and largest numeric triangle bytes; refusal precedes dense representation allocation. |

These changes are part of the selected numerical binding. Operator materialization does
not become a new quasi-Newton algorithm, and its representation cost is not linear in
L-BFGS history. A native success or stalled candidate establishes no global certificate.
Named native optimization/iterate statuses are checked against the upstream C header;
`UNO_INFEASIBLE_SMALL_STEP` is `Numerical`. Callback and scope causes retain precedence.

## Dense SQP support derived from the admitted allowance

Let `n` be the original model variable count, `k` its positive L-BFGS history, and `B`
the actual finite foreign-memory allowance in bytes. The bridge first reserves the
existing history/vector projection:

```text
H(n,k) = 64*k*k + 64*n*k + 128*n + 64*k
A = B - H(n,k), requiring H(n,k) < B
```

Both original optimality/restoration histories use the original model extent `n`.
The materialization check uses the actual native subproblem dimension `d`, including
any added restoration/slack variables, rather than assuming `d=n`:

```text
t = d*(d+1)/2
D(d) = 64*t + 16*d = 32*d*d + 48*d
require d <= INT32_MAX, t <= INT32_MAX, D(d) <= A
```

The 64-byte coefficient covers the existing COO/index/permutation, CSC and model-copy
projection; `16*d` covers two action vectors. Thus the largest dimension permitted by
this projection is the largest integer `d` satisfying those inequalities, equivalently
`min(65535, floor((sqrt(2304 + 128*A) - 48)/64))` for nonnegative `A`. This mathematical
expression explains the bound; the actual guard retains checked integer arithmetic.
The smaller allowance bound usually dominates the index ceiling.

For the one-variable, history-four control, `H(1,4)=1664` and `D(1)=80`. An allowance
of 1700 admits history but leaves only 36 bytes, so the actual subproblem is refused
before producing a dense Hessian or any operator column. SLP has no such histories or
materialization; its bridge accepts a zero SQP materialization allowance and reports no
materialization work. SLP still requires its ordinary complete task resource admission.

These are explicit admission projections, not allocator interception or a measured
maximum workload. Other native/library allocations remain covered by the enclosing
foreign allowance and lifetime. A larger allowance does not qualify a larger scientific
problem. No fixed 1450-variable capacity claim follows from a default-envelope example.

## Upstream equivalence inquiry

**Interface-checked, 2026-10-21:** the pinned upstream
[HiGHS representation](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/uno/ingredients/subproblem_solvers/HiGHS/HiGHSQuadraticProgram.cpp)
requires an explicit Hessian matrix. The published
[v2.9.1 representation](https://github.com/cvanaret/Uno/blob/3e2fd1b9bc965c50195de22a439a84b407f8c347/uno/ingredients/subproblem_solvers/HiGHS/HiGHSQuadraticProgram.cpp)
still has that requirement, and its
[HiGHS solver](https://github.com/cvanaret/Uno/blob/3e2fd1b9bc965c50195de22a439a84b407f8c347/uno/ingredients/subproblem_solvers/HiGHS/HiGHSSolver.cpp)
does not implement the three selected HiGHS policy changes above. A version bump alone
therefore does not replace this binding while preserving its admitted meanings.

There is a genuine upstream matrix-free candidate: pinned
[BQPDQuadraticProgram](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/uno/ingredients/subproblem_solvers/BQPD/BQPDQuadraticProgram.cpp)
accepts a positive Hessian operator, and
[BQPDSolver](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/uno/ingredients/subproblem_solvers/BQPD/BQPDSolver.cpp)
consumes an initial point. This could preserve outer SQP/L-BFGS while changing the QP
engine. Technical end-to-end equivalence has not been established: provider acquisition,
ABI, tolerances, starting/feasibility behavior, finite scope, memory ownership, serial
execution, observations and original-space controls need a separate qualified composition.
No licensing rejection or missing-source claim is implied.

The pinned
[subproblem factory](https://github.com/cvanaret/Uno/blob/805a27ecf802eec33d1aea85ae73efa8a24f71be/uno/ingredients/subproblem_solvers/SubproblemSolverFactory.hpp)
restricts the Woodbury alternative to the appropriate equality-only/no-bounds/no-trust-region
route; it cannot cover the current bounded trust-region profile. Uno IPM would change the
major method. The selected outcome therefore retains the declared HiGHS composition;
revisit when an upstream HiGHS operator route or a qualified BQPD composition preserves
the required behavior. Inspection covers the named pinned sources and v2.9.1, not every
upstream branch or future release.

The targeted controls live in `pse-uno-sys` and `pse-backend-native::uno::tests`:
`operator_storage_limit_is_terminal_before_matrix_allocation`,
`bounded_slp_solves_and_reports_effective_curvature`, and
`actual_operator_storage_refusal_preserves_typed_resource_failure`. Their execution and
assembled acceptance remain recorded by the owning Plan49 runtime/qualification scope.
