> Supporting evidence for [the solver methods, selection and diagnostics review](../../reviews/design_review_solver-methods-selection-and-diagnostics_2026-10-12.md). Wave-1 worker return, B3 (library-research): IDAS, Diffsol, CVODES/ARKODE, PETSc TS, published unchanged by the coordinator. Baseline: `ff5a4aa2f` plus the Plan 36 working tree (diff fingerprint `45ceea51b0698679`). It is read-only and static: nothing was executed. Its judgments are the worker's; the principal review owns the verdicts.

I'm returning the B3 review evidence below in Markdown. The work was read-only. I ran no builds, tests or cargo, and wrote no files.

# B3: dynamics and sensitivity solver libraries (IDAS, Diffsol, CVODES/ARKODE, PETSc TS)

## Summary

1. **Unbound IDAS diagnostics: the brief's claim is confirmed.** The adapter never calls these, although all exist in the bundled SUNDIALS 7.1.1 headers:
   - `IDAGetIntegratorStats`, `IDAGetLastOrder`/`IDAGetCurrentOrder`, `IDAGetLastStep`/`IDAGetCurrentStep`;
   - `IDAGetNumLinSolvSetups`, `IDAGetNumJacEvals`, `IDAGetNumLinConvFails`, `IDAGetNumStepSolveFails`;
   - `IDAGetErrWeights`/`IDAGetEstLocalErrors`, which identify the variables that limit the step size;
   - `IDAGetSensStats`, `IDAGetAdjCheckPointsInfo`;
   - every SUNDIALS error-handler and logger API.

   *(Interface-checked; grep over `crates/**/*.rs`.)*
2. **Dynamics statistics are not persisted: confirmed.** `dynamics::Report.statistics` is a `Vec<serde_json::Value>`. Its consumers are memory-byte accounting, shooting stitching and accuracy contrast. No result table receives it. *(Interface-checked.)*
3. **`Auto` is refusal-ordered: confirmed.** It takes the first non-refused candidate from the fixed list `[Diffsol, Idas]`. No stiffness, event-frequency, size or sensitivity-shape feature enters the choice. *(Interface-checked.)*
4. **The bundled build cannot produce step-level telemetry.** The `sundials-sys` build script passes no logging, monitoring or profiling defines. The generated `sundials_config.h` in `target/` shows `SUNDIALS_LOGGING_LEVEL 2`, with monitoring and profiling undefined. IDAS 7.1.1 also has no monitor function. Per-step observation therefore needs `IDA_ONE_STEP` polling or a custom SUNDIALS build. *(Interface-checked.)*
5. **The `arkode` feature gives no ARKODE bindings.** `sundials-sys` 0.6.2's `wrapper.h` never includes any `arkode/*.h` header, so the feature only builds and links the library. ARKODE also cannot integrate a singular mass matrix such as diag(I,0). CVODES is bindable just by enabling the feature, but it covers mass-free ODEs only. *(Interface-checked.)*
6. **PETSc 3.24 TS discrete adjoints exist only for TSRK and TSTHETA (beuler/CN).** BDF, ARKIMEX and ROSW forward integration is reachable through the existing IFunction/IJacobian bridge with a type switch. TSEvent supports a direction per event and terminal events, and its post-event callback runs in both the forward and adjoint passes. *(Interface-checked from docs.)*
7. **Upstream has moved since the pins.**
   - SUNDIALS is at 7.9.0 (2026-09-03). The releases that matter for IDA(S) are 7.6, which adds constraint-failure counters and `IDA_TOO_CLOSE` and fixes linear-solver counter resets, and 7.8, which fixes an IDAS quadrature predictor bug after `IDACalcIC` (local exposure is guarded off).
   - Diffsol is at 0.17.1. It brings batched sensitivities and adjoints and a breaking diffsol-la 0.2 change, but no new schemes.

## Baseline and scope

- **Baseline.** Working tree at HEAD `ff5a4aa2f` plus the Plan 36 diff.
- **Pins** (`Cargo.toml` around lines 236–246):
  - `sundials-sys =0.6.2`, `default-features = false`, features `kinsol`, `klu`, `build_libraries`, `static_libraries`. The backend `idas` feature adds `sundials-sys/idas` (`crates/pse-backend-native/Cargo.toml:41`).
  - `diffsol =0.16.2` with `faer`; `diffsol-la =0.1.1`.
  - PETSc 3.24.0 configured `--with-mpi=0`, no Fortran, 32-bit indices (`[workspace.metadata.pse.petsc]`).
