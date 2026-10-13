> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R4 (library-research), Part B: obligations mapped onto the R4 libraries. Facts A and B of X2 correct its §1.1 statements about retained symbolic analysis. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R4 Part B: mapping the W-A/W-B obligations onto the numerical and native-session libraries

- **Inputs.** The W-A ledger (`w-a-interactive-obligations.md`), the W-B ledger (`w-b-multicase-obligations.md`) and the R4 Part A file (`r4-numerical-methods-native.md`).
- **Baseline and method.** `main` at `6c368c88a`. Static reading only; nothing was built, run or probed.
- **Labels.**
  - *Interface-checked*: a header or source path was read.
  - *Implemented*: a code path exists.
  - *Proposed*: a statement about the target.
- **Abbreviations.**
  - **§17.6** and **§18.8** are blueprint sections in `docs/authoritative_design/sections/numerical-execution.md`.
  - `BN/` = `crates/pse-backend-native/src/`.
- **One Part A nuance, corrected here.** FERAL's parallel pool is per `Solver`, and it is never the global rayon pool. Its *size*, however, comes from `RAYON_NUM_THREADS` or the hardware count (`vendor/feral/src/numeric/solver.rs:680-710`). Callers cannot inject a size; they can only switch it on or off per backend (`FeralConfig.parallel`). That is the substance of register R-41, and it is why `BN/pounce.rs::feral_threads` only allows FERAL to run parallel when admission covers the whole pool.

---

## 1. Obligations in scope, grouped by kind

### 1.1 Native session lifetime and retention

**Ledger facts:**
- Each Python `solve_*` call gets a fresh `pse-math` session thread, closed after the run (W-A 1.15, 2.7; W-A (c)).
- Each independent or fan-out study point gets a fresh `Staged` session. A session is retained only inside a `Staged` sequence (horizon steps, initialization stages and blocks) or a single-successor local continuation chain (W-B 1a stage 8a; W-B (c)).
- Durable attempts never carry native state across points (W-B 1b).
- KINSOL child sessions are cached per worker thread (`BN/implicit.rs:124-128`, `thread_local! SESSIONS`).
- HiGHS updates its model in place for compatible layouts.
- A transient fit builds a new dynamics worker and integrator for every oracle call (W-B 3, stage 4b).

**Per-library retention rules.** "Per thread" here means: owned by one thread for its whole life, as both the library and §18.8 require. All rows are *Interface-checked*, Part A §1.10 and below.

| Library | Retainable native state | Unit of retention | What a re-solve keeps |
|---|---|---|---|
| Ipopt | `IpoptApplication` + TNLP; `ReOptimizeTNLP` with `warm_start_same_structure` | instance, used by one thread | KKT structure and option state; with warm-start options, the primal-dual seed and μ (B1 §1.3) |
| POUNCE | `TnlpPresolveSession` (`pounce-rs/src/session.rs:377-510`), which holds `Rc<RefCell<dyn TNLP>>` and so is **`!Send`**, thread-bound by type; FERAL `pattern_reused` | instance on one thread | presolve transform, last solution (`solve_warm_last`), FERAL symbolic pattern |
| KINSOL / IDAS / CVODES | solver memory plus `SUNContext` ("thread-safe provided each thread has its own context"); `KINSetNoInitSetup`; `IDAReInit`/`IDASensReInit`/`IDAQuadReInit`/`IDAReInitB` | one context per thread | allocation, linear-solver object, and KLU symbolic analysis (`SUNLinSol_KLU` refactors). `IDAReInit` resets order and step history and forces one setup (B3 §1.3) |
| Diffsol 0.16.2 | the problem object; the solver borrows it (`Bdf<'_, Eqn, LS>`); `set_state`/`checkpoint`; `OdeEquations::set_params(&mut self)` | instance | the solver is rebuilt after a parameter change because it borrows the problem. Symbolic reuse lives in the project-owned `diffsol-la` `LinearSolver` (`BN/dynamics/linear.rs`), which can hold it across solver instances |
| HiGHS | `Highs` object; model retained; basis and factor retained across `change*` calls **unless** `Highs_clearSolver` runs (B4 §1.3); executor per calling thread | instance + calling thread | basis, factor, model; hot start |
| Clarabel | `update_P/q/A/b` | instance | symbolic factorization (pattern unchanged, presolve and chordal off) |
| SCIP | reoptimization (objective changes only); solution pool | instance | tree for objective-only sequences |
| PETSc | SNES/KSP/PC/Mat objects; symbolic LU skipped on an unchanged pattern; Jacobian and PC lagging | **process** (non-thread-safe build, admission mutex `BN/petsc.rs:64`) | factor symbolic structure, lagged Jacobian. Locally refused today (`ReusePolicy::RequireReuse`, B2 §2.3) |
| Uno | one solve per handle in the bridge (B1 §3.3) | instance | none through the bridge |
| IBEX | none; process lock (`BN/root_isolation.rs:45`) | process | none |

