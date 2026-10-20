# Native and mathematics diagnostic capabilities

Supporting inquiry for the [principal review](../../reviews/design_review_pipeline-diagnostics-and-review-artifacts_2026-10-18.md); the [evidence overview](README.md) records integration and execution limits.

Source investigation for the pipeline diagnostics and offline review-artifact inquiry,
2026-10-18. Baseline: coordinator-specified `e9d24396c` plus the existing working tree.
This is bounded library evidence for Core 3.4 / process-simulator 1.5 review, not the
overall verdict. **Interface-checked:** declarations and implementation bodies below
were inspected. Existing consumer behavior is described from source, without new runtime
qualification. Tests, builds, services and timing measurements: **not_run**. Neither
`docs/library-utilization.jsonl` nor `docs/capability-maps` was consulted.

## Consumer baselines and evidence strength

`Cargo.toml` is the version/feature authority: Symbolica **3.0.1** with GMP/MPFR;
faer **0.24.4**, `std,sparse-linalg`; Clarabel **0.11.1**, `serde`; HiGHS Rust
**2.4.0** / `highs-sys` **1.15.0**; `scip-sys` **0.1.28**, with the native image
SCIP **10.0.2**. The image pins Ipopt **3.14.20**, MUMPS **5.9.1**, SPRAL
**2025.09.18** and oneMKL **2026.1.0** (`docker/solvers/Dockerfile`). These are build
inputs, not proof that a particular solve used a particular linear backend.

POUNCE **0.12.0** and FERAL **0.18.0** resolve together to maintained fork
`paul-heyse/pounce@cfa07f0d4089bbab756def9df25dbca740547f03`.
The inspected Cargo checkout contains its `pounce-common`, `pounce-feral`,
`pounce-linsol` and `vendor/feral` sources. SUNDIALS uses local
`vendor/sundials-sys` **0.6.2**, bundling **7.1.1** with project modifications;
Diffsol **0.16.2** and `diffsol-la` **0.1.1** are also local maintained sources.
Their added observers, setup-cause counters and replay/budget getters must not be
attributed to an unmodified published crate.

PETSc is **3.24.0**, MPIUNI, real/double, 32-bit indices, no OpenMP/CUDA, patched
by `docker/solvers/petsc-pse.patch`. Uno is revision
`805a27ecf802eec33d1aea85ae73efa8a24f71be`, patched by
`docker/solvers/uno-pse.patch`. Project C bridges in `pse-petsc-sys` and
`pse-uno-sys` define the actual exposed ABI. The local skills' captured Symbolica
3.0.0 and upstream solver references are discovery aids; actual consumer sources
take precedence. Current Context7 documentation was fetched for SUNDIALS, Ipopt,
PETSc, HiGHS, Clarabel and SCIP, then checked against pinned/local contracts.
Context7's generated SCIP REST-shaped examples are not C API contracts.

## Current signals and concrete extension candidates

All candidates are **Proposed**, without a claimed latency improvement. Different
solver classes need different signals; a single mandatory metric population would
erase useful distinctions.

