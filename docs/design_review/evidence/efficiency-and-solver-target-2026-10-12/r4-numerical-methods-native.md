> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R4 (library-research), Part A: numerical methods and native sessions. Facts A and B of X2 correct its §2.2 and Part B statements about retained symbolic analysis. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R4 (Part A): numerical methods and native sessions

Inquiry R4 of the consolidated efficiency and solver target review. `library-research`, read-only and static. Nothing was built, run or probed.

- **Baseline.** `main` at `6c368c88a`, clean, 2026-10-12.
- **Prior evidence used as the baseline.** The solver evidence B1–B5 in `docs/design_review/evidence/solver-methods-and-diagnostics-2026-10-12/`: `nlp-solver-capabilities.md` (B1), `nonlinear-and-linear-solver-capabilities.md` (B2), `dynamics-solver-capabilities.md` (B3), `lp-conic-global-solver-capabilities.md` (B4), `candidate-libraries.md` (B5a), `reference-practice-and-selection-literature.md` (B5b) and `method-landscape-and-selection.md`. That evidence was taken at `ff5a4aa2f` plus the Plan 36 working tree.
- **How this file treats B1–B5.** It records **deltas only**. Every capability inventory, statistics list and "bound versus unbound" table in B1–B4 still stands unless a delta below says otherwise. Section references such as "B1 §1.4" point into those files.
- **Evidence labels.**
  - Source and header facts are *Interface-checked*, with the path named.
  - Release and upstream facts are *Interface-checked* against GitHub releases, queried 2026-10-12.
  - Anything about this repository's future shape is *Proposed*.
  - No claim here is *Tested* or *Measured*.

Abbreviations: `BN/` = `crates/pse-backend-native/src/`, `RT/` = `crates/pse-runtime/src/`, `IPOPT/` = `.claude/skills/native-solver-libraries/content/corpus/ipopt@3.14.20/src/`, `PETSC/` = `~/.cache/pse-arrow-petsc-source/petsc-3.24.0/`, `POUNCE/` = `~/.cargo/git/checkouts/pounce-8316ac8e070882e5/ff63869/crates/`, `SUN/` = `~/.cargo/registry/src/*/sundials-sys-0.6.2/`, `HIGHS/` = `~/.cargo/registry/src/*/highs-sys-1.15.0/HiGHS/highs/`.

---

## 0. What changed since B1–B5

### 0.1 Code and pins

Sixty-six commits since `ff5a4aa2f`. The solver-relevant code delta is Plan 36's working tree, now committed (commit `677394474` "continued plan 36 work"), plus Plan 37 changes outside the native backend.

- **No solver pin moved.** The `Cargo.toml` diff adds only `surrealdb-syn`/`surrealdb-sql`.
- **Native backend delta.** Only one item matters here: `BN/dynamics/linear.rs` now owns a KLU refactor policy that copies SUNDIALS 7.1.1's (`klu_l_refactor`, then `klu_l_rcond`, then `klu_l_condest` gates; around lines 230–262). `rcond` and `condest` still only gate a refactor and are not exported.
- **B1–B4 "unbound" claims re-confirmed at HEAD.** I grepped `crates/pse-backend-native`, `crates/pse-ipopt-sys` and `crates/pse-petsc-sys` (`*.rs`, `*.cpp`, `*.c`, generated and bindings excluded) for `Statistics()`, `TimingStats`, `timing_statistics`, `SNESSetConvergenceHistory`, `SNESMonitorSet`, `IDAGetIntegratorStats`, `IDAGetLastOrder`, `SCIPcollect`, `Highs_getRunTime`, `KINSetDampingFn` and `IDA_ONE_STEP`. There were no hits. *Interface-checked.*

### 0.2 Upstream releases since B1–B5

Nothing new up to 2026-10-12. Checked: POUNCE latest v0.12.0 (2026-09-19); Uno latest v2.9.0, with the pin a post-2.9.0 commit; SUNDIALS latest v7.9.0 (2026-09-03). The HiGHS, Clarabel, SCIP, PETSc, Diffsol and egobox statuses are as B3/B4 recorded them on the same day. FERAL 0.19.0 (2026-10-10, noted by B2) is still unreviewed.

### 0.3 Framing changes that alter B1–B5 conclusions

1. **No HSL.** B5a ranked HSL MA27 (+MC19) second among new libraries, and B1 §6 called HSL admissible. Both are withdrawn by the brief's requirement. Consequences:
   - **No equilibration NLP scaling in Ipopt.** `nlp_scaling_method=equilibration-based` and `linear_system_scaling=mc19` stay unavailable (B1 §1.1). Only HSL-free scaling remains (§1.2).
   - **POUNCE's `ma57` linear solver (`pounce-hsl`) is out.** FERAL is POUNCE's only KKT factorization.
   - **Uno's interior-point presets must use SPRAL or MUMPS** (§1.1.3).
   - **IDAES iteration-path parity cannot be matched.** IDAES 2.13's `ipopt_v2` defaults to `linear_solver=ma57` (`.claude/skills/pyomo-and-solvers/content/sources/idaes-pse-2.13.0/idaes/config.py`, around lines 367–370). The legacy `ipopt` config sets no linear solver, so idaes-ext's Ipopt falls back to its MA27 default. This corrects B5a, which said only "MA27". It is *Interface-checked* for the config. Parity comparisons can be solution-level only, never iteration-level.
2. **Parallel execution is in scope.** The B1–B5 README listed "parallelism that comes from code architecture is out of scope", so B1–B5 did not inventory the libraries' concurrency contracts. §1.10 adds that inventory. It contains two contracts that bind any design (*Interface-checked*):
   - **Ipopt serializes every MUMPS call process-wide.** `static std::mutex mumps_call_mutex` guards init, factor and solve (`IPOPT/Algorithm/LinearSolvers/IpMumpsSolverInterface.cpp`, around lines 46–52, 106, 240, 414, 479, 574 and 671). Ipopt's ChangeLog says calls into MUMPS "are now protected by a mutex … [to prevent] calling MUMPS concurrently from several threads". In one process, concurrent Ipopt+MUMPS solves therefore queue on their linear algebra.
   - **The pinned PETSc build is not thread-safe.** It has no `--with-threadsafety` (`Cargo.toml` `[workspace.metadata.pse.petsc]` options; `PETSC/config/PETSc/options/libraryOptions.py:18,41-48`). PETSc's manual says "standard PETSc is not thread-safe" (`PETSC/doc/manual/getting_started.md:612-620`). The adapter enforces a process-wide PETSc admission mutex (`BN/petsc.rs:64-82`). NASM and field-split subdomains in this MPIUNI build therefore run serially in one process.

