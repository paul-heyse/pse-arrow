# Native solvers: capability-gap inventory (contributor return)

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Capability-gap inventory: native solver libraries. Its §0 and W5/C6 framing of vendored SUNDIALS changes as a fork was retracted under premise P4.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

I made no file edits and ran no builds, tests or services. Baseline: HEAD `0530dcaa1` (source `a791c533e`). I used the coordinator's census §4 and §7 for usage and checked anything consequential against source.

**Sources used:**
- Context7 `/llnl/sundials` (custom `SUNLinearSolver`).
- Skill corpus: Ipopt 3.14.20, Clarabel 0.11.1, FERAL 0.18.0.
- POUNCE fork checkout at `~/.cargo/git/checkouts/pounce-8316ac8e070882e5/7f5e2e9`.
- Pristine `sundials-sys-0.6.2` in the cargo registry, diffed against `vendor/sundials-sys`.
- Uno source at `805a27ec` (`~/.cache/pse-arrow-uno-805a27-source`).
- PETSc 3.24.0 source (`~/.cache/pse-arrow-petsc-source/petsc-3.24.0`).

**Evidence labels.** Source reading is *Interface-checked*. Recommendations are *Proposed*. Nothing here is *Tested* or *Measured*.

## 0. Census correction (resolves census "Unresolved 1")

The vendored SUNDIALS C sources **are modified**. That contradicts `vendor/native-supplier-notes.md:29` ("All upstream feature selections and numerical Rust/C sources are preserved").

`diff` against the pristine crate shows these changed files (counted in changed lines):

| File | Changed lines |
|---|---|
| `src/sunlinsol/klu/sunlinsol_klu.c` | 189 |
| `include/sunlinsol/sunlinsol_klu.h` | 43 |
| `src/idas/idas.c` | 131 |
| `src/idas/idaa.c` | 83 |
| `src/idas/idaa_io.c` | 22 |
| `src/idas/idas_io.c` | 17 |
| `include/idas/idas.h` | 20 |

`idas_ic.c` and `idas_impl.h` also differ.

The added API is:
- KLU: `SUNLinSol_KLUFixedPattern`, `KLUBeginApplication`, `KLUGetMemUsage`, `KLUGetWork`, `KLUSetObserver` (`vendor/sundials-sys/vendor/include/sunlinsol/sunlinsol_klu.h:140-163`).
- IDAS: `IDASetCheckpointLimit`, `IDAGetCheckpointUsage`, `IDASetAcceptedStepBudget`, `IDAGetAcceptedBudgetSteps`, `IDASetReplayStepLimit`, `IDAGetReplayNumSteps` (`include/idas/idas.h:459-478`).

This is the same pattern DA-018 criticises for Uno: project semantics carried inside a library fork. DA-018 names only Uno. §3-C6 below gives the upstream extension point that replaces part of it.

## 1. Capability surface at the pins (relevant topics)