| Producer / current source | Existing consumer signal and meaning | Useful candidate or remaining limit |
|---|---|---|
| Ipopt sequence bridge: `crates/pse-ipopt-sys/bridge/pse_ipopt_sequence.cpp`, `src/sequence.rs`; consumer `pse-backend-native/src/ipopt/sequence.rs` | `Statistics()->NumberOfEvaluations`, iteration and CPU/system/wall totals; timing tasks for overall algorithm, initialization, search direction, acceptable trial, scaling, symbolic factor, numeric factor and backsolve. Task records carry enabled/running/available state. Intermediate callback uses `get_curr_iterate(..., false, ...)` and `get_curr_violations(..., false, ...)`. | Join already available task/iterate records with demand records before adding counters. Ipopt's unscaled coordinates mean unscaled relative to the submitted NLP, which can itself be a normalized/reduced model. Task times are inclusive and nested; summing parent and children double-counts. These tasks do not establish MUMPS/Pardiso/SPRAL internal retry, refinement or allocation detail. |
| POUNCE: `pse-backend-native/src/pounce.rs`, `pounce/observed.rs`, `pounce/records.rs`; fork `pounce-common/src/observed.rs`, `pounce-feral/src/lib.rs`, `pounce-linsol/src/summary.rs` | Existing observations distinguish monolithic, Schur-F and Schur-S factor attempts, failures, interrupted operations and backsolve calls. `native_factor.v1` reports input/factor nonzeros, fill ratio, symbolic reuse, requested/actual ordering and availability. Warm-start/crossover records preserve block verdicts, residuals, complementarity, phases and decline causes. Summary includes inertia/pivot/fill information. | Actual internal numeric retry and actual refinement-step counts are explicitly unknown in `Observation::record`; do not infer them from `Solver::factor` calls or configured refinement bounds. A fork extension could emit inner attempt/refinement outcomes at the owning loop, preserving parent call and path. This requires a real producer contract, not renaming outer counters. |
| FERAL fork `vendor/feral/src/numeric/solver.rs`: `FactorStats`, `OrderingInfo`, `Solver::last_factor_stats` | Existing library snapshot includes matrix/factor nnz, fill ratio, inertia, minimum/maximum absolute pivot, symbolic reuse, scaling outcome, ordering selection/escalation, supernodes and largest front; static pivot perturbation count also exists in the struct. Snapshot is unavailable before an eligible factor and cleared on hard failures. POUNCE currently forwards a selected subset in its detail event. | Selected scaling/escalation/front/perturbed-pivot details could explain changed factor behavior without a fresh solve. Source documents snapshot construction as O(1); that is an implementation claim, not measured observer overhead. Pivot extrema are for the actual scaled/permuted/regularized matrix, not physical variable sensitivities or a condition number. Avoid stale cumulative summaries on retained applications. |
| KINSOL: `pse-backend-native/src/kinsol.rs`, `kinsol/linear_work.rs`; patched SUNDIALS KLU | Existing final getters include nonlinear/function/Jacobian counts, backtracks, beta-condition failures, linear iterations/failures, preconditioner and Jv work, last linear flag, function norm and step length. Patched `SUNLinSol_KLUGetWork` distinguishes issued/completed setup, symbolic, numeric factor, refactor, rcond, condest and solve; `SUNLinSol_KLUSetObserver` brackets actual phases. | Preserve direct/Krylov applicability: zero Krylov iterations on a direct solve is not evidence of no linear work. `KINGetNumLinFuncEvals` counts residual calls made by the default difference-quotient routines, not all callback work. Existing baseline/delta checks refuse reset/overflow/absent-source interpretations. Residual norm and step length use KINSOL's supplied scaling; report that scale before comparing runs. |
| IDAS: `pse-backend-native/src/dynamics/idas.rs`; `vendor/sundials-sys/vendor/include/idas/idas.h`, `src/idas/idas_io.c` | Current source collects steps, residual/nonlinear/linear/setup/Jacobian/Jv/preconditioner work and failures, root evaluations, initialization backtracks, sensitivity/quadrature counters, order/step/internal time, checkpoint usage and replay/accepted-budget steps. It distinguishes initialization's artificial step from actual integration. Backward/replay work has separate owner paths. | **Unused in this consumer:** `IDAGetErrWeights`, `IDAGetEstLocalErrors`, `IDAGetTolScaleFactor` (no occurrences in `dynamics/idas.rs`). The vector getters copy `ida_ewt`/`ida_ee` with `N_VScale`; scalar getter copies `ida_tolsf`. A bounded weighted local-error projection could identify difficult state coordinates. Sample at valid owner returns, record accepted-step/error-state provenance and algebraic-error suppression. O(n) vector copy/projection requires admitted buffers; it is not a physical global-error certificate. |
| Diffsol: `vendor/diffsol/src/ode_solver/mod.rs`, `bdf.rs`; consumer `dynamics/integrator.rs`, `dynamics/linear.rs` | Existing `OdeSolverStatistics` and `live_statistics_for` include steps, error-test and nonlinear failures/iterations and linear setups classified by checkpoint, first/second convergence failure, error-test failure and successful-step cause. The project faer/KLU owners distinguish symbolic/factor/backsolve work and typed factor failures. Statistics are emitted with generation and segment context. | Use setup-cause distributions to distinguish checkpoint reconstruction from solver difficulty. A setup is not automatically a symbolic analysis, fresh factor, or linear iteration. BDF/RK/explicit methods have different applicable work. Checkpoint restore can reinitialize the linearization and trigger work; reconstructing a trajectory is not a cost-free read. Do not combine replay and forward accepted steps into one unexplained total. |
| faer 0.24.4 via project `dynamics/linear.rs`, `conditioning.rs`, `square_response/actions.rs` | Project owners already bracket symbolic LU, numeric factor and substitutions, admit scratch and mark factor validity/failure. `FaerLu` checks pattern equality and nonfinite outputs; a symbolic factor survives changing numeric values within that pattern. | Expose the existing owner status/extent before assuming faer supplies a global solver telemetry object. A pivot/rank finding belongs to its actual factor operation; faer LU completion alone is not a nonsingularity or conditioning certificate. Exact scope of additional sparse factor internals remains unqualified. |
| HiGHS: `pse-backend-native/src/highs.rs`, `highs/diagnostics.rs` | Typed `Highs_getInfoType` dispatch preserves native integer/int64/double metrics; existing callback data carries simplex/IPM progress, nodes and incumbent information. Explicit diagnostic APIs already support IIS, basis inverse, presolved model, ranging and relaxation/fixed-LP inquiries. | Information getters are observation; IIS/relaxation/pricing/ranging operations may do additional analysis or solve a copied model. Label that work separately. Native LP/QP column and row indices require presolve/reconstruction attribution before presenting model names. No inspected public info field establishes universal factor/refinement counts. Native bundled version and actual linked archive should accompany an external pack. |
| Clarabel: `pse-backend-native/src/conic.rs`; 0.11.1 `solver/implementations/default/info.rs`, `solver/core/solver.rs`, `timers/timers.rs` | Existing termination callback and final projection contain mu, sigma, step length, primal/dual costs and residuals, infeasibility residuals, absolute/relative gap, kappa/tau ratio, iterations, solve time and linear solver name/thread/direct/nnzA/nnzL. | Public `solver.timers: Option<Timers>` exists, but `Timers` keeps nested timer map private and exposes `total_time`/`print`, not a structured subtree reader in this release. Detailed phase export needs a library access change or owner-side instrumentation, not parsing printed text. Linear nnz is not a factor-call count. Cone coordinates, especially PSD `svec`, require their actual transform. |
| SCIP: `pse-backend-native/src/scip.rs::selected_plugin_statistics` | Existing selected plugins: dualinfer, pseudoobj, gomory, nonlinear and Ipopt NLPI. Source records stage, presence, reset policy, timing and native counters. It explicitly warns NLPI initialization scope can include merged worker statistics. | Keep absent plugin separate from stage-inapplicable or unqueried. Preserve cumulative/reset semantics; never manufacture per-attempt deltas. `SCIPgetNLPs` means number of LPs, not nonlinear programs; `SCIPgetNLPIterations` means LP iterations. Deeper selected NLP/subsolver inquiry must respect its stage and copy/worker ownership. |
| PETSc: `pse-petsc-sys/bridge/pse_petsc.c`; consumer `petsc/logging.rs` | Existing project getters expose SNES reason/iterations/function evaluations/linear-iteration total/function norm; latest KSP iterations/reason; TS steps/nonlinear iterations/rejections/failures/time/dt. A separately owned selected log handler records 13 events including KSPSetUp, PCSetUp/Apply, SNES line search and TS function/Jacobian work. | Event records preserve registered/activated/active-depth/entered-call states and completed time/flops. They are inclusive native event accounting, not exclusive phase time. Latest KSP count and SNES cumulative linear count have different populations. Additional monitors must run on serialized native owner and describe residual norm/preconditioning convention. No MPI/GPU conclusions apply to this configured route. |
| Uno: `pse-uno-sys/bridge/pse_uno.cpp`, consumer `pse-backend-native/src/uno.rs` | Existing bridge exposes iterations/evaluation/subproblem counts, objective, feasibility/stationarity/complementarity, CPU seconds, native acceptable/trial checkpoints, materialization count/extent and HiGHS feasibility-call/iteration statistics. | Consumer explicitly states private subproblem iteration detail unsupported. A native subproblem/linear breakdown needs an upstream/fork bridge with genuine parentage. CPU seconds are not elapsed seconds; feasible/restoration subproblem counters are not original-NLP iterations. |