---

## 1. Capability survey at the pinned versions (neutral; deltas over B1–B4)

Each family gives what it does, its contract, its limits, newer releases, and the responsibility it could serve in a process simulator. Where B1–B4 already hold the inventory, only the pointer and the delta appear here.

### 1.1 Smooth NLP: interior point, SQP and penalty methods

#### 1.1.1 Ipopt 3.14.20 (C++ bridge `crates/pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`)

- **Does:** see B1 §1.1–1.4: filter line-search IPM, restoration, adaptive or monotone μ, exact or L-BFGS Hessian, warm start, derivative checker.
- **Contract (delta):**
  - Ipopt is thread-safe "as long as Ipopt objects (esp. SmartPtrs) are not shared between threads and a threadsafe linear solver is used" (`IPOPT/../ChangeLog.md:855-860`).
  - MUMPS is made safe by global serialization (§0.3).
  - SPRAL is OpenMP and requires `OMP_CANCELLATION=TRUE` (B1 §1.2).
  - MKL Pardiso honours per-thread `MKL_Set_Num_Threads_Local`, which the project already scopes (`BN/mkl.rs` `Threads::enter`).
- **HSL-free robustness levers not in B1** (*Interface-checked*):
  - `neg_curv_test_tol`/`neg_curv_test_reg` (`IPOPT/Algorithm/IpPDFullSpaceSolver.cpp:81-115`): the Zavala–Chiang inertia-free curvature test. It accepts a direction with wrong inertia when the curvature test passes, so it reduces reliance on exact inertia, which matters when MKL Pardiso's inertia is unreliable under perturbed pivots.
  - `dependency_detector=mumps` (`IPOPT/Interfaces/IpTNLPAdapter.cpp:125-140, 265-305`): detects linearly dependent equality constraints with MUMPS. Upstream labels it "experimental and does not work well", but it is the only HSL-free in-Ipopt rank signal.
  - MUMPS options: `mumps_pivtol`/`pivtolmax`, `mumps_scaling`, `mumps_permuting_scaling`, `mumps_pivot_order`, `mumps_mem_percent`, `mumps_dep_tol`.
  - SPRAL options: `spral_scaling` ∈ {none, mc64, auction, matching (default), ruiz (MC77 norm equilibration), dynamic}, plus switch rules, `spral_order`, `spral_pivot_method`, `spral_u`/`umax`, `spral_small`, `spral_nemin`, `spral_ignore_numa`.
  - Pardiso options: `pardisomkl_matching_strategy`, `pardisomkl_max_iterative_refinement_steps`, `pardisomkl_order`, `pardisomkl_redo_symbolic_fact_only_if_inertia_wrong`, `pardisomkl_repeated_perturbation_means_singular`, `pardisomkl_skip_inertia_check`.
- **Limits:**
  - Options are read only at `Initialize` (B1 §1.7).
  - Linear-solver statistics reach the application only through the journal (B1 §1.4).
  - sIPOPT is not built: `docker/solvers/build.sh` `build_ipopt` has no `--enable-sipopt`.
  - Ipopt has no internal parallelism other than its linear solver's.
- **Newer:** none (3.14.20 is the newest tag).
- **Serves:** large sparse nonconvex NLP (optimization, parameter estimation as NLP, simultaneous dynamic optimization), and reference-practice comparisons.

#### 1.1.2 POUNCE fork `ff6386944` (0.12.0) with FERAL 0.18.0

- **Does:** see B1 §2. Deltas found here (*Interface-checked*):
  - **Batched NLP:** `solve_nlp_batch`, `solve_nlp_batch_parallel`, `solve_nlp_batch_warm` and `solve_nlp_batch_parallel_warm` (`POUNCE/pounce-algorithm/src/batch.rs:615-709`). They use `install_serial_feral_backend` and `install_pooled_serial_feral_backend` (`:193, :380`), which is outer-parallel and inner-serial on rayon.
  - **Batched convex QP:** `solve_qp_batch`, `solve_qp_batch_parallel[_warm]` and `solve_qp_multi_rhs[_parallel]` (`POUNCE/pounce-convex/src/batch.rs:77-188`). The module docs state the contract:
    - outer-parallel with inner-serial FERAL, because nesting oversubscribes cores;
    - `time_limit` is **per instance**, not per batch;
    - cancellation order depends on rayon scheduling (`:20-56`).
  - **FERAL parallelism is per backend, not global.** It is controlled by `FeralConfig.parallel: Option<bool>` and a `min_par_flops` gate, which defaults to 10⁸ flops (`POUNCE/pounce-feral/src/lib.rs:540-565`). FERAL builds its own sized `rayon::ThreadPool` and "never [uses] the global rayon pool" (`vendor/feral/src/numeric/solver.rs:695-710`).
  - **Reusable sessions:** `TnlpPresolveSession` (`POUNCE/pounce-rs/src/session.rs:377-510`) provides `solve_cold`, `solve_warm`, `solve_warm_last`, `invalidate`, `enable_iter_history` and typed option setters. It keeps the presolve transform across re-solves.
  - **Other method modules:** QP homotopy (`POUNCE/pounce-qp/src/homotopy.rs`), the sensitivity Schur stack (`pounce-sens-core/src/{schur_driver,reduced_hessian,step_calc}.rs`), and the convex arm's crossover, simplex, active set, HSDE IPM and SOS (`pounce-convex/src/`).
- **Contract:** pure Rust, so the solver's internal state is reachable directly. A cooperative stop keeps the result (B1 §2.7).
- **Limits:**
  - Young upstream with fast cadence: 0.10 to 0.12 in six weeks.
  - The pinned code is a fork.
  - The sensitivity *crate* `pounce-sensitivity` (sIPOPT analogue) is not pinned; only `pounce-sens-core` is.