| Topic | Library capability at pin (source) |
|---|---|
| Fixed-point / recycle maps | KINSOL has `KIN_FP` and `KIN_PICARD`, Anderson (`KINSetMAA`/`DampingAA`/`DelayAA`/`OrthAA`) and `KINSetDamping`. PETSc 3.24 has `SNESNRICHARDSON`, `SNESANDERSON` and `SNESNGMRES` (`include/petscsnes.h:29,34,42`). |
| Recovery / second opinion | `pounce_algorithm::second_opinion::{second_opinion_rungs, SecondOpinionAvailability, SecondOpinionTrigger}`. `pounce_rs::restoration::second_opinion_driver` (`pounce-rs/src/restoration.rs:68`). No equivalent ladder exists in Ipopt, Uno or KINSOL. |
| Continuation correctors | `pounce_sens_core::boundcheck::{step_along_path, refine_step_onto_bounds, path_direction}` (`pounce-sens-core/src/boundcheck.rs:786,1275,1974`). `pounce-sensitivity` (feature `sensitivity`, off) adds `corrector::CorrectorReport`: Newton iterations against the held IPM factor. |
| Schur separators | POUNCE consumes `NlpOracle::solve_separator` (`pse-backend-native/src/lib.rs:566-568`). `pounce_sens_core::{schur_driver, schur_data}`. |
| Factor reuse: KINSOL / KLU | `KINSetNoInitSetup`, `KINSetMaxSetupCalls`. Upstream `SUNLinSolSetup_KLU` keeps symbolic analysis **and** pivot order (`klu_refactor` with an rcond fallback). `SUNLinSol_KLUReInit` frees both. Pristine `sunlinsol_klu.c:145-170,240-330`. |
| Factor reuse: Ipopt | `ReOptimizeTNLP` plus `warm_start_same_structure`. In 3.14.20 the "same structure" path skips `GetSpaces` and `DetermineScaling` (`IpOrigIpoptNLP.cpp:239-298`). It **does not** keep linear-solver symbolic analysis: `TSymLinearSolver` resets `initialized_ = false` unconditionally (`IpTSymLinearSolver.cpp:129-131`), so `InitializeStructure` reruns. |
| Factor reuse: POUNCE / FERAL | `reset_for_new_application` and neutral FERAL symbolics. `pounce_presolve::warm::{PresolveFingerprint, PresolveMap, project_warm_point}` (`pounce-presolve/src/warm.rs`). `pounce_rs::session::TnlpPresolveSession` (`session.rs:396-510`). |
| Batch: POUNCE | `pounce_algorithm::batch::{solve_nlp_batch_parallel(_warm), install_pooled_serial_feral_backend}` (`batch.rs:381-710`). It requires `T: TNLP + Send`. |
| Warm starts: HiGHS / Clarabel / Uno | HiGHS: `Highs_setBasis`/`getBasis`, `setSparseSolution`, plus an implicit retained basis that is lost after `Highs_clearSolver`. Clarabel: `update_P/q/A/b`, refused under presolve or chordal decomposition (`data_updating.rs:55-181`). Uno: `uno_set_initial_dual_iterate`. |
| Re-initialisation: IDAS / PETSc | IDAS: `IDAReInit`, `IDASensReInit`, `IDAQuadReInit`, `IDAAdjReInit`. PETSc: retained SNES, `KSPSetReusePreconditioner`, `PCFactorSetReuseFill`/`ReuseOrdering`, `SNESSetLagJacobianPersists` (snes.c:3804, itfunc.c:256, factor.c:678). |
| Typed status | Uno C constants `UNO_SUCCESS..UNO_USER_TERMINATION` and `UNO_NOT_OPTIMAL..UNO_UNBOUNDED_OBJECTIVE` (`interfaces/C/Uno_C_API.h:44-59`). SUNDIALS 7.1.1 `SUNContext_PushErrHandler`, `ClearErrHandlers`, `GetLastError` (`include/sundials/sundials_context.h:32-45`). |
| Memory query / limit: SUNDIALS, KLU, FERAL, SCIP | SUNDIALS: `KINGetWorkSpace`, `KINGetLinWorkSpace`, `N_VSpace`. KLU: `Common.memusage`/`mempeak`. FERAL: `SymbolicFactorization::work_estimate() -> WorkEstimate{factor_bytes, peak_bytes, max_front_bytes}`, available after analysis and before numeric factorisation (`feral/src/symbolic/mod.rs:900-935`). SCIP: `limits/memory`. |
| Memory query / limit: PETSc, SuiteSparse | PETSc `PetscMallocSet(imalloc, ifree, irealloc)`, which must precede `PetscInitialize` (`src/sys/memory/mal.c:196-213`). `PetscMallocGetCurrentUsage`/`MaximumUsage` and `PetscMallocPush/PopMaximumUsage` (mtr.c:539-614) need the tracing allocator. SuiteSparse `SuiteSparse_config_{malloc,calloc,realloc,free}_func_set` (`SuiteSparse_config.h:291-294`). |
| Uno sparse routes at `805a27ec` | IPM preset `"ipopt"` → `EQPSolver` or `WoodburyEQPSolver` over MUMPS, SSIDS or HSL (`SubproblemSolverFactory.hpp:35-60`; `Presets.cpp:76-110`). `HAS_MUMPS` is found automatically by CMake (`CMakeLists.txt:229-233`). Every trust-region or inequality-QP route needs BQPD or HiGHS (`IQPSolver`). Hessian models: exact matrix or `uno_set_lagrangian_hessian_operator`, LBFGS, LSR1. |
| Sensitivities | `pounce-sens-core` (used). `pounce-sensitivity` and sIpopt are built but unused (census). IDAS forward and adjoint sensitivities are used. |

