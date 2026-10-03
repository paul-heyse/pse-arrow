# Candidate libraries for solver-pipeline mechanisms the pinned stack lacks

Wave 1 library research for the solver acceleration and globalization design review (2026-10-03).
Scope: capability, contract and integration fit only. No benchmarking. Licences are not a rejection
reason. Background inputs are the colleague input
(`docs/design_review/evidence/solver-acceleration-2026-10-03/colleague-input.md`), the
`native-solver-libraries` skill, register R-47 (`docs/adr/register.md`), and the pinned versions in
the workspace `Cargo.toml` (`sundials-sys =0.6.2` with bundled SUNDIALS 7.1.1 and only the `kinsol` and
`klu` features enabled; the `pounce-*` family at `=0.12.0`; `feral =0.18.0`; `diffsol =0.16.2`;
`highs =2.4.0`; `clarabel =0.11.1`; `scip-sys =0.1.28`).

## Evidence labels used here

- **[src]**: observed in upstream source I downloaded or read in the local cargo registry. The file
  and version are given.
- **[doc]**: the library's own documentation (official site, Context7 or docs.rs).
- **[mem]**: my recollection, not re-verified in this pass. Treat it as a lead.
- **[obs]**: an inference from source or docs. It is not a documented guarantee.

Scratch copies of the sources I read are under
`<scratch>/src/`: PETSc
`main` (3.26.0-dev) `tr.c`, `linesearchbt.c`, `al.c`, `posindep.c`, `snesngmres.c`, `aspin.c` and
`petscsnes.h`; `russell_nonlin-3.3.1`; `polsys-0.1.2`; `ceres-solver-0.5.1` and
`ceres-solver-sys-0.5.3`; a shallow clone of Uno at `83ec44e` (2026-10-03, C API 2.9.0); and
`pounce-sensitivity-0.12.0`. I read the other `pounce-*` crates, `sundials-sys-0.6.2` and `feral-0.18.0`
from `~/.cargo/registry/src/index.crates.io-*/`.

## Hard requirements (the R-47 set, plus the brief)

A candidate is checked against these:

- **H1.** It consumes analytic sparse Jacobian/Hessian callbacks or JVPs.
- **H2.** A recoverable evaluation failure rejects the trial. It does not abort the solve.
- **H3.** It supports bounds or sign constraints.
- **H4.** It supports cancellation and time budgets.
- **H5.** It returns typed termination reasons.
- **H6.** It is deterministic.
- **H7.** It works with single-threaded worker ownership.
- **H8.** It builds and links on Linux alongside the existing SUNDIALS, MKL and FERAL toolchain.
- **H9.** It is maintained.

---

## 1. Candidate profiles

### 1.1 PETSc (C; SNES, TS, TAO). Version: `main` reports 3.26.0 (dev). No system install on this host.

Mechanisms PETSc owns as library code:

- **SNES types [doc]:** NEWTONLS, NEWTONTR, NEWTONAL (arc-length), NRICHARDSON, NCG, NGMRES, ANDERSON,
  FAS, NASM, ASPIN, NGS, VINEWTONRSLS/VINEWTONSSLS, MS, COMPOSITE and SHELL.
- **Nonlinear preconditioning:** `SNESGetNPC` / `SNESSetNPC` with `SNESSetNPCSide` (left or right).
  This is the Brune–Knepley–Smith–Tu composition architecture.
- **TS:** `TSPSEUDO`.
- **TAO:** BNTR/BNLS/BQNLS bound-constrained Newton, BRGN (Gauss–Newton/LM with bounds),
  POUNDERS and ALMM.

Requirement fit:

- **H1, fits.** `SNESSetFunction` and `SNESSetJacobian` take an assembled `MATSEQAIJ`, whose pattern
  is preallocated once and whose values are refilled. `MATSHELL` gives JVP / matrix-free operation.
  Bounds go through `SNESVISetVariableBounds` (VI solvers).
- **H2, depends on the SNES type [src].** The user marks a failed evaluation by putting NaN/Inf in
  the residual and optionally calling `SNESSetFunctionDomainError`.
  - **Recoverable:** the backtracking line search (`linesearchbt.c` around l.140–160) halves λ while
    the trial objective is Inf/NaN. It only declares `SNES_DIVERGED_FUNCTION_DOMAIN` at `minlambda`.
    NEWTONLS(bt) and every line-search-based type therefore have recoverable trials.
  - **Aborts [src]:** NEWTONTR (`tr.c`). `SNESNewtonTRObjective` calls `SNESCheckFunctionDomainError`
    on the trial, which sets `snes->reason` to `DIVERGED_FUNCTION_DOMAIN`/`NANORINF`. The loop then
    reaches `if (snes->reason < 0) break` (around l.798) even though `rho` was set to `eta1`. A failed
    trial point does not just shrink the region; it ends the solve.
  - **Aborts [src]:** NEWTONAL (`al.c` l.382, 547) and NGMRES (`snesngmres.c` l.187–263) also use
    `SNESCheckFunctionDomainError` directly, so they abort as well.
- **H3, partly.** `SNESVISetVariableBounds` gives true box bounds for the VI Newton solvers. That is
  stronger than KINSOL's sign constraints. Other SNES types have no bounds.
- **H4, fits.** `SNESSetConvergenceTest` or `SNESSetConvergedReason` from a monitor can set
  `SNES_DIVERGED_USER` (-12) [src `petscsnes.h`]. There is no built-in wall-clock budget; the caller
  checks the clock in the monitor or convergence test.