- **Newer:** none.
- **Serves:**
  - NLP with in-process observation and structured statistics;
  - many structurally identical NLPs or QPs, where a sweep fits batch form;
  - parametric sensitivity and predictor steps;
  - KKT conditioning and inertia (FERAL).

#### 1.1.3 Uno `805a27ec`

- **Does:** see B1 §3.
- **Delta under "no HSL"** (*Interface-checked* in the pinned source, extracted to scratch): `CMakeLists.txt` compiles `MUMPSSolver.cpp` when `find_package(MUMPS)` finds the **upstream** MUMPS library layout (`libdmumps`, `libmumps_common`, `libpord`, `libseq`; around lines 229–265). It compiles `SSIDSSolver.cpp` when `find_library(spral)` succeeds (around lines 267–285).
  - The solver image builds MUMPS through coin-or ThirdParty-Mumps as `libcoinmumps` (`docker/solvers/build.sh` `build_mumps`), which does not match that layout.
  - The image's `libspral` would satisfy the SPRAL probe.
  - So the HSL-free route to Uno's `ipopt` preset (interior point with primal-dual inertia correction) is a Uno rebuild against the image's SPRAL. *Proposed*, unverified.
- **Unchanged from B1:** the bridge allows only `filtersqp`/L-BFGS and `filterslp`, binds no Hessian, refuses re-solves, and stops by throwing.
- **Serves:** a second NLP engine with genuinely different globalization (trust-region filter or funnel SQP), and a "second opinion" of a different method family.

### 1.2 Sparse symmetric-indefinite KKT factorization without HSL

| Component (pinned) | Where it can run | Inertia | Threading contract | Structured statistics reachable | Notes |
|---|---|---|---|---|---|
| MUMPS 5.9.1, sequential with METIS (image) | inside Ipopt; SCIP's nested Ipopt | yes, INFOG(12) | **one process-wide mutex inside Ipopt** (§0.3); `libseq`, no MPI; its BLAS is MKL with threads | none (journal only; B2 §4) | robust, usually slower; `dependency_detector=mumps` |
| SPRAL SSIDS 2025.09.18 (OpenMP, METIS, hwloc; CPU) | inside Ipopt; Uno if rebuilt | yes | OpenMP team per calling thread; `OMP_CANCELLATION`/`OMP_PROC_BIND` required | none from Ipopt | its own scaling (`ruiz`, `mc64`, `matching`) partly stands in for MC19 |
| oneMKL Pardiso | inside Ipopt; Clarabel (`clarabel-pardiso`) | yes, less reliable under perturbation | MKL threads, per-thread local setting | none from Ipopt; Clarabel exposes none | matching strategy and iterative refinement |
| FERAL 0.18.0 (Rust) | POUNCE; direct `feral` (conditioning and certified inertia) | yes | per-backend rayon pool; serial mode | **full `FactorStats`** (B2 §4): fill, pivots, `n_tiny`, ordering, scaling, κ₁ estimate, refinement | METIS, AMD, SCOTCH, KaHIP orderings; MC64-symmetric scaling; static pivoting |
| QDLDL (Clarabel) | Clarabel | quasidefinite | serial | none (B2 §4) | needs a quasidefinite KKT |
| faer 0.24.4 sparse LDLᵀ/LBLᵀ/LU | project code | derivable from LDLᵀ | `Par` | symbolic sizes only (B2 §4) | already used for Newton/KKT pieces; Clarabel's faer backend would pull faer 0.21.9, a second faer (`crates/pse-backend-native/Cargo.toml:38`) |
| PETSc native LU and Cholesky | PETSc KSP/PC | `MatGetInertia` (Cholesky) | process mutex (§0.3) | `MatGetInfo`, zero-pivot row | no MUMPS or SuperLU in this PETSc build (B2 §2) |

**HSL-free scaling inventory** (*Interface-checked*):
- Ipopt: `nlp_scaling_method` ∈ {`gradient-based`, `user-scaling`}; `linear_system_scaling=slack-based`.
- Linear-solver-internal scaling: SPRAL `ruiz` (MC77-type equilibration) or `mc64`; MUMPS `mumps_scaling`/`permuting_scaling`; Pardiso matching.
- POUNCE `nlp_scaling_method=curvature-based` (B1 §2.2); FERAL MC64-symmetric and inf-norm.
- Clarabel Ruiz equilibration.
- No HSL-free substitute for MC19 *NLP-level* equilibration exists inside Ipopt. User-scaling from a project-computed equilibration is the library-compatible route. *Interpretation.*

### 1.3 Square nonlinear systems and fixed points

- **KINSOL (SUNDIALS 7.1.1):** see B2 §1.
  - **Delta: thread contract.** "Thread-safe provided that each thread has its own context object" (`SUN/vendor/include/sundials/sundials_context.h:15-16`). Separate KINSOL, IDAS or CVODES instances on separate threads, each with its own `SUNContext`, are a library-supported concurrency shape.
  - **Setter surface in 7.1.1** (`SUN/vendor/include/kinsol/kinsol.h:103-149`) also includes `KINSetReturnNewest` (fixed point), `KINSetMaxSubSetupCalls`, `KINSetResMonParams` and `KINSetRelErrFunc`. These are unbound per B2.
  - KINSOL's internal difference-quotient Jv is used when no `jtimes` is supplied. Dense and band difference-quotient Jacobians are available.
  - Fixed-point damping is `KINSetDamping`/`KINSetDampingAA` (constant). The per-iteration `KINSetDampingFn`/`KINSetDepthFn` need SUNDIALS 7.3+, and `sundials-sys` 0.6.2 vendors 7.1.1 (B2 §1.6).
- **PETSc 3.24 SNES:** see B2 §2. Every type exists in the pinned header (`PETSC/include/petscsnes.h:25-46`): NEWTONLS, NEWTONTR, NEWTONTRDC, NEWTONAL, QN, NGMRES, ANDERSON, NRICHARDSON, NGS, NCG, FAS, NASM, ASPIN, COMPOSITE, PATCH, MS, VINEWTONRSLS/SSLS, KSPONLY. The line searches are bt, secant, cp, nleqerr, basic/none, bisection, ncglinear and shell (`:730-739`).
  - **Composition APIs:** `SNESSetNPC` (`:948`); `SNESCompositeSetType`/`AddSNES` (`:1235-1236`); `SNESNASMSetSubdomains`/`SetType`/`SetDamping`/`GetSNES` (`:1203-1210`); `SNESNewtonALSetFunction`/`GetLoadParameter`/`SetCorrectionType` (`:1327-1356`).
  - **Observation:** `SNESMonitorSet` (`:61`), `SNESSetConvergenceHistory`/`Get…` (`:67-68`), `SNESConvergedReasonViewSet` (`:89`).
  - **Finite differences:** `-snes_mf`, `-snes_mf_operator`, and `MatFDColoring` for colored FD Jacobians (serial here).