## 2. What is already adopted (verified; strengths to preserve)

- **KINSOL is used thoroughly.**
  - All four strategies (`kinsol.rs:1797-1802`), Anderson, eta forms, sign constraints, fixed-pattern KLU, and per-solve reset of every refreshable option (`kinsol.rs:1748-1796`).
  - The start-of-application setup policy is explicit and history-free: `KINSetNoInitSetup(mem, 0)`, `kinsol.rs:1774`.
  - The status map covers every pinned code (`kinsol.rs:2169-2201`).
  - Workspace queries are library calls (`kinsol.rs:1140-1215`).
- **Fixed-point maps are already library-iterated.**
  - `kinsol::Function::{Equations, Picard, FixedPoint}` (`kinsol.rs:243-258`).
  - `recycle.rs:3` ("no recycle iteration algorithm lives here").
  - PS-09 holds. The gap is only in the seam (DA-016).
- **POUNCE.**
  - Library second-opinion rungs are lowered into explicit, separately admitted profiles (`pounce/profiles.rs:36-131`).
  - Active-set SQP working-set warm start.
  - FERAL `reset_for_new_application` with neutral parking (`execution.rs:1046-1056`; `pounce/retained.rs:215`).
  - `pounce_sens_core` drives KKT analysis, sensitivities and path stepping over an **original-coordinate** FERAL factor that does not depend on the solver (`kkt.rs` header; `kkt/path.rs:16-19`). This is stronger than sIpopt or `pounce-sensitivity`, which use the solver's internal factor.
- **Ipopt.** The C++ bridge uses `OptimizeTNLP` and `ReOptimizeTNLP` with a checked bound-classification compatibility test. `warm_start_same_structure` is deliberately `no` (`pse_ipopt_sequence.cpp:232-262,298-309`).
- **HiGHS.** Explicit basis and sparse start; `reset_for_new_application` (`highs.rs:653`); per-solve `Highs_clearSolver` (`highs.rs:963`).
- **Clarabel.** Reusable-data updates, with refusal under equilibration (`conic.rs:622-625`).
- **SCIP.** Reoptimization; `limits/memory` is derived from the foreign allowance (`scip.rs:1340-1405`). This is the one *library-enforced* memory limit in the family.
- **IDAS.** `IDAReInit` at scheduled changes, forward sensitivities, adjoint, `IDACalcIC` (`dynamics/idas.rs` header).
- **Staged sequences.** A worker-owned `Retained` survives the steps of a sequence. `reset_independent` parks only HiGHS, POUNCE (when neutral) and SCIP (`execution.rs:1020-1063`; `math/staged.rs:675,1866`).

## 3. Unused or under-used capabilities

### C1 — Declared fixed-point and Picard as seam representations (the libraries already own the iteration)

**(i) Owns.** DA-016 bullet 1 completely:
- Runtime constructs `kinsol::Session` itself (`math/initialization.rs:1271`; `workflow/strategies/conditional.rs:912`) and calls `quality::qualify` itself (`:1288`; `:922`).
- DA-017 "KINSOL publishes `declared_fixed_point` that routing never derives".

