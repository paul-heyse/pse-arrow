# basin

Realizations considered:
- **Not in the workspace lock.** `basin` 3.1.0 (published 2026-10-26) would be a new
  dependency.
- **Indirect arrival.** `basin` ^1.13 would arrive through `egobox-gp` 0.37.1 and
  `egobox-ego` 0.42.0 if the egobox family moved.
- **Dependencies.** Required: `rand` ^0.10, `rand_chacha` ^0.10, `rand_distr` ^0.6,
  `num-traits` and `web-time`. `rand_chacha` 0.10 is not yet in the lock; `rand` 0.10.3
  and `rand_distr` 0.6.0 are.
- **Backends.** Optional features include `faer_v0_24` and `nalgebra_v0_35`, plus
  nalgebra-sparse 0.12.
- **License and toolchain.** MIT OR Apache-2.0, edition 2024. MSRV is 1.87, or 1.89 with
  `nalgebra_v0_35`.

Research date 2026-10-27.

Sources:
- Downloaded crate `basin-3.1.0`: `README.md`, `Cargo.toml`, and
  `src/solver/{levenberg_marquardt.rs, bounded_levenberg_marquardt.rs, trust_region_reflective.rs, least_squares_diagnostics.rs}`,
  `src/core/executor.rs` (`CancellationToken`) and `src/core/math/faer_backend.rs`.
- crates.io metadata.
- Larsson (2026), arXiv:2608.11279, cited only as the authors' reference. Not read.

## Types

### basin/backend-generic-nonlinear-least-squares
- **Need it serves:** Dense or sparse nonlinear least squares on a chosen linear-algebra
  backend, unconstrained or box-bounded. Methods: Gauss–Newton, LM, bounded LM and
  trust-region reflective.
- **Recognizable code shapes:**
  - LM routes forced onto an algebra family different from the rest of the codebase.
  - Bounded fits handled by reparameterization.
  - Unbounded-only LM.
- **Mechanisms and APIs:**
  - **`LevenbergMarquardt<V, M, F>`.**
    - `with_damping(LmDamping::{TrustRegion (default), Nielsen})`.
    - Stopping tests: `with_relative_model_reduction_tolerance`,
      `with_relative_step_tolerance`, `with_gradient_orthogonality_tolerance`,
      `with_absolute_gradient_tolerance`, `with_relative_trust_radius_tolerance` and
      `with_no_progress_check`.
    - Damping controls: `with_initial_step_bound` (default 100) and `with_tau`.
    - Trial and diagnostic controls: `with_max_inner_attempts` and
      `with_trial_diagnostics`.
    - Linear solve: Cholesky on the normal equations, or `with_pivoted_qr()`, which
      returns `LevenbergMarquardtQr`.
  - **`BoundedLevenbergMarquardt`.** Coleman–Li scaling with strictly interior steps and
    Nielsen damping.
  - **`TrustRegionReflective`.** Dense; SciPy `trf`-style candidate selection; a
    pure-Rust one-sided Jacobi SVD; `BoxBounds`. Finite equal bounds fix a coordinate.
  - **Robust losses.** `RobustLeastSquares`.
  - **Executor.** `Executor::new(problem, solver, state).max_iter(n)`, with observers,
    checkpoints and a `CancellationToken`. Cancellation is cooperative, between
    iterations; a problem callback can stop sooner by returning a typed error.
  - **Native decision record.** `least_squares_diagnostics` gives per-trial
    observations of native decisions.
  - **Backends.**
    - `Vec<f64>` with `DenseMatrix` uses a pure-Rust Cholesky, QR and SVD.
    - nalgebra and ndarray use the same pure-Rust dense kernels through `MatrixIndex`
      for TRF. For LM, the docs list them at the dense tier.
    - faer `Col`/`Mat`: LM with the faer backend calls `faer::linalg::solvers::Llt` and
      faer `matmul`.