- **H5, fits.** `SNESConvergedReason` is a closed enum covering FNORM_ABS/RELATIVE, SNORM, USER,
  FUNCTION_DOMAIN, LINEAR_SOLVE, NANORINF, MAX_IT, LINE_SEARCH, INNER, LOCAL_MIN, DTOL,
  JACOBIAN_DOMAIN, TR_DELTA and USER [src]. `TSConvergedReason` and `TaoConvergedReason` are the same.
- **H6 and H7, fit with configuration [obs].** Configure with `--with-mpi=0` (MPIUNI) and no
  OpenMP/threads, and the build is serial and deterministic. A `SNES` object is owned by one thread.
  PETSc's global state (`PetscInitialize`, the options database, the error handler) is per process.
  Use the object API (`SNESSetType`, `...Set...`) rather than the options database. Install a
  non-aborting error handler with `PetscPushErrorHandler(PetscReturnErrorHandler)` [mem]. Callbacks
  return `PetscErrorCode`, so Rust panics stay behind `catch_unwind` and become codes.
- **H8, feasible but significant.** No distro or system PETSc here; only Open MPI pkg-configs exist. A
  source build uses `./configure --with-mpi=0 --with-blaslapack-dir=$MKLROOT --with-debugging=0` plus
  optional `--download-suitesparse/--download-mumps` (serial MUMPS works under MPIUNI [mem]). It is a
  Python-driven configure and an estimated tens-of-minutes build. The SUNDIALS and MKL toolchain can
  coexist. A bundled-build `-sys` crate in the style of `sundials-sys` `build_libraries` is feasible.
- **H9, fits.** PETSc is very active: NEWTONAL is a recent addition and the 3.25 enum renames are
  visible.

Rust binding options:

- **`petsc-rs` (gitlab.com/petsc/petsc-rs).** Experimental, not on crates.io; last commit 2025-06-03.
  It depends on `mpi` 0.8 / `mpi-sys` with `build-probe-mpi`, so it requires a real MPI and does not
  fit an MPIUNI build. It would compile against Open MPI on this host but adds an MPI runtime for no
  benefit. It also exposes only part of the SNES composition API.
- **Recommended route: a repository-owned `-sys` crate.** Run bindgen over `petscsnes.h`, `petscts.h`,
  `petscmat.h` and `petscvec.h` with an allowlist, against an MPIUNI build. MPIUNI makes `MPI_Comm` a
  plain integer, so `PETSC_COMM_SELF` binds trivially [obs].

Fit by mechanism:

- **Arc-length (NEWTONAL) [src `al.c`].**
  - The formulation is a load-parameter one: residual `F(x, λ)` via `SNESSetFunction`, reading λ with
    `SNESNewtonALGetLoadParameter`, plus the "tangent load" `Q = ∂F_ext/∂λ` via
    `SNESNewtonALSetFunction`. λ is bounded in [`lambda_min`, `lambda_max`], defaulting to [0, 1].
  - Correction is EXACT (Ritto-Corrêa & Camotim 2008) or NORMAL. There is a spherical/cylindrical
    regularization ψ², a fixed `step_size` and `max_continuation_steps`.
  - The first iteration uses a predictor whose sign is chosen by `dot + ψ²·Δλ`, which handles
    direction through folds.
  - Missing: no adaptive step control, no secant predictor option, no fold or bifurcation detection
    or report, and no step retry on failure, because a NaN aborts.
  - It is generic enough for any `F(x, λ)` with a parameter derivative (H1 fits). Its ingredients are
    narrower than a continuation library's.
- **TSPSEUDO [src `posindep.c`].**
  - SER step control: `dt_new = inc·dt·‖F_prev‖/‖F‖`, or `inc·dt0·‖F0‖/‖F‖` with
    `increment_dt_from_initial_dt`, capped by `TSPseudoSetMaxTimeStep`.
  - `TSPseudoSetTimeStep` accepts a user dt rule. `TSPseudoSetVerifyTimeStep` lets a user verifier
    reject a step; the step is retried and reported as `TS_DIVERGED_STEP_REJECTED` past `max_reject`.
  - The inner solve is a full SNES, so it inherits the SNES failure semantics. With NEWTONLS(bt), a
    NaN trial backtracks; an inner SNES failure leads to a TS step rejection [mem].
  - This is the closest exact library match to Kelley–Keyes PTC.
- **NGMRES, ANDERSON, NASM, ASPIN, COMPOSITE, NPC.**
  - These are the only production library implementations of the Brune et al. composition algebra.
  - NASM/ASPIN without a DM require `SNESNASMSetSubdomains(snes, n, subsnes, iscatter, oscatter,
    gscatter)`: each subdomain gets its own SNES with block residual/Jacobian callbacks and VecScatters
    [mem].
  - Integration cost is high: a block-local residual callback per subdomain, or a callback that
    restricts the global one.
  - NGMRES aborts on NaN [src]. ANDERSON was not inspected; it is likely the same pattern [obs].
- **NEWTONTR.** Dogleg or Steihaug-CG trust region with `-snes_tr_fallback_type dogleg` [doc]. It
  fails H2 [src], as above.
- **TAO BRGN/BNTR.** Bound-constrained LM/Gauss–Newton and trust-region Newton. A NaN in the
  objective is handled by TAO line searches [mem; not verified].

### 1.2 SUNDIALS beyond KINSOL (bundled 7.1.1 in `sundials-sys 0.6.2`)

