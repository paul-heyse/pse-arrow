# faer (sparse LU)

Realizations considered: installed `faer` 0.24.4 (features `std`, `sparse-linalg`). 0.24.4
(crates.io 2026-06-24; GitHub release 2026-07-01) is the newest release, so there is no upgrade
route. A "faer 0.2x/0.3x" newer than the pin does not exist as of the research date.
Research date 2026-10-27. Sources: installed `faer-0.24.4/src/sparse/linalg/{lu.rs,cholesky.rs,qr.rs}`,
`src/operator/mod.rs`; crates.io version list; GitHub releases of sarah-quinones/faer-rs. Context7
was not consulted for this narrow source question.

## Types

### faer/sparse-lu-scratch-and-reuse
- **Need it serves:** a sparse LU with partial pivoting whose transient workspace is sized before
  allocation and whose symbolic analysis is reused across numeric refreshes.
- **Recognizable code shapes:** `SymbolicLu::try_new`, `factorize_numeric_lu_scratch` → `StackReq`,
  `MemBuffer`/`MemStack`, a retained `NumericLu` refreshed by `factorize_numeric_lu`.
- **Mechanisms and APIs:** `sparse::linalg::lu::factorize_symbolic_lu`, `SymbolicLu::{factorize_numeric_lu_scratch,
  factorize_numeric_lu, solve_in_place_scratch, solve_transpose_in_place_scratch}`, `NumericLu::new()`,
  `LuRef::{solve_in_place_with_conj, solve_transpose_in_place_with_conj}`, `Par::Seq`; errors
  `LuError::{SymbolicSingular{index}, Generic(FaerError::{OutOfMemory, IndexOverflow})}`; internal growth
  uses `try_reserve`, so allocation failure is returned rather than aborting.
- **Obligations discharged:** ordering (COLAMD), supernodal/simplicial dispatch, the scratch
  requirement as a `StackReq` before allocation, multi-RHS and transposed solves, fallible
  allocation of factor storage.
- **Residual caller obligations:** detecting a numerical zero pivot (faer can finish at an exact
  zero pivot; only a nonfinite solution reveals it), any condition estimate, retained-storage
  accounting.
- **Semantic conditions:** factor (L/U values) storage is sized dynamically during numeric
  factorization. `NumericLu` hides its retained capacity; the simplicial variant exposes
  `l_factor_unsorted`/`u_factor_unsorted` (post-hoc nnz), and the supernodal variant exposes only
  dimensions and supernode counts.
- **Version / build conditions:** 0.24.x API.
- **Limits and negative results:** no condition-number estimator anywhere in faer 0.24.4 (searched
  `src/` for `cond`, `rcond`, `condest`, `condition`). No a-priori LU fill. By contrast, faer's sparse
  Cholesky/LDLᵀ and QR symbolic objects expose `len_val()` (exact factor size from symbolic
  analysis). The `operator::BiPrecond` trait (apply and transpose-apply) is a natural host for
  wrapping an LU as an inverse operator, but it ships no estimator.
- **Structural fill bound (added 2026-10-27):** public lower-level APIs
  (`sparse::linalg::colamd::order`, `qr::column_counts_ata`, `SymbolicLu::col_perm`) allow a
  George–Ng structural upper bound on LU fill (the fill of the Cholesky factor of AᵀA under the
  column order). That bound does not cover the supernodal path's relaxed-supernode padding or the
  `try_reserve` growth, so it is not a complete storage bound; a `SymbolicLu` storage-bound API
  would be an upstream addition.
- **Evidence:** Interface-checked: source files above; crates.io/GitHub version listings.

### faer/runtime-simd-dispatch
- **Need it serves:** knowing whether faer's kernels meet a fused-multiply-add-free or
  cross-CPU bit-reproducibility premise.
- **Mechanisms:** faer 0.24.4 declares `pulp` with `features = ["x86-v3"]` unconditionally, so
  `default-features = false` on faer does not remove it. pulp selects its V3 path at run time
  (`Arch::new()`), and that path uses `_mm256_fmadd_pd`. Dense kernels, including the `matmul`
  that supernodal sparse LU calls, can reach it. With `std` on x86_64, matmul also directly
  selects `private_gemm_x86::InstrSet::{Avx512, Avx256}` by CPU features; small products use
  `nano_gemm::planless` and another path uses `gemm`. Pulp is not the only dispatch owner.
- **Semantic conditions:** results can differ between CPU classes with and without FMA under the
  same build. A compile-time rule against `target-cpu=native` does not reach this route.
- **Residual caller obligations:** a no-FMA policy requires a configuration/patch covering the
  full consumed kernel closure; a pulp-only patch is insufficient. ISA attribution explains
  execution but does not remove FMA. No complete no-FMA route or performance cost was established.
- **Evidence:** Interface-checked (faer 0.24.4 `Cargo.toml`, pulp 0.22.3 `src/x86/v3.rs`), 2026-10-27.
  Not run on a host without FMA.

## Current applicability

### [CBO-026](../../../library_leverage/codebase/native-solver-integration.md#cbo-026) — Sparse factorization wrappers (faer part)
- **Types:** faer/sparse-lu-scratch-and-reuse
- **Realization:** installed 0.24.4; no upgrade route.
- **Conclusion:** ruled out here for a condition estimate or storage size — confirmed absent.
  The research question "does a newer faer expose them?" is moot (no newer release).
- **What it would discharge / what remains local:** faer remains the strongest of the three for
  allowance-before-scratch. A condition estimate comes from composing FERAL's public driver over
  faer's solves (see `feral.md`). The two near-isomorphic faer wrappers (square response and
  arclength tangent) are a local consolidation question; no faer facility merges them.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** There are three faer LU owners, not two: Diffsol `FaerLu`
is the third. Diffsol `FaerLu` maps every faer failure to an untyped `LinearSolverError::Other`,
which the integrator reports as numerical, so `FaerError::OutOfMemory` is misclassified, and its
scratch is never compared with an allowance. The comparative review recommends one crate-local
faer-LU helper for the three owners (scratch admission, `LuError`/`FaerError` classification, a
finite-solve check and one shared `opaque_allowance(n)`) and rejects composing FERAL's estimator
over faer. See the [comparative review](../../reviews/native-solver-integration.md#cbo-026).

**Reviewer correction (2026-10-27).** faer's run-time FMA dispatch (type above) contradicts the
"no FMA" rule the first review pass listed for these owners; the review records it as remedy-open.
See the [comparative review](../../reviews/native-solver-integration.md#faer-kernels-and-the-fma-free-premise).

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) — Prepaid accounting (faer part)
- **Types:** faer/sparse-lu-scratch-and-reuse
- **Conclusion:** fits with conditions — the scratch prediction is real and already used. Factor
  storage has no prediction, so a conservative caller bound (or post-hoc observation) remains
  necessary for LU. For Cholesky/LDLᵀ/QR, `len_val()` would discharge it.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The structure-blind complete bounds (n²·64, n²·32) stay
unresolved pending a measurement; the George–Ng structural bound (see the type) is not complete,
so a tighter bound would need an upstream `SymbolicLu` storage-bound API. See the
[comparative review](../../reviews/native-solver-integration.md#cbo-001-native).
