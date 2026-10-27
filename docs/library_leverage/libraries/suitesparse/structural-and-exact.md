# SuiteSparse (BTF, CXSparse, SPEX)

Realizations considered:
- **Installed:** the vendored `suitesparse_sys` 0.1.4. Its source tree is SuiteSparse 7.7.0,
  containing BTF 2.3, CXSparse 4 and SPEX 3.1.0. The workspace enables `klu`, `amd`, `colamd`
  and `btf` as static libraries, so the BTF library is already compiled whenever a dependent
  native feature is built. `cxsparse` and `spex` are build features of the same crate and are
  not enabled.
- **Upgrade route:** SuiteSparse v7.14.1 (released 2026-09-14) carries SPEX 3.2.4 (SPEX date
  2025-07-25), which adds exact LDLᵀ.

Research date 2026-10-27.

Sources:
- Vendored headers `vendor/{BTF/Include/btf.h, CXSparse/Include/cs.h, SPEX/Include/SPEX.h}`.
- Sources `BTF/Source/btf_maxtrans.c` and `CXSparse/Source/{cs_maxtrans.c, cs_dfs.c}`.
- `SPEX/Include/SPEX.h` at the `v7.14.1` and `dev` refs via
  `gh api repos/DrTimothyAldenDavis/SuiteSparse/contents/...`. The `dev` copy is saved as
  `<cluster>/SPEX_dev.h`.

## Types

### suitesparse/structural-permutations
- **Need it serves:** maximum transversal, Dulmage–Mendelsohn (DM) decomposition and
  block-triangular form (BTF) of sparse patterns, without recursion and with a work limit.
- **Recognizable code shapes:**
  - Hopcroft–Karp/DFS matchers that run on dedicated large-stack threads.
  - Strongly connected component (SCC) passes over matched dependency graphs.
  - Hand-built CSC patterns used only for structure.
- **Mechanisms and APIs:**
  - BTF:
    - `btf_maxtrans(nrow, ncol, Ap, Ai, maxwork, &work, Match, Work)` is MC21-style with an
      explicit stack. `maxwork` caps work at `maxwork·nnz(A)`, and `work == -1` reports that the
      cap was hit.
    - `btf_strongcomp(n, Ap, Ai, Q, P, R, Work)` is a non-recursive Tarjan.
    - `btf_order` runs both.
  - CXSparse:
    - `cs_maxtrans` uses `cs_augment` with explicit `js/is/ps` stacks.
    - `cs_dmperm(A, seed) -> csd {p, q, r, s, cc[5], rr[5], nb}` gives the coarse and fine DM
      decomposition of rectangular matrices, plus the BTF blocks.
    - `cs_scc` covers SCCs.
  - Rust FFI is the bindgen output of `suitesparse_sys` with the `btf` or `cxsparse` feature.
- **Obligations discharged:**
  - stack-safe matching;
  - a deterministic work bound for the matching (BTF);
  - the full fine DM, including rectangular parts (CXSparse);
  - SCC ordering.
- **Residual caller obligations:**
  - CSC construction and index mapping.
  - Domain tie-breaking. The output order follows column/row index order and, for
    `cs_dmperm`, a `seed` (0 means natural order).
  - Typed refusal.
  - Inter-block closure. Neither library provides a block DAG or a transitive closure.
  - Cancellation. There is no callback, so the work cap is the only bound.
- **Semantic conditions:**
  - The output is deterministic for a fixed input order and `seed = 0`.
  - The work limit makes truncation explicit: an incomplete matching is reported.
  - The libraries use C `malloc` (CXSparse) or caller-supplied workspace (BTF: `Work` of 5·ncol
    and 4n). Allocation is predictable from n and nnz, so it can be charged before calling.
- **Version / build conditions:**
  - BTF is present and compiled in the installed vendored crate, but only behind native solver
    features.
  - CXSparse needs the `cxsparse` feature.
  - Either way, a pure-Rust crate would take a native dependency.
- **Limits and negative results:** BTF's `maxtrans` handles square or rectangular input but gives
  no DM coarse classification. Use `cs_dmperm` for that.
- **Evidence:** Interface-checked (headers and source comments).

### suitesparse/spex-exact-ldl
- **Need it serves:** exact (rational or integer-preserving) symmetric factorisation, used to
  decide definiteness without rounding.
- **Recognizable code shapes:**
  - Hand-written LDLᵀ over arbitrary-precision rationals.
  - Sylvester-criterion determinant loops.
- **Mechanisms and APIs:**
  - SPEX 3.2: `SPEX_ldl_analyze`, `SPEX_ldl_factorize`, `SPEX_ldl_solve`, `SPEX_ldl_backslash`.
  - Ordering through AMD/COLAMD, with the option `algo = SPEX_LDL_LEFT | SPEX_LDL_UP`.
  - `SPEX_cholesky_*`, present at 3.1.0.
  - Input as `SPEX_matrix` in CSC with an MPZ, MPQ or FP64 type.
  - Errors `SPEX_ZERODIAG` ("an LDL factorization is not possible"), `SPEX_NOTSPD` (Cholesky)
    and `SPEX_UNSYMMETRIC`.