- **diffsol-nl 0.1.1:** see B2 §3. It has an in-library mean-rate contraction estimator. Register R-47 excludes it as the nested inner solver.
- **argmin 0.11.0:** `BrentRoot`, `BrentOpt`, `GoldenSectionSearch`, `Newton`, `GaussNewton`(LS), `TrustRegion` (Cauchy/Dogleg/Steihaug), quasi-Newton, and the line searches `Backtracking`, `MoreThuente` and `HagerZhang` (`~/.cargo/registry/src/*/argmin-0.11.0/src/solver/`). Its `Executor` has observers and termination hooks. There is no Levenberg–Marquardt (B5a §2). Its argmin-math faer backend stops at faer 0.23.
- **Serves:** per-block and whole-flowsheet square solves, tear fixed points, bounded square systems (VI), arclength continuation.

### 1.4 Least squares and bounded minimization

- **PETSc TAO 3.24** (`PETSC/include/petsctao.h:205-238`):
  - least squares: BRGN (Gauss–Newton with regularizers including LM; B2 §2.1), POUNDERS (derivative-free);
  - bounded minimization: BNLS, BNTR, BNTL, BQNLS, BQNK*, BLMVM, TRON, GPCG, BQPIP;
  - general constrained: ALMM (augmented Lagrangian), PDIPM (primal-dual IPM), IPM;
  - complementarity: SSILS, SSFLS, ASILS, ASFLS;
  - TAOSNES and TAOSHELL.
  - Observation: `TaoMonitorSet`, `TaoSetConvergenceHistory`, `TaoGetSolutionStatus`.
  - Unbound locally (`pse_petsc.c` has no `tao`, B5a).
- **Contract:** serial in this build, behind the process mutex.
- **Limits:**
  - BRGN's subsolver choice decides bound handling.
  - PDIPM and ALMM are not filter IPMs. I did not verify their maturity for nonconvex process NLP.
- **Serves:** parameter estimation and data reconciliation (bounded LM and Gauss–Newton), derivative-free calibration (POUNDERS).

### 1.5 Composition and decomposition (method-level structure)

- **PETSc SNES/PC composition:**
  - nonlinear preconditioning (left or right NPC);
  - additive or multiplicative COMPOSITE;
  - NASM and ASPIN over user subdomains (scatters from BTF/SCC index sets);
  - FAS;
  - linear PCFIELDSPLIT with Schur factorization variants and saddle-point detection (`PETSC/include/petscpc.h:303-326`);
  - ASM;
  - KSP `ComputeExtremeSingularValues`.
  - All of it is serial in MPIUNI without threadsafety: subdomain parallelism in PETSc comes from MPI ranks.
- **POUNCE KKT:**
  - Schur augmented, low-rank augmented (SMW) and slack-scaled KKT variants (B1 §2.1);
  - the sens-core Schur driver;
  - `solve_many_cached` multi-RHS backsolves (`POUNCE/pounce-algorithm/src/kkt/pd_full_space_solver.rs:999,1248`).
- **SUNDIALS:**
  - per-instance `SUNContext` threading (§1.3), so multiple-shooting segments can run concurrently;
  - an OpenMP `NVector` exists behind a `sundials-sys` feature (`SUN/wrapper.h:16`), but is not enabled.
- **SCIP 10.0.2:** concurrent racing solve; the image builds it `THREADSAFE=ON`, `TPI=tny` (`docker/solvers/build.sh` SCIP cmake around lines 474–486).
- **Serves:** block-structured flowsheets, KKT systems with arrow or block-angular structure, multi-period design, shooting.

### 1.6 Continuation, path following and parametric sensitivity

- **PETSc:** SNESNEWTONAL (arclength with load parameter) and TSPSEUDO (used locally).
- **POUNCE:** `step_along_path`, `path_direction`, fix-relax (B1 §2.1), QP homotopy, `final_mu` threading for path following.
- **Ipopt:** warm start plus `ReOptimizeTNLP`. sIPOPT is not built.
- **Serves:** studies, sweeps, homotopy initialization, NMPC-like sequences.

### 1.7 Dynamics

See B3 for IDAS, Diffsol, CVODES/ARKODE and PETSc TS. Deltas:

- **Thread contract:** SUNDIALS per-`SUNContext` (§1.3).
- **PETSc TS:** behind the process mutex.
- **diffsol 0.16.2 features** (*Interface-checked*, `diffsol-0.16.2/Cargo.toml` `[features]`):
  - `diffsl` (DSL compiled by Cranelift or LLVM 15–18);
  - `diffsl-external[-dynamic|-f64]`, where the model functions come from externally compiled symbols or a dynamic library;
  - `cuda`.
  - None is enabled (`default-features = false, features = ["faer"]`).
  - `diffsl-external-*` is an interface through which externally generated code, such as evaluator code generated elsewhere, can drive Diffsol. *Interpretation*; it belongs with R2.
- **Serves:** dynamics from the same definitions; forward and adjoint sensitivities for fitting and dynamic optimization; shooting.

### 1.8 LP, MILP, QP, conic and MINLP

See B4. Deltas:

- **HiGHS 1.15 executor.** `Highs::initializeMultiThreading` calls `initialize_scheduler(options.threads)`. A different `threads` value after first initialization is a `kError`, "can be destroyed by calling `Highs::resetGlobalScheduler()`" (`HIGHS/lp_data/Highs.cpp:5111-5132`). The executor handle is `static thread_local` (`HIGHS/parallel/HighsTaskExecutor.h:40-48`), so each calling OS thread owns one executor with its own worker threads. The default thread count is (hardware_concurrency+1)/2 (`HIGHS/parallel/HighsParallel.h:20-29`).
  - The project already serializes resets across direct HiGHS and Uno's HiGHS subproblems (`BN/highs_lifecycle.rs`, `Highs_resetGlobalScheduler(1)` at around line 110). That is project machinery imposed by this library contract.