- **Present in the vendored tree but not enabled:** `arkode`, `cvode`, `cvodes`, `ida`, `idas`,
  `sunnonlinsol` and `sunadaptcontroller`. They are Cargo features of `sundials-sys` [src
  `sundials-sys-0.6.2/Cargo.toml`]. Enabling them is a feature flag, not a new dependency.
- **SUNNonlinearSolver (Newton, FixedPoint) [doc, mem].** These are integrator-internal modules. They
  can be driven standalone (`SUNNonlinSolSetSysFn`, `SUNNonlinSolSolve`). A positive return from the
  system function gives `SUN_NLS_CONV_RECVR`. They have no globalization (no line search or trust
  region), so they add nothing that KINSOL lacks for steady roots.
- **KINSOL failure semantics by strategy [src `vendor/src/kinsol/kinsol.c` 7.1.1].**
  - `KINFullNewton` (l.1760–1784) and `KINLineSearch` halve the step on a recoverable func failure
    (`retval > 0`). They give up with `KIN_REPTD_SYSFUNC_ERR`.
  - `KINFP` (l.2858–2873) only checks `retval < 0`. A positive (recoverable) return is ignored and the
    returned `fval` is used as the next G value.
  - So a fixed-point/Anderson map driven through KINSOL must never return a recoverable failure; it
    must abort or succeed. This matters for any composition that uses KINSOL FP+AA as the outer
    accelerator over block sweeps. The sweep map must absorb its own failures.
  - The existing backend already forbids constraints with KIN_FP/KIN_PICARD
    (`crates/pse-backend-native/src/kinsol.rs:227`).
- **ARKODE/CVODE/IDA as a pseudo-transient carrier [mem, doc].**
  - Implicit integrators retry with a smaller h on a recoverable RHS/residual failure. They support
    sign constraints (`*SetConstraints`) and typed flags.
  - Their step control is error-based, not SER. PTC by composition therefore means either:
    - driving the integrator one step at a time (`ARKodeSetFixedStep` or max-step changes) with an
      external SER rule, or
    - accepting error control as the time-step policy, as in Pattison–Baldea pseudo-transient models.
  - `sunadaptcontroller` offers I/PI/PID/ExpGus controllers. They are not residual-ratio controllers.

### 1.3 Trilinos NOX/LOCA (C++)

- **NOX [doc]:** line-search Newton, trust-region and inexact-trust-region (dogleg) solvers, tensor,
  Broyden, Anderson-accelerated fixed point, and status tests.
- **LOCA [doc]:**
  - natural and pseudo-arclength continuation (`LOCA::MultiContinuation::ArcLengthGroup`);
  - Constant/Secant/Tangent/Random predictors and adaptive step size;
  - turning-point, pitchfork and Hopf location and tracking;
  - an artificial-parameter homotopy group (`LOCA::Homotopy::Group`, `DeflatedGroup`);
  - bordered-system solvers.
  It is the most complete continuation library found.
- **Fit:** H1 fits through a `Group` abstraction (computeF, computeJacobian, applyJacobianInverse). The
  user `Interface::computeF` returns `bool`. How NOX line searches treat `false` was not verified
  [mem: some line searches treat it as a failed step, others abort]. H4 fits through custom
  `StatusTest`s. H5: `StatusType` plus the test tree.
- **Integration cost: highest of all candidates.**
  - C++ only, with a large Teuchos/Thyra/Tpetra/Kokkos dependency tree. Trilinos 17 removed Epetra;
    the Epetra stack was archived by the end of 2025 [web: trilinos release notes / dealii list].
  - The modern path is Tpetra/Thyra ModelEvaluator.
  - A Rust bridge would need a `cxx` shim plus a Trilinos CMake superbuild.
  - Maintenance: active, but NOX/LOCA is a secondary priority inside Trilinos.
- **Use as a design reference** for continuation semantics, such as bordered solves and predictor
  choices. It is not a link target unless the maintainer accepts a Trilinos build.

### 1.4 russell_nonlin 3.3.1 (Rust, crates.io, updated 2026-09-24) — new lead

- **Capabilities [src].**
  - Natural-parameter and pseudo-arclength continuation, solving `G(u, λ) = 0` by Euler–Newton
    predictor–corrector.
  - Initial tangent from the bordered system, with the tangent sign kept along the branch so folds
    are traversed.
  - Automatic step control: `DeltaLambda` with Söderlind-class controllers and tangent-angle `Rdiff`.
  - `Stop` criteria on λ, a u-component, `‖u‖` or step count.
  - Typed `Status`: BorderingSmallDenominator, LargeDelta, ReachedMaxIterations,
    Continued{Residual,Delta}Divergence, Rejection, SecondaryUpdateError, UnmetStopCriterion,
    SmallStepsize, SecondaryUpdateTerminate, ContinuedFailure/Rejection, NanOrInf{Residual,Delta},
    Success.
- **H1, fits.** `calc_gg` and `calc_jac` take `&mut CooMatrix` (a sparse Jacobian) plus the `∂G/∂λ`
  vector. The linear solver is `russell_sparse` with `Genie::Umfpack` by default, or MUMPS or KLU
  (system SuiteSparse / MUMPS libraries).
