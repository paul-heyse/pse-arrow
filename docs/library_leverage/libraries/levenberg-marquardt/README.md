# levenberg-marquardt (rust-cv)

Realizations considered:
- **Installed.** `levenberg-marquardt` 0.15.0 from git rev `e67066cf13` (merged
  2026-06-17, PR #32 "Bump nalgebra to 0.35"), on `nalgebra` 0.35.0.
- **Latest crates.io release.** 0.15.0 (2025-08-03), on `nalgebra` ^0.34.
- **No newer release.** Upstream issue #33, "New release with updated nalgebra
  version?", is open. Open PRs include #34 (perf), #30 (num-dual forward-AD Jacobians)
  and #28 (IEEE-754 zero tolerance checks).

Research date 2026-10-27.

Sources:
- GitHub `rust-cv/levenberg-marquardt`: commits, issues and pull requests, read through
  `gh api`.
- crates.io dependency metadata.
- Installed `levenberg-marquardt-0.15.0` source (`src/qr.rs`, `src/lm.rs` listing).
- The repository uses `with_ftol`, `with_xtol`, `with_gtol`, `with_scale_diag`,
  `with_patience` and `minimize`, in `fitting/residual_policy.rs` and
  `fitting/oracle/residual.rs`.

## Types

### levenberg-marquardt/minpack-style-lm
- **Need it serves:** Unconstrained nonlinear least squares with MINPACK-`lmder`-style
  criteria, over small dense problems.
- **Recognizable code shapes:**
  - An implementation of `LeastSquaresProblem<F, M, N>`, with `set_params`, `params`,
    `residuals` and `jacobian`, each returning `Option`.
  - `LevenbergMarquardt::new().with_*().minimize(problem)` returning
    `(problem, MinimizationReport)`.
- **Mechanisms and APIs:**
  - Builder settings: `with_ftol`, `with_xtol`, `with_gtol`, `with_stepbound`,
    `with_patience`, `with_scale_diag(bool)` and `with_tol`.
  - `TerminationReason` variants.
  - An internal `PivotedQR` written in MINPACK style, with an `lmpar` analogue.
  - Optional `minpack-compat` behavior.
- **Obligations discharged:** The LM iteration, the trust-region parameter, scaling, and
  termination classification.
- **Residual caller obligations:**
  - Bounds. There are none; a caller would need transformations or a different solver.
  - Jacobian supply.
  - Cancellation, done only by returning `None` from callbacks.
  - Mapping results into nalgebra `DMatrix`/`DVector`.
- **Semantic conditions:**
  - Its QR and solve kernels are scalar Rust loops, not BLAS or `matrixmultiply`. They
    are deterministic across CPUs, subject to Rust's no-contraction semantics.
  - The default criteria are not MINPACK's unless configured.
- **Version / build conditions:**
  - nalgebra 0.35 support exists only at the git revision. The released 0.15.0 is on
    0.34.
  - The current API has no faer or ndarray backend.
- **Limits and negative results:** No bounds, no general constraints and no robust losses.
- **Evidence:** Interface-checked: GitHub commit, issue and PR listings (2026-10-27),
  crates.io metadata and the installed `qr.rs`.

## Current applicability

### [CBO-035](../../../library_leverage/codebase/numerical-workflows.md#cbo-035) — Separate dense-algebra universes
- **Types:** `levenberg-marquardt/minpack-style-lm`.
- **Realization:** Installed git rev. No released route exists to the same code on
  nalgebra 0.35.
- **Conclusion:** fits as is; the CBO's question "is there a released LM crate on
  nalgebra 0.35" has a negative answer as of 2026-10-27. Corrections to the premise:
  - The nalgebra 0.35 universe is not carried by LM alone. `nalgebra` 0.35.0 is also
    required by `diffsol-la` (non-optional, native `solver-diffsol` builds), `num-dual`,
    `feos`/`quantity` (xtask) and pse-runtime's direct dependency.
  - Removing LM would therefore not remove nalgebra 0.35 from native builds. It would
    remove only pse-runtime's direct nalgebra use, which appears in two files.
- **What it would discharge / what remains local:** Unchanged from today.
- **Prerequisites, configuration or extension:** Keep the git pin until a release lands,
  as the pin reason in `[workspace.metadata.pse.pins]` already says.
- **Semantic gaps against the CBO's preservation constraints:** None. The LM criteria
  policy and non-MINPACK defaults are preserved by staying.
- **Unsettled premises and what would settle them:** Whether upstream issue #33 leads to
  a 0.16 release.
- **Evidence:** Interface-checked: GitHub and crates.io, plus `Cargo.lock` reverse
  dependencies of `nalgebra` 0.35.0.