- **Clarabel 0.11.1:**
  - `max_threads` is consumed by MKL Pardiso, and by the unused faer LDL backend (`Par::rayon`) (`clarabel-0.11.1/src/.../ldlsolvers/{pardiso.rs:199, faer_ldl.rs:100-103}`).
  - It has `presolve_enable`, `chordal_decomposition_*`, `iterative_refinement_*` and static and dynamic regularization settings.
  - It has no warm start (B4).
- **SCIP 10.0.2:**
  - concurrent mode exists, used deterministically by the adapter;
  - reoptimization covers objective changes only;
  - the structured `SCIPcollect*Statistics` and `SCIPwriteOrigProblem` are unbound (B4 §3.4);
  - 10.1.0 adds solving-phase queries (B4 §3.6).

### 1.9 Auxiliary

- **egobox 0.38.1:** `q_batch` and `qei_strategy` (q-EI batch proposals), `timeout`, `warm_start`/`hot_start` (file-based, unused because of `no_outdir()`), TREGO and CoEGO (`egobox-ego-0.38.1/src/solver/egor_config.rs:207-733`). Batch proposals are the library's own case-level parallelism hook. *Interface-checked.*
- **IBEX 2.9.1:** see B4 §4. A process-global lock; no session.

### 1.10 Concurrency and threading contracts (new inventory)

| Library | Unit of safe concurrency | Internal parallelism control | Constraint for case-level parallelism in one process |
|---|---|---|---|
| Ipopt 3.14.20 | one `IpoptApplication` per thread | via the linear solver | MUMPS: global mutex. SPRAL: an OpenMP team per caller (ICV per thread). Pardiso: per-thread MKL threads. |
| MUMPS 5.9.1 seq | serialized by Ipopt | none (BLAS threads) | queues |
| POUNCE/FERAL | one application per thread; batch APIs | `FeralConfig.parallel`, `min_par_flops`, per-solver pool | outer × inner nesting oversubscribes; the library documents outer-parallel/inner-serial |
| Uno | per instance, but its HiGHS subproblems share the HiGHS executor rules | `USE_OPENMP=OFF` in the pinned build | goes through the project HiGHS lifecycle lock |
| PETSc 3.24 (MPIUNI, no threadsafety) | **one per process** | none (no OpenMP, no MPI) | serialize, or use separate processes, or rebuild `--with-threadsafety`, which then allows one object set per thread |
| SUNDIALS 7.1.1 | one `SUNContext` per thread | OpenMP NVector (not enabled); KLU serial | safe with a context per thread |
| Diffsol 0.16.2 | per solver instance (`Send` bounds not checked) | faer `Par` in its linear algebra | — |
| HiGHS 1.15 | one `Highs` per thread; executor per calling thread | `threads`, `parallel`, `mip_*`; reset is global per thread | the thread count is fixed per thread until reset |
| SCIP 10.0.2 (THREADSAFE, TPI=tny) | one SCIP per thread | concurrent solve (`parallel/mode`); `lp/threads` | fine |
| Clarabel | per solver | Pardiso threads via MKL | fine |
| IBEX | process-global lock | none | serial |

*Interface-checked*, except the Diffsol `Send` bounds, which were not inspected.

### 1.11 Observation and cooperative stop (delta: the full table under one frame)

| Library | Per-iteration hook | Cooperative stop that keeps iterate and statistics | Current local stop |
|---|---|---|---|
| Ipopt | `intermediate_callback` + `IpoptData`/`IpoptCq` (B1 §1.4) | return false: `User_Requested_Stop`, `finalize_solution` runs, `Statistics()` exists | throws, so the iterate and statistics are lost (B1 §1.7) |
| POUNCE | intermediate callback, plus Rust state | return false: result kept | returns false on evaluation failure only |
| Uno | termination callback with residuals and multipliers; notify-acceptable | return 0: `USER_TERMINATION` with the iterate | throws |
| KINSOL | none in 7.1.1 (logging compiled out). Fixed point: the map callback runs once per iteration | none. A negative callback return is an unrecoverable failure. What `uu` holds on that path was not verified. | callback latch |
| PETSc SNES/TS/TAO | `SNESMonitorSet`, convergence test, `SNESSetConvergenceHistory`; `TSSetPostStep`; `TaoMonitorSet` | set a user converged or diverged reason in the test or monitor; the iterate is in the solution Vec | convergence test → `SNES_DIVERGED_USER` |
| IDAS | `IDA_ONE_STEP` loop (unused) | return between steps; the state is in `yy`/`yp` | `IDA_NORMAL` with callback latching |
| Diffsol | caller-driven `step()` | stop between steps | already step-driven (B3 §2.3) |
| HiGHS | callbacks 1–7 | `user_interrupt`: interrupt status; solution and basis retained (simplex/IPM; not QPASM/PDLP) | used |
| SCIP | events | `SCIPinterruptSolve`; `SCIPsolve` can resume | used |
| Clarabel | termination callback | `CallbackTerminated` with the current iterate | used |
| argmin | `Observe` and termination hooks | `TerminationReason` | (surrogate loop only) |

---

## 2. The six uses (deltas plus this brief's framing)

### 2.1 Selection and diagnosis signals

The B1–B4 inventories stand. Additions:

- **Under no HSL, FERAL is the only KKT factorization with typed per-factorization statistics.** These are inertia, `n_tiny`, pivot extrema, fill, ordering and scaling, κ₁ and the refinement trajectory. MUMPS, SPRAL and Pardiso inside Ipopt surface only journal text, plus δw and `info_string` (B1 §1.4, B2 §4).
- **Ipopt rank signals without HSL:**
  - `dependency_detector=mumps` (experimental);
  - the inertia-free `neg_curv_test_*` path, which changes what δw means: with it on, regularization is no longer a pure inertia signal. *Interpretation.*
- **PETSc's library-owned per-iteration records:**
  - `SNESSetConvergenceHistory` (residual norms and linear iterations per iteration, with no callback);
  - `KSPComputeExtremeSingularValues` (a cheap condition estimate);
  - `MatFactorGetErrorZeroPivot`.