- **H2, partly [src `enums.rs` `try_again`, `solver.rs` l.234–263].**
  - `Err` from `calc_gg` or `calc_jac` propagates with `?` and aborts.
  - A NaN residual gives `NanOrInfResidual`, which is not `try_again`, so it stops.
  - A step can be rejected and retried with a smaller step only through the "secondary state" hooks:
    an `update_secondary_state` returning `Err` gives `SecondaryUpdateError`, which is retried, with
    backup/restore callbacks.
  - It is usable by mapping recoverable evaluation failure onto that hook [obs], but awkward.
- **H3:** no bounds. **H4:** `update_secondary_state` returning `Ok(true)` terminates
  (`SecondaryUpdateTerminate`), which serves as a cancellation hook. There is no time budget.
- **H6/H7.** Callbacks are `Arc<dyn Fn + Send + Sync>`, so they are `Fn`, not `FnMut`; mutable state
  goes through `&mut A args`. UMFPACK is deterministic serially.
- **H8.** It needs system SuiteSparse (UMFPACK) or MUMPS-seq through `russell_sparse`, which is a
  second sparse stack beside FERAL/KLU. Feasible on Linux.
- **H9.** Single maintainer (cpmech), active in 2026. It has no fold or bifurcation detection API.

### 1.5 HOMPACK / HOMPACK90 (Fortran 77/90; TOMS 652 and 777) and PITCON (Fortran 77; MIT per Burkardt)

- **HOMPACK90 [doc netlib, mem].**
  - Probability-one homotopy `ρ(a, λ, x) = λF(x) + (1−λ)(x−a)` (or a user ρ). Three trackers: ODE
    (FIXPDF/S), normal flow (FIXPNF/S) and augmented Jacobian (FIXPQF/S); sparse variants use packed
    storage with preconditioned iterative linear algebra [mem].
  - User routines have fixed external names (F, FJAC/FJACS, RHO, RHOJAC/RHOJS). The state is global
    and not reentrant [mem].
  - There is no recoverable-failure channel, no bounds and no cancellation hook beyond the IFLAG
    return.
  - The `polsys` crate (0.1.2, 2026-07) shows the Fortran-in-build.rs pattern, but wraps only
    POLSYS_PLP (polynomial systems) [src]. That is irrelevant to general residuals.
- **PITCON [doc].** Local parameterization, limit-point and target-point location. Dense (DENSLV) or
  banded (BANSLV) linear algebra only. The user's FX reports errors through IERROR. F77 is the
  maintained form; F90 (pitcon7) exists.
- **Verdict.** Valuable algorithm references (path tracking, curvature step control, limit points).
  Poor link targets: fixed-name callbacks, dense or band storage, no H2/H3/H4.

### 1.6 MINPACK (fortran-lang/minpack, modern Fortran with C API)

- **[src `include/minpack.h`].** `minpack_hybrj`/`hybrd` (Powell hybrid dogleg) and `minpack_lmder`
  (LM). Callbacks are `(n, x, fvec, fjac, ldfjac, int* iflag, void* udata)`, with a **dense column-major
  Jacobian**. Setting `iflag < 0` terminates (cancellation, H4). There is no recoverable-trial path,
  no bounds and no sparsity. `diag`/`mode` give Moré scaling.
- **Verdict.** A reference for Moré scaling and the Powell hybrid method. Usable only for small dense
  blocks, such as tearing blocks of 8 to a few hundred unknowns.

### 1.7 Ceres Solver 2.x (C++) with `ceres-solver` 0.5.1 / `ceres-solver-sys` 0.5.3 (cxx; `system` or `source` features)

- **Capabilities [doc, src].** Trust-region LM, DOGLEG (traditional and subspace) and line-search
  minimizers for bounded nonlinear least squares. Bounds are per-parameter (`set_lower_bounds`) and
  enforced by projection. `jacobi_scaling` gives Moré column scaling. Sparse linear solvers: sparse
  normal Cholesky, sparse Schur, iterative Schur, CGNR.
- **H2, fits [doc].** `CostFunction::Evaluate` returns `bool`. On `false` the step is invalid and the
  minimizer retries with a smaller region or a better-conditioned system, up to
  `max_num_consecutive_invalid_steps` (default 5). This is the best H2 behaviour among the C/C++
  trust-region candidates.
- **H4/H5.**
  - Upstream `IterationCallback` returns SOLVER_ABORT or SOLVER_TERMINATE_SUCCESSFULLY, and
    `max_solver_time_in_seconds` gives a time budget. `TerminationType` covers CONVERGENCE,
    NO_CONVERGENCE, FAILURE, USER_SUCCESS and USER_FAILURE.
  - The Rust crate exposes the time limit and the summary, **but not IterationCallback** [src
    `solver.rs`]. The cxx shim would need extending.
- **H1, partly.** Residual blocks over parameter blocks. A general sparse Jacobian maps onto
  per-equation residual blocks over scalar parameter blocks: correct, but with overhead. More
  importantly, Ceres solves the least-squares problem `min ½‖F‖²`. For square systems, normal-equation
  solvers square the condition number. DENSE_QR avoids that but is dense.
- **H6/H7:** `num_threads(1)`. **H8:** needs Eigen and abseil/glog. The `source` feature builds a
  bundled Ceres. Feasible.

### 1.8 Rust pure crates

- **argmin 0.11.0 [mem, doc].** Trust region (Cauchy, dogleg, Steihaug), Gauss–Newton and
  quasi-Newton for unconstrained minimization. Operators return `Result`; an `Err` aborts. There are
  no bounds. `TerminationReason` includes a timeout and an interrupt. Mostly dense backends.
