# argmin / argmin-math

Realizations considered:
- **Installed.** `argmin` 0.11.0 with default features off, and `argmin-math` 0.5.1.
  pse-runtime uses only `argmin::core::{CostFunction, Problem, Solver, State,
  TerminationStatus}`, for egobox interop in the surrogate library.
- **Newer releases.** None. argmin 0.11.0 (2025-09-28) and argmin-math 0.5.1
  (2025-10-02) are the latest.
- **Upstream main.** `argmin-math`'s `Cargo.toml` on main still lists `faer_v0_20` to
  `faer_v0_23` and `nalgebra_v0_29` to `nalgebra_v0_34`.

Research date 2026-10-27.

Sources:
- crates.io dependency metadata for argmin-math 0.5.1.
- GitHub `argmin-rs/argmin` main `crates/argmin-math/Cargo.toml`, and the latest commits
  through 2025-10-02.

## Types

### argmin/backend-generic-local-solvers
- **Need it serves:** Backend-generic unconstrained local optimization, including
  Gauss–Newton and trust-region subproblem solvers (Cauchy, Dogleg, Steihaug).
- **Recognizable code shapes:**
  - `Executor::new(problem, solver).configure(|s| s.param(x0).max_iters(n))`.
  - `ArgminOp`/`CostFunction`/`Gradient`/`Jacobian`/`Hessian` impls.
- **Mechanisms and APIs:**
  - `GaussNewton`, `GaussNewtonLS`, `TrustRegion<Dogleg | Steihaug | CauchyPoint>`,
    L-BFGS, BFGS and Newton.
  - Math backends selected by `argmin-math` version features.
- **Obligations discharged:** The iteration drivers and line searches.
- **Residual caller obligations:** Bounds, which argmin's local solvers do not support,
  and least-squares-specific damping.
- **Semantic conditions:** As for whichever backend is selected.
- **Version / build conditions:** argmin-math 0.5.1 supports faer only up to 0.23 and
  nalgebra only up to 0.34. It supports neither faer 0.24 nor nalgebra 0.35.
- **Limits and negative results:**
  - Search scope: argmin-math 0.5.1 features and main `Cargo.toml`, and argmin's solver
    list as known from its documentation.
  - There is no Levenberg–Marquardt solver and no box-bounded least-squares solver.
- **Evidence:** Interface-checked: the crates.io dependency list and the main
  `Cargo.toml` (2026-10-27).

## Current applicability

### [CBO-035](../../../library_leverage/codebase/numerical-workflows.md#cbo-035) — Separate dense-algebra universes
- **Types:** `argmin/backend-generic-local-solvers`.
- **Realization:** Installed 0.11.0 / 0.5.1. No upgrade exists.
- **Conclusion:** ruled out here. argmin-math cannot bind to the workspace's faer 0.24 or
  nalgebra 0.35, and argmin offers no LM or bounded least-squares method that would
  replace the LM route.
- **What it would discharge / what remains local:** Nothing.
- **Prerequisites, configuration or extension:** n/a.
- **Semantic gaps against the CBO's preservation constraints:** n/a.
- **Unsettled premises and what would settle them:** An argmin-math release adding
  faer 0.24 or nalgebra 0.35. None was visible on main as of 2026-10-02.
- **Evidence:** Interface-checked.