## Causal and coordinate implications

The evaluator is already a useful producer, not merely a solver callback counter.
`pse-math/src/assembly_observation.rs` separates demand, completion/failure,
actual evaluation, cache hit/projection and miss causes (empty, invalidated,
changed input, derivative-order upgrade). It distinguishes value/first/second
achievement, directional and transpose/Hessian actions, and static product census.
`pse-kernels/src/lib.rs::ProviderObservation` records actual provider calls,
completion/failure, elapsed duration and requested order. The assembly completion
path in `assembly.rs` explicitly retains current provider effects: a cache hit cannot authorize
skipping effects whose owner has not certified reuse. Candidate explanations
should join these existing records to solver-demand phases rather than count
Symbolica derivative construction as runtime numerical evaluation.

Useful relationships are conditional hypotheses: repeated order upgrades plus
new evaluation can explain derivative work; repeated checkpoint-caused setup can
explain reconstruction work; numeric factors without symbolic analyses can
establish a reuse pattern; failed factors followed by new ordering/scaling can
identify a recovery path. Counts and ordering support these explanations but do
not establish elapsed-time benefit or prove that the numerical model caused the
observed slowdown. Preserve an explicit unresolved link when parents, selection
or detailed records are unavailable.

`pse-math/src/derived.rs` defines original coordinates/constraints,
normalization identity, incidence and reconstruction contracts.
`pse-runtime/src/workflow/modeling/diagnostic_source_map.rs` exposes semantic
occurrence/body/source identities, selected free coordinates, frozen physical
scales and source ranges, and explicitly supplies no native dual/stationarity
pullback. A native factor pivot position, presolved row, cone slot, integrated
state, reconstructed variable and authored source member are different
coordinates. An explanation needs the particular transform/reconstruction and
its validity; nearest-index or shared-body matching is insufficient. A physical
unit is meaningful only after this mapping. Native residual and convergence
quantities must retain their norm/scaling/constraint-population semantics.