**Ownership.** Libraries own retention of their own native state **wholesale**, but only within a thread (for PETSc and IBEX, within the process). No surveyed library retains or transfers native state across threads or processes. That is consistent with §18.8 ("Mutable native state is constructed, used and destroyed on the owning `pse-math` thread") and §17.6 ("Mutable native state never crosses a worker boundary").

**Necessity.** The functional target needs value-only edit → re-solve without rebuilding structure, plus fast studies and recycles. Retaining native state is a means to that, not a requirement in itself. What the libraries allow and what the current rules impose differ in four places:

- **Per-call session closure for Python `solve_*`.** This is a current implementation rule (`run.rs::launch` opens and closes a `Staged` per run; W-A (a)). No library requires it. §17.6 separates numerical start policy from allocation reuse (`ReusePolicy`) and allows reuse whenever `layout`, `profile` and `backend` match. A long-lived session thread owned by the Python `ModelingPackage` (or a `Runtime` lane) would satisfy every library's rules.
- **Fresh session per independent study point.** Also a current rule (W-B (c), "fan-out gets independent sessions"). The libraries allow one instance per lane thread to be reused across sequential points with a compatible layout. Ipopt `ReOptimizeTNLP`, POUNCE `solve_warm`/`solve_cold`, KINSOL `replace` and HiGHS in-place changes all keep structure. §17.6's compatibility stamp already defines when that reuse is valid.
- **Durable points.** The rule "never crosses a worker boundary" is compatible with libraries keeping state across points *on the same worker lane thread*. Only point-to-lane affinity would be needed. Nothing in the libraries prevents it.
- **Transient-fit integrator per oracle call.** Not required by IDAS. `IDAReInit` plus `IDASensReInit` on one memory per experiment keeps the KLU symbolic analysis and the allocation, and the oracle runs on the fit's NLP thread (W-B 3, stage 4b, "in-thread"). With Diffsol, the solver must be rebuilt per parameter set, but the project's `LinearSolver` can keep the symbolic analysis.

**Consequences.** Library-owned retention on long-lived lane or session threads would remove the per-call native construction listed in §3 below. It would weaken one-shot isolation: a corrupted native state could leak into later points. §17.6's `profile` stamp and a `Fresh` option exist for that case. It also needs an idle-session owner, which §18.8 already describes: "idle sessions retain resident memory … but no active permits".

### 1.2 Starts and seeds

**Ledger facts:**
- No warm start across Python `solve_bound` calls unless `with_stored_start`/`with_primal_start` is used, and Python does not expose either (W-A 2.7).
- Durable points re-read and decode stored seeds (W-B 1b, stage 8).
- HiGHS calls `clearSolver` before every solve (B4 §1.3).

**Ownership.** Partly library-owned. Every NLP and LP library *accepts* a seed:
- Ipopt: `warm_start_init_point`, primal-dual plus μ, and the unused `warm_start_entire_iterate`;
- POUNCE: `set_warm_start_iterate` and the SQP working set;
- HiGHS: `setSolution`/`setBasis`;
- Uno: primal only;
- KINSOL and IDAS: an initial vector.

A library keeps a seed implicitly only while its instance lives (Ipopt `ReOptimize`, POUNCE `solve_warm_last`, the HiGHS basis without `clearSolver`). No library persists seeds across processes, so durable seed storage stays the project's.

**Necessity.** Starting from the previous accepted result in an interactive loop is standard practice (§17.6 `PreviousAccepted`). Its absence from Python `solve_bound` is an API gap, not a library limit. `clearSolver` is a local choice that discards HiGHS's own hot-start state.

**Consequences.** If instances are retained (§1.1), the library's implicit warm state makes most explicit seed round-trips unnecessary for same-lane sequences. For Ipopt, the documented condition holds: an interior-point warm start pays only with reduced pushes and μ together (B1 §6).

### 1.3 Native exclusion and concurrency (product concurrency, not harness)

