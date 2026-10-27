# SUNDIALS (IDAS, KINSOL, SUNLinSol_KLU)

Realizations considered: installed `sundials-sys` 0.6.2 (vendored, locally patched; bundles
SUNDIALS 7.1.1; features `kinsol`, `klu`, `build_libraries`, `static_libraries`); upgrade routes:
SUNDIALS 7.8.0 (released 2026-06-25) and 7.9.0 (2026-09-03, latest). `sundials-sys` 0.6.2
(2024-12-25) is still the newest crates.io release, so any newer native tree means replacing the
bundled tree inside the vendored crate or using its standalone-provider branch.
Research date 2026-10-27. Sources: upstream `CHANGELOG.md` and `include/{idas/idas.h,kinsol/kinsol.h,sunlinsol/sunlinsol_klu.h,sundials/sundials_linearsolver.h}`
at github.com/LLNL/sundials `HEAD` (cloned to scratch); upstream commits a7393b501 (#888) and
c20033eb0 (#669); installed headers under `vendor/sundials-sys/vendor/include`; the
repository's supplier notes.

## Types

### sundials/idas-quadrature-history-after-calcic
- **Need it serves:** correct quadrature (and quadrature-sensitivity) predictor history when IDAS
  integrates quadratures after `IDACalcIC` corrects the initial state, or after `IDAReInit`.
- **Recognizable code shapes:** a local patch that recomputes `phiQ[1]`/`phiQS[1]` after
  `IDACalcIC`; forward-recomputation mismatches between an original adjoint forward pass and its
  checkpoint replay.
- **Mechanisms and APIs:** from 7.8.0, `IDAQuadSetup` (static) evaluates the quadrature RHS and
  quadrature-sensitivity RHS at the first `IDASolve` step instead of inside `IDAInitialSetup`,
  so the history uses the post-`IDACalcIC` state whether `IDAQuadInit` precedes or follows
  `IDACalcIC`. A failing RHS returns `IDA_QRHS_FAIL`/`IDA_FIRST_QRHS_ERR` (or the QS
  equivalents) from `IDASolve` before any step is taken.
- **Obligations discharged:** the defect itself, both plain and sensitivity quadratures, and
  re-initialization.
- **Residual caller obligations:** none for the defect; checkpoint budgets, accepted-step budgets
  and KLU fixed-pattern accounting are separate integrations and are not in upstream.
- **Semantic conditions:** the upstream fix evaluates at first `IDASolve`, not inside `IDACalcIC`.
  A caller that observes counters or callback failures between `IDACalcIC` and `IDASolve` sees a
  different ordering, but callback failures still stop before the first physical step. Counter
  values (`nrQe`, `nrQSe`) increment at that later point.
- **Version / build conditions:** fixed in SUNDIALS 7.8.0 (commit a7393b501, 2026-05-24); absent
  in 7.1.1–7.7.0.
- **Limits and negative results:** inspection covers the commit diff in `src/idas/idas.c` and the
  changelog; staggered-sensitivity corner cases were not re-derived.
- **Evidence:** Interface-checked: commit a7393b501 diff; CHANGELOG 7.8.0 "Fixed a bug where IDAS
  would incorrectly compute the quadrature predictor when `IDACalcIC` was used".

### sundials/workspace-reporting
- **Need it serves:** reporting how much memory a created integrator, nonlinear solver or linear
  solver holds, for admission or accounting.
- **Recognizable code shapes:** `KINGetWorkSpace`, `KINGetLinWorkSpace`, `IDAGetWorkSpace`,
  `SUNLinSolSpace_*`, `*BBDPrecGetWorkSpace` calls after object creation; `lenrw`/`leniw` × word size.
- **Mechanisms and APIs:** the work-space functions return real and integer word counts for a
  constructed object (post-hoc, after allocation). `SUNLinSol_KLUGetCommon/GetSymbolic/GetNumeric`
  expose the underlying KLU objects, whose `Common->memusage`/`mempeak` give actual bytes.
  `SUNMemoryHelper_GetAllocStats_*` reports statistics only for allocations made through a
  `SUNMemoryHelper` (device/managed memory paths), not for internal `malloc`s of serial solvers.
- **Obligations discharged:** post-hoc measurement of integrator/solver state.
- **Residual caller obligations:** any before-allocation prediction; vector-header and wrapper
  overheads that the word counts omit.
- **Semantic conditions:** post-hoc only. Counts are in words, not bytes.
- **Version / build conditions:** **all work-space functions were deprecated in 7.3.0 and are
  scheduled for removal in 8.0.0** (commit c20033eb0; headers carry
  `SUNDIALS_DEPRECATED_EXPORT_MSG("Work space functions will be removed in version 8.0.0")`).
  No replacement API is named in the changelog. 7.7.0 added `ARKodeInit` for users who measure
  memory before simulation (ARKODE only).
- **Limits and negative results:** no SUNDIALS release through 7.9.0 offers a before-allocation
  size prediction or an allocation budget for IDAS/KINSOL. A `SUNLinSol_KLUGetMemUsage` function
  is not upstream (searched `sunlinsol_klu.h` at `HEAD`).
- **Evidence:** Interface-checked: upstream headers at `HEAD`, CHANGELOG 7.3.0 and 7.7.0 entries.

### sundials/adjoint-checkpoint-introspection
- **Need it serves:** bounding or inspecting IDAS adjoint checkpoint memory.
- **Mechanisms and APIs:** `IDAAdjInit(Nd, interp)` fixes the steps between checkpoints;
  `IDAGetAdjCheckPointsInfo` and `IDAGetAdjCurrentCheckPoint` read checkpoints post-hoc;
  `IDASetMaxNumSteps`/`IDASetMaxNumStepsB` cap steps per call.
- **Limits and negative results:** no upstream API through `HEAD` limits the checkpoint population
  or charges an accepted-step budget across calls. ARKODE's 7.3.0 `SUNAdjointStepper` /
  `SUNAdjointCheckpointScheme` classes target explicit Runge–Kutta methods, not IDAS.
- **Evidence:** Interface-checked: `include/idas/idas.h` at `HEAD`; CHANGELOG 7.3.0.

### sundials/release-fixes-7.2-to-7.9
- **Need it serves:** decide which release a bundled 7.1.x tree should move to, and which local
  workarounds or reused-instance patterns each release affects.
- **KINSOL:**
  - 7.3: Anderson-acceleration errors are captured; damping on the first Anderson iteration was
    fixed, except that a caller setting `KINSetDamping` and `KINSetDampingAA` to the same value
    already avoids the defect.
  - 7.5: fixes the Anderson-depth reinitialization regression introduced in 7.3. A caller that
    reuses instances (session caches, restarts) should therefore move to at least 7.5, not 7.3 or
    7.4.
- **IDA(S):** 7.6 resets linear-solver counters on reinitialization, adds the return value
  `IDA_TOO_CLOSE` (callers mapping return codes need a case for it) and counts constraint failures
  separately.
- **Shared fixes:** 7.8: CVODES, IDAS and KINSOL no longer leak after a failed `malloc`. 7.2:
  `SUNMatScaleAddI` wrote out of bounds for unsorted indices (not reached by callers that enforce
  canonical CSC).
- **Build:** 7.7 namespaces the CMake option names (for example `SUNDIALS_ENABLE_KLU`); 7.9 adds
  `SUNDIALS_ENABLE_DEPRECATED_WARNINGS`.
- **Limits and negative results:** the visible changelog does not name `N_VSpace` or
  `SUNMatSpace` in the 7.3 work-space deprecation.
- **Evidence:** Interface-checked: upstream `CHANGELOG.md` at `HEAD` (2026-10-27).

## Current applicability

### [CBO-028](../../../library_leverage/codebase/native-solver-integration.md#cbo-028) — Vendored supplier numerics whose premises may have aged (SUNDIALS part)
- **Types:** sundials/idas-quadrature-history-after-calcic, sundials/adjoint-checkpoint-introspection
- **Realization:** installed 7.1.1; upgrade route 7.8.0 or 7.9.0, delivered by replacing the native
  tree in the vendored `sundials-sys` 0.6.2 (no newer crate exists) and re-porting the ten-file
  integration patch.
- **Conclusion:** fits with conditions — the recorded revisit trigger has fired: upstream fixed the
  defect in 7.8.0, so the local quadrature-history hunk can retire at a supplier move.
- **What it would discharge / what remains local:** discharges the `IDAInitialQuadDerivatives`
  call after `IDACalcIC` in `idas_ic.c` and the helper factoring in `idas.c`. The checkpoint
  population/high-water accounting, backward-count exposure, accepted-step budget and
  `SUNLinSol_KLUFixedPattern` remain local integrations, since upstream has no equivalent.
- **Prerequisites, configuration or extension:** rebase the remaining hunks onto 7.8+/7.9; rerun
  the three named quadrature controls without the local hunk. The CMake option names used by the
  build script were deprecated (namespaced, e.g. `ENABLE_KLU` → `SUNDIALS_ENABLE_KLU`) in 7.7.0;
  check whether the build script needs them.
- **Semantic gaps against the CBO's preservation constraints:** quadrature history is now refreshed
  at first `IDASolve`, not inside `IDACalcIC`, so any registered control that reads counters
  between the two calls may change. Bit-reproducibility of every dynamics result changes with
  the supplier move (as the CBO notes), so receipt hash identity moves.
- **Unsettled premises and what would settle them:** whether the controls pass unchanged against
  7.8+ without the local hunk — probe request P2.
- **Evidence:** Interface-checked (upstream diff and changelog); application Proposed.

**Reviewer correction (2026-10-27).** The target is SUNDIALS 7.9.0, and it must be at least 7.5:
instances are reused (the session cache, `solve_with_restarts`), so 7.3 or 7.4 would hit the
Anderson-depth reinitialization regression (see `sundials/release-fixes-7.2-to-7.9`). `kinsol.rs`
already sets both damping values equal, avoiding the 7.3 damping defect. The move also gains
captured Anderson errors (7.3) and no leak after a failed `malloc` (7.8), requires mapping
`IDA_TOO_CLOSE` and re-verifying the 11 version-coupled Rust sites. The bit-reproducibility change
is acceptable only once native build identity frames the vendored patch content: today
`BackendExecution::build()` returns `None` for the dynamics adapters and the KINSOL fingerprint
frames version macros and declaration text, so a patch-only change moves no identity. The review
makes that framing a prerequisite. See the [comparative review](../../reviews/native-solver-integration.md#cbo-028).

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) — Prepaid accounting (SUNDIALS workspace part)
- **Types:** sundials/workspace-reporting
- **Realization:** installed 7.1.1 (functions present, not yet deprecated there); any move to 7.3+
  emits deprecation warnings, and 8.0 removes the functions.
- **Conclusion:** fits with conditions — usable only as post-hoc measurement, and on a deprecation path.
- **What it would discharge / what remains local:** post-hoc retained-size reporting of
  constructed KINSOL sessions (as currently used). The before-allocation charge (BBD bytes,
  native header bytes) stays local because no prediction API exists.
- **Prerequisites, configuration or extension:** for KLU-backed solvers, upstream
  `SUNLinSol_KLUGetCommon` → `Common->memusage` gives actual bytes without a local accessor,
  unless the local fixed-pattern mode deliberately refuses native accessors (it does, per the
  supplier notes).
- **Semantic gaps against the CBO's preservation constraints:** post-hoc numbers cannot satisfy
  "reservation before allocation". The 8.0 removal makes the current reporter non-durable.
- **Unsettled premises and what would settle them:** what SUNDIALS 8.0 offers instead — watch
  the 8.0 release notes.
- **Evidence:** Interface-checked; application Proposed.