- **KLU `rcond`/`condest`** are computed for free in both KLU paths; they are exported in neither (§0.1).
- **SCIP 10.1 solving-phase flags and HiGHS solve-time and IPM-variant identity** need a bump or binding (B4).

### 2.2 Reuse across study points, recycles and re-solves

The B1–B4 hooks stand. Additions:

- POUNCE `TnlpPresolveSession` (presolve kept, warm or last re-solve) and the batch `*_warm` variants.
- FERAL pattern reuse.
- Ipopt `ReOptimizeTNLP` (used). `warm_start_entire_iterate` and `warm_start_target_mu` are unused.
- KINSOL `replace()` with `NoInitSetup` (used).
- IDAS `IDAReInit` keeps KLU symbolic but resets order and forces setup (B3 §1.3).
- HiGHS basis: cleared before every solve (B4 §1.3).
- Clarabel `update_*` (used).
- SCIP reoptimization (objective changes only).
- egobox in-run state.

**Library contract that matters for parallel reuse.** Retained native state is per instance and per thread for every library in §1.10. "Session retention" therefore implies thread affinity, and for PETSc it implies process affinity. *Interpretation.*

### 2.3 Per-block methods for BTF/SCC sequences

| Block shape | Library-owned options at the pins | Gaps |
|---|---|---|
| 1×1, closed form | none needed (or symbolic; see R2) | — |
| 1×1, bracketed | argmin `BrentRoot`; Symbolica certified cubic; IBEX enclosure (certification) | no TOMS748 or Illinois at the pins (B5a §5) |
| small dense square | KINSOL dense; PETSc NEWTONLS/TR/NLEQERR (per-call object setup behind the process mutex); diffsol-nl (R-47); argmin `Newton`/`TrustRegion` (dense) | many tiny blocks pay per-object setup in PETSc and KINSOL. No library offers "many tiny dense Newton solves" in batch. |
| one-sided bounds | KINSOL sign constraints (used) | — |
| two-sided boxes | PETSc VINEWTONRSLS/SSLS (unbound) | — |
| near-singular | PETSc NEWTONTR dogleg, NLEQERR; FERAL κ₁ and `n_tiny` | no rank-revealing sparse QR at the pins (B5a: SPQR) |
| large SCC | KINSOL with KLU or Krylov plus a BTF FERAL preconditioner (used); PETSc NASM/ASPIN (serial) | — |
| least squares | TAO BRGN-LM, POUNDERS (unbound); NLP routes | — |