**Ledger facts:** process-global PETSc, HiGHS-lifecycle and IBEX mutexes; MKL and OpenMP counts scoped per thread; FERAL's pool outside the CPU semaphore (R-41); faer `Par::rayon` inside the implicit child dispatching to rayon's **global** pool (W-B (b), nesting point 2).

**Ownership.** These obligations are **imposed by the libraries**. The project cannot remove them, only choose around them:

| Library-imposed constraint | Source | Effect on in-process concurrency |
|---|---|---|
| Ipopt's global MUMPS mutex | `IpMumpsSolverInterface.cpp:46-52` and call sites | every concurrent Ipopt-with-MUMPS solve in one process queues on its linear algebra. **MUMPS is the project default** (`BN/settings/ipopt.rs:53-65`, "MUMPS with METIS ordering"). SCIP's nested Ipopt also uses Ipopt's linear solver |
| PETSc built without threadsafety | `libraryOptions.py:18,41-48`; manual `getting_started.md:612-620` | one PETSc solve per process (`BN/petsc.rs` admission) |
| IBEX | process lock | one per process |
| HiGHS thread-local executor; thread count fixed until a global reset | `Highs.cpp:5111-5132`; `HighsTaskExecutor.h:40-48` | concurrent HiGHS is allowed (`BN/highs_lifecycle.rs` takes a shared read lock for solves). Resets need exclusive idleness |
| SPRAL OpenMP team per calling thread; MKL Pardiso per-thread count | Part A §1.10 | safe per thread; the project scopes ICVs (`BN/mkl.rs`) |
| FERAL pool size not injectable | `solver.rs:680-710` | all-or-nothing internal parallelism (R-41) |
| POUNCE `solve_*_batch_parallel` uses whatever rayon pool is current (global by default) | `pounce-algorithm/src/batch.rs:641`; `pounce-convex/src/batch.rs:95` | the caller must `install` a scoped pool, as the project does (`BN/pounce.rs::with_threads`) |
| SUNDIALS: one `SUNContext` per thread | `sundials_context.h:15-16` | safe per thread |
| SCIP `THREADSAFE`, `TPI=tny` | image build flags | safe per instance; concurrent racing within one solve |

**Necessity.**
- *Target need:* the sixteen-case and sweep premises need concurrent solves.
- *Fixed by the library:* the MUMPS and PETSc constraints apply under any design.
- *Choices, not library facts:*
  - MUMPS as the default Ipopt linear solver (ADR-0108 settings);
  - PETSc's configure flags, which are a pin choice (`Cargo.toml [workspace.metadata.pse.petsc]`);
  - the faer global-pool dispatch, which is project code (`pse-math/src/implicit.rs:1049`, no `install`).

**Consequences.** The library-compatible routes to in-process case parallelism are:
- Ipopt with SPRAL or Pardiso;
- POUNCE/FERAL (no mutex);
- SUNDIALS with a context per thread;
- HiGHS, Clarabel and SCIP per thread.

PETSc and IBEX cases parallelize only across processes. W-B route C (worker slots) already supplies such processes. Ipopt with MUMPS behaves like process-scoped work unless it runs in separate processes.

### 1.4 Case- and structure-level parallelism (absent today)

**Ledger facts:** independent BTF blocks and SCCs never run in parallel, and all blocks share one backend; fit experiments run sequentially; multiple-shooting windows run sequentially; durable workers never batch, while local studies can use POUNCE's convex batch (W-B 2, 3, 4c, 5).

**Ownership and library permission.** All *Interface-checked* against Part A §1.10:

- **Independent blocks within one topological level of the block DAG.** The libraries permit one KINSOL context per thread, or POUNCE instances per thread. No library schedules blocks; ordering is the project's (structure from R2). *Partly* library-owned: per-block solves yes, block scheduling no.
- **Fit experiments.** Per-thread IDAS memories with separate contexts, Diffsol instances, and per-thread evaluators are all permitted. The NLP oracle is a synchronous callback, so the parallelism would run inside the callback, as nested work borrowing the job's allowance under §18.8 ("synchronous nested work borrows its allowance"). No library owns this fan-out.
- **Multiple-shooting windows.** One IDAS memory per window per thread is permitted. Adjoints need one memory per window, because `IDAAdjReInit` frees checkpoints (B3 §1.3). PETSc TS would serialize.
- **Batches of structurally similar points.** Library-owned **wholesale** for NLP and convex QP by POUNCE: `solve_nlp_batch_parallel[_warm]`, `solve_qp_batch_parallel[_warm]`, with inner-serial FERAL. The batch API needs the problems up front as a `Vec<T>`. The durable loop claims one candidate per action (W-B 2 route C), so it never forms a batch. That is a project scheduling choice, not a library limit.
- **Racing configurations.** Library-owned **wholesale** by SCIP's concurrent solve (deterministic mode in the adapter). No NLP library offers racing; POUNCE's second-opinion ladder is sequential within one solve.