- **gomez 0.5.1 [doc].** Trust-region nonlinear-equation solver with box bounds, but **derivative-free
  only** (finite differences, dense). Fails H1.
- **levenberg-marquardt 0.15.0 [doc].** MINPACK port on nalgebra, dense, no bounds.
  **levenberg-marquardt-sparse 0.2.0** (PCG normal equations) and **trust-region-least-squares
  0.11.0** (dense TRF, SciPy-equivalent, bounds) are newer and low-adoption. Not audited.
- **sim-lib-numbers-continuation 0.1.1** (pseudo-arclength, 31 recent downloads). Immature. Not audited.
- **Search coverage for "no Rust PETSc/LOCA equivalent":** I searched crates.io for petsc, homotopy,
  arclength continuation, pseudo-arclength, numerical continuation, bifurcation, trust region, dogleg,
  powell hybrid, minpack, anderson acceleration, nonlinear equations solver, newton solver, kriging,
  radial basis, surrogate, gaussian process, interior point schur and nonlinear least squares,
  2026-10-03, top 8 results per query. GitHub-only crates were not searched exhaustively.

### 1.9 Uno 2.9.0 (C++ with C API; Vanaret & Leyffer; MIT; clone at `83ec44e`, 2026-10-03) — new lead for (6)

- **Capabilities [src `uno/options/Presets.cpp`].** Presets are combinations of ingredients:
  - `filtersqp`: TR, Fletcher filter, inequality-constrained QP subproblem, feasibility restoration,
    exact Hessian;
  - `filterslp`: TR, Fletcher filter, zero Hessian, i.e. SLP;
  - `funnelsqp`: TR, funnel;
  - `ipopt`: LS, Wächter filter, interior point.
  Ingredients also include an ℓ1 merit and an ℓ1 relaxation. QP/LP solvers are BQPD (nonconvex QP,
  Hessian-vector products) or HiGHS. Linear solvers are MUMPS, MA27/57/86 and SSIDS.
- **H1, fits [src `interfaces/C/Uno_C_API.h`].** Sparse COO Jacobian and Lagrangian Hessian callbacks,
  plus Jacobian/Hessian operator (JVP/HVP) callbacks. Indexing is 0- or 1-based.
- **H2, fits [src].**
  - Callbacks return `uno_int`; non-zero means an evaluation error.
  - `TrustRegionStrategy.cpp` l.133–139 catches `EvaluationError` and **decreases the radius**.
  - `BacktrackingLineSearch.cpp` l.222 keeps backtracking.
- **H4:** `uno_termination_callback` (user termination) and a time limit. **H5:** typed optimization
  status (SUCCESS, ITERATION_LIMIT, TIME_LIMIT, EVALUATION_ERROR, ALGORITHMIC_ERROR,
  USER_TERMINATION) plus a solution status (FEASIBLE_KKT_POINT, FJ point, INFEASIBLE_STATIONARY, small
  step, diverging, unbounded).
- **H6/H7.** Build with `USE_OPENMP=OFF` (the default is ON). The Rust bridge is a hand-written
  bindgen over the C API, since no crate exists.
- **H8.** CMake. **Filter-SQP needs BQPD**, which comes as a precompiled binary via BQPD_jll or under
  an academic licence from Leyffer [src `docs/installation.md`]. HiGHS works only for convex QP
  subproblems. HiGHS is already pinned, and SLP with HiGHS is fully open. **H9:** very active (commit
  the day of this research).

### 1.10 POUNCE family 0.12.0 (pinned): capabilities not yet used for these mechanisms

- **`pounce-sens-core::boundcheck` [src].**
  - `refine_step_onto_bounds`: sIPOPT fix–relax over both halves, pin and release.
  - **`step_along_path` / `path_direction`:** applies the perturbation in segments and stops at each
    active-set breakpoint: a bound reached, a multiplier reaching zero, or a hold dropped. A drop needs
    no refactorization because holds are Schur rows.
  - The docs say the path is exact for a QP and a predictor for an NLP, since nothing is re-linearized
    between breakpoints. `rowlimit` covers constraint-row limits.
  - The repository uses `SensApplication`/`IndexSchurData` (`crates/pse-backend-native/src/kkt/
    advance.rs`, `kkt/sensitivity.rs`) but **not `step_along_path`**: a grep for `step_along_path`,
    `path_direction` and `refine_step_onto_bounds` in `crates/` found no match.
- **`pounce-sensitivity::corrector` [src].** Newton iterations on the barrier KKT system against the
  held factor, one backsolve each. The docs state it cannot release a strongly held bound and is not
  a re-solve substitute.
- **`pounce-qp::homotopy` [src].** Parametric active-set QP homotopy `(1−t)QP₀ + t·QP₁` with ratio
  tests (qpOASES lineage). It applies to QP only.
- **`pounce-algorithm` SQP [src].** Line-search filter SQP (`sqp/filter.rs`, Fletcher–Leyffer). There
  is **no trust-region SQP**: `sqp/line_search.rs` l.129 mentions a "separate trust region" only in
  passing [obs]. It has BFGS/L-BFGS and a partitioned quasi-Newton Hessian. `pounce-l1penalty` is an ℓ1
  penalty wrapper.