- **Obligations discharged:**
  - an exact, roundoff-free factorisation with a fill-reducing order;
  - the inertia, read from the signs of D when the factorisation succeeds.
- **Residual caller obligations:**
  - Zero-pivot cases: semidefinite and some indefinite matrices.
  - Witnesses.
  - Budgets and cancellation (no hooks).
  - Initialising GMP and MPFR (`SPEX_initialize`).
- **Semantic conditions:**
  - LDL "requires nonzero leading principal minors" in the chosen order; it does not do
    symmetric pivoting.
  - A matrix that is positive semidefinite but singular yields `SPEX_ZERODIAG`, and so does an
    indefinite matrix with a zero minor. That outcome is undecided, not a verdict.
  - Positive definite matrices succeed with D > 0. Indefinite matrices with nonzero minors
    succeed with a negative D entry.
- **Version / build conditions:**
  - LDLᵀ is absent at SPEX 3.1.0 (vendored SuiteSparse 7.7.0). Only Cholesky (SPD-only) is
    there.
  - It is present at SPEX 3.2.x (SuiteSparse ≥ the release carrying SPEX 3.2; confirmed at
    v7.14.1).
  - It needs GMP and MPFR at link time. A process that already links GMP through another crate
    must avoid duplicate or incompatible GMP symbols.
- **Limits and negative results:**
  - no semidefinite decision;
  - no negative-direction witness;
  - no work budget.
- **Evidence:** Interface-checked (headers).

## Current applicability

### [CBO-023](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-023) — Matching, DM, BTF and recursion guard
- **Types:** suitesparse/structural-permutations.
- **Realization:**
  - BTF: installed (vendored 7.7.0, `btf` already enabled, but compiled only for native solver
    features).
  - CXSparse: same vendored crate with `cxsparse` added.
- **Conclusion:** fits with conditions. This is the only inspected library route that removes the
  recursion guard and adds a deterministic work cap. It costs a native dependency in the
  structural layer, and it re-does DM/BTF that POUNCE already supplies.
- **What it would discharge / what remains local:**
  - *Discharged:*
    - the large-stack thread and its stack constant;
    - the unbounded matching work;
    - the second hand-wired decomposition. One CSC route could serve both sites.
  - *Remains local:*
    - mapping to and from SemanticId and the tie-break order;
    - limits and refusal types;
    - inter-block parameter propagation;
    - cancellation between stages.
- **Prerequisites, configuration or extension:**
  - make `suitesparse_sys` (with `btf`, plus `cxsparse` if DM is needed) a dependency of the
    structural layer;
  - a safe wrapper;
  - FFI-boundary unwinding discipline.
- **Semantic gaps against the CBO's preservation constraints:**
  - Matching cardinality and the DM parts are invariant across algorithms. Block order and
    tie-breaking follow index order, so SemanticId ordering must be imposed through the input
    permutation.
  - "Conservative support across all branches" is unaffected, because it concerns pattern
    construction.
- **Unsettled premises and what would settle them:**
  - Whether a native dependency is acceptable in a crate that is currently pure Rust. This is a
    comparative-review question.
  - Whether BTF's block order can be made to match the current lexicographic topological order.
    A probe comparing both on representative incidences would settle it. Not authorized.
- **Evidence:** Interface-checked; Proposed.

### [CBO-025](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-025) — Exact rational LDLᵀ for convexity and Gram checks
- **Types:** suitesparse/spex-exact-ldl.
- **Realization:** upgrade route only. This needs a vendored SuiteSparse with SPEX ≥ 3.2 and the
  `spex` feature, plus GMP/MPFR linkage.
- **Conclusion:** ruled out here, for two reasons:
  - SPEX LDL cannot decide the semidefinite case: a zero pivot gives `SPEX_ZERODIAG`.
  - It gives no witness, and the CBO's preserved diagnostics (`Indefinite` and `GramFactors`
    with zero-pivot rows) would need the local kernel as a fallback anyway.
- **What it would discharge / what remains local:**
  - *Discharged:* exact positive-definite and nonzero-minor indefinite verdicts.
  - *Remains local:* everything else.
- **Prerequisites, configuration or extension:**
  - SuiteSparse upgrade;
  - `spex` feature;
  - GMP coexistence with the existing arbitrary-precision stack.
- **Semantic gaps against the CBO's preservation constraints:**
  - no budget or cancellation;
  - the factors are integer-preserving (REF) rather than the local Gram-row form.
- **Unsettled premises and what would settle them:** none decisive.
- **Evidence:** Interface-checked.