**Mechanism.** Add `Problem::FixedPoint { map: Box<dyn FixedPointOracle>, … }` and `Problem::Picard { oracle, linear }` beside `Problem::Roots` (`execution.rs:452`). Give each a registry `NativeRepresentation` value (a registry or codegen change). Let `routing::problem_classes` emit `DeclaredFixedPoint` from a declared map. `execution::roots` then owns admission, qualification and reuse for both. The KINSOL adapter maps the variants onto the existing `kinsol::Function` variants.

**(ii) Removes.** The runtime's direct session and qualification paths, and the separate `restarts` wiring in `solve_declared_root`.

**(iii) Fit and limits.** Library behaviour is unchanged.
- PETSc `SNESANDERSON`/`SNESNRICHARDSON` on `F(x)=x−G(x)` could later serve the same class through the new variant. That is a lead, not recommended: no proven benefit, and it adds a second fixed-point path.
- KINSOL forbids constraints with `KIN_FP`/`KIN_PICARD` (`kinsol.rs:456`). That refusal stays with the adapter.

**(iv) Reverse check.** Preserves typed status, PS-09 (KINSOL iterates), and independent original-residual qualification (`FixedPointOracle::original_residual` stays the authority). It does not change explicit selection: `automatic_classes` stays as declared.

### C2 — Recovery profiles as an adapter method

**(i) Owns.** DA-016 bullet 2 completely. The POUNCE ladder is wired by name in `math/solves.rs:712,1127-1150,1278-1290` and in `strategy/target.rs:97-115,398-430`, each with its own settings extraction.

**Mechanism.** Add `BackendExecution::recovery_profiles(&self, controls, settings, last: &SolveReport, allow_replacement) -> Result<Vec<RecoveryProfile>>` and `recovery_capacity(...)`, both defaulting to empty. POUNCE implements them with the existing `second_opinion_profiles` and `second_opinion_capacity` (`pounce/profiles.rs:36,134`). The runtime iterates over adapters and never names `Backend::Pounce`.

**(ii) Removes.** The two runtime extraction copies and `pounce_recovery_baseline`.

**(iii) Fit.** This is exact: the library already returns pure descriptions (`second_opinion_rungs`), and the repository deliberately does not call the library's private retry driver (`profiles.rs:34-35`). Keep it that way. Do not adopt `run_second_opinion_ladder`: it would hide attempts inside one native call.

**(iv) Reverse check.** Every rung is still admitted and assessed separately as its own candidate, so typed status and independent qualification are preserved. Replacement starts still require explicitly permitted `StartOrigin::Auxiliary` recovery.

### C3 — Capability truth that depends on settings and method

**(i) Owns.**
- DA-017 Uno (`ExactHessianOrLimitedMemory` advertised, LM-only admitted, `execution/uno.rs:20,73-81`).
- DA-017 PETSc (NASM and pseudo-time published but refused, `execution/petsc.rs:74-76`).
- DA-016 arclength pinning (`math/solves/paths.rs:974-988`).
- DA-016 Schur gating (`solves.rs:5269`).
- DA-016 duplicated KINSOL JVP demand (`solves.rs:3893-3894` vs `execution/kinsol.rs:212-229`).

**Mechanism.** A library-backed per-method record replaces the one static `Capability`, for example `fn capability_for(&self, settings) -> Capability` with the published row as the union plus per-method rows. Add seam facts:
- `consumes_separator`: only POUNCE.
- `path_corrector`: Nlp + Root intent + `HessianMode::LimitedMemory`. Library fact: Ipopt and POUNCE both implement `hessian_approximation=limited-memory`; Uno LBFGS does too but only through the dense patch.
- The KINSOL demand stays with `required_artifact`; the runtime calls the adapter instead of re-deriving it.

The truth comes from library facts the repository already queries (`IpoptGetAvailableLinearSolvers`, linked features, Uno presets).

**(ii) Removes.** Runtime `Backend::` gates (census footprint ≈50) and the static-row mismatch.