**Reference practice** (IDAES 2.13, behaviour only). `BlockTriangularizationInitializer` checks for a perfect matching first. It solves 1×1 blocks with Pyomo `calculate_variable_from_constraint`: an exact solve if the residual is linear, otherwise Newton with a line search and `alpha_min` 1e-8. Larger blocks go to a block solver, typically Ipopt. A final full solve follows because SCC solving ignores scaling (Pyomo issue #3785) (`idaes/core/initialization/block_triangularization.py:95-140, 181-205`; `pyomo/contrib/incidence_analysis/scc_solver.py:92-121`; `pyomo/util/calc_var_value.py:32-71`). *Interface-checked.*

### 2.4 Decomposition and parallel execution

**Case-level.** Library-supported shapes:

- POUNCE `solve_nlp_batch_parallel[_warm]` and `solve_qp_batch_parallel[_warm]` (rayon, inner-serial FERAL, per-instance time limits);
- SUNDIALS instances with a context per thread;
- HiGHS and SCIP instances per thread;
- Ipopt with SPRAL or Pardiso per thread.

Two of these queue in one process:

- **Ipopt with MUMPS**, through Ipopt's MUMPS mutex;
- **every PETSc-backed method**, through the non-thread-safe build and the project's process mutex.

**Structure-level.**

- Multiple-shooting segments: library-supported concurrency (IDAS per context).
- PETSc NASM, ASPIN and field split: compositions are available, but serial here. MPI would be a process topology, not a thread one.
- POUNCE: Schur KKT and multi-RHS backsolves.
- SCIP: concurrent racing of configurations (deterministic or opportunistic).

**Library-internal controls:** HiGHS `threads`/`parallel`; SCIP `parallel/*`; FERAL `parallel`/`min_par_flops`; SPRAL OpenMP; MKL Pardiso threads; MUMPS none; Clarabel `max_threads`; SUNDIALS OpenMP NVector (not enabled); faer `Par`.

**Finite-difference column parallelism.**
- *Coloring* is library-owned: PETSc `MatFDColoring`/`-snes_fd_color`, POUNCE finite-difference Hessian with CPR/star coloring, Ipopt `jacobian_approximation=finite-difference-values`, KINSOL and IDAS difference-quotient Jv.
- None runs color groups in parallel in this build.
- Parallel FD would be project-owned evaluation over library-computed colorings. *Interpretation.* The `finitediff` crate was not inspected for this.

### 2.5 Cooperative policy stop

§1.11 gives the library-owned stop for each library.

- **Ipopt and Uno:** the bridges' exception route discards what the libraries would retain (B1 §1.7, §3.7).
- **KINSOL:** no cooperative stop exists in 7.1.1.
- **PETSc SNES/TS/TAO:** user converged reasons give a clean stop.
- **IDAS:** needs `IDA_ONE_STEP`.
- **Diffsol, HiGHS, SCIP, Clarabel:** already cooperative.

### 2.6 The KKT linear-solver story without HSL

- **Available at the pins:**
  - inside Ipopt: MUMPS (serialized), SPRAL (OpenMP, its own scaling), MKL Pardiso (MKL threads);
  - FERAL under POUNCE and directly (typed statistics, per-solver pools);
  - QDLDL or Pardiso inside Clarabel;
  - faer for project-owned factorizations.
- **Lost without HSL:**
  - MC19 equilibration in Ipopt;
  - MA57 in POUNCE;
  - IDAES iteration-path parity (MA57 or MA27);
  - the small-KKT, low-overhead regime that MA27/MA57 serve (B5a §1, literature).
- **Compensating library levers:**
  - SPRAL `ruiz`/`mc64` scaling;
  - MUMPS permuting scaling;
  - Ipopt `neg_curv_test_*` when inertia is unreliable;
  - FERAL METIS ordering for collocation structure (B1 §2.2);
  - a Uno rebuild against SPRAL for an HSL-free IPM preset.
- **Unbuilt routes worth naming:**
  - MUMPS inside PETSc (`--with-mumps-serial`), for PETSc Cholesky/LU with `MatGetInertia` and the MUMPS getters;
  - SuiteSparse SPQR (B5a) for rank.
  - Both are *Proposed*; neither was examined further.

---

## 3. Reference practice (IDAES 2.13, behaviour only)

B5b holds the full reference. Additions relevant to R4:

- Default solvers are Ipopt with HSL (§0.3).
- `nlp_scaling_method=gradient-based`, `tol` 1e-6 and `max_iter` 200 are the `ipopt_v2` defaults (`idaes/config.py:340-370`).
- Writer-level model scaling is on.
- The BT initializer is as described in §2.3.
- `IpoptConvergenceAnalysis` (B5b §1.2) records iterations, restoration iterations, regularized iterations and time per sweep point. Every one of these is available from Ipopt's statistics, callback and `IpoptData` (B1 §1.4).

---

## 4. Current use (pointers only)

- **Seam:** `BN/execution.rs` (one adapter per `Backend`, a capability record, settings admission, native session lifecycle, warm payload); `BN/execution/*.rs` (ipopt, pounce, pounce_convex, uno, petsc, kinsol, dynamics, highs, clarabel, scip, factorable, sos); `BN/routing.rs` (class routing; automatic preference orders eligible adapters).
- **NLP:** `crates/pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`, `BN/ipopt/{sequence,settings,runtime}.rs`; `BN/pounce.rs`, `BN/pounce/{profiles,retained,observed,records,equalities}.rs`, `BN/tnlp.rs`; `crates/pse-uno-sys/bridge/pse_uno.cpp`, `BN/uno.rs`, `BN/uno/`.
- **Roots, recycle and continuation:** `BN/kinsol.rs`, `BN/kinsol/block.rs`, `BN/recycle.rs`, `BN/tears.rs`, `BN/square_response.rs` and `BN/square_response/`, `BN/petsc.rs`, `BN/petsc/{blocks,flow,derived}.rs`, `crates/pse-petsc-sys/bridge/pse_petsc.c`, `BN/kkt/{path,sensitivity,advance,activity}.rs`, `crates/pse-math/src/continuation.rs`.
- **Dynamics:** `BN/dynamics.rs`, `BN/dynamics/{idas,linear,integrator,anchored}.rs`.
- **LP, conic, MINLP:** `BN/highs.rs`, `BN/highs/diagnostics.rs`, `BN/highs_lifecycle.rs`, `BN/conic.rs`, `BN/scip.rs`, `BN/root_isolation.rs`.
- **Diagnostics:** `BN/conditioning.rs` (direct FERAL), `BN/jacobian_diagnostics.rs`, `BN/derivative_diagnostics.rs`.
- **Threads:** `BN/mkl.rs` (`Threads::enter`, OpenMP and MKL per worker); `BN/execution/{pounce,pounce_convex}.rs` (`scope_threads`, `with_threads`); `BN/execution/scip.rs:66-120`; PETSc, KINSOL and Uno refuse more than one thread (`BN/execution/{petsc.rs:77, kinsol.rs:263, uno.rs:74}`).
- **Strategy, solves and staged sessions:** `RT/math/strategy/{admission,target,path_control,accuracy_refinement}.rs`, `RT/math/solves.rs` and `RT/math/solves/`, `RT/math/staged.rs` (worker-owned session, one native thread per session, permits per step).
- **Surrogate:** `RT/math/surrogate/library.rs` (egobox, argmin).

---

## 5. Library-shaped allocations for method choice and execution

Each sketch assumes the three-layer selection requirement. Layer 1 (a priori rules) and layer 3 (advisory history) stay project-owned in every sketch, because no library offers cross-library method choice. What differs is how much of layer 2 (in-solve switching) and of execution the libraries own. All sketches are *Proposed*.

### A. PETSc as the composable method framework (SNES, TS, TAO, KSP/PC)

Square, least-squares, bounded and dynamic problems go through one PETSc object model, with composition expressed as SNES, TS and TAO types and their nesting. Ipopt or POUNCE are kept for general constrained NLP, and HiGHS, Clarabel and SCIP for their classes.

- **Library discharges:**
  - Newton, TR, QN/Broyden, NGMRES/Anderson, NLEQ-ERR, VI boxes and arclength;
  - in-solve composition and fallback through COMPOSITE (additive or multiplicative) and NPC, which is a library-owned form of layer-2 switching *within* one solve;
  - NASM and ASPIN subdomain solves;
  - field-split and Schur preconditioning;
  - bounded least squares (BRGN-LM, POUNDERS, BNTR, TRON);
  - TS integrators with `TSAdjoint` (RK/Theta only) and `TSEvent`;
  - colored FD Jacobians, JFNK and lagging;
  - uniform monitors, convergence history, typed converged reasons and `-log_view`-style profiling.
- **Remains the project's:**
  - building Vec/Mat callbacks from compiled evaluators;
  - deriving subdomain and field index sets from BTF/SCC structure;
  - the typed vocabulary over PETSc's string options database (the current refusal of raw options in `BN/petsc.rs:585-593` points the same way);
  - mapping reasons to typed outcomes;
  - consistent DAE initialization (TS has no `IDACalcIC`);
  - layers 1 and 3.
- **Lost or weakened:**
  - IDA's variable-order BDF and `IDACalcIC` maturity, and BDF adjoints (TSAdjoint covers RK and Theta only; B3 §4);
  - KINSOL sign constraints and its cheap fixed-point path;
  - **any thread parallelism in one process**, unless PETSc is rebuilt `--with-threadsafety` (shared libraries already on) for one object set per thread, or MPI is used for subdomain parallelism, which is a process topology;
  - per-object setup cost for many tiny blocks.
- **Integration, lifecycle, testing and upgrade cost:**
  - one C bridge grows substantially;
  - PETSc deprecates APIs on a roughly six-month cadence (3.25 and 3.26 renamed SNES reasons and deprecated `SNESLINESEARCHBASIC`; B2 §2.6);
  - the MPIUNI build stays.
- **Makes unnecessary** (responsibility level):
  - project-owned acceleration and escalation for square systems and tears where SNES composition covers them;
  - separate least-squares routing through NLP adapters;
  - parallel per-library observation vocabularies for the PETSc-covered classes.

### B. A Rust-native core: POUNCE/FERAL for optimization, SUNDIALS for roots and dynamics

POUNCE owns NLP, QP and convex solving, sensitivities, presolve sessions and batches, all on FERAL KKT factorizations. SUNDIALS (KINSOL, IDAS, optionally CVODES) owns roots and dynamics with a context per thread. HiGHS, Clarabel and SCIP keep their classes. Ipopt is kept only as a reference engine.

- **Library discharges:**
  - typed, structured statistics (POUNCE `SolveStatistics` with per-iteration `IterRecord`, FERAL `FactorStats`) with no journal parsing;
  - in-process access to internal state (`IpoptData`-equivalent) for layer-2 cues;
  - cooperative stop that keeps the result;
  - batched NLP and QP with outer-parallel/inner-serial pools and warm variants;
  - presolve reuse across re-solves;
  - sensitivity predictor steps;
  - second-opinion ladders and dual-divergence retries inside the solver;
  - safe concurrency with a SUNContext per thread.
- **Remains the project's:**
  - layers 1 and 3;
  - cross-library transitions (warm payload translation);
  - batch time budgets, since POUNCE's are per instance;
  - nested-pool policy between FERAL pools, rayon batch workers, MKL and OpenMP;
  - a KINSOL cooperative stop, which is absent in 7.1.1 (and `KINSetDampingFn`/`DepthFn` need a SUNDIALS 7.3+ build outside `sundials-sys` 0.6.2);
  - fork maintenance.
- **Lost or weakened:**
  - Ipopt's reference status and its MUMPS, SPRAL and Pardiso choices as the primary path;
  - maturity, since POUNCE is young and moves fast;
  - IDAES reference practice centres on Ipopt.
- **Cost:**
  - tracking the fork against a fast upstream;
  - FERAL 0.19 is already released and unreviewed;
  - testing per-instance budgets and cancellation order in batches, which upstream says depends on scheduling.
- **Makes unnecessary:**
  - journal or text observation routes for KKT statistics;
  - the Ipopt-MUMPS serialization problem for concurrent cases;
  - a separate project-owned batch scheduler for structurally identical NLP or QP points.

### C. Class-specific adapters behind one plan (the library-maximal form of today's shape)

Each class keeps its best-fit library:

- square: KINSOL, plus PETSc SNES for VI, TR, NLEQERR, QN and NASM;
- NLP: Ipopt, POUNCE and Uno;
- least squares: TAO;
- dynamics: IDAS and Diffsol;
- LP/MILP/QP: HiGHS;
- conic: Clarabel;
- MINLP: SCIP;
- scalar: argmin.

One project plan owns selection, transitions and observation normalization. Every library is driven through its own cooperative stop and warm or retained session.

- **Library discharges:** every method; its native statistics; warm start and reuse; native stop semantics; library-internal threading.
- **Remains the project's:**
  - all three selection layers;
  - one observation schema mapped from about ten statistics vocabularies (B1–B4 tables);
  - a cooperative-stop contract per library (§1.11);
  - warm-payload translation between solvers;
  - **per-library concurrency rules** (§1.10): the MUMPS mutex, the PETSc process mutex, HiGHS executor resets, OpenMP and MKL per-worker ICVs, FERAL pools and SCIP TPI. These rules are library-imposed obligations, not optional design.
- **Lost or weakened:** composition inside a single solve; switches happen only at stop-and-resolve boundaries (B1 §4). Every library keeps its own build, ABI and upgrade cadence.
- **Cost:**
  - N bridges, the highest of the three sketches;
  - per-library qualification;
  - pinned-native composition through the image and the cache (ADR-0108, ADR-0157).
- **Makes unnecessary:** project-owned numerical methods wherever a pinned library has one. B5a's "wire first" list (VI, BRGN-LM, QN, NLEQERR, NEWTONAL, BrentRoot) is that inventory.

**Shared observation across A–C.** In every sketch, layers 2 and 3 consume the same small set of native cues: residual norms, step acceptance, regularization or inertia, restoration, μ or trust-region radius, and factorization statistics. The libraries produce these at very different fidelity:

| Fidelity | Libraries |
|---|---|
| typed | POUNCE, FERAL, PETSc history, IDAS getters, HiGHS info, SCIP datatree |
| callback-only | Ipopt and Uno per-iteration data |
| text-only | MUMPS, SPRAL and Pardiso inside Ipopt |
| absent | KINSOL per-iteration data in 7.1.1 |

The fidelity of a library is a property of that library, independent of which allocation is chosen. *Interpretation.*

---

## 6. Search coverage, uncertainties and unfinished items

- **Not verified:**
  - Uno's SPRAL detection against the image's `libspral`: is the `SPRAL_INCLUDE_DIR` the CMake script expects provided?
  - whether `uu` holds the last iterate after a KINSOL unrecoverable callback return;
  - Diffsol `Send` bounds;
  - TAO PDIPM and ALMM maturity;
  - PETSc `--with-threadsafety` interactions with logging and the existing bridge;
  - `finitediff` coloring and parallelism.
- **PETSc TS adjoint scope and TSBDF order adaptivity** remain documentation-sourced (B3 uncertainty).
- **Search coverage for the concurrency facts:**
  - Ipopt MUMPS mutex: the full `IpMumpsSolverInterface.cpp` plus the ChangeLog.
  - PETSc threadsafety: `config/` and `doc/manual/getting_started.md`.
  - HiGHS executor: `parallel/HighsParallel.h`, `HighsTaskExecutor.h`, `lp_data/Highs.cpp:5110-5140`.
  - SUNDIALS: `sundials_context.h`.
  - SCIP: image build flags only, no header read.
- **Part B** (mapping the W-A/W-B obligations onto these libraries) is not started.

**Files written:** this file only. Scratch: the pinned Uno source archive was extracted read-only to `/tmp/claude-1000/-home-paul-pse-arrow/ec287db0-3345-4c95-b8a8-8582acffa694/scratchpad/r4-uno-src/` (from `~/.cache/pse-arrow/native/sources/55618cdb….tar.gz`, the archive whose sha256 matches the pin).