- **Obligations discharged:**
  - LM and trust-region iteration.
  - Bound handling (Coleman–Li, reflection).
  - Rank-aware subproblems.
  - Robust losses.
  - Cooperative cancellation.
  - Native trial diagnostics.
- **Residual caller obligations:**
  - Residual and Jacobian callbacks.
  - Mapping the termination code to domain outcomes.
  - Covariance and statistics; there is no fit-statistics layer.
  - Choosing a backend whose kernels meet any determinism requirement.
- **Semantic conditions:**
  - **Not a MINPACK reproduction.** The docs state that the default trust-region damping
    "does not reproduce MINPACK's complete algorithm or stopping behavior".
  - **Scaling is always on.** Marquardt scaling is a monotone `diag(JᵀJ)`. No on/off
    switch analogous to `with_scale_diag(false)` was found in `levenberg_marquardt.rs`.
  - **Determinism depends on backend.** The pure-Rust `Vec` path is scalar; no
    `mul_add` was found in `core/math` or the solver files. The faer backend inherits
    faer's runtime-dispatched SIMD and FMA kernels.
- **Version / build conditions:** Exact backend version features avoid resolution drift.
  If unification enables several versions, basin implements every enabled one.
- **Limits and negative results:**
  - TRF has no sparse or LSMR path.
  - Bounded LM uses normal equations only, with no QR.
  - No general constraints inside the least-squares solvers. General constraints go
    through augmented-Lagrangian or barrier wrappers.
- **Evidence:** Interface-checked: the doc comments and builder signatures in the
  downloaded 3.1.0 source listed above. Not compiled or run.

## Current applicability

### [CBO-035](../../../library_leverage/codebase/numerical-workflows.md#cbo-035) — Separate dense-algebra universes
- **Types:** `basin/backend-generic-nonlinear-least-squares`.
- **Realization:**
  - A new dependency, `basin` 3.1.0 with `faer_v0_24` or `nalgebra_v0_35`. It adds
    `rand_chacha` 0.10.
  - Alternatively it arrives indirectly at ^1.13 through an egobox family upgrade. A
    direct 3.x would then coexist with egobox's 1.x line; this was not verified.
- **Conclusion:** fits with conditions. It is the only inspected released route
  offering LM, bounded LM and trust-region reflective on either the workspace's faer 0.24
  or nalgebra 0.35. It could lift the CBO's "unbounded" restriction. It would change the
  LM algorithm and stopping semantics, however, and the CBO names those as part of fit
  identity.
- **What it would discharge / what remains local:**
  - **Discharged.**
    - The LM iteration and bounded variants.
    - nalgebra use in pse-runtime, if the faer or `Vec` backend is chosen.
    - The git pin.
  - **Local.**
    - Mapping the criteria policy: the current `ftol`/`xtol`/`gtol`/`scale_diag`/
      `patience` set onto basin's tolerances, with no exact `scale_diag(false)`
      equivalent.
    - Termination mapping.
    - Fit identity versioning.
    - Cancellation, through callback errors or a token.
- **Prerequisites, configuration or extension:**
  - Choose the `Vec` backend to stay scalar and FMA-free, or the faer backend to share
    the universe and accept FMA kernels.
  - Re-establish the evidence fields that record LM criteria.
- **Semantic gaps against the CBO's preservation constraints:**
  - "The LM criteria policy … is part of fit identity" and "the non-MINPACK defaults
    are a contract". basin's tests and damping differ from rust-cv LM, so a switch is a
    deliberate fit-identity change, not a transparent replacement.
  - There is no scaling-off switch.
- **Unsettled premises and what would settle them:**
  - Whether the comparative review values bounded LM/TRF more than LM continuity.
  - Whether the LM route outperforms the NLP Gauss–Newton route at all (the CBO's own
    question).
  - basin's maturity: version 3.x reached in 2026, with ~106k downloads.
  - A behavioral comparison on the eligible fit fixtures would settle the trade-off.
- **Evidence:** Interface-checked. The application is Proposed.