**(iii) Limits.** Some facts are composition-dependent: PETSc NASM needs subdomain blocks. To be served by the adapter, `Problem::Roots` needs an optional decomposition input. Without one, report it as "composition-only" in the record rather than advertising it.

**(iv) Reverse check.** This tightens DP-19 and G7 truthfulness. Refusals stay typed and happen before effects.

### C4 — Uno without the dense patch: a sparse route exists, but not for the current SQP method (remedy partly open)

**(i) Obligation.** DA-018 and the Uno part of DA-022 (the quadratic `count*64+n*16` projection).

**Library facts.**
- At `805a27ec`, every trust-region SQP route (`filtersqp`, `funnelsqp`) sends curvature subproblems to `IQPSolver(QPSolverFactory)`, which means BQPD or HiGHS (`SubproblemSolverFactory.hpp:56-59`).
- BQPD is not in the image. HiGHS QP needs an explicit PSD Hessian, which is why the patch materialises L-BFGS with n Hessian-vector products.
- Upstream has **no sparse, unpatched filter-TR-SQP** at this revision. For the SQP method the remedy is **open**.
- What upstream does offer is the `"ipopt"` preset: an interior-point method with line search and `EQPSolver`, or `WoodburyEQPSolver` for L-BFGS, over **MUMPS 5.9.1 or SPRAL (SSIDS)**. Both are already in the image for Ipopt (`docker/solvers/Dockerfile:7`). CMake picks MUMPS up through `find_package(MUMPS)` (`CMakeLists.txt:229-233`). That route needs no HiGHS, no materialisation and no feasibility-LP seed.
- `uno_set_lagrangian_hessian_operator` (`Uno_C_API.h:258`) allows exact curvature through Hessian-vector products.

**(ii) Removes, if adopted for an IPM profile.** The patch hunks for that profile and the quadratic allowance.

**(iii) Fit and limits.**
- This is a *different method* (Uno-IPM). Its distinct value over Ipopt and POUNCE is Uno's filter/funnel and restoration variants. Judge it as a method addition, not as a fix to the SQP route.
- One process-wide MUMPS link must be shared with Ipopt (`libcoinmumps`). Verify there is one copy.
- MUMPS is sequential, which matches the single thread Uno already admits.

**(iv) Reverse check.** Keeping the SQP-LBFGS profile requires the patch, and dropping it would remove an existing capability. So do not retire SQP to fix DA-018. Record "SQP: remedy open (needs BQPD or a later Uno with a sparse QP route)".

### C5 — Native session continuity across independent applications

**(i) Owns.** "Native session continuity across calls/points/steps" for KINSOL, and partly DA-014 (dynamic sweeps).

**KINSOL (clear win).** `Retained::reset_independent` drops any session without an accepted neutral reset (`execution.rs:1057-1062`), and KINSOL has none. Yet the KINSOL session is already history-free per application:
- `KINSetNoInitSetup(0)` (`kinsol.rs:1774`).
- Every refreshable option is reset (`:1781`).
- `clear_callback_setup` drops preconditioner and block factors (`:1556-1563`).
- `SUNLinSol_KLUBeginApplication` forces a fresh numeric factor while keeping symbolic analysis (`kinsol.rs:1638`; patched header `:140-145`).
- `Session::replace` swaps the function on a compatible layout (`:1242`).

Adding `kinsol::Session::reset_for_new_application` (clear the callback owner and seeds, keep memory and KLU symbolic) and a `has_neutral_state` predicate lets independent study points and requests reuse the KINSOL memory, vectors and KLU analysis. This is the same contract HiGHS and POUNCE already meet.

**IDAS.** The IDAS memory plus `IDAReInit` and `IDASensReInit` across study points becomes possible once DA-014's value binding for dynamics exists, since the trajectory is outside the seam (`execution/dynamics.rs:60-104`). Lead only, because it depends on DA-014.

