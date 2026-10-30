# Native solver integration

This page covers `pse-backend-native` and the `pse-*-sys` crates, the vendored suppliers under
`vendor/`, the solver prefix patches under `docker/solvers/`, and the native dynamics and square-root
adapters (IDAS, KINSOL, Diffsol). Contracts are owned by
[§15–§18](../../authoritative_design/sections/numerical-execution.md#section-15). The baseline is
on the [entry page](README.md). Following the
[supplier notes](../../../vendor/native-supplier-notes.md), vendored accounting, checkpoint
budget and fixed-pattern hooks are classified as integration, not as core-solver forks.

**System picture.** A static adapter table (`execution::LINKED`) maps each backend to one adapter
over one callback failure boundary (`callback::CallbackState`) and independent original-space
qualification. Representation runners (NLP, roots, coefficients, cones, dynamics) drive the adapters.
Consumers are `pse-runtime` workflows (staged steps, initialization, fitting, accuracy evidence,
shooting), implicit selection in `pse-compiler`, and `recycle.rs`.

The area is strongly library-led. No project integrator or project solver exists:

- **Presolve, derivative checks and KKT backsolve** come from POUNCE crates; **tear selection** is a
  HiGHS MILP with petgraph as an independent witness; **recycle** is a KINSOL fixed-point map.
- **HiGHS** use includes IIS, ranging, Hessians, bases, callbacks and lexicographic objectives. **SCIP**
  use includes exact solving, reoptimization, IIS, indicator and SOS constraints, and concurrency.
- **IBEX** contractors compose the validated proofs. **FERAL** supplies certified inertia and
  Hager–Higham condition estimates.
- **IDAS and KINSOL** use analytic Jacobians and products, with stepping, error control, roots,
  checkpoints and consistent initial conditions owned by the libraries.
- Faer scratch is requested from the library and admitted before allocation (`sparse_lu.rs`).

Legitimately custom: outcome truthfulness (original-coordinate quality, ray verification,
trial-failure classification), per-solver status tables, exception-shielded C++ bridges, and the
hub-and-spoke cone maps with independent certificate recheck. Opportunities cluster around
**process-wide library state, arithmetic policy across the stack, repeated preparation (IBEX, KLU,
Diffsol, IDAS), dense diagnostics and admission over sparse operators, supplier currency, and
mechanical native boundaries**, not around reinvented solvers.

Vendored and patched delta classification, as read from the supplier notes, the delta records and the
patches (patch hunks were read for Uno, PETSc and the SUNDIALS sites cited; others by record):

| Tree | Delta | Class | Entry |
|---|---|---|---|
| `vendor/faer`, `vendor/faer-traits` | 3 + 1 files: native fields select `pulp::Scalar`, nonfused `*_e_*` operations, private GEMM dispatch deleted | Numerical and execution policy fork | [CBO-116](#cbo-116) |
| `docker/solvers/uno-pse.patch` | 374 lines against Uno `805a27ec`, HiGHS subproblem solver | Mixed: scope, counter, readback and exception routes are integration; four declared numerical changes | [CBO-028](#cbo-028) |
| `docker/solvers/petsc-pse.patch` | 21 lines in `petscstring.h` | Defect fix; upstream candidate | [CBO-028](#cbo-028) |
| SUNDIALS `kinsol_aa.c`, `sundials_nvector.c` | Destructor uses allocated depth; clone-array cleanup count | Upstream defects held only locally | [CBO-028](#cbo-028) |
| SUNDIALS `sunlinsol_klu.c/.h` | Opt-in fixed pattern, work counters, phase observer, memory getter | Integration: reuse plus diagnostics | [CBO-126](#cbo-126) |
| SUNDIALS `idaa.c`, `idaa_io.c`, `idas.c`, `idas_io.c`, `idas_impl.h`, `idas.h` | Checkpoint population limit, accepted-step budget, replay step limit | Admission and accounting hooks | none |
| `vendor/sundials-sys` wrapper | `build.rs` freshness, namespaced 7.9 options, bindgen | Build and provenance; native tree is 7.9.0 with ten locally modified files | none |
| `vendor/diffsol`, `diffsol-la`, `diffsol-nl`, `suitesparse-sys` | Manifest path selection, `build.rs` freshness | Build and provenance; numerical sources unchanged | [CBO-027](#cbo-027) |
| POUNCE and FERAL fork (`paul-heyse/pounce`) | 15 commits ahead, 441 behind upstream | Mostly admission and diagnostics, by commit subject only | [CBO-121](#cbo-121) |
| `symbolica` | Bounded square-free and VAS-CF overloads with callbacks | Admission integration | [CBO-021](mathematics-and-compilation.md#cbo-021) |
| `paste`, `pest_derive`, `py_literal` | Input-channel refusal, inlined grammar | Assurance integration | [CBO-061](developer-tooling.md#cbo-061) |
| `scripts/surrealdb-source/*.patch` | Connection ownership, admission, SSI controls, rollback | Mostly integration; one changes transaction outcome semantics | [CBO-070](persistence-and-transport.md#cbo-070) |

The size of `vendor/deltas/sundials-sys.patch` (about 172,000 lines) overstates the burden: it is
measured against the published 7.1.1-bundling wrapper archive, and the local native integration
relative to 7.9.0 is the ten files above.

**Coverage on this page.** Mechanisms are *Interface-checked* by static reading; every value claim is
*Proposed*; nothing was built, run or measured. Read closely: `root_isolation.rs` and
`native/root_isolation.cpp` (IBEX itself only at symbol and header level, its `.cpp` sources being
unavailable), the Uno patch and bridge solve path, the HiGHS lifecycle, all production code of
`dynamics/idas.rs`, `integrator.rs`, `integrator/adjoint.rs`, `linear.rs`, `layout.rs`, `anchored.rs`
and `kinsol.rs`, and the SUNDIALS integration sites cited. Read representatively: `scip.rs`, `pounce.rs`,
`tnlp.rs`, `presolve/*`, `ipopt*`, `petsc*`, `kkt/*`, `conditioning`, `square_response`, `solve/*`,
`docker/solvers/build.sh`. Skimmed: `conic*` and `execution/pounce_convex` (heads), PETSc bridge bodies,
`kinsol/restarts.rs`. Not read: test modules (names only), `solve.rs` line by line, the `HiGHS`
`solve_inner` body, the Diffsol 0.16.2 to 0.18.0 source diff beyond its file list, and POUNCE fork
commit bodies or hunks.

<a id="cbo-030"></a>

## CBO-030 — Process-wide library state makes independent solves exclude each other

- **Operation and consumers.** Any parallel workload (16 lanes, studies, sweeps, implicit selection
  inside evaluation, accuracy evidence) that reaches IBEX, PETSc, concurrent SCIP or Uno.
- **Current mechanism.** Each library's global state has one process-level gate:
  - IBEX: `root_isolation.rs` `static IBEX: Mutex<()>`, stated as "process-global random/contractor
    state". Every proof, refinement and point enclosure runs under it.
  - PETSc: `petsc.rs` `static PROCESS: Mutex<Phase>`; initialization is once per process.
  - SCIP: `scip.rs` `CONCURRENT` plus `CONCURRENT_FAILED`, for one process-static thread pool. A failed
    teardown leaks a `Quarantine` that holds the gate and refuses later solves, serial ones included.
  - HiGHS: `highs_lifecycle.rs` `LIFECYCLE: RwLock<()>`. `exclusive()` makes Uno exclude every HiGHS
    session in the process, because Uno needs the serial scheduler and `Highs_resetGlobalScheduler`.
  - Uno: a fifth gate, `ProcessGate`/`ProcessPermit` in `pse_uno.cpp`, nested inside the Rust
    exclusion.

  `execution::acquire_quiescent` now shares the acquire-and-quiesce protocol across PETSc, SCIP and both
  HiGHS paths. The residue is `root_isolation::acquire_library` (about 90 lines, its own error type, an
  optional `Execution`) and the C++ permit.
- **Why it matters.** The stated parallelism is lane-level. A gate turns one slow library into a
  process-wide throughput ceiling and binds failure scope: one uncertain teardown disables a capability
  until process exit. A process boundary could make hard kill a real cancellation for libraries without
  interrupts.
- **Patterns.** LOP-X018, LOP-08, LOP-09.
- **Evidence.** The gate code above; `execution::acquire_quiescent`; `highs_lifecycle::{read, exclusive,
  reset}`; `scip::ConcurrentAdmission::acquire`. `nm -AC` of the prepared `libibex.a` shows unsynchronized
  `RNG::{x,y,z,seed}`, `Expr` function-local statics, name counters, `StaticTimer`, `CtcAcid::nbvarstat`
  and FILIB `extended_err`.
- **Footprint.** Gates and same-file callers inspected. IBEX is symbol-level only. Contention under real
  workloads, the job and worker model, and PETSc and SCIP beyond their comments are unread.
- **Preservation constraints.** Callback containment, typed-cause precedence, cancel and deadline meaning,
  charging order, one BLAS and OpenMP per process, no hidden fallback. Solver callbacks evaluate
  in-process programs, so a split suits only callback-free libraries first; IBEX receives a whole DAG.
- **Counterevidence and uncertainty.** A subprocess costs startup and a new lifecycle. The IBEX lock is
  justified for covering ([CBO-117](#cbo-117)) but unproved for point arithmetic ([CBO-118](#cbo-118)),
  so narrowing may beat isolating. A newer HiGHS may offer instance scheduling. No contention data exists.
- **Status.** Current mechanism.
- **Research questions.** Which IBEX statics does expression construction write? Does a newer HiGHS avoid
  the global scheduler? Is PETSc re-initialization per helper process viable?

<a id="cbo-116"></a>

## CBO-116 — The faer arithmetic fork: enforced per supplier, with no stack-wide statement of coverage

- **Operation and consumers.** All consumed binary64 faer work: sparse and dense LU, rank and covariance
  SVD, PSD EVD, matrix products. Consumers are response actions, arclength, fitting covariance and Diffsol.
- **Current mechanism.** `vendor/faer` and `vendor/faer-traits` select `pulp::Scalar` (one lane) and delete
  the private GEMM dispatch, guarded by the controls that `vendor/faer-arithmetic.md` describes. That note's LLVM-inspection command
  names `scripts/faer_arithmetic.py`, which `600d4df30` deleted; `scripts/probes/faer-nonfused.rs` and
  `src/arithmetic_tests.rs` remain, and whether the inspection control survives elsewhere was not established. FERAL meets the same no-FMA policy with SIMD retained (runtime `pulp`
  dispatch, separate mul and sub kernels, `NumericParams::fma` false), so SIMD loss is faer-specific.
  Enforcement is per supplier and wider than faer: `-ffp-contract=off` in `pse-backend-native/build.rs`,
  `pse-ipopt-sys/build.rs` and `native_compiled.rs`, PETSc flags in `native/providers.rs`, and the
  `target-cpu=native` ban. No single statement says which suppliers are covered. `matrixmultiply` is in the lock
  through `ndarray` and `nalgebra`, the note excludes the LM route by inspection, and the egobox route has no
  statement. No such flag was found under `docker/` or `scripts/`, so HiGHS, SCIP and MUMPS are unknown.
- **Why it matters.** This is a real arithmetic fork, not a diagnostics patch, and faer alone paid for the policy with SIMD lanes. The controls do not measure throughput, so the cost is unrecorded, and the hold makes any faer move a re-audit. The note instructs rerunning its controls on a supplier update while naming a deleted script. A lane-preserving realization might meet the actual requirement with a smaller fork (*Proposed*).
- **Patterns.** LOP-X006, LOP-11, LOP-10.
- **Evidence.** `vendor/deltas/faer-traits.patch` (head), `vendor/faer-arithmetic.md`, the `faer` pin
  reason, `[patch.crates-io]`, commit `74cc0636c`, FERAL `NumericParams`. A search for production bit
  comparison on faer output (`to_bits` over the response, `sparse_lu`, arclength and `pse-math` diagnostics
  paths) found only input hashing and memo keys.
- **Footprint.** Policy, manifests and one patch head read. The original rationale (no ADR text found
  beyond ADR-0191's scalar witness), horizontal reductions, and `pse-runtime` or test bit assertions are unread.
- **Preservation constraints.** No FMA contraction in the consumed closure, faer `Par` and algorithms,
  stable rank and pivot verdicts within tolerance, and arithmetic source in native build identity.
- **Counterevidence and uncertainty.** Cross-CPU identity may be deliberate, and a host change would then
  alter lane-dependent results. Faer 0.24.4 is still newest, so rebase pressure is nil. Small
  factorizations may gain little.
- **Status.** Current mechanism.
- **Research questions.** Does the promise need "no FMA" or "same result on this host"? Which faer kernels
  have lane-dependent reductions? What does a throughput comparison show on supernodal LU and fitting?

<a id="cbo-121"></a>

## CBO-121 — The POUNCE fork is far behind upstream

- **Operation and consumers.** NLP, presolve, QP, sensitivities and FERAL factorization for
  `pse-backend-native`, `pse-math` and `pse-structural`, all at one fork revision (`141a9d00`, recorded in
  the workspace pins).
- **Current mechanism.** GitHub compare of `jkitchin/pounce:main` with the fork reports status diverged:
  15 commits ahead, 441 behind, 108 files, merge base 2026-09-19. By file: 79 in that fork's `vendor/feral`, 11
  `pounce-algorithm`, 5 `pounce-nlp`. By subject, the local commits are mostly admission and accounting and
  diagnostics. Behavioural exceptions: rebinding Schur structure when curvature sparsity changes, admitting
  regular sparse MC64 factors, and preserving outward constant intervals through FBBT. Upstream subjects
  include FERAL 0.19 and 0.20 (one cited as fixing a collocation stall), block-parallel KKT factorization,
  an inertia-trust correction, bound-multiplier handback in presolve and convex infeasibility fixes.
- **Why it matters.** The upstream work has not been assessed against this deliberate pin, and local hooks may need
  reapplying later. Presolve multiplier commits may bear on [CBO-120](#cbo-120). A rebase would invalidate
  source-derived bounds such as `pse-structural` `allocation_bound` (see [CBO-001](cross-cutting-mechanisms.md#cbo-001)).
- **Patterns.** LOP-X004, LOP-11, LOP-10.
- **Evidence.** `gh api` compare (status, counts, file list, subjects) and the local checkout git log.
- **Footprint.** Subjects and paths only; no commit body or hunk was read, so every classification is by subject.
- **Preservation constraints.** Storage bounds, factor snapshots and reset controls consumed by admission and
  diagnostics; one coherent revision across the POUNCE family and FERAL.
- **Counterevidence and uncertainty.** Upstream moves daily, so a rebase is a moving target needing numerical
  requalification. A frozen qualified solver is legitimate, and upstreaming hooks needs the upstream
  maintainer's agreement. Upstream performance claims are unverified here.
- **Status.** Current mechanism.
- **Research questions.** Which of the 15 could be proposed as public hooks? Which of the 441 change a
  qualified numeric result or resolve a locally recorded limitation?

<a id="cbo-028"></a>

## CBO-028 — Supplier deltas with numerics inside integration: Uno, and upstream-eligible fixes

- **Operation and consumers.** The pinned supplier numerics under every dynamics, QP and NLP solve.
- **Current mechanism.**
  - **SUNDIALS.** The native tree is 7.9.0 and the local quadrature workaround is gone (`vendor/native-supplier-notes.md`).
    Two local fixes are real upstream defects, not integration: `kinsol_aa.c` (destructor uses allocated
    depth) and `sundials_nvector.c` (clone-array cleanup). Build fingerprints now frame `sundials-source.txt`
    (`sundials_profile.rs`), and `build-source.rs::authenticated` refuses stale delta receipts.
  - **PETSc.** The 21-line `petsc-pse.patch` fixes `petscstring.h` bound handling.
  - **Uno.** `uno-pse.patch` (374 lines, revision `805a27ec`) is mostly scope, accounting and readback
    integration, with four declared numerical changes: forwarded HiGHS tolerances, zero QP regularization,
    a feasibility-LP seed and basis, and dense L-BFGS materialization (`docker/solvers/uno-binding.md`).
    Materialization applies Uno's operator to every coordinate column per new iterate: `n` Hessian-vector
    products and `n²` storage, with admission formulas `32d² + 48d` that mirror it.
- **Why it matters.** The defect fixes and the seed correct upstream behaviour. A rebase carries numerics and
  integration together, and a rival QP route could remove the largest numerical hunk.
- **Patterns.** LOP-11, LOP-10, LOP-X004.
- **Evidence.** The supplier and binding notes, the whole Uno patch, `pse_uno.cpp` scope and solve, the full
  PETSc patch. Assessment of 2026-10-27: Uno v2.9.2 (2026-10-21) changed the HiGHS QP path and seven commits
  touch patched files, but the seed addresses HiGHS 1.15.0 QP initialization, so it probably survives (unconfirmed).
- **Footprint.** Upstream status of the two SUNDIALS fixes after 7.9.0 is unchecked. `uno.rs` and `uno/callback.rs` were outlined only.
- **Preservation constraints.** Registered controls and tests, serial determinism, original deadlines,
  terminal cause order, receipt hash identity.
- **Counterevidence and uncertainty.** Materialization reflects Uno's operator interface. The maintainers
  retained the composition after an Interface-checked inquiry on 2026-10-21 (v2.9.1). A supplier move
  changes identity and needs requalification, which is a cost, not a barrier. Value depends on Uno use at size.
- **Status.** Current mechanism, with a recorded revisit condition.
- **Research questions.** Are the two SUNDIALS fixes in releases after 7.9.0? Does a newer Uno change QP warm
  start or L-BFGS handling with HiGHS? Does another QP route avoid materialization (a BQPD candidate is noted)?

<a id="cbo-027"></a>

## CBO-027 — Diffsol's infallible traits force panic control flow and local workarounds

- **Operation and consumers.** Delivering cancellation, deadline, evaluation failure and step-limit outcomes
  through Diffsol, and running Newton linear solves, for every Diffsol trajectory and adjoint.
- **Current mechanism.**
  - `Shared::abort` stores the failure and unwinds with a private payload; `contain` classifies it and the
    `segment!` macro catches once to read statistics.
  - `dynamics/linear.rs` (`FaerLu`, `Klu`) replaces Diffsol's solvers. `set_linearisation` and `set_sparsity`
    return nothing, and the vendored `faer/sparse_lu.rs` and `suitesparse/klu.rs` `expect` on failure (dense
    `faer/lu.rs` does not on that path), so `linear/terminal.rs` is a thread-local first-cause channel.
  - Workarounds: `state_mut()` repairs a zero BDF sensitivity coefficient, `adjoint_start` rebuilds the state
    to avoid a step-size probe, `Replay` wraps `OdeSolverMethod`, and `Counted` exists for statistics.
  - Pin 0.16.2; the Diffsol trees differ from upstream only in manifest and `build.rs`.
- **Why it matters.** Ordinary outcomes travel as panics through library frames, which is part of why unwinding is
  a workspace invariant. The larger half is the operator callbacks, not the solvers. Do not budget this as line
  deletion: `linear.rs` also carries observation, admission of faer scratch and fixed-pattern refactor policy.
- **Patterns.** LOP-10, LOP-11, LOP-03.
- **Evidence.** `integrator.rs`, `integrator/adjoint.rs`, `linear.rs`, vendored `diffsol-la` sources, 0.18.0 and
  diffsol-la 0.3.0 `op/nonlinear_op.rs` and `linear_solver/mod.rs`. Open upstream issue #419 was seen by title only.
- **Footprint.** `linear/terminal.rs` read through call sites. A comment in `dynamics.rs::admit_adjoint` still
  gives the crate-private reset setter as the reason for refusing events; the barrier for ODEs is smaller than it says
  (see [CBO-148](capability-gaps.md#cbo-148)).
- **Preservation constraints.** First-cause attribution, statistics surviving a failed segment, panic distinct from typed
  abort, and failed Newton factorization staying recoverable by step reduction.
- **Counterevidence and uncertainty.** At 0.18.0 the operators and `set_linearisation` remain infallible, so a move
  ([CBO-129](#cbo-129)) retires little here. The abort boundary is contained and tested. A fallible route is an upstream
  or vendored change, which would end the numerically untouched Diffsol posture. Local checkpoint recording stays for budgets.
- **Status.** Current mechanism.
- **Research questions.** Would a small vendored stop check in the step loop remove most callback aborts? Is an upstream
  issue viable?

<a id="cbo-129"></a>

## CBO-129 — A Diffsol supplier move is worth assessing on its contents

- **Operation and consumers.** The Diffsol route, which `Method::Auto` tries first.
- **Current mechanism.** Pinned at 0.16.2; 0.18.0 was current on crates.io at audit. By changelog and targeted source:
  - Fix: 0.18.0 re-evaluates the IC Jacobian at the current iterate (#392). The pinned code calls `reset_jacobian` in an outer
    setup loop. Whether this bears on consistent initialization here is not established.
  - Capability: `StepVjp`, required `solve_transpose_in_place` in diffsol-la, batched sensitivities and adjoints (0.17.0, breaking),
    trait-level `get_statistics`, which would remove `Counted`.
  - Performance: fused backwards-difference kernels, native faer sparse gemv, `SmallVec` BDF storage.
  - Cost: breaking operator and linear-solver layers, the batching refactor touching `Equation`, `Operator`, `Replay` and `Passes`, and
    new fields in option structs that deliberately fail the serde remotes.
- **Why it matters.** The pin receives no correctness fixes otherwise, and the local workarounds are written against its behaviour.
  Two workarounds may be retired (BDF coefficient repair, inferred from code) and one is unchanged (`LS::default()` construction,
  see [CBO-127](#cbo-127)).
- **Patterns.** LOP-11, LOP-12.
- **Evidence.** 0.18.0 `CHANGELOG.md`, `ode_solver/step_ad.rs`, `bdf.rs`, `problem.rs`, diffsol-la 0.3.0 `linear_solver/mod.rs`.
- **Footprint.** A 21,000-line source diff was seen by file list only; upgrade effort is an estimate, not traced through compilation.
- **Preservation constraints.** Exact pin and family movement of `diffsol`, `diffsol-la`, `diffsol-nl`, local deltas reapplied, one faer
  version, method contrast and adjoint tests agreeing, and profile identity moving with option fields.
- **Counterevidence and uncertainty.** A move changes numerics and needs requalification. No model here is known to be affected by #392.
  Compatibility with the vendored faer 0.24.4 is unchecked, and the feature-gated algebraic floating-point reductions in 0.17 would
  need the nonfused audit of [CBO-116](#cbo-116) if enabled.
- **Status.** Proposed.
- **Research questions.** Which faer version does 0.18.0 require? Does #392 reproduce on a repository model?

<a id="cbo-126"></a>

## CBO-126 — Three KLU owners with different reuse and observation

- **Operation and consumers.** Sparse Newton factorization in KINSOL, IDAS and Diffsol.
- **Current mechanism.** KINSOL uses the locally patched `SUNLinSol_KLUFixedPattern` (symbolic reuse, counters, phase observer, memory
  getter; `kinsol.rs::Session::new`, `kinsol/linear_work.rs`). IDAS uses stock `SUNLinSol_KLU` for forward and backward problems
  (`idas.rs::linear_solver`, `create_backward`), with none of those. Diffsol uses a Rust owner over direct `klu_l_*` (`linear.rs::Klu`) whose
  `refactor` copies the bundled SUNDIALS 7.9.0 setup policy.
- **Why it matters.** One mechanism has three lifecycle and telemetry forms. By reading C source, IDAS appears to repeat symbolic analysis
  at every native restart: `IDAReInit` clears `ida_SetupDone`, and `SUNLinSolInitialize_KLU` then sets `FIRSTFACTORIZE` for non-fixed solvers.
  The backward route restarts per observed sample through `restart_backward` and `IDAReInitB`, so a fit with many samples may
  re-analyse the same transposed pattern about once per sample per direction. IDAS reports no KLU work, so the cost is invisible.
- **Patterns.** LOP-09, LOP-07, LOP-03.
- **Evidence.** `sunlinsol_klu.c` (`SUNLinSolInitialize_KLU`, `SUNLinSolSetup_KLU`), `idas.c`, `idas_ls.c`, `idaa.c`, `idas.rs`,
  `kinsol/linear_work.rs`, `linear.rs::Klu`. Established by reading, not a run.
- **Footprint.** IDAS backward KLU creation seen by search. The full `sunlinsol_klu.c` diff is unread.
- **Preservation constraints.** Fresh numeric factors per application, failed factors invalidated, the `rcond`/`condest` refactor policy,
  Diffsol's recoverable-versus-terminal split. Fixed mode needs the canonical pattern in the matrix before construction, as KINSOL does
  and the IDAS paths do not.
- **Counterevidence and uncertainty.** Symbolic analysis may be cheap against the first numeric factor; nothing was measured. The
  second-order `2n` block pattern differs from the forward one. Diffsol's route can run without SUNDIALS linked, which justifies a
  separate owner, and SUNDIALS calls KLU internally, so one Rust owner cannot serve all three.
- **Status.** Current mechanism.
- **Research questions.** Can `SUNLinSol_KLUFixedPattern` serve IDAS forward and backward problems across `IDAReInit`? Can one module own the common
  KLU observation vocabulary?

<a id="cbo-125"></a>

## CBO-125 — Direct-factor admission charges dense fill

- **Operation and consumers.** Admitting direct factorization before native work: IDAS forward and backward KLU, sample elimination,
  the Diffsol faer scratch ceiling and the KINSOL block preconditioner.
- **Current mechanism.** `PreparedLayout::direct_bytes` charges `n * n * 32` as an allowance "independent of sparse input nonzeros".
  `prepare_storage` repeats it for observation, `Profile::checkpoint_bytes` adds a second for the backward problem, `sample_elimination_bytes`
  a third, `FaerLu::set_sparsity` caps scratch with it, and `kinsol/block.rs::checked_extent` uses a Markowitz bound plus an `n * n` projection.
  `idas.rs::create_backward` publishes `size * size * 32 * directions`, which is `128 n² d` on the second-order route. `initialize_consistent`
  requires a finite allowance, so KLU consistent initialization always pays the dense charge.
- **Why it matters.** Under a finite allowance the charge grows quadratically whatever the sparsity: 3.2 GB at n = 10,000 and 320 GB at n = 100,000,
  before the backward term. A sparse flowsheet DAE could be refused for a factor far smaller. Krylov routes are then the only way through, and
  their preconditioners are thin ([CBO-150](capability-gaps.md#cbo-150)).
- **Patterns.** LOP-X001, LOP-05.
- **Evidence.** `dynamics/layout.rs`, `idas.rs::prepare_storage`, `dynamics.rs`, `linear.rs`, `kinsol/block.rs`. `Klu::publish_return` already reads
  `common.memusage`, `mempeak`, `lnz`, `unz` after the fact; no analysis result is consulted before admission.
- **Footprint.** The runtime callers computing the allowance are unread.
- **Preservation constraints.** Prepay before native effects, typed finite refusal, no partly valid factor after failure, unbounded local mode working.
- **Counterevidence and uncertainty.** A symbolic estimate is not a hard bound: partial pivoting can exceed it and KLU grows factors while factoring.
  The dense bound is the honest conservative contract unless a hard ceiling (allocation control) or a deliberately weaker contract is accepted.
  The local default is unbounded, so the bite depends on how often a finite allowance is selected, which was not established. See also
  [CBO-001](cross-cutting-mechanisms.md#cbo-001).
- **Status.** Current mechanism.
- **Research questions.** Do `klu_analyze` with `memgrow`/`initmem` and SuiteSparse allocator hooks allow an analysis-derived estimate with a hard cap
  failing as out-of-memory? Does faer symbolic LU (already consumed by `sparse_lu::scratch`) or a FERAL analysis-phase fill estimate serve the same way?

<a id="cbo-127"></a>

## CBO-127 — Diffsol rebuilds patterns and symbolic analysis per segment

- **Operation and consumers.** Trajectories with scheduled inputs or event modes, shooting windows, and Diffsol adjoints.
- **Current mechanism.** `integrator.rs::build_problem` constructs five `Operator`s per segment. `Operator::for_mode` calls `oracle.support()` (an owned
  `Vec`) and builds source, state and parameter patterns, two transposes and four refill maps. Each new solver takes `LS::default()` and `set_sparsity`,
  which runs `klu_l_analyze` or `sparse_lu::symbolic`. The forward-sensitivity route builds a transition problem per event. The adjoint route, after
  each sample jump, calls `split` and `resumed`, and the vendored and 0.18.0 constructors create one `NewtonNonlinearSolver::new(LS::default(), ..)`
  plus another for the augmented solver, so analysis repeats per observed sample. `layout.rs` builds CSC by hand while `Operator` uses faer's
  `try_new_from_indices`: two scatter-map constructions over one support. IDAS, by contrast, shares one immutable `Arc<PreparedLayout>` across modes.
- **Why it matters.** State patterns do not depend on the segment, only parameter columns do. Work scales with segments times operators on the route chosen
  first, and horizon and shooting workflows have many segments. `LS::default()` is unchanged in 0.18.0, so a supplier move does not address it.
- **Patterns.** LOP-07, LOP-09, LOP-06.
- **Evidence.** `integrator.rs`, `linear.rs`, `layout.rs`, `integrator/adjoint.rs`, 0.18.0 `bdf_solver_adjoint_from_state`.
- **Footprint.** `linear/terminal.rs` (a plausible home for an attempt-scoped cache) read through use sites only.
- **Preservation constraints.** Per-mode support differences, a changed pattern as a contract failure, factors private to an attempt, per-segment parameter axes.
- **Counterevidence and uncertainty.** Support can legitimately differ per mode, and the `Function::Initial` parameter pattern is deliberately widened. The `Default`
  bound is no barrier because the repository owns the `LS` types. No cost was measured, and small models likely do not care.
- **Status.** Current mechanism.
- **Research questions.** Does `Bdf::new_augmented` taking a caller-supplied linear solver (0.18.0) allow a prepared solver across BDF and SDIRK, or is an
  attempt-scoped symbolic cache the simpler seam?

<a id="cbo-128"></a>

## CBO-128 — IDAS gradients and sensitivities repeat preparation

- **Operation and consumers.** IDAS forward sensitivities, backward residuals and quadratures, and repeated gradient evaluations in fitting and shooting.
- **Current mechanism.** Two manifestations:
  - **Full partials in callbacks.** `residual_b`, `quadrature_b`, `residual_bs`, `jacobian_bs`, `quadrature_bs` and `sensitivities` call `c.partials(t, x)` and then multiply.
    No `jacobian_transpose_action` is used in `idas.rs`, while the Diffsol route uses the compiled cotangent product. Backward callbacks allocate a `Vec` per call.
    `N_VEnableFusedOps_Serial` is never called. `DynamicWorker::evaluate` already caches by an identical bit key.
  - **Storage rebuilt per gradient.** `attempt` retains `IdasWorkspace` only for non-adjoint, first-order, non-BBD requests. `IDAAdjReInit` frees all checkpoints, so each
    non-final segment is replayed (a second forward integration), where Diffsol keeps all segments (memory against time). `bbd::install` calls `IDABBDPrecInit` on every restart and
    replay, and `IDABBDPrecReInit` is unused. `Function::Initial` is evaluated twice in `attempt`.
- **Why it matters.** A multi-segment gradient does roughly two forward integrations and rebuilds storage, and fitting does many gradients. Sensitivity cost scales with
  parameters. Compounds with [CBO-126](#cbo-126).
- **Patterns.** LOP-07, LOP-09, LOP-05, LOP-04, LOP-03.
- **Evidence.** `idas.rs` (callbacks, `attempt`, `backward`, `replay`, `restart`), `idas/bbd.rs`, `idaa.c::IDAAdjReInit`, `idas_bbdpre.c` by symbol, `integrator.rs::Operator::transpose_product`.
- **Footprint.** `DynamicWorker` partly read (owned by workflow code). Whether a retained session could reset the backward problems was not traced.
- **Preservation constraints.** Fresh-point validity, domain failure semantics, aliased outputs published after input borrows end, no uncertain reset escaping on failure, checkpoint permission not renewed on reuse.
- **Counterevidence and uncertainty.** Retention narrowing appears deliberate (ADR-0205, read by search). One IDAS adjoint memory holds one trajectory, so avoiding replay needs one memory per segment. When the full Jacobian is needed at setup anyway,
  the cached form may win. Fused-operation benefit is modest. Nothing was measured.
- **Status.** Current mechanism.
- **Research questions.** Which fused operations do IDAS sensitivities call in 7.9, and what fraction of backward steps set up a Jacobian in representative fits? Can `IDABBDPrecReInit` replace repeated init under `IDAReInit`?

<a id="cbo-118"></a>

## CBO-118 — Point interval enclosures rebuilt per call under the proof lock

- **Operation and consumers.** `Ibex::{enclose_point_arithmetic*, enclose_arithmetic_box*}`, consumed by `pse-runtime` `engineering_accuracy/arithmetic.rs`,
  `kkt_accuracy/arithmetic.rs`, `certified_accuracy.rs`, `objective_accuracy.rs`, `dynamics_accuracy.rs::point_uncertainty` and `output_program.rs` to certify original outputs and KKT quantities.
- **Current mechanism.** Three manifestations of one need:
  - **Native rebuild.** `point_arithmetic_core` encodes a new transport, takes the global lock, and `pse_ibex_point_arithmetic` builds expression nodes, `Function`, `System`
    and (at order 2) `diff()` per output, evaluates once, and destroys everything. Consumers construct `Ibex::default()` per use. No covering or contraction occurs.
  - **Protocol repeated.** About five call sites repeat the budget, projection, retained-bytes, workspace and radius protocol, with uneven refusal: only the KKT variant maps `Incomplete`
    to a typed reason. `point_arithmetic_program_for_order` (`pse-math/src/factorable.rs`) reprojects the DAG at every call, and `charge(available)` transiently claims all remaining capacity.
  - **Local widening.** `dynamics_accuracy.rs` builds boxes and widens one ulp around an IBEX call. This is small.
- **Why it matters.** Program-dependent preparation repeats per point and serializes against covering proofs ([CBO-030](#cbo-030)). Separating forward interval evaluation from validated covering could allow
  unlocked, retained evaluation.
- **Patterns.** LOP-07, LOP-09, LOP-01, LOP-04.
- **Evidence.** `root_isolation.rs` `point_arithmetic_core`, `acquire_library`; `native/root_isolation.cpp` `Graph`, `pse_ibex_point_arithmetic`; the two arithmetic files and most of `certified_accuracy.rs`, read.
- **Footprint.** `objective_accuracy.rs` and `derived_accuracy.rs` confirmed for the budget and radius parts only. Call frequency per solve is unknown. The `pse_ibex_certify` proof paths share the rebuild shape, partly required.
- **Preservation constraints.** Outward rounding including elementary functions and rational constants, guard ordering, domain-boundary refusal, `abs` value-only rule, `Incomplete` versus `Interrupted`, byte admission before work.
- **Counterevidence and uncertainty.** If enclosures are rare or small the cost is negligible. One arithmetic owner for proof and enclosure may be deliberately simpler assurance. Guard validity is value-dependent, so value-free preparation may be unsound. Retaining IBEX
  `Function`s needs the global-state question of CBO-030 answered. A second interval library needs its own rounding claim.
- **Status.** Current mechanism.
- **Research questions.** Are an IEEE 1788 library or FILIB used directly adequate for forward and reverse mode? Can an IBEX `Function` be retained and reused from one thread safely?

<a id="cbo-117"></a>

## CBO-117 — Covering proofs consume an unseeded process-lifetime random stream

- **Operation and consumers.** `pse_ibex_certify`, through `exclude`, and every consumer of selection charts, cell counts and refusal outcomes.
- **Current mechanism.** `GuardContractor` configures `LinearizerXTaylor::RANDOM_OPP` under `CtcPolytopeHull`. By symbol reference the linearizer draws from the global `RNG`. A search of
  `native/root_isolation.cpp` and `root_isolation.rs` finds no `RNG::srand`. The corners taken therefore depend on how many draws earlier proofs in the process consumed, in an order that varies across lanes with lock order.
- **Why it matters.** Soundness holds for any corner, but near a cell limit `Resource` versus `Unique` and `proof_cells` can depend on unrelated earlier work. That complicates replay and evidence reuse. `RNG::srand` is public and deterministic corner modes exist,
  so the outcome could become a function of the request. It would also remove one reason for the lock ([CBO-030](#cbo-030)).
- **Patterns.** LOP-X006, LOP-X018, LOP-03.
- **Evidence.** `native/root_isolation.cpp` `GuardContractor`, `exclude`; `ibex_Random.h`; `nm` references to `RNG::rand`.
- **Footprint.** Inferred from symbols and headers, since IBEX sources are absent. Not executed.
- **Preservation constraints.** Soundness, the budget account, "numerical proposals supply no proof authority", and `Ibex::identity`, which would need to reflect a changed policy.
- **Counterevidence and uncertainty.** Differences may appear only near limits. Deterministic corners can converge worse. Reseeding per request needs the lock or per-thread state anyway. No consumer known compares `proof_cells` across runs.
- **Status.** Current mechanism.
- **Research questions.** Which `LinearizerXTaylor` modes are deterministic, and how well do they contract?

<a id="cbo-120"></a>

## CBO-120 — NLP presolve bound passes disabled for lack of multiplier recovery

- **Operation and consumers.** Continuous NLP through `presolve::pipeline` for Ipopt and POUNCE.
- **Current mechanism.** §18.3 records that the pinned binding cannot recover general original-bound multipliers from `LinearBounds` or `Fbbt` tightening, so both passes are ineligible. `Auto` records a reason, and interval analysis reduces to an infeasibility check whose geometry is discarded.
- **Why it matters.** Tightening is a common generic presolve step, and the library is the maintainer's own fork, so the gap may be closable at its source. Tightened domains could improve start quality and conditioning. Upstream subjects about handing back transferred bound multipliers ([CBO-121](#cbo-121)) may bear on the premise.
- **Patterns.** LOP-12, LOP-03, LOP-X004.
- **Evidence.** §18.3, `presolve/pipeline.rs` head, `presolve/records.rs`, `presolve::tests::bounds` by name.
- **Footprint.** Docs and heads. The `pounce-presolve` postsolve design is unread.
- **Preservation constraints.** Original complementarity stays mandatory. The project records effects and inverse source attribution, not a second transformation IR. The ℓ1 route's Auto-to-Off rule stands, and no pass may silently weaken a multiplier.
- **Counterevidence and uncertainty.** Bound-multiplier postsolve after chained FBBT is hard and can yield non-unique multipliers; an active tightened bound implied by several rows needs a dual distribution or a resolve. Interior-point methods can behave worse with bounds tightened near a solution.
- **Status.** Current mechanism (a recorded limitation in §18.3).
- **Research questions.** How do PaPILO (in the solver prefix), HiGHS presolve and POUNCE's sibling tools report dual postsolve for bound tightening?

<a id="cbo-122"></a>

## CBO-122 — Ipopt reuse keeps objects but discards numerical preparation

- **Operation and consumers.** Staged sequences and sweeps of one structure through the Ipopt adapter.
- **Current mechanism.** The bridge retains the C++ application and TNLP and calls `ReOptimizeTNLP`, but every call sets `warm_start_same_structure=no`, so spaces, scaling and linear-solver symbolic state are rebuilt. POUNCE contrasts: `pounce/retained.rs` keeps reset-safe baseline symbolics of leased FERAL backends across applications. Bound updates pass through the classification-preserving `update_bounds`.
- **Why it matters.** Symbolic reanalysis for MUMPS, SPRAL or Pardiso repeats per step on the default NLP route, while the bridge already computes a structure signature that could bound preparation lifetime.
- **Patterns.** LOP-07, LOP-09.
- **Evidence.** `pse_ipopt_sequence.h` and `.cpp` (solve area), test `retained_application_refreshes_gradient_scaling_from_current_coefficient_and_start`, §18.3 option hygiene.
- **Footprint.** Ipopt internals of same-structure paths per linear solver are unread.
- **Preservation constraints.** No stale value-dependent scaling, no option leakage, `RequireReuse` refusal, dependency-map refusal, original acceptance.
- **Counterevidence and uncertainty.** The setting is a deliberate, tested correction: Ipopt's same-structure mode retains value-dependent gradient scaling and requires bound-class constancy. It may be sound only when `nlp_scaling_method=none`, which §18.3 allows under a policy. Benefit scales with analysis cost, unmeasured.
- **Status.** Current mechanism.
- **Research questions.** Which Ipopt objects does `warm_start_same_structure=yes` keep, and is retaining only the symmetric solver's symbolic state possible?

<a id="cbo-119"></a>

## CBO-119 — Rank and spectral diagnostics densify sparse operators

- **Operation and consumers.** Arclength fold and branch probing, regular-square response spectrum, Jacobian degeneracy rank and general matrix diagnostics, for continuation and structural diagnostics on large Jacobians.
- **Current mechanism.** `kkt/path/arclength.rs::svd` computes a full dense SVD with full U and V. `square_response.rs` `spectrum` and `rank_scratch` are dense. `jacobian_diagnostics.rs::rank` goes to `pse_math::diagnostics::analyze_matrix` behind a hard `dense_entries` gate, building dense Grams; beyond it an investigation gets no answer. The pinned faer tree carries `operator::{partial_svd, partial_eigen, partial_self_adjoint_eigen, lsmr}`, unused here by grep. FERAL LU and inertia give a rank signal elsewhere.
- **Why it matters.** Near-singular analysis matters most on large cases, where the dense gate refuses. A few extreme singular triplets answer dependence, null direction and fold test function. The response path also has no condition estimate: it uses a local `matrix_norms` for backward-error bounds, while `conditioning.rs` uses FERAL's `condition_estimate_1`. These answer different questions, so whether the FERAL estimator (a Hager–Higham operator is public) could run over other factors is a lead, not a defect.
- **Patterns.** LOP-12, LOP-03, LOP-06.
- **Evidence.** `arclength.rs::svd`, `jacobian_diagnostics::rank`, `square_response` scratch and spectrum heads, `conditioning`, `vendor/faer/src/operator`.
- **Footprint.** `pse-math::diagnostics` lightly read. Typical workload size was not established.
- **Preservation constraints.** Scale-aware rank semantics, thresholds, typed "unresolved" rather than a false verdict, "numerical rank is not a certificate", prepaid faer `StackReq` scratch, cancellation between work units, and the arithmetic of [CBO-116](#cbo-116).
- **Counterevidence and uncertainty.** Partial Krylov methods can miss clustered smallest singular values, the case of interest; shift-invert needs a factor of a possibly singular matrix. If small block diagnostics are the real workload, dense is right. SLEPc is a heavier alternative.
- **Status.** Proposed.
- **Research questions.** Is faer `partial_svd` usable for smallest triplets through an inverse operator? Does a sparse rank-revealing factorization serve better?

<a id="cbo-123"></a>

## CBO-123 — Native boundary declarations exist twice with no generated link

- **Operation and consumers.** Every change to the project-owned C ABIs: the IBEX transport and the PETSc, Uno and Ipopt-sequence bridges.
- **Current mechanism.** Only upstream Ipopt C uses committed bindgen (`xtask/src/codegen/ipopt.rs`). Others are duplicated by hand: `pse-petsc-sys/src/lib.rs` against `bridge/pse_petsc.h`, `pse-uno-sys/src/lib.rs` against `pse_uno.h`, `pse-ipopt-sys/src/sequence.rs` against `pse_ipopt_sequence.h`, including struct layouts and integer status spaces. IBEX is weakest: `root_isolation.rs` has seven `repr(C)` types duplicating `PseRootNode` and others in `native/root_isolation.cpp`, with no shared header and bare integer op codes. `NativeRequest` is spelled field by field in at least four read paths. A search for `offset_of!` in the sys crates and `root_isolation.rs` found none.
- **Why it matters.** A mismatched struct, op or status number at a validated-proof boundary is silent undefined behaviour or a false proof input, and each bridge function needs several edits.
- **Patterns.** LOP-02, LOP-X003.
- **Evidence.** The headers, Rust declarations in part, `pse-uno-sys/build.rs`, `pse-petsc-sys/build.rs`. `pse_uno.cpp` carries `static_assert`s tying status constants to `Uno_C_API.h`, a real link on the native side only.
- **Footprint.** Negative search limited to `offset_of!` and `size_of` shapes. Live ABI tests in `pse-petsc-sys/src/tests.rs` exercise layouts indirectly.
- **Preservation constraints.** Exception shields, `noexcept`, build identity including bridge sources, one-owner `unsafe` reasons, bindings available without a solver prefix.
- **Counterevidence and uncertainty.** Small stable ABIs and direct tests may suffice. A C++ interop layer would conflict with the C-only PETSc bridge and the panic choices. Build-time bindgen needs libclang, already in the toolchain.
- **Status.** Current mechanism.
- **Research questions.** Bindgen from the project's own headers or header generation from Rust? Is a C++ interop crate suitable for the two C++ bridges only?

<a id="cbo-124"></a>

## CBO-124 — Oracle wrappers forward by hand, and optional facts default silently

- **Operation and consumers.** Wrappers over `NlpOracle` and `NleOracle`: `transform::{Pinned, Unconstrained}`, `structural::{NlpWitness, RootsWitness}`, `callback::CountedNlp`, `initialization::LeastDeviationOracle`, `transport::Roots` and `presolve/pipeline::Transport`.
- **Current mechanism.** Each hand-forwards the trait, about 15 methods. Six NLP methods have default `None`-style bodies, so a wrapper that omits a forward loses a fact with no compile error. In outline, `transform::Pinned` forwards `solve_separator`, `normalization`, `constraint_sources` and `derivative_facts` but not `structural_analysis` or `presolve_facts`.
- **Why it matters.** Each new optional capability needs a decision in every wrapper, and the failure mode is silent loss, not an error. The translators themselves are not duplicated: one `tnlp::Adapter` and one `foreign_nlp` dispatch serve the three NLP solvers.
- **Patterns.** LOP-02.
- **Evidence.** The trait in `lib.rs`, outlines of the files above, `callback.rs` `CountedNlp`.
- **Footprint.** Body-level reading of forwarding only for `callback.rs` and `transform.rs` outlines.
- **Preservation constraints.** Dropping a fact can be correct (a pin changes presolve validity), so explicit omission must stay possible.
- **Counterevidence and uncertainty.** The omissions may each be correct and the wrappers are few. A delegation macro or an `inner()` default is cheap but not free, and removing the defaults would need no library.
- **Status.** Current mechanism.
- **Research questions.** Would a delegation derive keep explicit omission readable, or does removing the trait defaults suffice?

<a id="cbo-130"></a>

## CBO-130 — Hybrid trajectory ordering written separately per backend

- **Operation and consumers.** Segment-by-segment trajectory semantics: coincident roots, scheduled changes, ambiguous guards, sample-at-transition, event limits, terminal events, conservation settlement and endpoint capture.
- **Current mechanism.** `idas.rs::{run, segment}`, `integrator.rs::{run, drive, record_start, capture_endpoint}` and `integrator/adjoint.rs::Passes::run` each encode these decisions, with near-identical branches for `max_events` and the deferred post-transition sample. Shared helpers (`observe_conservation`, `settle_transition`, `guards_at_zero`, `sample_jump`) cover conservation and jumps, not ordering. The backward sweeps repeat too: `Session::backward` and `Passes::run` have the same structure (latest sample first, jump at stop, restart, step, carry the adjoint, initial-value correction).
- **Why it matters.** A semantic change to event ordering needs parallel edits and differences can hide. `dynamics/method_contrast_tests.rs` exists to catch drift. The backward-sweep driver is the more mechanical duplication.
- **Patterns.** LOP-09.
- **Evidence.** Both `run` bodies, `drive` and all of `Passes`, read.
- **Footprint.** Inspected as above.
- **Preservation constraints.** IDAS's NORMAL-mode output service differs from Diffsol's step-driven loop and `state_mut_back`. IDAS restarts one native memory while Diffsol builds a new problem, and sensitivity seeding differs.
- **Counterevidence and uncertainty.** No library owns this: it is a local consolidation with an owned trait or state machine, and shared tests may be the better remedy. Keeping both backends is deliberate, because their capabilities are not nested (§13 method table).
- **Status.** Current mechanism.

<a id="cbo-131"></a>

## CBO-131 — SUNDIALS resource ownership hand-written per adapter

- **Operation and consumers.** Creating, attaching, parking, reinstalling and freeing native objects, and entering Rust from native callbacks, in IDAS and KINSOL.
- **Current mechanism.** `NativeStorage` in `idas.rs` holds 18 fields with hand-written `take`, `install` and `Drop`, and reuse branches through `prepare_storage` and `new_storage`. `Session` has parallel fields. `kinsol.rs::Session` has its own `Drop`, `check`, `index`, `values` and `publish`, and `local_error.rs::Scratch` another context. The callback disciplines differ: KINSOL uses `CallbackState::evaluate` with preallocated buffers, IDAS uses `catch_unwind` per callback plus `finish_callback`, typed `input!` guards and mostly per-call `Vec`s.
- **Why it matters.** Free order, aliasing and detach rules are stated in comments at each site, and each new native object adds edits in several lists. Typed owners could encode order and make parking mechanical.
- **Patterns.** LOP-08, LOP-09.
- **Evidence.** `idas.rs` session and storage sections, sample callbacks, `kinsol.rs::Session` and callbacks, `local_error.rs::Scratch`, `bbd.rs`.
- **Footprint.** Inspected as above.
- **Preservation constraints.** Free order (integrator memory, solvers, matrices, vectors, context last), owner-thread affinity, no oracle pointer surviving while parked, aliased outputs published after input borrows end, panic containment.
- **Counterevidence and uncertainty.** The code is careful and tested, and a generic layer can hide the aliasing rules. The benefit is change cost, not behaviour. Whether a maintained safe Rust SUNDIALS crate covers IDAS adjoints was not researched, and is doubted.
- **Status.** Current mechanism.

## Examined and not retained

- **`highs::Model` retained mostly as an owner beside direct C calls.** `highs.rs` uses `ColProblem`, `Model::try_new` and `try_set_option`, and otherwise about 60 `Highs_*` calls through `as_mut_ptr`. A thin RAII owner would cut little against a small gain and some rewrite risk.
- **SCIP through raw `scip-sys` rather than `russcip`.** Recorded by ADR-0105: the adapter needs nonlinear expressions, event handlers and copy callbacks.
- **Greedy row colouring in `adjoint_init`.** Small, correct for its need and exercised only in Hessians.
- **Replacing the accepted-step and replay limit patches with `IDASetMaxNumSteps`.** Refuted by the hidden `IDA_ONE_STEP` replay loop, as the C comment records.
- **Factor owners for conditioning (FERAL), responses (faer) and roots (KLU).** Each serves a distinct contract (inertia and condition estimate, actions with backward error, SUNDIALS integration). Retained only as the residues in CBO-119 and CBO-126.
- **Hand-built IBEX chart chain and certify width schedule.** This is proof orchestration composed from library contractors and a bisection stack, not generic continuation code.

## Retired opportunities

<a id="cbo-026"></a>

- **CBO-026 — Sparse factorization wrappers split across faer, FERAL and KLU.** Retired 2026-10-30: the main manifestation was resolved by `cb7b65b62`, where `backend/sparse_lu.rs` is the shared faer owner for `kkt/path/arclength.rs`, `square_response/actions.rs` and `dynamics/linear.rs`, and the remaining split follows different contracts. Successors: [CBO-119](native-solver-integration.md#cbo-119) (the FERAL condition-estimate lead) and [CBO-126](native-solver-integration.md#cbo-126). Last described at [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/native-solver-integration.md#cbo-026).

<a id="cbo-029"></a>

- **CBO-029 — Two NLP-oracle callback translators for three native NLP solvers.** Retired 2026-10-30: the premise was a misreading. `foreign_nlp` is one C-ABI dispatch for Uno and the Ipopt sequence bridge, and `tnlp::Adapter` is the POUNCE trait implementation, which sit on different language boundaries; `ipopt.rs` re-exports `sequence::Sequence`, so no third Ipopt route exists. Successor: [CBO-124](native-solver-integration.md#cbo-124). Last described at [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/native-solver-integration.md#cbo-029).

<a id="cbo-031"></a>

- **CBO-031 — Convex and conic encodings hand-synchronized per solver.** Retired 2026-10-30: the premise is gone. `conic.rs` maps each cone directly to Clarabel's types with a test per mapping, POUNCE-convex uses one `Target` table in both directions, and `certificate.rs` converts HiGHS rays once and verifies against original data. Successor: none. Last described at [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/native-solver-integration.md#cbo-031).