FERAL's `Solver::with_static_pivot_threshold` documentation makes an especially relevant
distinction: static pivot perturbation factors `A + Δ`, and reported inertia is
for that perturbed matrix. Any original-model second-order interpretation must
establish the additional contract; exposing the native inertia count alone does
not establish it.

## Safe observation and unanswered investigation questions

Read scalars at owner-thread safe returns or documented callbacks while handles
are alive. Keep callback storage stable, contain Rust panic/C++ exceptions, and
clear observers before destruction. Existing KLU, Ipopt, POUNCE and PETSc bridges
already demonstrate these patterns. Selected observation failure must remain
separate from the numerical outcome. Vector snapshots require bounded allocation
and copying; fresh derivative checks, condition estimates, IIS and replay require
explicitly identified additional numerical work. No cost or noninterference
measurement was performed in this investigation.

Questions for the overall design review:

1. Can an analysis distinguish issued/completed/failed work, cumulative lifetime,
   retained-attempt deltas, generation resets and missing collection at every join?
2. Which model/state mappings suffice for local-error contributions, reduced KKT
   factors, presolved LP rows, Schur blocks and transformed cone coordinates?
3. Which native details materially change a user's next action: inner factor
   retries/refinement, IDAS local-error contributions, FERAL scaling/order
   escalation, or Clarabel phase breakdown? Select by concrete diagnosis.