**PETSc.** "Fresh scoped SNES handles" (`execution/petsc.rs:31`). PETSc reuses LU symbolic analysis automatically when a retained SNES/KSP gets a matrix with the same nonzero pattern. Making that history-free needs `KSPSetReusePreconditioner(PETSC_FALSE)` and lag-persistence off. Proposed, lower priority.

**(ii) Removes.** Repeated KINSOL and KLU construction per independent point.

**(iv) Reverse check.** No numeric history crosses applications, so typed reuse provenance (`ReuseHistory`) and explicit-start policy are preserved. **Rejected variant:** `KINSetNoInitSetup(1)` across calls. It would reuse the predecessor's numeric factor and violate history independence. (With fixed-pattern KLU, `BeginApplication` would force a fresh numeric factor anyway, so the gain would be nil.)

### C6 — Replace the vendored KLU patch with SUNDIALS' own extension point

**(i) Owns.** Part of the native patch burden (the DA-018 pattern applied to SUNDIALS) and KLU size modelling (DA-022).

**Mechanism.**
- Build a custom `SUNLinearSolver` with `SUNLinSolNewEmpty` and its own `ops` (initialize, setup, solve, space, free). Context7 `/llnl/sundials` documents this as the supported way to implement custom modules.
- Back it with direct `suitesparse_sys` `klu_l_analyze`/`klu_l_factor`. The repository already owns exactly this for Diffsol (`dynamics/linear.rs:24,251-292`).
- `setup` does a full `klu_factor` per application while keeping symbolic analysis, which is the semantics the patch adds. Upstream cannot express it, because `KLUReInit` frees symbolic analysis and the default `refactor` carries pivots.
- `space` reports `memusage`.

**(ii) Removes.** The 189- and 43-line KLU diffs, and a second KLU owner (one KLU owner for KINSOL, IDAS and Diffsol).

**(iii) Limits.** IDAS checkpoint, accepted-step and replay limits have **no upstream cap**. Driving `IDASolveF` in `IDA_ONE_STEP` mode and reading `ncheck` would enforce them only after at most one more checkpoint has been allocated, which weakens admission before allocation. Those IDAS patches therefore stay: **remedy open**.

**(iv) Reverse check.** Preserves the fresh numeric factor per application and the observers. Cancellation still drains through the existing synchronous owner.

### C7 — Memory admission queried or enforced by the library (DA-022 native part)

| Library | Capability | Fit |
|---|---|---|
| PETSc | `PetscMallocSet` installed just before `PetscInitializeNoArguments` (`pse_petsc.c:200`). A thread-local current-owner allocator gives **exact** accounting and a **hard limit**: refusal raises `PETSC_ERR_MEM`, which already maps to a typed error. | Replaces the shape models in `petsc/derived.rs:143-187`, `blocks.rs:686` and `flow.rs:608`. That code itself says "PETSc has no hard per-instance memory limiter", but PETSc has this hook. Limits: process-global; frees need a header prefix; MKL/BLAS allocations bypass it. |
| KLU (SuiteSparse) | `SuiteSparse_config_*_func_set` with a thread-local owner: exact KLU factor accounting plus refusal (`KLU_OUT_OF_MEMORY`). | Covers KINSOL, IDAS and Diffsol KLU. Limits: global; KLU's routing through `SuiteSparse_malloc` is unverified (probe P5). |
| FERAL | `work_estimate().peak_bytes` read after symbolic analysis and before numeric factorisation. | `kkt.rs:764` reads `factor_bytes` only after the fact. Use `peak_bytes` to admit the numeric factor before allocating it. POUNCE-internal FERAL is not reachable this way. |
| SUNDIALS | `KINGetWorkSpace`/`IDAGetWorkSpace`/`N_VSpace` after construction. | Already used. Pre-construction shapes remain models; SUNDIALS has no allocator hook for serial `N_Vector`. |
| Ipopt / MUMPS, HiGHS, Uno, Clarabel | No query or limiter is reachable through the bridges. | Lead only. The models stay; DA-022's drift risk stays for these. |