**Necessity.** The target needs large SCCs with many small blocks, fitting with many observations, and long trajectories with sensitivities. Block-level, experiment-level and window-level parallelism are means to that. The current rules impose sequential evaluation by construction: one backend per block sequence, sequential loops in `oracle.rs:136` and `shooting.rs`. No library imposes it.

**Consequences.** Lane-level instance retention (§1.1) and structure-level fan-out compete for the same threads. Library pools add a third level: SPRAL OpenMP, Pardiso MKL, HiGHS, FERAL and faer. POUNCE upstream's guidance is "outer-parallel, inner-serial" (`pounce-convex/src/batch.rs:20-45`). It is the only explicit nesting rule any surveyed library publishes. *Interface-checked.*

### 1.5 Cancellation, cooperative stop and drain

**Ledger facts:** cancellation is a flag polled by native callbacks; threads are joined before permits are released; heartbeat loss cancels native work (W-B 1b, stage 10).

**Ownership.**
- Libraries own **cooperative stop with the iterate retained** for: Ipopt (`intermediate_callback` returning false), POUNCE, Uno (termination callback returning 0), PETSc (user converged or diverged reason), HiGHS (`user_interrupt`), SCIP (`SCIPinterruptSolve`, resumable), Clarabel, and Diffsol and IDAS between steps.
- KINSOL 7.1.1 has none (Part A §1.11).
- Drain and join stay the project's under §18.8. No library joins its caller's thread.

**Necessity.** Policy stops for layer-2 switching need the iterate and statistics, and so does diagnosable failure. The Ipopt and Uno bridges stop by throwing, which loses both (B1 §1.7, §3.7). That is a bridge choice.

### 1.6 Observation for the three selection layers

**Ledger fact:** today's in-run observation is progress events tapped to the store, in batches of 64 or every 200 ms (W-A 1.14; W-B 1b, stage 12).

**What each library can supply per layer.** *Interface-checked*, from Part A, B1–B4 and §1.

| Library | Layer 1: a priori facts it can compute before or at setup | Layer 2: in-solve cues (hook) | Layer 3: final record for history |
|---|---|---|---|
| Ipopt | none of its own beyond dimensions; `derivative_test` (option) | `intermediate_callback`: mode, inf_pr, inf_du, μ, ‖d‖, δw, α, line-search trials; `IpoptData` `info_string` | `Statistics()` (iterations, evaluation counts, infeasibilities, times; **only on cooperative return**); `TimingStats()` with `timing_statistics=yes`; linear-solver internals are journal-only |
| POUNCE/FERAL | FERAL symbolic: `work_estimate`, fill, supernodes, ordering used | intermediate callback plus Rust state; per-factorization `FactorStats` (inertia, `n_tiny`, pivots, κ₁) | `SolveStatistics` with `IterRecord` trajectory; restoration, quality escalations, dual divergence; `timing_stats()` |
| Uno | auto-preset thresholds (n+m, nnz) | termination callback: residuals, multipliers | result residuals (currently dropped), counts, CPU time |
| KINSOL | KLU `Symbolic.nblocks`/`maxblock` (if bound) | fixed point: the map callback (exact per-iteration history reconstructible); Newton: none in 7.1.1 | 12 counters, function norm, step length; KLU `rcond` (computed, not exported) |
| PETSc SNES/TAO/TS | `MatGetInfo` fill | `SNESMonitorSet`, `SNESSetConvergenceHistory` (library-owned record), KSP residual history and extreme singular values | iteration, linear-iteration and function-evaluation counts, converged reason |
| IDAS | Jacobian pattern; KLU symbolic | `IDA_ONE_STEP` loop: order, h, error-test failures | `IDAGetIntegratorStats`, error weights × local errors (stiff-state ranking), sensitivity and checkpoint statistics (all unbound) |
| Diffsol | — | `step()` loop: `order()`, `state().h` | `OdeSolverStatistics` (setups by cause) |
| HiGHS | presolved dimensions; basis availability | callback: iterations, MIP bounds, gap | 26 of 38 `HighsInfo` values bound; run time and IPM identity unbound |
| SCIP | — | events; 10.1 solving-phase flags (needs a bump) | `SCIPcollect*Statistics` datatree (unbound) |
| Clarabel | — | termination callback: μ, σ, step, residuals | full `DefaultInfo` (bound) |

