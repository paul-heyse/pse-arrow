# faer (dense factorizations for covariance square roots)

Realizations considered: installed `faer` 0.24.4, with the workspace features `std` and
`sparse-linalg`. Its dependencies are `pulp` 0.22.x with feature `x86-v3`, `gemm` 0.19,
`nano-gemm` 0.2 and `private-gemm-x86`. No upgrade was considered; 0.24.4 is the latest
release. Research date 2026-10-27.

Sources:
- Installed `faer-0.24.4/Cargo.toml` (features and pulp `x86-v3`).
- `faer-0.24.4/src/linalg/matmul/mod.rs` (`simd.mul_add` in the kernels).
- The list of `faer-0.24.4/src/linalg/**` files that use `mul_add`, including Cholesky
  LLT update, LDLT factor, Bunch–Kaufman, Householder and SVD.
- `pulp-0.22.3/src/lib.rs` (`Arch::new()` runtime dispatch).
- The repository's existing use of faer in `uncertainty/psd.rs`
  (`self_adjoint_evd_scratch`, `Par::Seq`).

## Types

### faer/dense-covariance-square-root
- **Need it serves:** A matrix square root of a symmetric PSD covariance, used to turn
  independent standard normal or QMC deviates into correlated parameter samples or sigma
  points.
- **Recognizable code shapes:**
  - Hand-written Cholesky.
  - `Σ = L Lᵀ` then `θ = θ̂ + L z`.
  - Eigen-decomposition followed by `V √Λ`.
- **Mechanisms and APIs:**
  - `Mat::llt(Side::Lower)` / `faer::linalg::solvers::Llt`, which fails on a matrix that
    is not positive definite.
  - `Mat::ldlt`.
  - `self_adjoint_eigen` / `linalg::evd::self_adjoint_evd` with
    `ComputeEigenvectors::Yes`, giving `Σ = V Λ Vᵀ`. This accepts singular PSD matrices
    and allows clipping or refusing negative eigenvalues.
  - `Par::Seq` to disable thread parallelism.
- **Obligations discharged:** Numerically stable factorization and eigen-decomposition.
- **Residual caller obligations:**
  - The PSD policy: refuse negative eigenvalues rather than clamp them.
  - Choose LLT versus EVD for singular covariances.
  - The transform `θ̂ + L z` itself.
  - Determinism, if bit-reproducibility is required (see below).
- **Semantic conditions:**
  - faer's dense kernels are SIMD-vectorized through `pulp` with runtime CPU-feature
    dispatch (`Arch::new()`). They use fused multiply-add (`simd.mul_add`) where the CPU
    supports it. Blocked algorithms also fix a summation order that differs from naive
    loops.
  - Consequently results are deterministic on a given CPU class, but they can differ in
    the last bits between x86-64 machines with and without FMA/AVX2 and between x86-64
    and aarch64.
  - `Par::Seq` removes thread nondeterminism but not SIMD dispatch.
  - No public switch forcing scalar, non-FMA kernels was found in the inspected sources.
- **Version / build conditions:** `pulp` is compiled with `x86-v3`. The `nightly` feature
  would add `x86-v4`.
- **Limits and negative results:**
  - Search scope: the faer 0.24.4 `Cargo.toml` features and the `pulp` lib docs. No
    "deterministic" or "scalar-only" mode was found.
  - This is an absence of evidence, not proof that none exists.
- **Evidence:** Interface-checked: the files above. The FMA effect was not measured
  across CPUs.

## Current applicability

### [CBO-034](../../../library_leverage/codebase/numerical-workflows.md#cbo-034) — Only first-order uncertainty propagation
- **Types:** `faer/dense-covariance-square-root`.
- **Realization:** Installed 0.24.4.
- **Conclusion:** fits with conditions.
  - The square root is available, and EVD is already in use for PSD admission (eigenvalues
    only). Sigma-point or sampling routes would extend that call to eigenvectors.
  - faer does not meet a cross-CPU, FMA-free bit-reproducibility premise. If correlated
    draws must regenerate bit-identically across machines, the parameter-count-sized
    factorization would need the repository's existing scalar FMA-free style (as the
    covariance sandwich already does), or the draws must be recorded as inputs.
- **What it would discharge / what remains local:**
  - **Discharged.** The factorization.
  - **Local.**
    - The PSD and negative-eigenvalue refusal policy ("refused, not clamped").
    - Sigma-point weights and points: the unscented transform has no credible library in
      the inspected Rust ecosystem; see the README.
    - The per-sample fully qualified solves.
    - The output statistics.
- **Prerequisites, configuration or extension:** `Par::Seq`. Use EVD with eigenvectors
  for singular PSD covariances.
- **Semantic gaps against the CBO's preservation constraints:** Bit-reproducibility
  across CPUs is not provided, for the SIMD and FMA reasons above.
- **Unsettled premises and what would settle them:**
  - Whether the reproducibility contract is per platform or universal.
  - A two-CPU comparison (FMA versus non-FMA) of faer `llt` on a small covariance would
    settle observed variation. That probe is not authorized here.
- **Evidence:** Interface-checked. The application is Proposed.

### [CBO-035](../../../library_leverage/codebase/numerical-workflows.md#cbo-035) — Separate dense-algebra universes
- **Types:** `faer/dense-covariance-square-root`, used as the backend condition.
- **Realization:** Installed.
- **Conclusion:** fits with conditions as the target universe for a least-squares route,
  via `basin`'s faer 0.24 backend (see `basin.md`). Note that faer's FMA kernels would
  then enter the LM normal-equation solves, where today's MINPACK-style QR in
  `levenberg-marquardt` is scalar.
- **What it would discharge / what remains local:** n/a beyond `basin.md`.
- **Prerequisites, configuration or extension:** See `basin.md`.
- **Semantic gaps against the CBO's preservation constraints:** A determinism change
  relative to the scalar QR route.
- **Unsettled premises and what would settle them:** Whether the LM fit identity
  includes bitwise iterates or only criteria and policy.
- **Evidence:** Interface-checked.
