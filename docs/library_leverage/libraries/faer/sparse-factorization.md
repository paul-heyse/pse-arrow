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
- **Evidence:** Interface-checked: source files above; crates.io/GitHub version listings.

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

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) — Prepaid accounting (faer part)
- **Types:** faer/sparse-lu-scratch-and-reuse
- **Conclusion:** fits with conditions — the scratch prediction is real and already used. Factor
  storage has no prediction, so a conservative caller bound (or post-hoc observation) remains
  necessary for LU. For Cholesky/LDLᵀ/QR, `len_val()` would discharge it.
- **Evidence:** Interface-checked; application Proposed.