**Ownership and necessity.**
- **Layer 1:** libraries own almost nothing. The a priori facts are structural (R2) and come from project analysis plus FERAL or KLU symbolic phases.
- **Layer 2:** libraries own the cue *sources*. The switch decision, and the cooperative stop that hands over the iterate, are project-owned.
- **Layer 3:** libraries own the raw statistics. Normalizing them across vocabularies and keeping them is the project's (R3).
- "Diagnostics and measurement are product capability" (maintainer requirement) makes layers 2 and 3 necessary. Today's progress-event tap carries only a small part of what the libraries offer (B1–B4 "unbound" tables, re-confirmed at HEAD, Part A §0.1).

---

## 2. Refined allocations

Part A sketched A (PETSc framework), B (Rust-native core) and C (class-specific adapters behind one plan). Here they are checked against the ledgers. All are *Proposed*.

**A. PETSc as the composable framework**
- **Undermined by:**
  - The ledgers' concurrency facts. PETSc is process-exclusive (W-B (a), the native threads row), while the product needs sixteen in-process lanes (W-B 2 route A) and in-process sweeps (1a).
  - Under A, every square, least-squares or dynamics case would serialize per process unless PETSc is rebuilt `--with-threadsafety` (one object set per thread) or cases move to worker processes.
  - Structure-level parallelism (NASM, field split) would need MPI ranks, which means processes.
- **Supported by:** single-session sequential work: horizon steps, staged initialization, recycle sweeps (W-B 4b, 5). There, one long-lived PETSc object set per session thread is retainable and composition inside one solve would replace stop-and-restart.
- **Refinement:** A is viable only as a per-session engine for sequential single-case work, or with a thread-safe rebuild. It does not fit as the case-parallel workhorse.

**B. Rust-native core (POUNCE/FERAL plus SUNDIALS)**
- **Supported by:**
  - No process-global mutex in POUNCE, FERAL or SUNDIALS.
  - POUNCE convex batch is already used in local studies (W-B 1a, stage 8b).
  - KINSOL child sessions are already cached per thread (W-B (c)), which matches SUNDIALS's per-context rule.
  - Transient fits are IDAS or Diffsol on the fit thread, where per-experiment memories could be retained (§1.1).
  - Typed per-iteration and per-factorization records answer layers 2 and 3.
- **Undermined by:**
  - The durable path never batches.
  - IDAES reference practice is Ipopt-with-HSL.
  - POUNCE's `!Send` session (thread-bound by type) means retention needs lane-thread affinity, not a pool.
  - FERAL's non-injectable pool size (R-41).
  - KINSOL 7.1.1 has no cooperative stop and no per-iteration Newton record.
- **Refinement:** B gives the best library fit for in-process case parallelism, studies and diagnosis, provided the scheduling unit becomes "a lane thread owning library instances across compatible points" and durable lanes can claim groups for batching.