- **`pounce-feral::schur::FeralSchurSolver`, `AlgorithmBuilder::set_kkt_schur(indices, cfg)` [src].**
  - A Schur-complement KKT backend over a caller-supplied F/S partition, with Sylvester inertia. The
    method follows the reduced-space / variable-aggregation path of Parker, Garcia & Bent.
  - It falls back transparently when the partition is unsuitable. It is serial and exact-Hessian FERAL
    only, and dense in S.
  - Not used in the repository (no grep hit for `set_kkt_schur`).
- **`pounce-presolve` [src].** Incidence, matching, DM, BTF, components, coupling classification,
  `block_solve` (damped Newton for ≤ 8-dim blocks with a `BlockSolver` trait for larger),
  `linear_eq_elim` (affine one-nonzero-per-row elimination), `trivial_elim`, `reduction_frame`
  (postsolve stack) and FBBT.

### 1.11 Surrogates: egobox (Rust; `egobox-gp` 0.36.4, `egobox-moe` 0.35.5, `egobox-ego` 0.40.2, updated 2026-09-22)

- **Capabilities [doc].** GP/kriging with several mean and correlation kernels; PLS-kriging for
  higher dimensions; sparse GP; mixture of experts (`GpMixture`, clustered, hard or smooth
  recombination). Predictions include mean, variance, `predict_gradients` and
  `predict_var_gradients`. Also LHS designs and EGO with constraints. Models serialize with serde.
- **Fit.**
  - The surrogate gives values and gradients, which makes it usable as an algebraic auxiliary model
    inside Ipopt/POUNCE.
  - Determinism needs a seeded RNG for hyperparameter optimization [mem].
  - linfa/ndarray, with a pure-Rust linalg default and an optional BLAS feature.
  - It is **not a model-management framework**. Trust-region model management (Alexandrov–Dennis;
    Eason–Biegler TRF, whose Pyomo `contrib.trustregion` is the IDAES-lineage reference) is absent
    from every candidate.
- **Alternatives.** `ferreus_rbf` 0.2.2 (RBF interpolation), `friedrich` (GP) and NLopt (BOBYQA
  model-based derivative-free TR; dense; `forced_stop`).

### 1.12 Block-structured / Schur decomposition references not linkable or not fitting H7

- **MadNLP (Julia)** with Schur/parallel KKT, **PIPS-NLP** (archived) and **HiOp** (LLNL C++;
  PriDec primal decomposition requires MPI [mem]).
- These are design references. Their parallel Schur method conflicts with H7 single-thread ownership
  unless the scenarios are spread across workers by the caller.
- **SCIP's Benders framework** (already pinned through `scip-sys`) is a library-owned decomposition
  for two-stage MINLP [mem].

---

## 2. Mechanism-by-mechanism recommendation

Recommendation classes are **library-owned (L)**, **composition of pinned-library solves (C)** and
**bespoke unavoidable (B)**. Size estimates are in Rust lines and are rough [obs].

### (1) Natural-parameter and pseudo-arclength continuation (tangent/secant predictor, step control, folds)

- **Class: C, with a small B core.** Leading library option: PETSc NEWTONAL; runner-up russell_nonlin.
- **PETSc NEWTONAL.** It is a real arc-length corrector, but:
  - it aborts on a NaN trial (H2 fails);
  - it has a fixed step size and no step adaptation or retry;
  - it gives no fold report;
  - it has a load-parameter API.
- **russell_nonlin.** It has the best feature match in Rust: pseudo-arclength, tangent, Söderlind
  step control and a sparse COO Jacobian. But:
  - H2 is only reachable through the secondary-state hook;
  - it has no bounds;
  - it uses a second sparse stack (UMFPACK/MUMPS).
