> Supporting evidence for [the solver methods, selection and diagnostics review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, B5a (library-research): free candidate libraries not yet bound, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Licence and price facts were taken from vendor pages on 2026-10-12. Its judgments are the worker's; the principal review owns the verdicts.

# B5a: Free solver libraries pse-arrow does not yet bind

## Summary

1. **Six of the eleven gaps can be closed without a new library.** The pinned PETSc 3.24 prefix already contains the missing methods; the bridge just doesn't expose them (*Interface-checked*: pinned headers, and a search of the bridge and its Rust callers).
   - `SNESVINEWTONRSLS`/`SSLS` gives a two-sided-box square solver.
   - `SNESQN` gives Broyden.
   - `SNESLINESEARCHNLEQERR` gives the affine-covariant (NLEQ-ERR) damping.
   - `SNESNEWTONAL` gives arclength continuation.
   - `TAOBRGN` with `TAOBRGN_REGULARIZATION_LM` gives a bounded Levenberg–Marquardt least-squares solver.
   - argmin 0.11.0, already pinned, has `BrentRoot`, a bracketed scalar root finder.
   - The bridge today selects only `newtontr` and `nasm` (plus pseudo-transient) and exposes no TAO at all.
2. **SuiteSparse SPQR is the most material new library.** It is the only candidate that adds new mathematics: sparse rank-revealing QR. faer 0.24.4's sparse QR does no pivoting. The cheapest route is to extend the existing SuiteSparse build in `scripts/native_cache.py` (`KLU_OPTIONS`) and write a minimal owned FFI to `SuiteSparseQR_C.h`, instead of using the broken `suitesparse_sys` feature.
3. **For a non-academic personal user, only MA27, MA28 and MC19 are free.** They come in the Coin-HSL Archive (2024.05.15, personal use, no redistribution). MA57, HSL_MA77/86/97, MC64 and MC68 are free only to academic staff and students; otherwise they cost about £2,098 per named user.
   - The value of MA27 is IDAES parity (its default linear solver) and the small-KKT performance regime. It is not new mathematics; SPRAL SSIDS, MUMPS and Pardiso are the same class.
   - This bears on R-34's revisit trigger ("a licence route for HSL becomes available").
4. **Further distinct capability, in order of materiality:**
   - convex-MINLP outer approximation and ESH (SHOT, or Bonmin), which is algorithmically distinct from SCIP's spatial branch and bound;
   - parametric active-set warm-started QP (qpOASES or DAQP), partly duplicated by the POUNCE QP homotopy;
   - NLEQ2's rank-reduction Newton;
   - probability-one homotopy (HOMPACK90 as a reference; it can be formulated on the existing arclength machinery);
   - Ceres' robust-loss LM and covariance.
5. **Duplicates with an exclusion reason:** MINPACK/cminpack, the `levenberg-marquardt` crate, Couenne, UMFPACK, SuperLU, STRUMPACK, OSQP, ProxQP, PIQP, russell_sparse/nonlin, Bertini and PHCpack. Each has a revisit trigger, given below.

No files were written; no scratch was used.

---

## Baseline facts from the repository

All of these are *Interface-checked*.

- **The PETSc build is minimal.** The cached prefix (`~/.cache/pse-arrow/native/petsc/<hash>/`, `PETSC_VERSION_MINOR 24`, `libpetsc.so.3.24.0`) was configured with only `--with-mpi=0 --with-fc=0 --with-openmp=0 --with-shared-libraries=1 ...` (from `.complete.json`). `petscconf.h` has no `PETSC_HAVE_SUPERLU`, `SUITESPARSE`, `MUMPS` or `STRUMPACK`. So PETSc supplies no UMFPACK, SPQR, SuperLU or MUMPS as `MatSolverType` in this build.
- **The pinned PETSc headers contain the missing methods:**
  - `petscsnes.h`: `SNESNEWTONAL "newtonal"`, `SNESLINESEARCHNLEQERR "nleqerr"`, `SNESVINEWTONRSLS`, `SNESQN`, `SNESNEWTONTRDC`;
  - `petsctao.h`: `TAOBRGN`, `TAOPOUNDERS`, `TAOBNTR`, `TAOBQNLS`, and `TAOBRGN_REGULARIZATION_LM` ("Levenberg-Marquardt, ½ xᵀ diag(JᵀJ) x"), with `TaoBRGNGetDampingVector()`.
- **The PETSc wiring is narrow:**
  - `pse_petsc_snes_type` takes a string (`crates/pse-petsc-sys/bridge/pse_petsc.c:258`);
  - the Rust side passes only `c"newtontr"` (`BN/petsc.rs:742`) and `c"nasm"` (`BN/petsc/blocks.rs:818`);
  - the method enum is `NewtonTrustRegion | NonlinearAdditiveSchwarz | PseudoTransient` (`crates/pse-model/src/generated/enums.rs:14408–14419`);
  - `pse_petsc.c` contains zero `tao` occurrences.
- **KINSOL** owns globalization and Anderson acceleration, with Picard and fixed-point representations, and has sign constraints only (`BN/kinsol.rs:7–10, 42–52, 93–103`). `BN/recycle.rs:3` says "no recycle iteration algorithm lives here".
- **argmin 0.11.0** (`Cargo.toml:267`, default features off; argmin-math 0.5.1 with its default `primitives,vec,rand` in the lock) is used only in `RT/math/surrogate/library.rs` (the `CostFunction` loop, about lines 4 and 371–428).
  - The registry source at `argmin-0.11.0/src/solver/` has `BrentRoot` and `BrentOpt`, `TrustRegion` with `CauchyPoint`/`Dogleg`/`Steihaug`, `GaussNewton`/`GaussNewtonLS`, `Newton`, quasi-Newton, Nelder–Mead and others.
  - It has **no Levenberg–Marquardt**.
  - argmin-math's faer backend stops at `faer_v0_23`; the workspace is on faer 0.24.4.
- **No scalar bracketing solver in pse-math.** I grepped `PM/` for `brent|bisect|regula|illinois|toms748|ridders|bracket` and found no solver. Scalar work goes through:
  - Symbolica certified cubic roots (`PM/implicit_cubic.rs:3, 87`, `"symbolica.cubic-roots.v2"`);
  - IBEX validated interval covering (`BN/root_isolation.rs:3`), acquired as the `isolation` capability.
  This search covered `PM`, `BN` and `RT` source, excluding generated files.
- **Continuation exists.** `PM/continuation.rs:1` ("Oriented arclength equation families. Native libraries own correction"), plus the POUNCE natural-parameter walk and QP homotopy (`BN/kkt/path.rs`, `BN/kkt/path/qp.rs`).
- **Native acquisition.** `scripts/native_operation.py:39` lists the capabilities `compiler, solver, klu, isolation, uno, petsc`. `scripts/native_cache.py:111` (`KLU_OPTIONS`) already builds SuiteSparse from source with `SUITESPARSE_ENABLE_PROJECTS=suitesparse_config;amd;btf;colamd;klu` and `KLU_USE_CHOLMOD=OFF`. The `docker/solvers/` image has MKL as its single BLAS, plus MUMPS 5.9.1 with METIS, SPRAL 2025.09.18, Ipopt 3.14.20 (`--without-hsl --disable-linear-solver-loader`) and SCIP 10.0.2 (`docker/solvers/README.md`, `build.sh:8–49`).
- **HSL exclusion record:** ADR-0108 and register R-34 (`docs/adr/register.md:45`; trigger "a licence route for HSL becomes available").

---

## Candidates

### 1. HSL: MA27, MA57, MA77, MA86, MA97, MC64, MC68

**Cost and licence** (*Interface-checked* on the STFC store pages, 2026-10-12):
- **Coin-HSL 2024.05.15** contains MA27, MA57, HSL_MA77, HSL_MA86 and HSL_MA97. Only MA27 is marked "Free to all". The rest are "Free to academics" under the HSL Academic Licence, which is limited to staff and students of a recognised educational institution for personal teaching and research. Commercial alternatives:
  - Named User: £2,098 per user;
  - Small Site: from £10,259.
- **Coin-HSL Archive 2024.05.15** is free and perpetual, "for personal use only", and "may not be redistributed in whole or in part, as source or in binary form". Search results and third-party build notes give its contents as MA27, MA28 and MC19. The store page itself does not list the packages; one probe below would confirm it.
- **LibHSL** (latest 2025.7.21) is academic.
- **For a hobbyist without an academic affiliation, only MA27, MA28 and MC19 are obtainable at no cost.** Whether the maintainer qualifies academically is a fact only they can supply.

**Maintenance:** active. Coin-HSL 2024.05.15; ThirdParty-HSL wrapper `releases/2.2.6` (EPL-2.0), last pushed 2025-09.

**Methods:** symmetric-indefinite LDLᵀ with threshold 1×1/2×2 pivoting and inertia.
- MA27: classic multifrontal, serial.
- MA57: multifrontal with METIS, static pivoting and better scaling.
- HSL_MA86: supernodal DAG on multicore.
- HSL_MA97: bit-reproducible multicore multifrontal.
- HSL_MA77: out-of-core.
- MC64: matching and scaling.
- MC68: orderings.
- MC19: equilibration scaling (Ipopt `linear_system_scaling=mc19`).
- MA28: unsymmetric (Ipopt `dependency_detector=ma28`).

**Gap filled:** "no HSL linear solvers for Ipopt". This is a regime and parity gap, not a mathematical one. IDAES' default is MA27 (ADR-0108 Context, from idaes-ext 3.4.2), so MA27 matters most for iteration-path parity.

**Reported performance** (literature practice, not measured here): MA27 and MA57 are usually fastest on small-to-medium KKT systems, because their per-factorization overhead is lower than SSIDS, Pardiso or MUMPS. MA86/MA97 and SSIDS win on large systems with many threads. SSIDS is RAL's open successor to the MA97 design.

**Binding:**
- **Route A:** rebuild Ipopt `--with-hsl` against ThirdParty-HSL with the Coin-HSL sources unpacked into `coinhsl/`.
- **Route B:** `--enable-linear-solver-loader` with `hsllib=libhsl.so`. The archive's meson build produces `libcoinhsl`.

ADR-0108 item 9 deliberately removes the loader, so Route A fits the "no runtime fallback" stance. Build cost is low: Fortran, and the gfortran already used for MUMPS. **Acquisition constraint:** sources are behind an STFC account and cannot be redistributed. They cannot be fetched by `docker/solvers/build.sh` from a public URL. The fit is a host-local, checksummed input to the native cache, consistent with the "local environment over portability" direction. *Proposed.*

**Diagnostics:** Ipopt exposes the `ma27_*` and `ma57_*` options (pivot tolerances, memory growth, `ma27_skip_inertia_check`, `ma27_ignore_singularity`). The HSL info arrays report inertia, 2×2 pivot counts, delayed pivots, factor nnz and flop counts. Ipopt's journal surfaces only part of this.

**Duplication:** same class as MUMPS, SSIDS and Pardiso (bound), and FERAL in POUNCE. MA28's dependency detection is duplicated by Ipopt `dependency_detector=mumps` with the linked MUMPS (Ipopt option set, recalled, not re-verified). Of the archive set, MA27 for parity and the regime is the non-duplicative reason.

**Revisit / trigger:** R-34 should be re-evaluated against the archive's personal licence for MA27 and MC19. MA57, MA97 and MC64 depend on academic eligibility.

### 2. MINPACK, cminpack, `levenberg-marquardt`, argmin

- **MINPACK and cminpack:**
  - Licence: BSD-like Minpack notice, free.
  - Version: cminpack v1.3.14, released 2026-07-25 and actively maintained. fortran-lang/minpack (modernised, with a C API) was last pushed 2024-07.
  - Methods: `lmder`/`lmdif` (Moré's LM trust region with diagonal scaling); `hybrj`/`hybrd` (Powell hybrid dogleg for square systems). Dense only, no bounds.
  - Diagnostics: an `info` code (1–8), `nfev`/`njev`, and `fjac`/`ipvt` for `covar`.
  - Binding: no maintained Rust sys crate; a hand FFI of about 10 functions; CMake (or a single Fortran file); trivial build.
- **`levenberg-marquardt` 0.15.0** (MIT, 2025-08): a pure-Rust LM using nalgebra. Upstream README describes it as a MINPACK lmder port; this is recalled, and docs.rs does not state it. It has `TerminationReason` and `MinimizationReport` (objective, termination); dense only, no bounds. **It depends on `nalgebra ^0.34`; the workspace pins 0.35.0, so adding it would create a second nalgebra.**
- **argmin 0.11.0 (pinned):** has no LM. `GaussNewton` uses normal equations (dense, `ArgminInv`). `TrustRegion` minimises a scalar objective with a Hessian and is not least-squares-specific.

**Duplication:** `TAOBRGN` with `-tao_brgn_regularization_type lm` (bounded LM, sparse, PETSc KSP or LU) is in the pinned PETSc. `SNESNEWTONTR` and `SNESNEWTONTRDC` cover the dogleg trust region for square systems. **Exclude MINPACK and the crate as duplicates.** Revisit if measured per-call PETSc setup overhead dominates for many tiny dense blocks (n ≤ about 10), or if the crate moves to nalgebra 0.35. The gap "no LM/TR least-squares" becomes "wire TAOBRGN-lm". *Proposed.*

### 3. Ceres Solver

- **Licence and version:** BSD-3; free. Latest release 2.2.0 (2023-10-13); master is active (pushed 2026-10-11).
- **Methods:** LM, and traditional and subspace dogleg; robust loss functions (Huber, Cauchy, Tukey and others); box bounds; inner iterations; a `Covariance` class; sparse normal Cholesky; Schur/iterative solvers; line-search methods.
- **Diagnostics:** rich `Solver::Summary` with per-iteration cost, cost change, gradient max-norm, step norm, relative decrease, trust-region radius, linear-solver iterations, timings and termination type.
- **Binding:** Rust crates `ceres-solver` 0.5.1 and `ceres-solver-sys` 0.5.3 (2026-04; a `cxx` bridge plus `ceres-solver-src`, which builds vendored with Eigen). This needs C++17 and Eigen, plus glog/miniglog in 2.2; optionally SuiteSparse and METIS. It doesn't match the C-ABI `*-sys` pattern but is feasible.
- **Distinctness:** robust losses and covariance are not offered by TAOBRGN; LM and dogleg duplicate it.
- **Materiality:** medium to low. It is relevant to parameter estimation and data reconciliation with outliers. *Proposed:* keep as a reference unless robust-loss estimation becomes a requirement.

### 4. NLEQ1, NLEQ2, NLEQ-ERR (ZIB, Deuflhard)

- **Licence:** the source headers allow use or modification "for your own non commercial purposes for an unlimited time", forbid redistribution without ZIB's permission, and require a separate agreement for commercial use. Free to obtain.
- **Versions and maintenance:** NLEQ1 2.3 (1992); NLEQ-ERR 1.1.1 (2006). The CodeLib test set was last updated in 2017. Effectively unmaintained.
- **Methods:** affine-covariant global Newton with a natural monotonicity test.
  - NLEQ2 adds rank reduction based on QR subcondition, which handles near-singular Jacobians.
  - NLEQ1 has band and Broyden options; NLEQ1S is sparse (it uses MA28).
- **Diagnostics:** contraction estimates Θ, the damping history λ and subcondition estimates.
- **Binding:** F77 and C; hand FFI; gfortran is present. Distribution is non-redistributable, so acquisition is host-local.

**Duplication:** `SNESLINESEARCHNLEQERR` in pinned PETSc 3.24 (contributed by Patrick Farrell; it cannot be combined with VINEWTONRSLS but can with VINEWTONSSLS) duplicates the error-oriented damping. NLEQ2's rank reduction is not duplicated.

**Verdict:** medium-low. Wire PETSc NLEQERR first. Revisit NLEQ2 if rank-deficient square blocks prove common after SPQR rank diagnostics exist.

### 5. Scalar root finders (1×1 blocks)

- **argmin `BrentRoot` (pinned):**
  - `new(min, max, tol)` requires a bracket and sign change, with relative tolerance only (`brentroot.rs:66`).
  - Termination is reported through `TerminationReason`; `IterState` carries iteration and cost counts.
  - It uses the anyhow-based `argmin::core::Error`, already accepted with a reasoned allow at `RT/math/surrogate/library.rs:371`.
- **`roots` 0.0.8** (BSD-2-Clause): Brent, Illinois regula falsi, secant, Newton and closed-form polynomial roots, all in pure Rust. Unmaintained since 2022-12 and not in the lock.
- **TOMS748:** no Rust crate (`toms748` is absent on crates.io); Boost.Math `toms748_solve` (C++) and SciPy are references.
- **faer:** no scalar root finder.
- **pse-math:** none (search coverage above). IBEX supplies certified enclosures, not a cheap safeguarded iteration.

**Verdict:** a duplicate exists in a pinned library (argmin `BrentRoot`). No new library is needed. *Proposed:* bracket from guard and bound intervals, then let IBEX or the original-residual check certify. A safeguarded Newton–bisection would be bespoke; argmin `Newton` plus a bracket check is the library-first alternative.

### 6. Homotopy and continuation

- **HOMPACK90** (ACM TOMS 777, Fortran 90, 1997; ACM software licence, free for non-commercial use; netlib; unmaintained): probability-one (Chow–Yorke) globally convergent homotopy, with fixed-point, zero-finding and curve-tracking variants (normal flow, augmented Jacobian, ODE-based), dense and sparse.
  - This is distinct from parameter continuation: it uses a random start *a* and a homotopy map ρ_a(λ,x) = λF(x) + (1−λ)(x−a).
- **PETSc `SNESNEWTONAL`** (pinned 3.24; it has `SNESNewtonALCorrectionType`) tracks arclength along a user parameter. With the existing `PM/continuation.rs` arclength families, the Chow–Yorke map can be formulated as a homotopy relation and corrected natively. *Proposed.*
- **Bertini 2** (v4.0.0, 2026-10-06), Bertini 1.x and **PHCpack** (v2.4.94, GPL-3): polynomial systems only. Symbolica certified roots and IBEX already cover the low-degree polynomial cases (cubic EOS). Excluded for process models with exp/log terms.
- **AUTO-07p** (v0.9.3, 2021): bifurcation detection (folds, Hopf, branch points) and periodic orbits. A driver program, not an embeddable library; use as reference.
- **russell_nonlin** 3.3.1 (MIT, 2026-09): natural and pseudo-arclength Euler–Newton, with good statistics (function and Jacobian evaluations, factorizations, accepted and rejected steps). It pulls in russell_sparse, which brings its own OpenBLAS or MKL and a locally compiled MUMPS/SuiteSparse. That conflicts with the single-BLAS invariant (ADR-0108) and duplicates the MUMPS already in the image. It also duplicates NEWTONAL and the existing arclength code. Excluded.

**Verdict:** the gap is a formulation (a probability-one homotopy map) on libraries already pinned. HOMPACK90 is reference practice.

### 7. MINLP

- **Bonmin** (EPL-1.0): last release 1.8.9 (2023-01-30); master was pushed 2026-10-09, so maintenance continues at low activity.
  - Methods: B-BB (NLP-based branch and bound), B-OA, B-QG and B-Hyb for convex MINLP, using Ipopt, Cbc and Clp.
  - Binding: C++ `TMINLP` API, no C API; built with coinbrew (Cbc, Clp, Cgl, Osi); medium-high complexity.
- **Couenne** (EPL-1.0): 0.5.9 (tag date 2026-09-17). Spatial branch and bound for nonconvex MINLP, which **duplicates SCIP 10.0.2** (bound). Exclude; revisit only if a nonconvex class is shown where SCIP fails and Couenne succeeds.
- **SHOT** (EPL-2.0): latest release 1.1.0 (2021-10); master pushed 2026-10-11; 2.0 (Python interface, HiGHS as MIP subsolver) is announced but unconfirmed.
  - Methods: extended supporting hyperplane (ESH) and ECP outer approximation with primal heuristics. Strong on convex MINLP; a 1.0+ nonconvex mode.
  - Needs a MIP solver (Cbc free; HiGHS on 2.0) and Ipopt; C++; there is an OSiL/AMPL path.
- **Juniper:** Julia, not usable. **Minotaur:** repository not found at the queried path; not assessed.

**Distinctness:** NLP-based branch and bound, OA, QG and ESH differ from SCIP's LP-based spatial branch and bound with outer-approximation cuts. On convex process-synthesis superstructures (the classic DICOPT and GAMS reference practice), they often give different and sometimes much better performance.

**Verdict:** medium materiality for process synthesis. SHOT is the stronger maintained free option. Cost: a C++ bridge plus a MIP-solver choice.

### 8. QP for warm-started sequences

| Candidate | Licence / version | Method | Rust route | Diagnostics | Duplication |
|---|---|---|---|---|---|
| **qpOASES** | LGPL-2.1, 3.2.2 (2025-02-05) | Parametric "online active set" homotopy between consecutive QPs; hot start reuses the working-set factorization (MPC/SQP design) | No crate. C++; qpOASES_e is a pure-C variant (exists; not re-verified) | `nWSR` used, objective, primal/dual solution, return codes (e.g. `RET_MAX_NWSR_REACHED`) | **Partially duplicated** by the POUNCE QP homotopy (`BN/kkt/path/qp.rs`) |
| **DAQP** | MIT, v0.10.0 (2026-09-26) | Dense dual active set; warm start from the active set; branch and bound for binaries | No crate (searched `daqp`); small C, trivial FFI | Exit flag, iterations, timing | Distinct regime: many small dense QPs |
| **OSQP** | Apache-2.0, v1.0.0 (2025-03); crates `osqp` and `osqp-sys` 1.0.1 (official, bundled C) | ADMM; warm start; factor caching on value updates; infeasibility certificates; polishing | Ready crate | `OSQPInfo` (status, iterations, primal/dual residuals, gap, rho updates, phase timings; from OSQP docs, recalled) | Class duplicate (Clarabel, HiGHS QP, POUNCE QP); moderate accuracy |
| **ProxQP** (proxsuite) | BSD-2, v0.7.3 | Proximal augmented Lagrangian; strong warm start; dense and sparse | C++/Eigen only, needs a `cxx` bridge | iter, mu/rho updates, residuals, gap, timings | Regime duplicate |
| **PIQP** | BSD-2, v0.6.4 (2026-08) | Proximal interior point | C interface; no crate | iter, residuals, timings | IPM, weak warm start; duplicates Clarabel |

**Verdict:** only parametric active-set methods (qpOASES, DAQP) are distinctive for warm-started sequences. POUNCE already supplies a QP homotopy, so first establish what the POUNCE QP lacks: hot-start working-set reuse, degeneracy handling. *Proposed.*

### 9. Sparse linear algebra

- **SuiteSparse 7.14.1** (2026-09-14): SPQR and UMFPACK are GPL-2+, CHOLMOD is mixed LGPL/GPL, KLU is LGPL. All free.
  - **SPQR** gives multifrontal rank-revealing sparse QR (`SuiteSparseQR_C` returns the rank) and min-norm or basic least squares. Diagnostics include `SPQR_istat`, `SPQR_tol_used`, `SPQR_norm_E_fro` and memory counters (see `docs/plans/36-investigation-evidence/sparse-rank.md`).
  - **UMFPACK** gives unsymmetric multifrontal LU with an `Info[]` array (rcond, flops, nnz L/U, memory).
  - **Binding:** `suitesparse_sys` 0.1.4 is broken for SPQR. Its `wrapper.h` includes the non-existent `spqr.h` (sparse-rank evidence). The fitting route extends `KLU_OPTIONS` to `...;klu;cholmod;spqr[;umfpack]` with MKL as BLAS, and adds an owned bindgen or hand FFI to `SuiteSparseQR_C.h` (a new `*-sys` crate in the `pse-ipopt-sys` pattern). The C++ runtime and full link closure come from SuiteSparse's CMake/pkg-config. Medium build cost. *Proposed.*
- **faer 0.24.4:** sparse LU (supernodal and simplicial), Cholesky and LDLᵀ, AMD and COLAMD, and sparse QR. **The sparse QR is non-pivoting:** `faer-0.24.4/src/sparse/linalg/qr.rs` uses `linalg::qr::no_pivoting` throughout, so it reveals no rank. Dense column-pivoted QR and SVD exist.
- **SuperLU 7.0.1** (BSD-style; 2025-05): threshold partial-pivoting supernodal LU, ILUTP, and `dgssvx` rcond/ferr/berr. The `superlu-sys` 0.4.2 crate depends on `openblas-src`, which **violates single-BLAS**, so an owned sys crate would be needed. Duplicates faer LU and KLU; its ILU and condition estimates are a minor addition. SUNDIALS ships only a SuperLU_MT interface module, not the library (covered by B3).
- **STRUMPACK 8.1.0** (2026-10-05): rank-structured compression (HSS, BLR, HODLR). Heavy C++ build; not material at process-flowsheet scale. Exclude.
- **russell_sparse 3.3.1:** UMFPACK, KLU and MUMPS through its own native stack, which conflicts with ADR-0108's single BLAS and adds a second MUMPS. Exclude.

### 10. Optional

- **CasADi 3.8.1** (LGPL-3): reference only. Its rootfinder plugins (`kinsol`, `newton`, `nlpsol`) and Opti/SQP method (with qpOASES as the default QP for warm starts) are useful reference practice.
- **SUNDIALS ARKODE and CVODES:** covered by worker B3.

---

## Probes not run (would settle material doubts)

1. **Archive contents and personal-licence route.**
   - Question: does the Coin-HSL Archive 2024.05.15 contain MA27, MA28 and MC19, and is it obtainable by a non-academic account?
   - Action: register on licences.stfc.ac.uk, download `coinhsl-archive-2024.05.15`, then `tar tf` it.
   - Expected: `ma27`, `ma28` and `mc19` source directories and a `meson.build`.
2. **TAOBRGN-lm with bounds.**
   - Question: does it work through the pinned prefix?
   - Action: compile a tiny C program against `$PETSC_DIR` that sets `TaoSetType(tao, TAOBRGN)`, `-tao_brgn_regularization_type lm` and `TaoSetVariableBounds`, and run it via `scripts/pse-env --native -- ./a.out` on the Rosenbrock residual with an active bound.
   - Expected: `TAO_CONVERGED_*`, with `TaoGetSolutionStatus` giving iterations, f and gnorm, and the bound active at the solution.
3. **SNESVINEWTONRSLS with two-sided bounds** and `SNESLINESEARCHNLEQERR` with VINEWTONSSLS.
   - Action: the same harness with `SNESVISetVariableBounds`.
   - Expected: `SNES_CONVERGED_*`, and `SNESVIGetActiveSetIS` returns the active indices.
4. **SPQR from an extended SuiteSparse build.**
   - Action: run `native_cache` with `SUITESPARSE_ENABLE_PROJECTS=...;cholmod;spqr` and `BLA_VENDOR=Intel10_64lp`, then compile `SPQR/Demo/qrsimplec.c`.
   - Expected: the rank printed for a rank-deficient matrix.

---

## Ranked shortlist

The ranking is by how much each candidate adds that is genuinely new and material for a process simulator.

**Wire first (duplicates already pinned; not new libraries):**
- PETSc `SNESVINEWTONRSLS`/`SSLS` closes the box-square gap.
- `TAOBRGN`-lm closes the LM least-squares gap.
- `SNESQN` closes the Broyden gap.
- `SNESLINESEARCHNLEQERR` closes the affine-invariant Newton gap.
- `SNESNEWTONAL` with a Chow–Yorke formulation closes the probability-one homotopy gap.
- argmin `BrentRoot` closes the scalar 1×1 gap.

Wegstein has no library; it would be bespoke or reference practice, and KINSOL Anderson already exists.

**New libraries:**
1. **SuiteSparse SPQR (+CHOLMOD).** Distinct: sparse rank-revealing QR, which faer's non-pivoting QR cannot supply. Feeds structural and numerical singularity diagnosis, degrees-of-freedom analysis and identifiability. Low marginal acquisition cost through the existing SuiteSparse build.
2. **HSL MA27 (+MC19), Coin-HSL Archive.** Free for personal use. A regime and IDAES-parity gain, not new mathematics. MA57, HSL_MA97 and MC64 only if academic eligibility holds.
3. **SHOT** (or Bonmin). Distinct convex-MINLP algorithms (ESH/OA, NLP-BB) beside SCIP's spatial branch and bound.
4. **qpOASES or DAQP.** Parametric active-set warm starts. Partly duplicated by the POUNCE QP homotopy; justify only after a gap in POUNCE QP is established.
5. **NLEQ2.** Rank-reduction Newton. Non-redistributable licence; consider after SPQR-based rank diagnostics.
6. **Ceres.** Robust losses and covariance, for outlier-robust estimation only.

**Duplicates, excluded, with their revisit triggers:**

| Candidate | Duplicated by | Revisit when |
|---|---|---|
| MINPACK, cminpack, `levenberg-marquardt` | TAOBRGN-lm, SNESNEWTONTR/TRDC | PETSc overhead on tiny blocks is measured to dominate, or the crate moves to nalgebra 0.35 |
| Couenne | SCIP | A nonconvex class fails in SCIP |
| UMFPACK | faer LU, KLU | An rcond requirement appears |
| SuperLU | faer LU, KLU; its crate breaks single-BLAS | An ILU need appears |
| STRUMPACK | — (scale) | n ≫ 10⁶ |
| OSQP, ProxQP, PIQP | Clarabel, HiGHS QP, POUNCE QP | A low-accuracy, warm-start QP regime is required |
| russell_sparse, russell_nonlin | Single-BLAS conflict; NEWTONAL and existing arclength | — |
| Bertini, PHCpack | Polynomial only; Symbolica and IBEX cover the cases | — |
| AUTO-07p, HOMPACK90, CasADi | Reference only | — |

## Sources

- STFC Coin-HSL: https://licences.stfc.ac.uk/product/coin-hsl
- Coin-HSL Archive: https://licences.stfc.ac.uk/product/coin-hsl-archive
- LibHSL: https://licences.stfc.ac.uk/product/libhsl
- HSL licensing: https://hsl.rl.ac.uk/licensing.html
- HSL academic access: https://hsl.rl.ac.uk/academic_access.html
- ThirdParty-HSL: https://github.com/coin-or-tools/ThirdParty-HSL
- CasADi, Obtaining HSL: https://github.com/casadi/casadi/wiki/Obtaining-HSL
- PETSc SNESType (3.26 docs; pinned 3.24 headers checked locally): https://petsc.org/release/manualpages/SNES/SNESType/
- PETSc SNESLINESEARCHNLEQERR: https://petsc.org/release/manualpages/SNES/SNESLINESEARCHNLEQERR/
- ZIB NLEQ codes: https://www.zib.de/codelib/newton_testset/methods/nleq1.f and https://www.zib.de/codelib/NewtonLib/nleq_err/readme
- cminpack v1.3.14: https://newreleases.io/project/github/devernay/cminpack/release/v1.3.14
- levenberg-marquardt docs: https://docs.rs/levenberg-marquardt/0.15.0/levenberg_marquardt/
- russell_nonlin: https://github.com/cpmech/russell/tree/main/russell_nonlin
- SHOT: https://github.com/coin-or/shot
- GitHub API, release, tag and licence metadata, queried 2026-10-12, for:
  - ceres-solver, Bonmin, Couenne, qpOASES;
  - osqp, piqp, proxsuite, daqp;
  - SuiteSparse, superlu, STRUMPACK;
  - PHCpack, auto-07p, b2, casadi, spral, Ipopt, faer-rs.
- crates.io API, queried 2026-10-12.