- **Sources read:**
  - `~/.cargo/registry/src/*/sundials-sys-0.6.2/{build.rs,wrapper.h,vendor/include/{idas,cvodes,arkode}/*.h,vendor/src/idas/{idas.c,idaa.c},vendor/src/sunlinsol/klu/sunlinsol_klu.c,vendor/cmake/SundialsBuildOptionsPre.cmake}`
  - `diffsol-0.16.2/src/ode_solver/{mod.rs,method.rs,problem.rs,jacobian_update.rs,config.rs,tableau.rs}`
  - Local adapters, `crates/pse-petsc-sys/bridge/pse_petsc.{h,c}`, blueprint §13.6 and §25.
  - GitHub release notes for LLNL/sundials and martinjrobins/diffsol.
  - Context7 `/websites/petsc_release`.
  - The `native-solver-libraries` skill. Its `dynamics.ida-idas` and `dynamics.diffsol` routes are thin and carry no statistics or method-choice content.

---

## 1. IDAS (SUNDIALS 7.1.1, bundled)

### 1.1 Methods and applicability *(Interface-checked, `idas.h`/`idas_ls.h` unless stated)*

- **BDF, variable order 1–5, variable step, fixed-leading-coefficient form.**
  - Residual form F(t, y, y′) = 0, for index-1 DAEs and implicit ODEs.
  - `IDASetMaxOrd` and `IDASetMaxStep`/`IDASetMinStep` exist. Step-adaptivity controls: `IDASetEtaFixedStepBounds`, `IDASetEtaMin/Max/Low/MinErrFail/ConvFail` (idas.h:205–217).
  - BDF orders 3 and above are not A-stable. Eigenvalues near the imaginary axis force order reduction or error-test failures. IDA has no stability-limit detection, unlike CVODE's `CVodeSetStabLimDet`.
  - After every `IDAReInit` the integrator restarts at order 1 with a fresh initial step: `ida_kused = 0`, `nst = 0` (idas.c:838–856).
- **Consistent initialization: `IDACalcIC(icopt, tout1)`.**
  - `IDA_YA_YDP_INIT` (1) computes the algebraic y and the differential y′, given the differential y. It needs `IDASetId`.
  - `IDA_Y_INIT` (2) computes all of y given y′.
  - Tuning (idas.h:182–188): `IDASetNonlinConvCoefIC`, `IDASetMaxNumStepsIC`, `IDASetMaxNumJacsIC`, `IDASetMaxNumItersIC`, `IDASetLineSearchOffIC`, `IDASetStepToleranceIC`, `IDASetMaxBacksIC`.
  - Results come from `IDAGetConsistentIC` and `IDAGetNumBacktrackOps`. Sensitivity ICs come from `IDAGetSensConsistentIC`. Backward problems use `IDACalcICB`/`IDACalcICBS`.
- **Constraints: `IDASetConstraints`** takes sign codes 0, ±1, ±2. A failed constraint reduces the step. 7.1.1 has no constraint-failure counter (added in 7.6).
- **Roots:** `IDARootInit`, `IDASetRootDirection`, `IDASetNoInactiveRootWarn`, `IDAGetRootInfo`, `IDAGetNumGEvals`.
- **Sensitivities:**
  - Forward: `IDASensInit` with `IDA_SIMULTANEOUS` or `IDA_STAGGERED`, plus `IDASetSensErrCon`, `IDASetSensDQMethod` and sensitivity quadratures.
  - Adjoint: `IDAAdjInit(Nd, IDA_HERMITE|IDA_POLYNOMIAL)`, `IDASolveF`/`IDASolveB`, `IDAInitB`, and `IDAInitBS` for backward problems that depend on forward sensitivities. Second-order (forward-over-adjoint) is supported this way.
  - Checkpointing is fixed-interval (every Nd steps), with Hermite or polynomial interpolation. There is no revolve-style binomial schedule.
  - **`IDAAdjReInit` frees every stored checkpoint** (`idaa.c:259–264`), which confirms the Plan 36c note.
