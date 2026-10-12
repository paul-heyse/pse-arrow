> Supporting evidence for [the solver methods, selection and diagnostics review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, B2 (library-research): KINSOL, PETSc SNES/TAO, diffsol-nl and the sparse factorization statistics inventory, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Its judgments are the worker's; the principal review owns the verdicts.

# B2: nonlinear-equation and least-squares solvers, plus the sparse factorization statistics inventory

**Baseline.** HEAD `ff5a4aa2f` plus the Plan 36 working tree, read as it is. Everything was read-only: I built nothing, compiled nothing, ran no tests and wrote only an empty scratch directory. Every claim is *Interface-checked* (I read the source or API) unless labelled *Proposed*.

### Summary
1. **KINSOL reports final totals only. Confirmed.** The adapter reads 12 counters, `KINGetFuncNorm` and `KINGetStepLength` after `KINSol` returns (`BN/kinsol.rs:1367-1424`). No per-iteration record exists. The vendored SUNDIALS 7.1.1 is built with `SUNDIALS_LOGGING_LEVEL=2`, so KINSOL's per-iteration INFO messages are compiled out, and profiling is OFF. For `KIN_FP` (the recycle path), KINSOL calls the map exactly once per iteration, so the project could reconstruct an exact per-iteration history inside its own residual callback at no extra cost.
2. **PETSc per-iteration norms are discarded. Confirmed.** `convergence()` receives `iteration, xnorm, ynorm, fnorm` and only forwards them to `SNESConvergedDefault` (`BN/petsc.rs:518-555`). The NASM parent and child callbacks do the same (`BN/petsc/blocks.rs:496-575`). The library's own `SNESSetConvergenceHistory` is not bound either.
3. **"No factorization statistics for MUMPS, KLU or QDLDL": partly refuted.**
   - **KLU:** the KINSOL path reads only `Common.memusage` (`BN/kinsol.rs:884-891`). The dynamics KLU path computes `rcond` and `condest` but uses them only as a refactor gate and never exports them (`BN/dynamics/linear.rs:212-258`).
   - **MUMPS and QDLDL:** confirmed. Ipopt and Clarabel keep these statistics internal.
   - **FERAL under POUNCE:** this path *does* capture statistics: factor count, pattern reuse, fill ratio, minimum and maximum pivot, inertia and nnz of A and L (`BN/pounce.rs:808-843`).
4. **Bound versus available.**
   - PETSc: only `newtontr` and `nasm` are bound, with KSP {gmres, fgmres, bcgs, preonly} and PC {none, jacobi, ilu, lu}. TAO is not bound at all.
   - Library-owned but unbound: VI solvers for two-sided boxes (`vinewtonrsls`), Anderson, QN/Broyden, NGMRES, the NLEQ-ERR line search, BRGN and POUNDERS.
   - The PETSc build has no external packages: `--with-fc=0` and none downloaded (`Cargo.toml:502-518`). So PETSc has no MUMPS, SuperLU, KLU or UMFPACK; its LU is PETSc's native SeqAIJ LU.
5. **Recycle acceleration.**
   - Library-owned: Anderson (KINSOL `KIN_FP` with `KINSetMAA`, or `SNESANDERSON`), Broyden (`SNESQN`), NGMRES, and Newton or JFNK on tears (KINSOL's default difference-quotient Jv, or PETSc `-snes_mf`).
   - Project composition needed: Wegstein and dominant-eigenvalue extrapolation. The new KINSOL damping callback cannot express dominant-eigenvalue extrapolation, because it restricts β to (0, 1].
6. **Newer releases that matter.**
   - SUNDIALS 7.3 adds `KINSetDampingFn` and `KINSetDepthFn`, per-iteration callbacks for fixed-point and Picard iterations.
   - SUNDIALS 7.8 adds `SUNLogger_SetQueueAndFlushMsgFns`, which allows typed in-process log capture.
   - SUNDIALS 7.5 fixes a KINSOL bug that suppressed logging messages.
   - `sundials-sys` 0.6.2 (2024-12-25) is still the latest crate, so reaching any of these needs a system SUNDIALS or a vendor bump.
   - PETSc 3.25 and 3.26 make no significant change to SNES or TAO monitoring.

---

### 1. KINSOL (SUNDIALS 7.1.1, vendored in `sundials-sys` 0.6.2)

**Build facts.**
- `build.rs:97-112` passes only BUILD_* and `ENABLE_KLU`.
- The CMake defaults are therefore in force: `SUNDIALS_LOGGING_LEVEL=2` (errors and warnings; `vendor/cmake/SundialsBuildOptionsPre.cmake:93`), `SUNDIALS_BUILD_WITH_PROFILING=OFF` (`:65`) and `SUNDIALS_INDEX_SIZE=64`.
- `BN/kinsol.rs` asserts that `sunindextype` is 8 bytes. KLU is therefore driven through `klu_l_*`.

**1.1 Methods and applicability.** The 7.1.1 header surface is `include/kinsol/kinsol.h:93-149` and `kinsol_ls.h:67-95`.

| Strategy | Applicability | Local binding |
|---|---|---|
| `KIN_NONE`: Newton with no globalization | Good initial guess; F is C¹ and J is nonsingular near the root | `Strategy::Newton` (`BN/kinsol.rs:1359-1364`) |
| `KIN_LINESEARCH`: Newton with backtracking, β-condition and `KINSetMaxBetaFails` | Poor initial guess; needs a merit decrease along a descent direction | `Strategy::LineSearch`, the default |
| `KIN_PICARD`: F = Lx − N(x) with constant L | L factored once; contractive when ‖L⁻¹N′‖ < 1. No constraints. | `Function::Picard`; constraints refused (`:231-235`) |
| `KIN_FP` with Anderson (`KINSetMAA`, `KINSetDampingAA`, `KINSetDelayAA`, `KINSetOrthAA`) | x = G(x) with G contractive or near-contractive; Anderson helps a moderate spectral radius. No constraints, no Jacobian. | `Function::FixedPoint`; recycle maps (`BN/recycle.rs:3`) |

Other controls:
- **Modified Newton:** `KINSetMaxSetupCalls` (default 10) and `KINSetNoInitSetup`. Residual monitoring is controlled by `KINSetNoResMon`, `KINSetMaxSubSetupCalls` (default 5) and `KINSetResMonParams`/`ConstValue` (ω_min 1e-5, ω_max 0.9; Context7, KINSOL guide). It applies only to matrix-based linear solvers.
  - Bound locally: `KINSetMaxSetupCalls(setup_interval)`, `KINSetNoInitSetup(reuse)` and `KINSetNoResMon(0)` (`:1330-1340`).
  - Not bound: `KINSetMaxSubSetupCalls` and `KINSetResMonParams`/`ConstValue`.
- **Eisenstat–Walker forcing** (Krylov routes only): `KINSetEtaForm`, `KINSetEtaConstValue` and `KINSetEtaParams` are all bound (`settings/kinsol.rs` `Eta`; `kinsol.rs:1355`). `KINSetNoMinEps` is not bound.
- **`KINSetConstraints`:** per-component sign conditions only: 0 none, 1 means ≥0, 2 means >0, −1 means ≤0, −2 means <0. It cannot express two-sided boxes. The adapter shifts one-sided bounds onto sign constraints through `offsets` (`:360`, `:3188-3231`) and refuses general boxes (test `general_boxes_are_not_sign_constraints`).
- **Steps and tolerances:**
  - Bound: `KINSetMaxNewtonStep` (`Method::max_newton_step`), `KINSetScaledStepTol` (derived from policy) and `KINSetFuncNormTol(accuracy.feasibility)` (`:1314-1318`).
  - Not bound: `KINSetRelErrFunc` (difference-quotient increment), `KINSetReturnNewest` and `KINSetMaxBetaFails`.
- **Scaling:** `u_scale` comes from `settings.variable_scales` (`:1306`) and is published to `fs`. Both scaling vectors are policy-derived, not caller inputs (`settings/kinsol.rs:1-5`).
- **Linear solvers** (`settings/kinsol.rs:23-50`): KLU (CSC), Dense, and SPGMR, SPFGMR, SPBCGS and SPTFQMR with an analytic Jv and right preconditioning.
  - The BTF-block FERAL LU preconditioner is in `BN/kinsol/block.rs`.
  - Not bound: `SUNLinSol_KLUSetOrdering` (default COLAMD) and KINSOL's internal difference-quotient Jv (used when no jtimes function is supplied).

**1.2 Upstream guidance** (KINSOL guide, Mathematics chapter, via Context7):
- Line search for poor initial guesses.
- Modified Newton with residual monitoring when Jacobian setup is expensive.
- Inexact Newton with Krylov and Eisenstat–Walker for large problems, matrix-free when only Jv is cheap.
- Picard when a constant, well-conditioned L captures the stiff linear part.
- Fixed point with Anderson when only a map G is available. Anderson depth around 5–10 is typical, and `DelayAA` lets an early contraction phase settle before acceleration starts.
- KINSOL has no trust region and no pseudo-transient continuation.

**1.3 Reuse hooks.**
- Setup reuse across solves is bound: `KINSetNoInitSetup` with `setup_usable` (`:1630-1637`). `replace()` keeps the native allocation and symbolic layout when the pattern is unchanged (`:910-960`).
- `SUNLinSol_KLU` reuses the symbolic analysis and calls `klu_refactor` on every setup after the first. After a refactor it computes `rcond` (and `condest` if rcond < ε^(2/3)) and refactors fully with pivoting if condest > ε^(-2/3) (`vendor/src/sunlinsol/klu/sunlinsol_klu.c:255-335`).
- `SUNLinSol_KLUReInit` exists for pattern changes.

**1.4 Statistics and diagnostics API (7.1.1).**
- Nonlinear: `KINGetNumNonlinSolvIters`, `KINGetNumFuncEvals`, `KINGetNumBetaCondFails`, `KINGetNumBacktrackOps`, `KINGetFuncNorm`, `KINGetStepLength`, `KINGetWorkSpace`, `KINPrintAllStats(FILE*, fmt)` and `KINGetReturnFlagName`.
- Linear-solver interface: `KINGetNumJacEvals`, `KINGetNumLinFuncEvals`, `KINGetNumPrecEvals`, `KINGetNumPrecSolves`, `KINGetNumLinIters`, `KINGetNumLinConvFails`, `KINGetNumJtimesEvals`, `KINGetLastLinFlag`, `KINGetLinWorkSpace`, plus `KINGetJac` and `KINGetJacNumIters` (unbound locally).
- KLU handles: `SUNLinSol_KLUGetCommon`, `GetNumeric` and `GetSymbolic` (`sunlinsol_klu.h:118-120`).

Per-iteration hooks in 7.1.1:
- **No user info handler.** The 7.x API removed it; the header has no setter.
- **SUNLogger.** Per-iteration lines (`nni`, `nfe`, `fnorm`) are emitted inside `#if SUNDIALS_LOGGING_LEVEL >= SUNDIALS_LOGGING_INFO` (`vendor/src/kinsol/kinsol.c:2548`, `:2933-2939`), so they are absent in this build. Enabling them needs a rebuild at level 3 or higher. In 7.1.1 output goes to a file name only.
- **SUNProfiler.** OFF. It would give timed regions anyway, not iteration data.
- **Reconstruction from callbacks.**
  - In `KINFP`, each iteration calls `kin_func` exactly once (`kinsol.c:2858`) and sets `fnorm = ‖fscale·(G(x)−x)‖` (`:2929`). The project's `residual()` callback (`BN/kinsol.rs:501-536`) already sees `x` and `G(x)` on every call, so an exact per-iteration history (‖Δ‖, component deltas) is available at zero extra cost. *Proposed:* record it in `CallbackState`.
  - For the Newton line search, callback invocations mix trial and accepted points. *Proposed:* read `KINGetNumNonlinSolvIters` from inside the callback to mark iteration boundaries. This needs the KINSOL memory pointer in `Context`, which it does not currently hold.

**1.5 Bound versus unbound.**
- Bound: final counters (`kinsol.rs:1386-1424`), KLU `memusage` only (`:884-891`), and success-free callback timing and counts (`CallbackState.counts`/`seconds`, `callback.rs:209-232`). Only *failure* events are pushed to `execution.progress` (`callback.rs:359-369`).
- **Recycle and tear history is totals only: confirmed.** `CausalMap` is a `FixedPointOracle` (`recycle.rs:375-395`), and the runtime defaults to `anderson: 0` (`pse-runtime/src/workflow/strategies/requests.rs:124`), which is damped successive substitution.

**1.6 Newer releases** (SUNDIALS CHANGELOG, read on GitHub main):
- **7.3:** `KINSetDampingFn` (`KINDampingFn(iter, u, g, qt_fn, depth, ud, *beta)`, β ∈ (0, 1]) and `KINSetDepthFn` (adaptive Anderson depth, with an option to remove a column). Both are per-iteration observation points for fixed-point and Picard.
- **7.5:** `KINSetMAA`/`OrthAA` may be called after `KINInit`; also the fix "logging messages would not be output" in KINSOL.
- **7.6:** the KINSOL info-logging function was always called.
- **7.7:** `SUNLogger_Set*File(FILE*)`; CMake renames to `SUNDIALS_ENABLE_PROFILING` and `SUNDIALS_ENABLE_KLU`.
- **7.8:** `SUNLogger_SetQueueAndFlushMsgFns`, which delivers log messages to user callbacks in-process; a KINSOL leak fix.

**Probe to settle a doubt (not run):** it is unclear whether KINSOL 7.1.1 INFO logging actually emits anything, given the 7.5 fix. Question: does a level-3 build print per-iteration `fnorm`? Command: rebuild the vendored SUNDIALS with `-DSUNDIALS_LOGGING_LEVEL=3` and set `SUNLogger_SetInfoFilename(logger, "kin.log")` on a 2×2 `KIN_LINESEARCH` fixture. Expected observation: one `KINSolInit` line, then lines with `nni`/`fnorm` for each iteration.

**1.7 Interrupt and outcome mapping.**
- Callback returns are 0 for success, positive for recoverable and negative for unrecoverable. The adapter's `result()` returns 1 for a recoverable trial and −1 for a terminal failure (`kinsol.rs:491-499`).
- KINFP and Picard only test for negative returns. The adapter therefore latches a terminal stop for map failures (`:518-534`). This is correct.
- Cancellation is checked after every callback (`callback.rs:330-335`).
- Native codes map to `Termination` (`kinsol.rs:1642-1675`), including `KIN_STEP_LT_STPTOL` → Limit and `KIN_LINESEARCH_BCFAIL` → Numerical.

### 2. PETSc 3.24.0 (SNES and TAO)

The pinned source is at `~/.cache/pse-arrow-petsc-source/petsc-3.24.0`. Configure options are at `Cargo.toml:502-518`; the patch `docker/solvers/petsc-pse.patch` only hardens `PetscStrlcat`.

**2.1 Methods** (`include/petscsnes.h:25-46`, `:730-739`; `petsctao.h:205-238`).
- **NEWTONLS** with the line searches `bt`, `secant` (formerly `l2`, deprecated in 3.24), `cp`, `nleqerr` (Deuflhard's affine-invariant NLEQ-ERR), `basic`/`none`, `bisection`, `shell` and `ncglinear`.
- **NEWTONTR:** fallback NEWTON, CAUCHY or DOGLEG (`SNESNewtonTRSetFallbackType`), QN model none, same or different (`SNESNewtonTRSetQNType`), and `SNESNewtonTRSetNormType`. NEWTONTRDC and NEWTONAL (arc-length) also exist.
- **QN:** LBFGS, BROYDEN, BADBROYDEN, with scaling none, scalar, diagonal or Jacobian and restart none, Powell or periodic.
- **NGMRES and ANDERSON:** select none, difference or line search; restart options.
- Others: NCG, NRICHARDSON, NGS, MS, FAS, NASM, ASPIN, COMPOSITE (additive, multiplicative, additive-optimal), PATCH, KSPONLY.
- **Nonlinear preconditioning:** `SNESSetNPC` and `SNESSetNPCSide`.
- **VI:** `SNESVISetVariableBounds(xl, xu)` gives true two-sided boxes on a *square* system, i.e. mixed complementarity. VINEWTONRSLS uses a reduced-space active set; VINEWTONSSLS uses a semismooth Fischer–Burmeister reformulation. `SNESVIGetActiveSetIS` returns the active set.
- **JFNK:** `-snes_mf` (no PC) and `-snes_mf_operator` (the user matrix as PC).
- **Lagging:** `SNESSetLagJacobian` and `SNESSetLagPreconditioner(Persists)`, which is modified Newton. Eisenstat–Walker via `SNESKSPSetUseEW`/`SetParametersEW`.
- **TAO least squares:**
  - BRGN: Gauss–Newton over a bounded subsolver, with regularization USER, L2PROX, L2PURE, L1DICT or LM (Levenberg–Marquardt, diag(JᵀJ)) (`petsctao.h:471-514`). Options `-tao_brgn_*` are at `brgn.c:356-361`.
  - POUNDERS: derivative-free model-based trust region honouring bounds (`pounders.c:91-95`, `:471`; options `-tao_pounders_delta`, `npmax`, `gqt`).
  - Bounded solvers: BNLS, BNTR, BNTL, BQNLS, BQNK*, BLMVM, TRON and GPCG.
  - Residual inputs: `TaoSetResidualRoutine`, `TaoSetJacobianResidualRoutine`, `TaoSetResidualWeights`.

**2.2 Upstream guidance.**
- The PETSc manual and Brune–Knepley–Smith–Tu (SIAM Review 2015) make NEWTONLS with `bt` the default. NEWTONTR is preferred for poor initial guesses or near-singular J. QN suits expensive Jacobians. NGMRES and Anderson work best as outer accelerators around an NPC such as NRICHARDSON, NGS or NEWTONLS. FAS and NASM suit spatial or block decomposition. `nleqerr` suits badly scaled problems where residual-norm merit functions mislead.
- Krylov versus direct: direct LU (PCLU) is robust for moderate n and irregular flowsheet sparsity. GMRES with ILU, ASM or a field split is needed at scale, and its quality depends on block structure. Matrix-free is only as good as its preconditioner.
- Pseudo-transient continuation (TSPSEUDO) belongs to the TS worker.

**2.3 Reuse.** Jacobian and PC lagging; the symbolic LU is reused when the matrix nonzero pattern is unchanged (`MatLUFactorSymbolic` is skipped on the same pattern); `TaoSetRecycleHistory` and `TaoGetLMVMMatrix`. Locally, `ReusePolicy::RequireReuse` is refused because each attempt creates fresh objects (`BN/petsc.rs:598-600`).

**2.4 Statistics API.**
- SNES: `SNESGetIterationNumber`, `SNESGetLinearSolveIterations`, `SNESGetNumberFunctionEvals`, `SNESGetConvergedReason(String)`, `SNESGetNonlinearStepFailures`, `SNESGetLinearSolveFailures`, `SNESGetFunctionNorm`.
- Per-iteration: `SNESMonitorSet(snes, fn(snes, it, fnorm))`, and **`SNESSetConvergenceHistory(a[], its[], na, reset)`**, a library-owned per-iteration record of residual norms and linear iterations that needs no callback (`snes.c:4478`).
- Trust region: `SNESNewtonTRGetTolerances` and `GetUpdateParameters`. The current radius is not exposed by a getter; it is visible through the TR pre/post check hooks.
- Jacobian checking: `SNESTestJacobian` / `-snes_test_jacobian`.
- KSP: `KSPGetConvergedReason`, `KSPGetResidualHistory`, `KSPComputeExtremeSingularValues` (a cheap condition estimate from GMRES with `KSPSetComputeSingularValues`).
- PC and factors: `PCGetFailedReason`, `MatFactorGetError`, `MatFactorGetErrorZeroPivot(pivot, row)`, `MatGetInfo` (`fill_ratio_given`/`needed`, `factor_mallocs`), `MatGetInertia`. MUMPS-specific getters (`MatMumpsGetInfog`/`Rinfog`/`NullPivots`) exist but are unusable here because MUMPS is not built.
- Logging: `-log_view`, `PetscLogStageRegister`, events.
- TAO: `TaoGetSolutionStatus`, `TaoGetResidualNorm`, `TaoSetConvergenceHistory`, `TaoMonitorSet`.

**2.5 Bound versus unbound.**
- The bridge (`crates/pse-petsc-sys/bridge/pse_petsc.c`) binds create, type, options, function, Jacobian, convergence test, tolerances, TR tolerances and update parameters, KSP and PC type, NASM, TS, and statistics. The statistics are reason, iterations, function evaluations, linear iterations and final norm (`:322-329`).
- Not bound: `SNESMonitorSet`, `SNESSetConvergenceHistory`, all KSP, PC and factor diagnostics, line-search selection, VI bounds, NPC, every type other than `newtontr` and `nasm`, and all of TAO.
- **The discarded per-iteration norms are confirmed** (`petsc.rs:518-555`; `blocks.rs:496-575`).
- The safe profile refuses raw options and bounds: `controls.options.is_empty()` is required (`petsc.rs:585-593`), and it is documented as an "unbounded root profile".

**2.6 Newer releases.**
- 3.25 (2026-03-30): `SNESSetObjectiveDomainError`; `SNES_DIVERGED_FNORM_NAN` renamed to `SNES_DIVERGED_FUNCTION_NANORINF`; `SNES_DIVERGED_OBJECTIVE_*`; `TaoTerm`; a separate TAO residual-evaluation counter; the BRGN prefix fix; MUMPS mixed precision and out-of-core.
- 3.26 (2026-09-28): `-snes_mf` respects an explicit PC type; FAS coarse-correction line search; `SNESLINESEARCHBASIC` deprecated; `TaoGetConvergedReasonString`; `KSP_DIVERGED_INNER_SOLVE_FAILED`.
- No new per-iteration monitoring primitive in either release.

**2.7 Outcome mapping.**
- A callback returning 1 becomes `SNESSetFunctionDomainError` or `SNESSetJacobianDomainError`; other non-zero values become `PETSC_ERR_USER` (`pse_petsc.c:47-58`).
- The convergence-test callback is the monitor-driven stop: a failed `state.evaluate`, including cancellation, sets `SNES_USER_DIVERGED` (`petsc.rs:546-553`).
- `SNES_TR_DELTA` and `STEP_RELATIVE` map to Limit (`petsc.rs:972`).

### 3. diffsol-nl 0.1.1 (a dependency of diffsol 0.16.2)

- It provides `NewtonNonlinearSolver<Ls, Lsearch>` with `NoLineSearch` or `BacktrackingLineSearch` (τ = 0.5, c = 1e-4, at most 10 steps) (`line_search.rs:44-105`).
- `Convergence` keeps a **mean convergence rate** `rate = (‖δ_k‖/‖δ_1‖)^(1/(k−1))` and η = rate/(1 − rate). It declares divergence when rate > 0.9 or when the iteration cannot converge within `max_iter` at the observed rate (`convergence.rs:68-131`). This is a working in-library contraction estimator; the same formula applies to recycle history.
- Statistics: only `niter()`, `eta()` and the line search's `n_iters`, plus `trace!` logs.
- Register R-47 excludes it as the nested inner solver (`docs/authoritative_design/sections/scope-and-open-design.md:131`).

### 4. Factorization statistics inventory (sole owner)

| Library (pinned) | Statistics available upstream | Locally bound |
|---|---|---|
| **KLU** 2.3.3 (SuiteSparse, `suitesparse_sys` 0.1.4; `klu.h:219-227`) | `Common`: `structural_rank`, `numerical_rank`, `singular_col`, `noffdiag`, `flops`, `rcond` (cheap min/max\|U_kk\| ratio), `condest` (Hager 1-norm), `rgrowth`, `work`, `memusage`, `mempeak`. `Numeric`: `lnz`, `unz`, `nzoff`, `max_lnz_block`. `Symbolic`: `nblocks`, `maxblock`. Functions `klu_rcond`, `klu_condest`, `klu_rgrowth`, `klu_flops`. | KINSOL: `memusage` only (`kinsol.rs:887-889`). Dynamics: computes `rcond`/`condest` as a gate (`dynamics/linear.rs:233-256`) but exports nothing. `singular_col` on `KLU_SINGULAR` is reduced to a message (`:364-369`). |
| **faer** 0.24.4 sparse LU, QR, LLᵀ/LDLᵀ/LBLᵀ | Symbolic `len_val`, `n_supernodes`, row and column permutations. Dense LU and QR report `transposition_count`; dense LDLᵀ reports `dynamic_regularization_count`. No condition estimate, no pivot growth, and no sparse inertia output (inertia derivable from the LDLᵀ diagonal). | None. Symbolic and numeric split reused (`BN/dynamics.rs:753-757`, `square_response/actions.rs:64`, `PM/implicit.rs:33`). |
| **FERAL** 0.18.0 | `FactorStats` (`numeric/solver.rs:102-147`): `nnz_a`, `nnz_l`, `fill_ratio`, inertia, minimum and maximum \|pivot\|, `pattern_reused`, `scaling_info`, `ordering_info` (requested versus used, escalated, supernodes, maximum front), `n_tiny` (statically perturbed pivots, comparable to MUMPS INFO(25)). Also `estimate_condition_1norm`, `RefinementDiagnostics` (κ₁ and refinement trajectory, `solve.rs:2340`), `quality_level`, `work_estimate`, `profile_report`. `SparseLu`: `condition_estimate_1`. | POUNCE path: factors, pattern reuse and changes, fill ratio, minimum and maximum pivot, inertia, nnzA/L (`pounce.rs:808-843`). Diagnostics: `jacobian_condition` and `kkt` inertia and κ₁ through separate factorizations (`conditioning.rs:19-156`; consumer `pse-runtime/.../diagnostics.rs:796`). Not bound: `n_tiny`, `scaling_info`, `ordering_info`, refinement. feral 0.19.0 was released 2026-10-10; I did not review it. |
| **MUMPS** 5.9.1 (inside Ipopt) | `ICNTL(11)`: 2 gives ω₁/ω₂ in RINFOG(7/8); 1 adds RINFOG(9) error bound and cond1/cond2 in RINFOG(10/11), described as "very costly". `ICNTL(24)` null-pivot detection gives INFOG(28) deficiency and `PIVNUL_LIST`; `ICNTL(25)` null-space basis; `ICNTL(56)` rank-revealing; `ICNTL(33)` determinant. INFOG(12) negative pivots (inertia), (13) delayed pivots, (3/4, 20) estimated and (9/10, 29) actual fill, (16–19) memory, (25) tiny pivots; RINFOG(1/3) flops (user guide §5, extracted pdf lines 2325-2372 and 2671-2755). | Not reachable. Ipopt reads INFOG(12) and uses ICNTL(24) only in dependency detection (`IpMumpsSolverInterface.cpp:430, 538, 676-725`). Statistics escape only through the journal. **Confirmed unbound.** |
| **SPRAL SSIDS** (inside Ipopt) | `inform`: `num_neg`, `matrix_rank`, `num_delay`, `num_factor`, `num_flops`, `maxfront`, `num_two`, `flag` | Inside Ipopt only (`IpSpralSolverInterface.cpp:574-674`) |
| **oneMKL Pardiso** (Ipopt and Clarabel) | iparm(14) perturbed pivots, (15–17) memory, (18) nnz of factors, (19) MFLOPS, (7) refinement steps, (22/23) inertia | Inside Ipopt only (`IpPardisoMKLSolverInterface.cpp:551-659`) |
| **HSL** MA27/MA57/MA86/MA97 (candidate) | MA27 INFO(15) negative eigenvalues; MA57 INFO(24) negative eigenvalues, INFO(25) rank; MA86/MA97 `num_neg`, `matrix_rank`, `num_delay`, `num_flops`, `num_factor`, `maxfront` | Inside Ipopt only |
| **QDLDL** (Clarabel 0.11.1) | `positive_inertia()`, `regularize_count()` (`qdldl.rs:105-111`) | Unreachable. `kktsolver` is a private boxed trait without statistics (`default/kktsystem.rs:18`; `quasidef/mod.rs:15-25`); `DefaultInfo` has none. **Confirmed.** |
| **UMFPACK** (candidate; `suitesparse_sys` feature `umfpack`) | Info[]: RCOND, LNZ/UNZ, FLOPS, PEAK_MEMORY, NUMERIC_SIZE, scaling minimum and maximum (from memory, not checked here) | Not built |
| **SuperLU** (candidate) | `dgssvx`: rpg (pivot growth), rcond, ferr/berr, `mem_usage_t` (from memory) | Not present |
| **PETSc native LU** | `MatGetInfo` fill ratios, `MatFactorGetErrorZeroPivot` | Unbound |

**Cross-cutting point.** Under Ipopt, every linear-solver statistic stays inside `TSymLinearSolver` and reaches the application only as journal text. The local Ipopt bridge is C++ and receives `IpoptData*` in `intermediate_callback` (`pse_ipopt_sequence.cpp:113-125`). *Proposed:* register a custom `Journal` on `J_LINEAR_ALGEBRA` at `J_DETAILED`, or read `IpoptData::TimingStats()`, as the only routes to these statistics. Both are text- or timing-based, not typed.

### 5. Decision guidance

**Square nonlinear systems.**

| Feature | Cheap, trustworthy estimate | Implication |
|---|---|---|
| Size and sparsity | nnz(J), n, symbolic fill (FERAL `work_estimate`, KLU `Symbolic.lnz` estimate) | Direct up to roughly 10⁵ with low fill. Otherwise Krylov with Eisenstat–Walker and block preconditioning. |
| Block structure | BTF and Dulmage–Mendelsohn blocks (already computed: `kinsol/block.rs`, presolve `btf_ms`); `KLU Symbolic.nblocks`/`maxblock` | Many small blocks favour block-sequential solves or a BTF preconditioner. One large SCC suggests NASM or ASPIN or a field split. |
| Nearness to singularity | KLU `rcond` after any refactor (free, already computed by SUNLinSol_KLU); `condest` for O(nnz) more; FERAL min\|pivot\| and `n_tiny`; MUMPS INFOG(28) | rcond < ε^(2/3) or tiny pivots favour trust region (NEWTONTR dogleg) over line search; rank deficiency suggests reformulation or diagnosis rather than retries |
| Bounds | Count of two-sided boxes versus one-sided | One-sided: KINSOL shifted sign constraints. Two-sided on a square system: SNES VINEWTONRSLS (library-owned, unbound) or an NLP. |
| Initial-guess quality | ‖F(x₀)‖ relative to scale; first-step β-condition failures; `KINGetNumBacktrackOps`/iterations | Good guess: Newton with no globalization or modified Newton. Poor guess: line search, then trust region, then pseudo-transient (TS) or homotopy. |
| Jacobian cost | Callback seconds per Jacobian versus per residual (already in `CallbackState.seconds`) | High ratio favours modified Newton (`msbset`, residual monitoring) or QN/Broyden (`SNESQN`, or TR with a QN model) |

**Flowsheet recycle (tear) maps.**
- The contraction factor ρ ≈ ‖Δ_k‖/‖Δ_{k−1}‖, with Δ_k = G(x_k) − x_k, can be computed exactly from the KIN_FP callback history (§1.4). diffsol-nl's mean-rate formula is a robust geometric average of the same quantity.
- Per-component Wegstein slopes s_i = Δg_i/Δx_i are cheap and diagnose which tears are slow or oscillatory.
- How to read ρ:
  - ρ ≪ 1: plain substitution.
  - ρ about 0.5–0.95 with one dominant mode: dominant-eigenvalue extrapolation or Anderson.
  - Several modes: Anderson (depth 3–10) or NGMRES.
  - ρ ≥ 1 or interacting tears: Newton or JFNK on the tears, or Broyden.
  - Oscillation (s_i < 0): damping, or bounded Wegstein q.
- Ownership:
  - **Library-owned:** Anderson (KINSOL `KIN_FP` with MAA — bound, default 0 — or `SNESANDERSON`); Broyden (`SNESQN` BROYDEN or BADBROYDEN on F = x − G(x)); NGMRES (`SNESNGMRES`); Newton on tears (KINSOL LineSearch on F = x − G(x) with SPGMR and KINSOL's internal difference-quotient Jv, unbound because the adapter requires an analytic Jv; or PETSc `-snes_mf`); damping (`KINSetDamping`, bound).
  - **Project composition:** Wegstein (a diagonal secant step with q bounds) and dominant-eigenvalue extrapolation (Orbach–Crowe). SUNDIALS 7.3's `KINDampingFn` restricts β to (0, 1], so it can express damping for oscillatory λ < 0 only, not the β = 1/(1 − λ) > 1 extrapolation. That is *Interface-checked* from the KINSOL guide via Context7, not from pinned source, since 7.3 is not vendored. `KINDepthFn` could implement adaptive Anderson depth once SUNDIALS reaches 7.3 or later.

**Least squares.**
- Small residual at the solution with an available Jacobian: Gauss–Newton or LM (TAO BRGN with LM regularization, bounded through the BNLS subsolver).
- Large residual: a full-Hessian or quasi-Newton method (BQNLS, BNTR).
- No derivatives or noisy residuals: POUNDERS (bounded, models built from about 2n+1 points).
- Cheap features to inspect: the conditioning of J at x₀ (FERAL QR or LU κ₁), the ratio of residual norm to scale, parameter bounds, and evaluation cost.
- None of TAO is bound locally. Current least-squares routing goes through the NLP paths; I did not inspect those (outside B2).

### 6. Search coverage and uncertainties
- **KINSOL symbols:** grep across `BN/kinsol.rs`, `kinsol/block.rs`, `execution/kinsol.rs`, `settings/kinsol.rs` and `recycle.rs`.
- **PETSc symbols:** all of `pse_petsc.c` plus `BN/petsc*`.
- **Factor-statistics fields:** grep of `crates/**/*.rs` (generated paths excluded) for `rcond|condest|rgrowth|numerical_rank|singular_col|mempeak|memusage|noffdiag|last_factor_stats|n_tiny|fill_ratio`.
- **No Wegstein or dominant-eigenvalue code:** grep of `crates/`, `python/pse` and `docs/authoritative_design` found no match.
- **MUMPS INFOG indices** are from the 5.9.1 user guide. The Pardiso, UMFPACK and SuperLU indices are partly from memory, flagged where so.
- **Uncertain:** whether 7.1.1 KINSOL INFO logging works at all (probe in §1.6). Whether `sundials-sys` can link a system SUNDIALS 7.8 with KLU and 64-bit indices without patching is unverified. Question: does a non-`build_libraries` build against SUNDIALS 7.8 compile the adapter? Command: `cargo check -p pse-backend-native --features kinsol` with `SUNDIALS_DIR` pointing at the install. Expected observation: success, or binding drift on renamed symbols.

**Files written:** none in the repository. The scratch directory `/tmp/claude-1000/-home-paul-pse-arrow/5335360e-031d-4d0d-ac32-d1df2e1106de/scratchpad/B2/` holds an extracted MUMPS user-guide text, `mumps.txt`.