**C. Class-specific adapters behind one plan (today's shape)**
- **Supported by:** the ledgers show C working, with library constraints already absorbed as project machinery: PETSc, HiGHS and IBEX locks, MKL and OpenMP scoping, the FERAL admission rule. Every class has its best library.
- **Undermined by:**
  - The defaults put the common NLP path on the one serialized linear solver (MUMPS).
  - Every library-retention opportunity is cut off by the per-call and per-point session rule (§1.1).
  - Observation is reduced to progress events.

**Refined synthesis: allocate execution by each library's concurrency class.** This is a library-shaped allocation of the *parallel-execution* responsibility, not of method choice:
1. **Thread-instance libraries:** SUNDIALS, HiGHS, Clarabel, SCIP, POUNCE/FERAL, and Ipopt with SPRAL or Pardiso. These run concurrently in one process, with instances retained on lane threads.
2. **Process-instance libraries:** PETSc, IBEX, and Ipopt with MUMPS. These run one per process for concurrency; they run on any session thread when sequential.
3. **Batch-native libraries:** POUNCE NLP and QP batches, SCIP concurrent racing, egobox `q_batch`. These are fed groups by the scheduler.

The project keeps:
- the mapping from each library to its class, since it follows from the library contracts above;
- scheduling and placement;
- capacity charging for library pools (R5).

---

## 3. Repeated or reconstructed work that the libraries would remove by construction

Each item names the ledger case and the library mechanism that already exists at the pins. All are *Proposed*, from *Interface-checked* mechanisms.

1. **A new native session per Python `solve_bound`** (W-A 2.7, (c) "Native session"). Ipopt `ReOptimizeTNLP`, POUNCE `TnlpPresolveSession::solve_warm_last`, KINSOL `replace` with `NoInitSetup`, HiGHS in-place `change*` with the retained basis, and Clarabel `update_*` keep structure, symbolic factorization and warm state inside one instance, if the instance outlives the call on a session thread.
2. **A new session per independent study point** (W-B 1a stage 8a; (c)). The same mechanisms apply per lane thread across compatible points (§17.6 compatibility stamp). Where points share a structure, POUNCE `solve_*_batch_parallel_warm` removes per-point session setup wholesale.
3. **Transient-fit re-integration from scratch** (W-B 3, stage 4b). IDAS `IDAReInit`/`IDASensReInit` on a per-experiment memory keeps the allocation and the KLU symbolic analysis across oracle calls. The integration itself must still rerun, because the trajectory depends on θ. Only the construction is removed.
4. **The HiGHS basis discarded by `clearSolver`** (B4 §1.3). HiGHS's own retained basis hot-starts after `change*` calls, so dropping the clear removes the export, round-trip and refactorization.
5. **PETSc per-attempt object creation** (B2 §2.3, `RequireReuse` refused). Kept objects skip the symbolic LU on an unchanged pattern and allow Jacobian lagging. Within one session only, because of the process mutex.
6. **Serialized Ipopt KKT work under concurrency** (§1.3). Choosing SPRAL, Pardiso or POUNCE/FERAL removes the queue by construction. MUMPS's mutex cannot be removed.
7. **Separately allocated KINSOL inner sessions for every new job thread** (W-B (c)). Already cached per thread. Long-lived lane threads would extend that cache's life by construction, since the cache ends when the thread ends.
8. **Lost iterate and statistics on policy stops** (Ipopt and Uno bridges throw). Each library's cooperative return keeps both (Part A §1.11).

Not removable by these libraries:
- re-reading durable seeds across processes or workers (§17.6 forbids crossing; no library persists native state);
- block, experiment and window scheduling (no library schedules these; the project or R2/R5 owns it);
- the MUMPS and PETSc exclusion itself.

---

## 4. Open premises that would change this mapping

| Premise | Would change | Evidence that settles it |
|---|---|---|
| A session thread may outlive a Python call or a study point (an owner for idle native instances) | §1.1 and §3 items 1, 2 and 7 | Design decision. §18.8 already admits idle sessions without active permits, and §17.6 defines compatibility. Settled by the coordinator's target, not by a library |
| Durable lanes may claim groups of compatible points | POUNCE batching in durable studies (§1.4) | Claim-protocol design (R1); the POUNCE batch API needs the problems up front |
| Ipopt's default linear solver can move off MUMPS without loss for small KKT systems | §1.3, §3 item 6 | Literature favours MA27/MA57 at small sizes (B5a), which are excluded. SPRAL and Pardiso per-call overhead at small n is unmeasured; a probe would compare iteration and factorization time per solve on a small and a medium fixture under SPRAL, Pardiso and MUMPS |
| PETSc can be rebuilt `--with-threadsafety` (shared libraries are already on) without bridge changes | allocation A's viability for case parallelism | Configure the pinned 3.24.0 with the flag, then run the existing PETSc fixtures from two threads with separate object sets |
| A `SUNContext` and its solver memory may be used sequentially on different threads | whether SUNDIALS instances can live in a pool rather than on a fixed thread | SUNDIALS documentation on context thread affinity. The pinned header only guarantees "each thread has its own context" |
| `diffsol` solver reconstruction per parameter set is cheap relative to integration, with `LinearSolver` symbolic retained | §3 item 3 for Diffsol | Static: does the project's `diffsol-la` `LinearSolver` keep its symbolic analysis across solver instances (`BN/dynamics/linear.rs`)? Then a timing probe |
| FERAL pool size becomes injectable (R-41; FERAL 0.19 unreviewed) | the nesting rule in §1.3 and capacity charging (R5) | FERAL 0.19 release notes and source |
| Enough independent blocks or SCCs per topological level exist in real flowsheets | value of block-level fan-out (§1.4) | Structure statistics from R2 (BTF level widths on reference packages) |
| KINSOL leaves the latest iterate in `uu` after an unrecoverable callback return | whether a KINSOL policy stop can hand over an iterate | KINSOL 7.1.1 source (`kinsol.c` return paths) or a probe |

**File written:** this file only.