- **Linear solvers:**
  - Matrix-based: dense, band, sparse KLU.
  - Krylov: SPGMR, SPFGMR, SPBCGS, SPTFQMR, PCG. User preconditioning via `IDASetPreconditioner`. The bundled BBD preconditioner targets parallel use.
  - KLU keeps its symbolic analysis for the life of the linear-solver object. Later setups run `klu_refactor`, which falls back to a full factor when `klu_rcond`/`klu_condest` reports poor conditioning (`sunlinsol_klu.c:255–330`).
- **Jacobian reuse:**
  - IDA has no Jacobian-evaluation-frequency control; that exists only in CVODE (`CVodeSetJacEvalFrequency`, `CVodeSetLSetupFrequency`).
  - IDA calls linear setup, which here means a new Jacobian plus a refactor, when nst = 0, when cj/cj_old leaves [(1−dcj)/(1+dcj), …], after a convergence failure, or when forced (idas.c:6284–6307).
  - `IDASetDeltaCjLSetup` (idas.h:191) tunes dcj (default 0.25). Other tuning: `IDASetEpsLin`, `IDASetLSNormFactor`, `IDASetLinearSolutionScaling`, `IDASetIncrementFactor`.

### 1.2 Method-choice controls and upstream guidance

- Upstream offers no automatic method choice inside IDA.
- Interpretation: IDA suits stiff index-1 DAEs, and its variable-order BDF is efficient when segments are long enough to ramp up order (tens of steps or more).
- For sensitivities, the upstream guide's rule is forward when parameters are few and adjoint when many parameters feed few functionals. CVODES and IDAS use the same principle.

### 1.3 Warm-start and reuse