- **LOCA** is the full-featured reference, at prohibitive cost.
- **Recommended shape.**
  - The predictor (secant from history, or tangent from one extra backsolve of `[F_x F_λ]` using the
    corrector's factor), step control and the arclength bordering row are bespoke, about 500–900 lines.
  - Correction is a KINSOL solve (natural) or a KINSOL solve of the bordered system (`n+1` unknowns,
    one extra dense row and column in the sparse Jacobian), or Ipopt/POUNCE for constrained
    corrections.
  - This keeps H2, H3 and H4 from KINSOL and the existing overlay machinery.
- **Fold handling.** Detect a sign change of `dλ/ds` or of the bordered determinant; KLU/FERAL can
  report the factor's sign or inertia [obs]. Fold location and branch switching stay bespoke.

### (2) Homotopy path tracking for constructed homotopies (Newton/fixed-point homotopy, Paloschi bounded, probability-one)

- **Class: C plus B.** The tracker is (1)'s machinery applied to `H(x, t)`.
- **No linkable library meets H1–H4:**
  - HOMPACK90 has fixed-name Fortran callbacks, no failure channel and no bounds;
  - LOCA's homotopy group is C++/Trilinos;
  - PETSc has no homotopy type beyond NEWTONAL's λ-scaling.
- **Bespoke parts:**
  - homotopy construction as a derived residual: Newton `F(x) − (1−t)F(x₀)`, fixed-point
    `t·F(x) + (1−t)(x − x₀)`, and Paloschi bounded homotopies with bound-preserving auxiliary terms
    that keep the original Jacobian sparsity, about 300–600 lines;
  - path tracking reused from (1).
- **Probability-one guarantees** (a random `a`, tracking through `λ` turning points) need arclength,
  not natural, parameterization. That is the same core as (1).
- **Symbolica** can generate the derived residual and its Jacobian, so derivatives stay library-owned
  [obs].

### (3) Pseudo-transient continuation (SER, TSPSEUDO)

- **Class: L if PETSc is admitted (TSPSEUDO); otherwise C plus small B.**
- **TSPSEUDO** provides the SER rule, a verify/reject hook, max dt and typed TS reasons. Use an inner
  NEWTONLS(bt) to keep H2. Bounds would need SNESVI inside, which is plausible [obs].
- **Composition alternative.** Each pseudo-step is a KINSOL solve of `M(x − x_k)/Δt + F(x) = 0`. The
  existing KINSOL session gives H2, H3 (sign constraints), H4 and H5. SER is about 100–250 lines
  (`Δt ← Δt·‖F_{k−1}‖/‖F_k‖`, capped, plus the transition to Newton).
- **Alternatively, IDA/ARKODE** (enable the `sundials-sys` features) integrate the pseudo-transient
  DAE with error-based control. This suits Pattison–Baldea style pseudo-transient models.
- **Recommendation: composition.** PETSc for this alone is not worth the build.

### (4) Nonlinear preconditioning and composition (left/right NPC, ASPIN, NASM, NGMRES/Anderson outer, nonlinear Gauss–Seidel)

- **Class: L only via PETSc. Otherwise C plus B.**
- **PETSc** is the only library implementing the composition algebra (`SNESSetNPC`/`NPCSide`,
  NGMRES, ANDERSON, NASM, ASPIN, COMPOSITE, FAS). Its costs:
  - an MPIUNI build;
  - a `-sys` crate;
  - per-subdomain SNES/scatter wiring;
  - NaN-abort in NGMRES (H2 is partial: line-search types recover, NGMRES/TR/AL do not).
- **Composition route from the pinned stack.**
  - The block nonlinear Gauss–Seidel sweep runs over the DM/BTF blocks already computed
    (`pounce-presolve` BTF plus block KINSOL solves). That gives a fixed-point map `G`.
  - Outer Anderson: KINSOL `KIN_FP` with `KINSetMAA`. Caveat [src]: KINFP ignores recoverable
    failures, so `G` must handle them internally or abort. KINSOL FP also forbids constraints.
  - Right-preconditioned Newton (`F(G(x))`) or left NPC (`x − G(x)` as the residual) can be posed as
    a derived residual to KINSOL Newton, with a JVP through finite differences of `G` or through block
    implicit derivatives.
- **Bespoke: about 600–1,200 lines** for sweep orchestration, ASPIN's preconditioned Jacobian
  `Σ R_iᵀ J_i⁻¹ R_i J` assembly or JVP, and failure and budget plumbing.
- **Recommendation: composition**, unless the maintainer wants PETSc for (3), (4) and (5) together. In
  that case PETSc's library ownership becomes worthwhile.

### (5) Trust-region nonlinear-equation solvers (Powell dogleg/hybrid, LM with bounds, Moré scaling)

- **Class: L.** The leading candidate depends on the role:
  - **Sparse square systems with bounds and recoverable trials: Ceres** (LM or DOGLEG on `½‖F‖²`, per
    parameter bounds, `Evaluate=false` gives an invalid-step retry, Jacobi/Moré scaling, time limit).
    Caveats: the normal-equation conditioning penalty for square systems, the residual-block mapping
    overhead, and the Rust crate lacking IterationCallback, so the shim needs extending.
  - **PETSc NEWTONTR** (dogleg) and **TAO BRGN/BNTR** (bounds) are library-owned, but NEWTONTR aborts
    on a NaN trial [src].
  - **MINPACK hybrj/lmder** for small dense blocks (Moré scaling via `diag`/`mode`; dense; no bounds).
- **Composition alternative.** LM-with-bounds is `min ½‖F‖² + μ/2‖Δ‖²` s.t. bounds, a sequence of
  HiGHS/Clarabel QPs or one POUNCE/Ipopt solve of the least-squares NLP, in which Ipopt's own
  restoration plays the LM role. That is already pinned but not a trust-region nonlinear-equation
  solver in the classical sense.
- **Recommendation:** Ceres for the classical TR/LM equation solver if a new C++ dependency is
  acceptable. Otherwise composition via POUNCE/Ipopt least squares.

### (6) Trust-region SQP / SLP-filter for auxiliary models

- **Class: L, with Uno the leading candidate.** Its TR presets are `filtersqp`, `filterslp` and
  `funnelsqp`.
- It meets H1 (sparse COO plus operators), H2 (TR radius shrink on evaluation error, [src]), H4
  (termination callback, time limit) and H5 (typed statuses). Its C API binds through bindgen.
- **Costs:**
  - BQPD binary or licence for nonconvex QP subproblems; HiGHS covers SLP and convex QP;
  - OpenMP off;
  - a CMake build with MUMPS. MUMPS is already in the native Ipopt build [obs].
- **POUNCE** (pinned) offers only line-search filter SQP. **PETSc/TAO** has no SQP-filter.
  **filterSQP/SNOPT/KNITRO** are commercial (not a rejection reason, but not obtainable code).

### (7) Generalized (active-set-changing) parametric sensitivity / path-following for NLPs

- **Class: mostly L (pinned) plus C.**
  - **`pounce-sens-core::boundcheck::step_along_path`** is library-owned piecewise-linear path
    following through active-set breakpoints. It covers bounds and, via `rowlimit`, constraint rows.
    The repository does not use it yet.
  - **`pounce-sensitivity::corrector`** is the held-factor corrector.
  - **`pounce-qp::homotopy`** gives the exact QP parametric path.
- **Composition for the NLP corrector:** re-linearize at breakpoints, i.e. a POUNCE/Ipopt warm-started
  solve or SQP steps, following Kungurtsev–Jäschke style predictor–corrector path following.
- **Bespoke:**
  - the outer loop deciding segment length, corrector acceptance and re-factorization, about 300–600
    lines;
  - an active-set-change certificate (strong regularity checks) that the libraries do not provide.
- **Caveat [src docs]:** the corrector cannot release a strongly held bound. A breakpoint path and a
  re-solve are needed there.

### (8) Multifidelity / surrogate model management and construction

- **Class: L for construction (egobox) plus B for management.**
- **egobox** gives GP/PLS-kriging/MoE values, gradients and variances, DOE and serde. **ferreus_rbf**
  is the RBF alternative.
- **No candidate provides trust-region model management** (first-order consistency correction,
  ratio test against the high-fidelity model, radius update; Eason–Biegler TRF). That is bespoke,
  about 600–1,000 lines, but each subproblem is an Ipopt/POUNCE solve, so it is composition-heavy.
- **Uno's TR machinery cannot host a surrogate-switching model** [obs]: its TR is internal to one model.

### (9) Multi-scenario / block-structured NLP decomposition (Schur-complement interior point)

- **Class: L (pinned, serial) plus C.**
  - **`AlgorithmBuilder::set_kkt_schur`** (`pounce-feral` `FeralSchurSolver`) gives a Schur-complement
    KKT over a caller partition. Linking/first-stage variables form S and scenario blocks form F. It
    has inertia via Sylvester's law and a transparent fallback.
  - It is serial and its S is dense, so it suits few linking variables. It is not yet used in the
    repository.
- **Parallel Schur IPM** (PIPS-NLP, MadNLP, HiOp PriDec) is either not linkable or requires MPI. It
  also conflicts with H7 unless scenarios are distributed by the caller (a bespoke coordinator over
  per-scenario factorizations).
- **Benders via SCIP** applies only to MINLP.
- **Recommendation:** adopt `set_kkt_schur` first. Parallel decomposition is B and large (more than
  1,500 lines) if ever needed.

### (10) Reduced-space / elimination support

- **Class: L (pinned) plus C.**
  - **`pounce-presolve`:** DM, BTF, components, coupling safety, `block_solve`, `linear_eq_elim`,
    `trivial_elim`, `reduction_frame` postsolve and FBBT.
  - **POUNCE Schur** gives variable aggregation (`set_kkt_schur`).
  - **Symbolica** gives symbolic substitution and derivative generation.
- **Nonlinear implicit elimination** (`y(z)` by an inner KINSOL with implicit-function derivatives) is
  composition, and the repository already has implicit-response machinery per the colleague input.
- **Bespoke:** choosing elimination sets across nonlinear blocks with conditioning and bound safety.
  Moderate size.

---

## 3. Cross-cutting findings

1. **The PETSc decision is the main fork.** PETSc alone would make (3), (4) and partially (1) and (5)
   library-owned. Its costs:
   - a source build (MPIUNI, MKL) and a new `-sys` crate. `petsc-rs` needs real MPI and is
     experimental (last commit 2025-06-03).
   - H2 is not uniform. Line-search types reject NaN trials recoverably; NEWTONTR, NEWTONAL and NGMRES
     abort [src]. An adoption would need to restrict itself to the line-search-based compositions, or
     accept abort-on-domain-error in TR/AL/NGMRES.
2. **KINSOL fixed-point semantics [src, 7.1.1].** A positive (recoverable) return from the FP/Picard
   map is ignored. Any Anderson-accelerated composition through KINSOL needs a map that never returns
   a recoverable failure.
3. **Already pinned but unused library capabilities:** `pounce-sens-core::step_along_path`,
   `pounce-sensitivity::corrector`, `set_kkt_schur`, the `pounce-presolve` elimination stack, and
   SUNDIALS ARKODE/CVODE/IDA features in the vendored tree. These are the cheapest library-owned wins.
4. **Trust-region SQP is missing from the pinned stack.** POUNCE SQP is line-search filter only. Uno
   is the only open library found with TR filter SQP/SLP, a C API and evaluation-error-driven radius
   reduction.
5. **Continuation and homotopy path tracking:** no linkable library satisfies H1–H4 together. The
   recommended design is a bespoke tracker core of roughly 0.8–1.5k lines whose every corrector is a
   pinned KINSOL/Ipopt/POUNCE solve. That keeps recoverable trials, sign constraints, cancellation and
   typed outcomes from the existing backends.

## 4. Uncertainties and unverified items

- Not verified:
  - how NOX line searches treat `computeF == false`;
  - HOMPACK90 sparse storage details;
  - PITCON's IERROR behaviour;
  - TAO's NaN handling;
  - PETSc NASM/ASPIN usability without a DM in serial;
  - SNES ANDERSON's NaN handling;
  - TSPSEUDO's behaviour when the inner SNES fails.
- PETSc behaviour was read from `main` (3.26.0-dev), not a tagged release. Release 3.24/3.25 may
  differ in detail.
- The Ceres behaviour on `Evaluate=false` comes from the docs (`max_num_consecutive_invalid_steps`).
  I did not read it in source.
- Low-adoption Rust crates (`levenberg-marquardt-sparse`, `trust-region-least-squares`,
  `sim-lib-numbers-continuation`, `fixed_point_acceleration`) were not audited.
- The bespoke-size estimates are judgement, not measurement.
