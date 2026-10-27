# SuiteSparse KLU

Realizations considered: installed `suitesparse_sys` 0.1.4 (vendored; features `static_libraries`,
`klu`, `amd`, `colamd`, `btf`) with its packaged KLU tree; KLU is also reached through SUNDIALS
`SUNLinSol_KLU` in the vendored `sundials-sys` 0.6.2. Newer SuiteSparse releases were not
assessed.
Research date 2026-10-27. Sources: installed `vendor/suitesparse-sys/vendor/KLU/Include/klu.h`,
`KLU/Source/{klu_factor.c,klu_kernel.c}`, `SuiteSparse_config/SuiteSparse_config.h`.

## Types

### klu/symbolic-estimates-and-measured-usage
- **Need it serves:** predicting, bounding or measuring KLU factor memory for admission, and
  estimating conditioning.
- **Recognizable code shapes:** `klu_l_analyze` followed by charges derived from `n`/`nnz`; local
  byte formulas beside `klu_l_factor`; use of `klu_l_rcond`/`klu_l_condest`.
- **Mechanisms and APIs:** `klu_l_symbolic { lnz, unz (double estimates from AMD/COLAMD), nzoff,
  nblocks, maxblock, est_flops, symmetry }`; `klu_l_numeric { lnz, unz, max_lnz_block,
  max_unz_block, nzoff }` (actual values after factor); `klu_l_common { initmem_amd, initmem, memgrow,
  memusage, mempeak, maxwork }`; `klu_l_refactor` (same pivots, reuses Numeric storage, allocates no
  new LU storage); `klu_l_rcond`, `klu_l_condest`, `klu_l_rgrowth`, `klu_l_flops`. Initial per-block
  LU allocation is `initmem_amd · Lnz_est + nk` units (or `initmem` times the block nnz when no
  estimate exists), growing by `memgrow · lusize + 2n + 1` when exhausted. Custom allocators
  can be installed through `SuiteSparse_config_malloc_func_set` (and calloc/realloc/free equivalents).
- **Obligations discharged:** a symbolic-phase fill estimate; exact post-factor usage and peak;
  allocation-free refactorization; two conditioning measures.
- **Residual caller obligations:** turning the estimate into a conservative charge. The estimate is
  not an upper bound, because pivoting can exceed it and growth then reallocates. Worst-case
  dense-block bounds remain the caller's.
- **Semantic conditions:** `memusage`/`mempeak` are post-hoc. After the first `klu_factor` of a
  fixed pattern, `klu_refactor` storage is exactly known, so later refreshes can be charged
  exactly. The SuiteSparse allocator hooks are **process-global function pointers**: they can
  refuse an allocation (KLU reports out-of-memory), but cannot attribute it to a particular
  concurrent owner without thread-local context, and they fire during, not before, factorization.
- **Limits and negative results:** no before-allocation upper bound for `klu_factor`.
- **Evidence:** Interface-checked: `klu.h` field comments; `klu_factor.c` (lsize formula),
  `klu_kernel.c` (growth); `SuiteSparse_config.h` (malloc setters).

## Current applicability

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) — Prepaid accounting (KLU part)
- **Types:** klu/symbolic-estimates-and-measured-usage
- **Realization:** installed pin.
- **Conclusion:** fits with conditions — a "bound first factor, then exact" scheme can use library
  numbers. Charge a conservative bound (or `initmem_amd·lnz` plus a growth margin) for the first
  `klu_factor`, then charge refreshes exactly from `Common.memusage` and the Numeric sizes,
  because `klu_refactor` allocates nothing.
- **What it would discharge / what remains local:** hand-derived bytes for retained KLU numeric
  objects after the first factor. The first-factor bound remains local.
- **Semantic gaps against the CBO's preservation constraints:** a global allocator hook would give
  allocation-time refusal, not prepay, and shares global state across concurrent owners — it does
  not meet "one deployment pool identity" without thread-local plumbing.
- **Evidence:** Interface-checked; application Proposed.

### [CBO-026](../../../library_leverage/codebase/native-solver-integration.md#cbo-026) — Sparse factorization wrappers (KLU part)
- **Types:** klu/symbolic-estimates-and-measured-usage
- **Conclusion:** fits with conditions — KLU stays required by the SUNDIALS fixed-pattern
  integration and offers its own `klu_condest`. To get one estimator per question, either keep
  `klu_condest` for KLU-owned factors or drive FERAL's `HagerHighamOperator` over
  `klu_l_solve`/`klu_l_tsolve` (see `feral.md`).
- **Evidence:** Interface-checked; application Proposed.