- `IDAReInit` resets the order, step history and counters, but keeps memory, the linear solver (and therefore KLU's symbolic analysis), roots and options.
- The first step after a reinit always forces a Jacobian evaluation and refactor (nst = 0 triggers setup).
- `IDASensReInit`, `IDAQuadReInit` and `IDAReInitB` are the companion resets.
- `IDAAdjReInit` deletes checkpoints. Keeping several segments' checkpoints therefore needs one IDAS memory per segment, or replay.

### 1.4 Statistics and diagnostics API at 7.1.1 *(Interface-checked, idas.h:246–293, 387–402, idas_ls.h:105–120)*

- **Integrator:**
  - `IDAGetIntegratorStats(nst, nre, nsetups, netf, qlast, qcur, hinused, hlast, hcur, tcur)`;
  - the individual getters (`NumSteps`, `NumResEvals`, `NumLinSolvSetups`, `NumErrTestFails`, `Last/CurrentOrder`, `ActualInitStep`, `Last/CurrentStep`, `CurrentTime`, `CurrentCj`, `TolScaleFactor`);
  - `IDAGetNumStepSolveFails`.
- **Nonlinear solver:** `IDAGetNonlinSolvStats`, `IDAGetNumNonlinSolvIters`, `IDAGetNumNonlinSolvConvFails`.
- **Linear solver:** `IDAGetNumJacEvals`, `NumPrecEvals`, `NumPrecSolves`, `NumLinIters`, `NumLinConvFails`, `NumJTSetupEvals`, `NumJtimesEvals`, `NumLinResEvals`, `IDAGetLastLinFlag`, `IDAGetJacCj/JacTime/JacNumSteps`.
- **Step-limiting variables:** `IDAGetErrWeights` and `IDAGetEstLocalErrors`. The element-wise product |ele_i · ewt_i| ranks the components that limit h.
- **Initial conditions:** `IDAGetConsistentIC`, `IDAGetNumBacktrackOps`.
- **Roots:** `IDAGetRootInfo`, `IDAGetNumGEvals`.
- **Sensitivities:** `IDAGetSensStats`, `IDAGetSensNonlinSolvStats`, `IDAGetNumStepSensSolveFails`, `IDAGetSensErrWeights`, plus quadrature (and quadrature-sensitivity) statistics.
- **Adjoint:** `IDAGetAdjCheckPointsInfo`, `IDAGetAdjCurrentCheckPoint`, `IDAGetAdjDataPointHermite`.
- **Bulk dump:** `IDAPrintAllStats(FILE*, SUNOutputFormat)`, table or CSV.
- **Logger, profiler and error handler:**
  - The build script passes no `SUNDIALS_LOGGING_LEVEL`, `SUNDIALS_BUILD_WITH_MONITORING` or `SUNDIALS_BUILD_WITH_PROFILING` defines (`build.rs` `build_vendor_sundials`).
  - The CMake defaults are logging 2 (errors and warnings only), monitoring OFF, profiling OFF (`SundialsBuildOptionsPre.cmake:58,65,93`).
  - Every generated `target/debug/build/sundials-sys/*/out/include/sundials/sundials_config.h` read shows `SUNDIALS_LOGGING_LEVEL 2`, monitoring and profiling `#undef`, and `SUNDIALS_INT64_T`.
  - The info and debug logs that hold per-step order, h and error-test detail need level 3 or higher, so they are unavailable in this build.
  - The default `SUNContext` error handler is `SUNLogErrHandlerFn`, which writes to stderr (`sundials_context.c:90`). `SUNContext_PushErrHandler` can capture code, function and message.

### 1.5 Bound versus unbound locally *(Interface-checked)*

**Bound** (`BN/dynamics/idas.rs`):

- Per-segment counters at `Session::statistics` (2157–2189): steps, residual evaluations, nni, ncfn, netf, IC backtracks, root evaluations, linear iterations, preconditioner evaluations, Jv products.
- Backward counters at `backward_statistics` (3130–3180): steps, nni, ncfn, netf.
- IC options: all seven tuning setters (1847–1898).
- `IDA_YA_YDP_INIT`/`IDA_Y_INIT` (1314–1318), `IDASetConstraints` (1504), `IDASetRootDirection`, `IDASetId`, `IDASetInitStep`, `IDASetStopTime`.
- KLU with analytic Jacobian, or SPGMR/SPFGMR with an analytic Jv and an optional Jacobi preconditioner (1704–1787).
- Simultaneous or staggered sensitivity correction, adjoint with `IDA_HERMITE` (1619), and second-order through `IDAInitBS`.

**Unbound.** The brief's list is confirmed in full:

- order history, step-size history, `nsetups`, `njevals`, `nlcfails`, step-solve failures;
- error weights and estimated local errors;
- sensitivity statistics, checkpoint information, last linear flag;
- `IDASetMaxOrd`, `IDASetMaxStep`, `IDASetEta*`, `IDASetDeltaCjLSetup`, `IDASetNonlinConvCoef`, `IDASetMaxConvFails`, `IDASetMaxErrTestFails`, `IDASetEpsLin`;
- `IDA_ONE_STEP` (only `IDA_NORMAL` is used, at 2130);
- any SUNDIALS error handler or logger. `NativeTermination.message` is always `None` (idas.rs:293).

**Search coverage:** `grep -rn` for each of these symbols over `crates/**/*.rs` found no hits.

**Persistence.** `Report.statistics` (`BN/dynamics.rs:2127–2128`) reaches:

- `RT/workflow/modeling/dynamics.rs:689`, which only counts bytes;
- `dynamics_accuracy.rs:2156,2270`, which nests it under `native_statistics` inside the same in-memory report;
- `shooting.rs:1525`, which concatenates per-window statistics.

`Report` does not derive `Serialize`. The `statistics` used in `analysis_tables.rs:566` and the `pse-py` getter is a different object, the modeling-diagnostics count map. Coverage was `rg "\.statistics\b"` over all `crates/`, and no persisted dynamics-statistics schema turned up. This confirms that dynamics statistics are never persisted, within that search.

**Adjoint replay.** `Session::restart` calls `IDAAdjReInit` at each segment restart (1943). `backward` then re-runs each earlier segment forward through `replay` (2598–2631: `IDAReInit` + `IDAAdjReInit` + `IDASolveF`). This costs roughly one extra forward pass per multi-segment adjoint, and the replay is required under the pinned semantics.

### 1.6 What newer releases add for IDA(S) *(Interface-checked, release notes)*

| Release | Relevant to the IDAS route |
|---|---|
| 7.2 | Uniform info-level logging plus the `suntools` log parser (still needs logging level ≥ 3); sparse `SUNMatScaleAddI` out-of-bounds fix for unsorted index values |
| 7.3 | `suncountertype`; workspace getters deprecated |
| 7.4 | Fix for an `IDAAdjInit` segfault after adjoint memory was freed |
| 7.6 | `IDASetMaxNumConstraintFails`, `IDAGetNumConstraintFails`, `IDAGetNumConstraintCorrections`; constraint failures no longer counted as step-solve failures; new `IDA_TOO_CLOSE` return; linear-solver counters now reset at reinit |
| 7.8 | Fix: IDAS quadrature predictor wrong after `IDACalcIC`, giving an inconsistent forward recomputation from a checkpoint and a possible segfault; also a fix for an unrecognized linear-solver flag being counted as success |
| 7.9 | IDA/IDAS logging duplicate-key fix |

- **The 7.8 bug.** Local exposure is guarded off: an adjoint together with declared quadratures is refused (`BN/dynamics.rs:1489–1492`). Backward quadratures (`IDAQuadInitB`) are a different code path.
- **The 7.6 counter-reset fix.** The adapter reads counters right after `IDASolve`, so the pre-7.6 bug of stale linear counters read before the next advance should not affect it. *(Interpretation.)*
- **Upgrade path.** `sundials-sys` upstream (Chris00) still shows 0.6.x. A newer SUNDIALS would mean dropping `build_libraries` and pointing `SUNDIALS_INCLUDE_DIR`/`SUNDIALS_LIBRARY_DIR` at a local install, which `build.rs` supports for versions 6 and up, or a project-owned `-sys` crate. 7.7 also renames the CMake options to `SUNDIALS_ENABLE_*`, and the 0.6.2 build script passes the old names.

### 1.7 Observation and interrupt hooks → typed outcome *(Interface-checked)*

- **Residual return.** The callback returns `+1` (recoverable) unless a terminal cause is latched, otherwise `-1` (`Context::failure`, idas.rs:195–201).
  - A panic latches `Termination::Panic` and returns −1 (`finish_callback`, 507–518).
  - On recoverable returns, IDA cuts the step or backtracks, and repeats end in `IDA_REP_RES_ERR`, categorized as Evaluation.
  - −1 gives `IDA_RES_FAIL`. `native_failure` (212–232) prefers the latched typed cause, then a callback witness, then `NativeStatus`.
  - All 60 pinned flags map in `termination` (234–297); `IDA_WARNING` maps to Inconclusive.
- **Stop between steps.** There is no native stop API. Cancellation and deadline arrive through callback latching, and `stopped` (327ff) maps them to `Termination::Cancelled` or `TimeLimit`. Coarse stops between outputs come from `IDASetStopTime` per sample and the step budget (`IDA_TOO_MUCH_WORK` maps to `StepLimit`, 2229).
- **Monitoring.** IDAS 7.1.1 has no `IDASetMonitorFn`; the call exists only in CVODE(S) and is deprecated in 7.7. Per-step observation therefore needs `IDA_ONE_STEP`.

---

## 2. Diffsol 0.16.2 (faer)

### 2.1 Methods *(Interface-checked)*

- **Schemes:** BDF (variable order, NDF-style), `tr_bdf2`, `esdirk34` and `tsit45` (`tableau.rs`), plus custom `Tableau` through `sdirk_solver`/`explicit_rk_solver` (`problem.rs`).
- **Problem form:** M y′ = f(t, y, p), with a constant mass matrix, which can be singular as diag(I,0) for index-1.
- **Events:** root finding detects any sign change, with no direction; resets via `apply_reset[_with_sens|_with_adjoint]`.
- **Sensitivities:** forward (`bdf_sens`, `*_sens`) and adjoint with checkpointing (`checkpointing.rs`, `adjoint.rs`).
- **Explicit `tsit45`:** admissible only for mass-free ODEs. Locally that is enforced (`BN/dynamics.rs:1723–1729`); non-stiffness is not checked.
- **Linear solvers:** pluggable through the diffsol-la `LinearSolver` trait. Locally these are a project-owned faer LU and KLU (`BN/dynamics/linear.rs`).
- **Callback limits:** ops return `()`, with no error channel (`op/nonlinear_op.rs:12`). A callback cannot signal a recoverable failure.

### 2.2 Controls and Jacobian reuse

`OdeSolverOptions` (`problem.rs:98–131`) carries:

- Newton limits: iterations, error-test failures, Newton failures, tolerance;
- step bounds: minimum step and growth/shrink bounds;
- PI-controller gains;
- Jacobian reuse: `update_jacobian_after_steps` (20), `update_rhs_jacobian_after_steps` (50), `threshold_to_update_jacobian` (0.3), `threshold_to_update_rhs_jacobian` (0.2).

`JacobianUpdate` forces a setup on any convergence or error-test failure or at a checkpoint. The local `ode_options::Remote` (`BN/dynamics.rs:2457–2475`) exposes every field. *(Interface-checked.)*

### 2.3 Statistics

- `OdeSolverStatistics` (`mod.rs:28–49`) counts steps, error-test failures, Newton iterations, Newton failures, and linear setups, total and split by cause (checkpoint, first or second convergence failure, error-test failure, step success).
- It has no RHS or Jacobian evaluation counts and no root-evaluation counts.
- Per-step order and step size are available through `OdeSolverMethod::order()` (method.rs:198) and `state().h` (state.rs:31).
- **Local binding:** `segment_statistics` (`integrator.rs:817`) serializes the whole struct per segment. `drive` (1220–1270) steps one at a time and emits a `simulation.step` progress event with only `time` and `steps`; it never reads `order()` or `h`, though both are there for free. *(Interface-checked.)*

### 2.4 Interrupts

`Shared::check`/`abort` (`integrator.rs:50–70`) stores the failure and calls `resume_unwind(Abort)` from inside callbacks. A failed linear factorization returns `LinearSolverError`, and Diffsol shrinks the step. Because of the missing error channel, recoverable evaluation failures are refused (`BN/dynamics.rs:1704–1707`).

### 2.5 Newer releases

- **0.17.0 (2026-09-26):** batched sensitivity and adjoint computation (breaking), diffsol-la 0.2 (breaking), a cuda-oxide backend, `SmallVec`/`SmallMat` in the tableaus and BDF.
- **0.17.1:** documentation build fix.
- Neither adds a scheme, a DAE capability or statistics fields. Moving would mean a diffsol-la major move on the `LinearSolver` trait the local faer and KLU solvers implement. *(Interface-checked, release notes.)*

---

## 3. CVODES and ARKODE (bundled but disabled)

### 3.1 CVODES *(Interface-checked)*

- **Binding work is minimal.** `wrapper.h` includes `cvodes/cvodes.h` under `USE_CVODES`, and `ParseSignedConstants` already covers the `CV` prefix. Enabling `sundials-sys/cvodes` yields bindings.
- **Capabilities:**
  - BDF (orders 1–5) or Adams (orders 1–12), with the Newton or fixed-point nonlinear solver;
  - `CVodeSetStabLimDet`, which detects BDF instability for oscillatory modes;
  - `CVodeSetJacEvalFrequency` and `CVodeSetLSetupFrequency`, which IDA lacks;
  - `CVodeSetConstraints`;
  - projection onto invariants (`cvodes_proj.h`, `CVodeSetProjFn`), relevant to conservation;
  - forward sensitivities and adjoints (`CVodeAdjInit`, `B`/`BS` variants);
  - `CVodeSetMonitorFn`, which only works with a monitoring build.
- **Limit:** ODEs only, so no algebraic states.
- **Interpretation:** CVODES would duplicate Diffsol BDF and `tsit45` for mass-free ODEs. Its distinct additions are Adams for non-stiff ODEs, stability-limit detection, projection and a mature adjoint. 7.8's `SUNNonlinearSolver_Auto`, which switches between fixed point and Newton based on a stiffness estimate, is not in 7.1.1.

### 3.2 ARKODE *(Interface-checked)*

- **No bindings.** `wrapper.h` has no `arkode` include, so the `arkode` feature links `libsundials_arkode` but generates no FFI. Using ARKODE needs project-owned bindgen over `arkode_{arkstep,erkstep,mristep,sprkstep,ls,butcher_*}.h`, which is the same pattern as `pse-petsc-sys`/`pse-ipopt-sys`.
- **7.1.1 capabilities:**
  - ARKStep: ERK, DIRK/ESDIRK and additive IMEX, with a mass matrix through `ARKodeSetMassLinearSolver`;
  - ERKStep;
  - MRIStep: MRI-GARK and IMEX-MRI-GARK. 7.2 adds adaptive multirate controllers, MERK and IMEX-MRI-SR;
  - SPRKStep: symplectic, not relevant to dissipative process models;
  - `ARKodeSetConstraints`, relaxation, `ARKodeSetAdaptController`, `ARKodeGetNumStepAttempts`, roots.
- **Limits:** the ARKODE mass matrix must be nonsingular, so diag(I,0) index-1 DAEs are out unless reduced to ODEs. The discrete ERK/ARK adjoint needs 7.3 or later, and Pre/Post step hooks need 7.7.
- **Interpretation:** ARKODE is applicable only to mass-free ODE models or reduced ODE subsystems. Multirate is attractive for process systems that couple fast subsystems (thermal, pressure) with slow ones (composition inventories), but only once algebraics are eliminated.

---

## 4. PETSc 3.24 TS

- **Local use** (`crates/pse-petsc-sys/bridge/pse_petsc.c`): TSPSEUDO only, through `TSSetType(TSPSEUDO)`.
  - Already bridged: `TSSetIFunction`/`TSSetIJacobian` (implicit F(t, u, u̇) with shift a), `TSSetPostStep`, `TSSetPreStage`, `TSSetFunctionDomainError`, `TSAdaptSetStepLimits`, `TSAdaptSetScaleSolveFailed`, `TSSetMaxStepRejections`/`MaxSNESFailures`.
  - Statistics: `TSGetStepNumber`, `TSGetStepRejections`, `TSGetSNESIterations`, `TSGetSNESFailures`.
  - *(Interface-checked.)*
- **Broader TS** *(Interface-checked, Context7 petsc_release docs)*:
  - **Implicit index-1 DAE integration** through `TSSetEquationType(TS_EQ_DAE_IMPLICIT_INDEX1)` with TSARKIMEX (fully implicit stages are needed when the mass matrix is not the identity), TSBDF (`-ts_bdf_order n`, adaptive dt, initial-guess extrapolation) and TSROSW.
  - TSROSW is linearly implicit (one linear solve per stage), works only for autonomous systems, and needs `-snes_lag_jacobian_persists` for Jacobian lagging.
  - The docs read do not show TSBDF varying its order automatically the way IDA does. Not verified.
- **`TSAdjoint`:** discrete adjoint "currently supported for `TSRK` explicit and `TSTHETA` (incl. BEULER, CN)". Examples include DAEs with discontinuities (ex9busadj). There is no BDF, ARKIMEX or ROSW adjoint.
  - TSTrajectory stores the forward pass (memory or disk); revolve checkpointing needs an external download that the local configure does not request.
  - Forward sensitivities exist for the same method family.
- **`TSEvent`:**
  - `TSSetEventHandler(nevents, direction[], terminate[], indicator, postevent)`. The post-event callback receives a forward or adjoint flag, so jump conditions can be applied in the adjoint pass.
  - `TSSetPostEventStep` and `TSSetPostEventSecondStep` control the first step sizes after an event.
- **Interpretation:**
  - Adding BDF, ARKIMEX or ROSW forward integration is mostly a type switch plus event and trajectory bridging.
  - The project-relevant gap PETSc fills is an adjoint across events. That is excluded in §25 for both local integrators, and PETSc offers it only with first- or second-order Theta methods.
  - TS has no counterpart to `IDACalcIC` (consistent initialization is the user's job), and its DAE BDF error control is less mature than IDA's.

---

## 5. Decision guidance for process dynamics (stiff, index-1, semi-explicit DAEs with events)

All items in this section are *Proposed*. The cited facts are *Interface-checked* above.

### 5.1 BDF versus ESDIRK versus multirate

- **Rule.** Variable-order BDF (IDA, Diffsol BDF) wins on long smooth segments. One-step L-stable ESDIRK or TR-BDF2 (Diffsol `esdirk34`, `tr_bdf2`) wins when events or scheduled changes are frequent. Every `IDAReInit` resets to order 1 with nst = 0 and forces a Jacobian and refactor, so with only a few steps between restarts BDF works at order 1–2 and never amortizes. ESDIRK keeps full order from the first step.
- **ESDIRK also suits** oscillatory or lightly damped modes, where BDF orders 3 and above lose A-stability and IDA has no stability-limit detection.
- **A priori features:**
  - number of scheduled changes plus authored events per horizon, against horizon / expected h;
  - presence of algebraic states, which forbids explicit methods and ARKODE;
  - an estimate of the Jacobian spectrum: Gershgorin bounds from the compiled sparse Jacobian, or a few power or Arnoldi iterations;
  - the ratio |Im λ| / |Re λ| for the dominant modes.
- **In-solve observations:**
  - steps per segment, already captured;
  - `IDAGetLastOrder`/`qcur` per segment, and Diffsol `order()`, both unbound;
  - netf at orders ≥ 3, as a sign of oscillation;
  - for `tsit45`, many error-test failures with h stagnating while the solution stays smooth, as a sign of stiffness.
- **Multirate (MRIStep)** applies only to mass-free ODEs and needs new bindings. Treat it as a later option for explicitly separated time scales.

### 5.2 Direct versus Krylov

- **Default to KLU.** Its symbolic analysis is reused across segments, and its numeric refactor is cheap.
- **Krylov pays off** only with a good preconditioner at large n. A Jacobi preconditioner is weak for process Jacobians dominated by strong off-diagonal coupling.
- **A priori:** n, nnz, and the predicted KLU fill (`klu_common` after `klu_analyze`: `lnz`, `unz`, flops).
- **In-solve:**
  - nli / nni (captured);
  - `IDAGetNumLinConvFails` (unbound);
  - KLU `rcond`/`condest` from `SUNLinSol_KLUGetCommon` (unbound). On the Diffsol side, `linear.rs` already computes these in its refactor policy but does not report them.

### 5.3 Forward versus adjoint sensitivities

- **Cost shape.** Forward costs O(n_p) extra solves sharing one factorization (simultaneous corrector). Adjoint costs one backward pass per functional plus checkpoint memory and, locally, a replay forward pass per earlier segment.
- **Choose forward** when n_p ≲ n_outputs × n_samples, which is typical for path constraints and output-matching fits, and whenever events are present (local adjoints refuse events).
- **Choose adjoint** for few functionals (an integral objective) with many interval parameters.
- **Observations:** `IDAGetAdjCheckPointsInfo`, a replay-steps counter, and `IDAGetSensStats`, so the choice can be tuned from history in the advisory layer.

### 5.4 Shooting versus collocation

- **Simultaneous FD/Radau** (local §13.4) handles unstable open-loop dynamics and path constraints with exact sparse KKT structure. It cannot take authored events, and its fixed mesh needs a priori resolution.
- **Multiple shooting** keeps adaptive error control and events and gives a smaller NLP, but needs a sensitivity for every node.
- **Single shooting** suits stable, short horizons only.
- **Features:** the sign of the dominant Re λ (instability favors collocation or multiple shooting), event presence (favors shooting), the number of path constraints relative to controls, and horizon length × stiffness relative to an affordable mesh.

### 5.5 Cheap trustworthy telemetry to add first

1. Per segment: `IDAGetIntegratorStats` (one call, ten fields), `IDAGetNumLinSolvSetups`/`IDAGetNumJacEvals`/`IDAGetNumLinConvFails`/`IDAGetNumStepSolveFails`.
2. On failure or at the end of a segment: the top-k entries of `|IDAGetEstLocalErrors · IDAGetErrWeights|`, which names the stiff or limiting states.
3. In the Diffsol `drive` loop: `order()` and `state().h` on the existing progress event.
4. `SUNContext_PushErrHandler` to fill `NativeTermination.message`.
5. Persist these with the trajectory result, so the advisory history layer has data.

---

## 6. Probes that would settle material doubts (described, not run)

1. **Order collapse at restarts.**
   - Question: does IDAS restart at order 1 and how long does it take to ramp up per segment on an event-heavy fixture?
   - Command: a temporary test calling `IDASolve(..., IDA_ONE_STEP)` and logging `IDAGetLastOrder`/`IDAGetLastStep`, run with `scripts/pse-env --native -- cargo t -p pse-backend-native --features idas <fixture>`.
   - Expected: order 1 after each `IDAReInit`, with mean order below 3 when there are fewer than about 20 steps per segment.
2. **ESDIRK versus BDF on the same fixture.**
   - Command: run the existing Diffsol benchmarks (`benches/benches/native_process`) with `method=esdirk34` against `bdf`.
   - Expected: lower `number_of_linear_solver_setups` and fewer steps per event for ESDIRK at high event density.
3. **Logging availability.**
   - Command: `grep SUNDIALS_LOGGING_LEVEL target/*/build/sundials-sys/*/out/include/sundials/sundials_config.h`.
   - Already observed from existing build output: `2`. A rebuild with `-DSUNDIALS_LOGGING_LEVEL=3` would need a patched or system SUNDIALS.

## Uncertainties

- The PETSc TS points come from documentation, not from source inspection at the 3.24.0 tag: the adjoint method list, whether TSBDF varies its order, and TSEvent details.
- Whether a newer `sundials-sys` exists on crates.io was not checked; only the GitHub repository head (0.6.1 in its `Cargo.toml`) was.
- The diffsol 0.16.2 statistics struct read from the registry includes per-cause setup counters. The source was taken as authoritative.