**(iv) Reverse check.** Hooks let admission happen *at* allocation, which is stronger than pre-charging. The runtime reservation must still be taken before the native call, so the order is preserved. A refused allocation must surface as `ResourceExhausted`, not as a crash. PETSc and KLU already return typed out-of-memory codes.

### C8 — Keep the POUNCE presolve transformation across sequence steps

**Library.** `pounce_presolve::warm::PresolveFingerprint` plus `PresolveTnlp::invalidate`.

**Current.** The repository rebuilds `PresolveTnlp`/`LinearEqElimTnlp` per attempt ("attempt-owned … not a retained cache", `presolve/pipeline.rs:144-146`). It does compare transformations for working-set transfer (`:626-631`).

**Proposed.** Keep the wrapper in the POUNCE `Session` while the fingerprint matches. Do not adopt `TnlpPresolveSession` itself: it owns the `IpoptApplication`, which conflicts with the adapter's session owner.

**Limits.** The fingerprint is blind to FBBT tapes and to auxiliary nonlinear data (`warm.rs` docs), so repository facts must invalidate explicitly. Benefit unmeasured.

### Considered and not recommended

| Candidate | Reason |
|---|---|
| Ipopt `warm_start_same_structure=yes` | No symbolic reuse in 3.14.20 (`IpTSymLinearSolver.cpp:129-131`). It inherits the predecessor's scaling, which the bridge rightly refuses (`pse_ipopt_sequence.cpp:298-301`). The gain is limited to spaces and maps. |
| sIpopt / `pounce-sensitivity` | Both bind to the solver's internal scaled factor, which weakens the original-space independent analysis already in `kkt`. |
| POUNCE NLP batch | `NlpOracle` is not `Send` (`lib.rs:565`). The library builds per worker. It competes with lane parallelism for admitted threads, so it is not a clear win. |
| HiGHS implicit hot start | Conflicts with the explicit-start policy. The basis is already exported and seeded explicitly. |

## 4. Clear wins on existing functionality no DA names

**W1. Uno: `INFEASIBLE_SMALL_STEP` overstates infeasibility.**
- `uno.rs:119` maps iterate 5 (`UNO_INFEASIBLE_SMALL_STEP`, `Uno_C_API.h:57`) to `Termination::Infeasible`.
- Ipopt's analogue `Search_Direction_Becomes_Too_Small` maps to `Numerical` (`ipopt.rs:59-62`).
- Only iterate 3 (`INFEASIBLE_STATIONARY_POINT`) is the counterpart of `Infeasible_Problem_Detected`.
- Fix: map 5 to `Numerical` (or `Inconclusive`).

**W2. Uno: the bridge latch is masked by Uno's native status.**
- After `uno_optimize`, the bridge records `native_finished` and then polls (`pse_uno.cpp:305-310`). `pse_uno_get_result` therefore succeeds even when the latch is `DEADLINE`, `MEMORY` or `ABANDONED` (`:131-132,312-317`).
- Rust then takes `native_known` and `termination(opt, iter)` (`uno.rs:413-420`). `USER_TERMINATION` (5) becomes `Cancelled`, and a caught HiGHS memory scope can become `ALGORITHMIC_ERROR` and then `Numerical`. The solve `status` is ignored.
- Fix: when `status ∈ {CANCELLED, DEADLINE, ABANDONED, MEMORY}`, `bridge_termination(status)` decides the category; keep the native codes as metrics.
- Evidence is *Interface-checked*. I did not establish whether a downstream owner reconciles this (probe P1).

**W3. Uno: use named constants.** The bridge should `static_assert` the `UNO_*` constants (`Uno_C_API.h:44-59`) against the `pse_uno` codes and export names, so `uno.rs:114-131` stops using literals. Also emit typed names instead of `Uno_optimization_{n}_iterate_{m}`. This closes DA-018's "status literals unchecked".