4. Can forward/replay/backward/checkpoint and current-provider-effect work remain
   independently explainable without retaining every iterate/vector?
5. Does an offline pack embed these semantics, selected observations, exact
   consumer/fork identity, availability and coordinate maps? URLs alone cannot
   make an offline pack self-contained. Snapshot the small relevant declarations
   or semantic extracts subject to the pack's publication policy; no runnable
   replay environment is implied.

Search limits: inspected the named consumer files, pinned declarations and
bounded implementation regions; did not audit every solver plugin, every FERAL
inner loop, every Symbolica evaluator, native binary symbols or runtime-loaded
library. A missing field in these routes means a scoped gap, not proof that the
whole upstream library cannot supply it. The two unused IDAS vector getters are
a specific source-search finding, not a runtime qualification claim.

## Upstream URLs for the reviewed capability families

- [Ipopt 3.14.20 interfaces](https://github.com/coin-or/Ipopt/blob/releases/3.14.20/doc/interfaces.dox),
  [timing statistics](https://github.com/coin-or/Ipopt/blob/releases/3.14.20/src/Algorithm/IpTimingStatistics.hpp).
- [POUNCE fork at the consumed revision](https://github.com/paul-heyse/pounce/tree/cfa07f0d4089bbab756def9df25dbca740547f03),
  [FERAL factor snapshots](https://github.com/paul-heyse/pounce/blob/cfa07f0d4089bbab756def9df25dbca740547f03/vendor/feral/src/numeric/solver.rs).
- [SUNDIALS 7.1.1 KINSOL API](https://github.com/LLNL/sundials/blob/v7.1.1/include/kinsol/kinsol_ls.h),
  [IDAS optional outputs](https://github.com/LLNL/sundials/blob/v7.1.1/src/idas/idas_io.c),
  [IDAS usage semantics](https://sundials.readthedocs.io/en/v7.1.1/idas/Usage/SIM.html).
- [Diffsol upstream](https://github.com/martinjrobins/diffsol),
  [faer 0.24.4 sparse solvers](https://docs.rs/faer/0.24.4/faer/sparse/linalg/solvers/index.html),
  [Symbolica 3.0.1 evaluator APIs](https://docs.rs/symbolica/3.0.1/symbolica/evaluate/index.html).
- [HiGHS callbacks](https://github.com/ERGO-Code/HiGHS/blob/master/docs/src/callbacks.md),
  [highs-sys 1.15.0 release source](https://docs.rs/crate/highs-sys/1.15.0/source/),
  [Clarabel 0.11.1 info](https://github.com/oxfordcontrol/Clarabel.rs/blob/v0.11.1/src/solver/implementations/default/info.rs),
  [Clarabel timer API](https://github.com/oxfordcontrol/Clarabel.rs/blob/v0.11.1/src/timers/timers.rs).
- [SCIP 10.0.2 solving-statistics API](https://scipopt.org/doc-10.0.2/html/scip__solvingstats_8h.php),
  [SCIP NLPI interfaces](https://scipopt.org/doc-10.0.2/html/scip__nlpi_8h.php).
- [PETSc 3.24.0 SNES interface](https://github.com/petsc/petsc/blob/v3.24.0/include/petscsnes.h),
  [PETSc logging interface](https://github.com/petsc/petsc/blob/v3.24.0/include/petsclog.h),
  [consumed Uno revision](https://github.com/cvanaret/Uno/tree/805a27ecf802eec33d1aea85ae73efa8a24f71be).

These URLs identify upstream evidence. Project-specific fork additions and actual
projection semantics are established by the local sources cited above; upstream
documentation alone cannot certify the patched consumer route.
