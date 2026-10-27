# FERAL

Realizations considered: installed `feral` 0.18.0 from the POUNCE fork at revision
7f5e2e9d78555bd53f780c1af186c8fd22c922f5 (`vendor/feral` in that tree; the same FERAL that
`pounce-feral` resolves). No newer fork revision or upstream FERAL release was assessed.
Research date 2026-10-27. Sources: installed source `~/.cargo/git/checkouts/pounce-*/7f5e2e9/vendor/feral/src/`
— `lib.rs` re-exports, `lu/{mod.rs,sparse_factor.rs,sparse_symbolic.rs,sparse_update.rs,condition.rs,sparse_solve.rs}`,
`numeric/{condition.rs,solver.rs}`, `symbolic/mod.rs` (`SymbolicFactorization`, `WorkEstimate`).

Provenance note (added 2026-10-27): the FERAL in use is the copy vendored inside a maintained
POUNCE fork (`paul-heyse/pounce`, extending upstream `a31dab9c`), not an upstream FERAL release.
`Solver::conservative_storage_bytes` and `with_bounded_storage_max_dimension` are fork additions
(commit `a695be61`, documented in the fork's `source-extension.md`), so new owner bounds of that
kind are fork changes following existing practice rather than upstream additions.

## Types

### feral/hager-higham-estimator-over-any-factor
- **Need it serves:** one 1-norm condition estimate (`κ₁ ≈ ‖A‖₁·‖A⁻¹‖₁`) for any factored
  square matrix, regardless of which library produced the factor.
- **Recognizable code shapes:** several factor wrappers that compute different conditioning or
  accuracy proxies for the same matrix; hand-written Hager/Higham or DLACON loops; KLU
  `klu_condest` beside another library's estimate.
- **Mechanisms and APIs:** public trait `feral::HagerHighamOperator { dim, apply_inverse(&mut [f64]),
  apply_inverse_transpose(&mut [f64]) }` and driver `feral::hager_higham_inverse_norm_1(&mut op)`.
  `feral::matrix_norm_1` supplies `‖A‖₁`. Built-in adapters: `SparseLu::condition_estimate_1(&b)`,
  `DenseLu::condition_estimate_1(&b)`, `Solver::estimate_condition_1norm(&matrix)` (LDLᵀ,
  symmetric: one solve per iteration). The driver implements Hager (1984) with Higham's (1988)
  alternating-sign refinement, LAPACK DLACON conventions, a cycle guard and an iteration cap.
  Unsymmetric operators cost about 2 solves per iteration, at most about 11 solves in total.
- **Obligations discharged:** the estimator algorithm, its termination and tie conventions.
- **Residual caller obligations:** implement the two solves over the foreign factor (forward and
  transposed). Map solve errors into `FeralError` (the trait's error type). Account for three
  `n`-length work vectors the driver allocates. Supply the matrix for `‖A‖₁` (factors do not
  retain it).
- **Semantic conditions:** the result is a **lower bound** on κ₁. It is deterministic for a
  deterministic operator (fixed start vector `1/n`, serial loop). The estimate describes whatever
  matrix the operator inverts, so a scaled (normalized) factor yields the scaled condition number.
- **Version / build conditions:** exported from `feral` 0.18.0 at the fork revision above.
- **Limits and negative results:** 1-norm only; no ∞-norm or componentwise (Skeel) estimate.
- **Evidence:** Interface-checked: `numeric/condition.rs` (trait and driver), `lib.rs` exports,
  `lu/condition.rs` adapters.

### feral/unsymmetric-sparse-lu
- **Need it serves:** factor and solve an unsymmetric sparse square matrix (Jacobian, basis),
  including transposed solves, with typed singularity.
- **Recognizable code shapes:** `SparseLu::factor_markowitz`, `SparseLu::factor(&a, &symbolic, params)`,
  `ftran`/`btran` calls.
- **Mechanisms and APIs:** `SparseColMatrix::from_sparse_columns`; `SparseLuSymbolic::{analyze, analyze_with,
  analyze_triangularized, analyze_amd_only, natural, with_order}`; `SparseLu::factor` (column order
  from a reusable symbolic, `LuPivoting::GilbertPeierls`) or `factor_markowitz` (own dynamic order,
  default `LuPivoting::Markowitz`); `refactor(&a, &symbolic)` (re-factors from scratch, reusing the
  ordering, not storage); `ftran`/`btran` (one right-hand side, in place), `ftran_refined`/`btran_refined`,
  `ftran_sparse`/`btran_sparse`; Forrest–Tomlin `update` for basis changes; `LuParams { pivot_threshold,
  zero_pivot_tol, on_singular: LuSingularAction, scaling: LuScaling, refine_steps, ... }`;
  diagnostics `factor_nnz()`, `growth()`, `used_markowitz()`.
- **Obligations discharged:** ordering, pivoting, singular-pivot detection with a typed
  `FeralError::SingularBasis { column }` (factor time, under `LuSingularAction::Fail`),
  refinement and transposed solves.
- **Residual caller obligations:** multiple right-hand sides (loop over columns), any
  before-allocation storage prediction, scratch admission, observation.
- **Semantic conditions:** LU applies its own row/column scaling (`D_row Π A D_col`) according to
  `LuParams::scaling`. A caller with its own normalization convention must choose the scaling
  mode deliberately to avoid double scaling. Markowitz accepts non-maximal pivots (`max|L| ≤ 1/u`),
  so element growth is bounded more weakly than partial pivoting, and its fill is decided
  during factorization. Solves use thread-local scratch buffers that persist on the calling
  thread. The LU path does not use FMA.
- **Version / build conditions:** as above.
- **Limits and negative results:** no a-priori fill or storage prediction for Markowitz or
  Gilbert–Peierls LU (pivoting is dynamic); no caller-provided scratch/arena; no multi-RHS solve;
  `refactor` reallocates.
- **Evidence:** Interface-checked: `lu/mod.rs` `LuParams` docs, `sparse_factor.rs`, `sparse_update.rs::refactor`,
  `sparse_solve.rs`.

### feral/symmetric-ldlt-symbolic-estimates
- **Need it serves:** before-numeric memory/work prediction and certified inertia for symmetric
  indefinite (KKT) factorizations.
- **Mechanisms and APIs:** `feral::symbolic::symbolic_factorize(&matrix, &params)` →
  `SymbolicFactorization { factor_nnz_estimate, factor_slack, contrib_sizes, peak_contrib_bytes, col_counts, ... }`
  and `work_estimate()` → `WorkEstimate { factor_nnz, factor_alloc_nnz, factor_bytes, peak_bytes, ... }`;
  `Solver::work_estimate()` (stable across refactorizations of one pattern); `Solver::with_parallel(false)`,
  `with_fma(false)` (default, bit-exact), `with_bounded_storage_max_dimension(n)` together with
  `Solver::conservative_storage_bytes(n, nrhs)` (a complete conservative owner extent for the serial
  AMD route); `inertia()`, `estimate_condition_1norm`, `solve_many*`.
- **Obligations discharged:** a symbolic-phase prediction derived from the library's own
  allocation policy (including its 1.2 slack), and a whole-owner conservative bound.
- **Residual caller obligations:** add held matrix storage and the caller's own buffers; delayed
  pivots can exceed the symbolic estimate unless the bounded-storage geometry is selected.
- **Semantic conditions:** `factor_bytes` is "what FERAL will try to allocate", not information-theoretic
  fill. `conservative_storage_bytes` is cubic in the bound dimension by construction.
- **Evidence:** Interface-checked: `symbolic/mod.rs` `WorkEstimate` docs; `numeric/solver.rs`
  (`work_estimate`, `with_bounded_storage_max_dimension`, `conservative_storage_bytes`).

## Current applicability

### [CBO-026](../../../library_leverage/codebase/native-solver-integration.md#cbo-026) — Sparse factorization wrappers split across faer, FERAL and KLU
- **Types:** feral/hager-higham-estimator-over-any-factor, feral/unsymmetric-sparse-lu
- **Realization:** installed pin.
- **Conclusion:** fits with conditions — the "one estimator per question" goal is reachable without
  changing factor libraries: FERAL's public estimator driver can run over the faer LU (and raw KLU)
  factors through `HagerHighamOperator`. Replacing the faer LU uses with FERAL `SparseLu` is
  unsettled.
- **What it would discharge / what remains local:** one shared estimator implementation and
  convention for every Jacobian factor. The local code would keep one small adapter per factor
  type (faer `solve_in_place_with_conj` / `solve_transpose_in_place_with_conj`; KLU
  `klu_l_solve` / `klu_l_tsolve`). The square-response `matrix_norms` helper answers a different
  question — normwise backward error, which needs ‖A‖∞ and ‖A‖₁ — so it is not a competing
  condition estimator. `feral::matrix_norm_1` could supply only its 1-norm half.
  If FERAL `SparseLu` replaced faer LU for the tangent and square-response uses, it would gain
  typed factor-time singularity, a built-in condition estimate and refined solves. It would lose
  faer's `MemStack` scratch with an a-priori `StackReq` (allowance before allocation), symbolic
  reuse with numeric-storage reuse, multi-RHS solves and faer's parallelism control. Its
  Markowitz fill cannot be predicted (see CBO-001).
- **Prerequisites, configuration or extension:** for a FERAL LU swap, choose `LuScaling` so that
  it does not double-scale the normalized matrix. Use `factor` + `SparseLuSymbolic` (Gilbert–Peierls)
  if symbolic reuse matters; `factor_markowitz` ignores the symbolic.
- **Semantic gaps against the CBO's preservation constraints:** "allowance charged before
  allocation" fails for FERAL LU numeric storage (no prediction, no caller scratch). FERAL keeps
  thread-local solve buffers outside any admission. Determinism: both are serial and FMA-free on
  these paths. "Singular vs fatal typed" is stronger in FERAL (`SingularBasis`) than in faer, which
  can finish at an exact zero pivot.
- **Unsettled premises and what would settle them:** whether FERAL-over-faer and FERAL-LU estimates
  agree closely enough on the actual Jacobians to merge the conditioning question — probe request P3.
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** The "one estimator per question" premise does not hold:
each question already has one estimator. The KLU `rcond`/`condest` in Diffsol is a
refactor-acceptance policy (the bundled SUNDIALS 7.1.1 KLU setup policy), not a diagnostic
estimator, and `matrix_norms` answers backward error. The comparative review rejects both FERAL
Hager–Higham over faer/KLU and a faer-to-FERAL LU swap, and recommends a crate-local faer-LU
helper for the three faer owners (square response, arclength and Diffsol `FaerLu`), with one
scratch-admission rule, one error classification, one finite-solve check and one shared
`n²·64 + 512n + 4096` allowance, which is duplicated verbatim today. See the
[comparative review](../../reviews/native-solver-integration.md#cbo-026).

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) — Prepaid accounting (FERAL part)
- **Types:** feral/symmetric-ldlt-symbolic-estimates, feral/unsymmetric-sparse-lu
- **Realization:** installed pin.
- **Conclusion:** fits for LDLᵀ (already used); ruled out for LU — no prediction exists.
- **What it would discharge / what remains local:** the KKT/conditioning LDLᵀ paths already charge
  `symbolic_factorize(...).work_estimate().peak_bytes` and `factor_bytes` before numeric work, so
  this is not a mirror. The KINSOL block's cubic Markowitz bound has no library replacement: FERAL
  exposes only post-hoc `factor_nnz()`, and the code comment "FERAL does not expose allocation
  capacities" is accurate at this revision. A library route would be an upstream addition, for
  example a conservative `SparseLu` owner bound analogous to `Solver::conservative_storage_bytes`.
- **Semantic gaps against the CBO's preservation constraints:** none for LDLᵀ. For LU, any bound
  must cover Markowitz's historical fill and capacity growth, which only the library author can
  derive authoritatively.
- **Unsettled premises and what would settle them:** whether the fork owner would add an LU owner
  bound (a supplier change, not a capability).
- **Evidence:** Interface-checked: local `conditioning.rs` and `kinsol/block.rs` read against the
  FERAL source.

**Reviewer correction (2026-10-27).** "A library route would be an upstream addition" should read
"a fork change, following the existing practice": the LDLᵀ owner bound is itself a fork addition
(`a695be61`), so a conservative `SparseLu` owner bound belongs in the same fork. Its single owner
bound must not refuse ordinary supported blocks, and a cubic LU bound must not be applied to the
whole-Jacobian diagnostic, where it would refuse work that runs today. The review also notes that
`conditioning::jacobian_condition` runs FERAL LU with no pre-admission. See the
[comparative review](../../reviews/native-solver-integration.md#cbo-001-native).