**W4. SUNDIALS: capture native error messages.** Use `SUNContext_ClearErrHandlers` plus `SUNContext_PushErrHandler` (7.1.1, `sundials_context.h:38-45`) on the KINSOL and IDAS contexts. That fills `NativeTermination.message`, which is currently `None` (`kinsol.rs:2197`), and stops default stderr logging. It needs no ABI change.

**W5. Correct the vendor notes.** `vendor/native-supplier-notes.md:29` must state the SUNDIALS C modifications (§0). This is DP-24 truthfulness.

**W6. Uno SLP admission (minor).** The SLP profile (`hessian_model=zero`, `pse_uno.cpp:270-271`) still requires `controls.hessian == LimitedMemory` (`execution/uno.rs:73-81`). The record should state each profile's own curvature (folds into C3).

**W7. Uno dual starts (optional, minor).** `uno_set_initial_dual_iterate` exists (`Uno_C_API.h:291`), but the record is `WarmCapability::Primal`. Promoting it to primal-dual for sequences is low-risk, but not urgent.

## 5. Unresolved questions and probes

| # | Question | Probe (read-only or targeted test) | Settles |
|---|---|---|---|
| P1 | Does any downstream owner correct the Uno category when the bridge latch is set (W2)? | `rg -n "Termination::Cancelled|stopped\(\)" crates/pse-backend-native/src/execution/runner.rs crates/pse-backend-native/src/uno.rs`. Then a targeted unit test with `status=DEADLINE` and `optimization=5`; expect `TimeLimit`. | Whether W2 is a live defect. |
| P2 | Does a KINSOL neutral reset keep any numeric history: Anderson state, `setup` or `raw_map` fields? | Read `Session` fields (`kinsol.rs:610-660`) and `solve` prologue `:1728-1740`. Test: two independent applications on a parked session give bit-identical reports versus a fresh session. | Whether C5 for KINSOL is safe. |
| P3 | Is Uno's IPM route with MUMPS viable in the image (one MUMPS copy, sequential, indexing)? | `nm -D` on the Uno archive and `libcoinmumps`. A CMake configure with `MUMPS_*` pointed at the image libraries. | Feasibility of C4. |
| P4 | Is PETSc's malloc hook compatible with MPIUNI and the bridge's init/finalise lifecycle, and do re-initialisations re-enter it? | Read `pse_petsc.c:185-203`; check `petscsetmallocvisited` (mal.c:216) against repeated `PetscInitialize`. | C7 PETSc. |
| P5 | Does KLU 2.x in `suitesparse_sys` 0.1.4 allocate only through `SuiteSparse_malloc`? | `rg -n "malloc\(|SuiteSparse_malloc" ~/.cargo/registry/src/*/suitesparse_sys-0.1.4/vendor/KLU/Source`. | C7 KLU. |
| P6 | Do the pounce-presolve fingerprint inputs cover every value-only rebind the repository performs? | Read `pounce-presolve/src/warm.rs` `PresolveFingerprint` fields against `presolve/pipeline.rs:560-631`. | C8 correctness. |
| P7 | Why is arclength limited to limited-memory Hessians (`paths.rs:1781-1783`: "library limited-memory curvature")? Is it a missing augmented-Hessian supply? | Read `paths.rs:1730-1790`. | Whether the C3 `path_corrector` fact needs LM or could admit exact Hessians. |
| P8 | IBEX root isolation. | Not examined beyond inventory (`root_isolation.rs`). | Coverage gap. |

**Search coverage for negative claims.**
- I found no use of `TnlpPresolveSession`, `pounce_presolve::warm`, `PetscMallocSet`, `SuiteSparse_config_*_func_set` or `SUNContext_PushErrHandler`. The search covered `crates/` (Rust and C/C++ bridges) and `vendor/sundials-sys`.
- The DA-024 native child lane has no library alternative in any in-use library (lead only).
- Silence elsewhere does not establish absence.

**Files written.** None.