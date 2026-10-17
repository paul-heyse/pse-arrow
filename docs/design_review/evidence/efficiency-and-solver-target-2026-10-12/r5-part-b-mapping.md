> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R5 (library-research), Part B: obligations mapped onto the R5 libraries. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R5 Part B: mapping the W-A / W-B obligations onto capacity, parallelism and the Python boundary

Supporting contribution; no verdict or finding IDs. Baseline `main` at `6c368c88a`. Static reading
only. Inputs: `w-a-interactive-obligations.md`, `w-b-multicase-obligations.md` (cited as W-A *n.m*,
W-B *Jn.s* or W-B (a)/(b)/(c)), and R5 Part A (`r5-capacity-parallelism-boundary.md`, cited as
"Part A §n"). A few extra source reads were made to settle mapping premises; they are cited inline.
Ownership labels: **wholesale** (a library capability can own it end to end), **partly**, **none**.
All alternatives are *Proposed*.

One ledger correction first: W-B (b) calls FERAL's pool "own global pool". At the pinned rev it is a
**per-`Solver`** lazily built rayon pool, sized from `RAYON_NUM_THREADS`/`available_parallelism`,
not injectable, dropped unjoined with the solver (Part A §6; `vendor/feral/src/numeric/solver.rs`
~l.425–720). The capacity consequence is the same (it sits outside admission) but its lifetime
follows the solver, not the process.

---

## 1. Obligations in scope, by kind

### 1.1 CPU admission (W-B §0, (b); W-A 1.1, 1.15)

Mechanisms: process `cpu` semaphore = min(`available_parallelism`, `pool_threads`) shared by compiler
and native work; `jobs` (population incl. idle sessions) and `active_jobs`; `try_acquire` refusal
for native entry; `admission_wait` (30 s) for queued entry; `ComputeAdmission` pause/try-resume
around native-exclusion waits; per-step permits in `Staged`.

- **Library ownership: partly.** tokio `Semaphore` owns counting, FIFO queueing, owned permits that
  cross threads, close-on-shutdown (Part A §1.3). It cannot know when a holder stops computing; that
  release-while-waiting (`pause`/`try_resume`) has no library equivalent in rayon, tokio or the native
  libraries (Part A §4.1). Width-to-library translation (setting OpenMP/MKL/HiGHS/SCIP/POUNCE counts
  from acquired permits) is project code by necessity: each library takes its count at entry.
- **Necessity.** *Target:* a finite CPU envelope for sixteen cases plus library teams, and no
  oversubscription by active teams. *Rule-imposed:* three separate semaphores, immediate refusal
  rather than queueing for native entry (`jobs` "including admission waiters", RC-D2 in the
  10-11 inquiry; ADR-0166 proposed), and the default `jobs = 4` (`MathPolicy::default`; Plan 36
  `coordination.md` "Retain those defaults"). The target needs one CPU number per process and a
  separate population/memory bound; it does not need the job slot to double as a thread bound.
- **Mapping consequences.** A single core semaphore acquired at actual team width and released
  while waiting (Part A A2) subsumes `cpu` + `active_jobs`; `jobs` remains only as a population or
  memory bound. Loses: nothing in the target; the lane-width rule `min(cores, jobs)` would become a
  pure policy choice instead of a coupling of two counters.

### 1.2 Thread lifecycle and drain (W-A 1.15, 4.1; W-B §0, J1a.8a, J1a.10, J1b.12, (a) native row)

Mechanisms: a fresh `pse-math` OS thread per job and per `Staged` session, joined through
`spawn_blocking(handle.join())`; permits and leases released only after join; stop drains issued
lanes; a scoped `pse-python-pull` thread per Arrow pull.

- **Library ownership: partly.** `std::thread::scope`, rayon `build_scoped` and `JoinHandle` witness
  joins; tokio-util `TaskTracker` (`close`/`wait`, `token()` held by a native thread until after its
  join) can own the drain census across tasks and native threads; `CancellationToken` owns signal
  fan-out (Part A §1.4). None can stop a running native call, so the cancel flag polled at native
  checkpoints and the join-before-release ordering stay project-owned.
- **Necessity.** *Target:* native objects are `!Send` and must be destroyed on their owning thread
  (§18.8); capacity must survive caller cancellation until join (§18.8). Both are genuine. *Not
  required by any contract:* a **fresh** OS thread per independent point. §18.8 requires an owning
  thread, not a new one; the per-point thread comes from `Staged::open_queued` per independent point
  (W-B J1a.8a, (c) "Native session … every independent study point").
- **Mapping consequences.** Long-lived owner threads (a fixed set per process, each serving many
  sessions in turn) keep the same owning-thread and join-witness contracts at session scope, while
  removing thread creation, one parked blocking-pool thread per live session, and — material for
  native libraries — the loss of per-thread library state (see §3). Weakens: a session that leaks
  thread-local state into the next session on the same thread; this needs an explicit reset contract
  (§18.8 already has one for "drained compatible empty scratch").

### 1.3 Library-internal team sizing (W-B (b) Library rows)

Mechanisms: POUNCE `with_threads` scoped pool with reserved stacks; FERAL parallel only when admitted
≥ its pool size; Ipopt SPRAL/Pardiso via `mkl::Threads`; HiGHS `threads`; SCIP `parallel/maxnthreads`;
Clarabel `max_threads`; surrogate local pool; Symbolica `cores = 1`; faer `Par::rayon(admitted)` in
`implicit.rs` dispatching to the **global** rayon pool.

- **Library ownership: partly.** Every library owns its team once given a count; none owns the count.
  faer has no pool of its own: `Par::Rayon(n)` runs on the ambient registry (Part A §1.2), so the
  `implicit.rs` site is the one place where structure-level work escapes both admission and any scoped
  pool. A `pool.install` around it (or calling it inside an existing scoped pool) would make rayon own
  it wholesale.
- **Necessity.** *Target:* only active teams count against CPU; idle persistent teams cost threads and
  stacks. *Rule-imposed:* FERAL "parallel only when admitted covers the whole pool" is a project rule
  compensating for non-injection (§18.8); under sixteen lanes it makes FERAL always serial, which
  `with_parallel(false)` already gives bit-identically.
- **Mapping consequences.** Deriving all counts from one per-job width removes per-library special
  cases except FERAL (not injectable) and HiGHS/OpenMP (per-thread persistent teams). Loses nothing.

### 1.4 Process-global native exclusion (W-B (a) native row)

PETSc process phase mutex, HiGHS lifecycle `RwLock`, IBEX mutex, MUMPS sections inside Ipopt.

- **Library ownership: none.** The libraries require exclusion and provide no in-process mechanism
  (Part A §4.2). A3 (process partitioning) makes it unnecessary per process; nothing in-process does.
- **Necessity.** *Target:* genuine for PETSc and IBEX. HiGHS 1.15's executor handle is `thread_local`
  (Part A §6), so a process-wide reset gate may be broader than the library needs; open premise P4.

### 1.5 Case-level concurrency and lane width (W-B J2 routes A–C, (b) Case rows)

`study_inner` lanes `min(cores, jobs)` via `FuturesUnordered`; worker `serve_owned_lanes` with
`maximum_in_flight == case_lanes` enforced at `publish_ready`; supervisor worker slots; concurrent
`RunHandle` supervisors; POUNCE convex batch only in the local route.

- **Library ownership: partly.** `FuturesUnordered`/`JoinSet` + `Semaphore` own bounded issuance and
  completion-order collection; rayon could own CPU-only fan-out but each durable lane is dominated by
  RPC stages (W-B J1b.7–14: discovery, claim, heartbeat every 10 s, progress, chunked publication),
  which must not occupy rayon workers. Process slots are owned by systemd/cgroups (Part A §1.7).
- **Necessity.** *Target:* sixteen independent cases concurrently with finite capacity. *Rule-imposed:*
  `maximum_in_flight == case_lanes` and the `case_lanes=16, cpu_threads=16, math_jobs=32` profile
  are deployment declarations (W-B J2 route C; `.config/agent-capacity.toml` is harness-side, W-B (a)
  "Harness and agent-tooling concurrency"). The ledger keeps these separate; only the product
  supervisor's execution profile is product capacity.
- **Mapping consequences.** Separating I/O stages (tokio, unbounded by CPU, bounded by memory and
  store) from the native stage (core semaphore) lets lane count exceed core count without
  oversubscribing CPU. Loses the simple "one lane = one case = one slot" accounting.

### 1.6 Structure-level teams (W-B J3.6, (b) Structure rows, J4c, J5)

Profile chains charge `threads × workers` through `quiescent_phase`; implicit factorization uses the
global pool; BTF blocks, SCCs, shooting windows and fit experiments are sequential (W-B J3 facts,
J4c, J5 "Structure-level parallelism … none found").

- **Library ownership: partly** (rayon scoped pools and `scope`/`par_iter` own fork-join, load
  balance and join; rayon owns no capacity). Detailed in §2.
- **Necessity.** *Target:* the workload premises include an interactive single case, long
  trajectories with sensitivities and fitting with many observations — all cases where cores are idle
  if only case-level parallelism exists. Under sixteen concurrent cases on sixteen cores, structure
  parallelism has no spare cores. Both premises are in the brief; the allocation has to serve both.

### 1.7 Process-level capacity (W-B J1b.2, J2 route C stages 1–3, (a) supervisor rows)

systemd scopes with `memory.max`/`cpu.max`, `verify_placement_hierarchy`, `taskset`, OpenMP re-exec,
host ledger, `primary-admission.json` under `fcntl`.

- **Library ownership: wholesale for enforcement** (cgroups, affinity); none for the ledger and
  supervisor protocol. Under affinity, `available_parallelism`, libgomp and FERAL/rayon defaults
  self-size correctly; HiGHS `threads=0` does not (`hardware_concurrency` ignores affinity, Part A §1.7).
- **Necessity.** *Target:* durable studies run by managed workers need a deployment envelope; the
  host ledger also serves agent tooling (W-B (a)), which is not product capacity.

### 1.8 Python boundary (W-A 1.1–1.3, 1.15, 1.18, 4.1–4.3, (a); W-B §0)

Process-static tokio `EXECUTOR` with `worker_threads = pool_threads`; one `PROCESS_RUNTIME` budget
per process; `detach` + 100 ms `block_on`/`check_signals` loop; scoped pull thread per Arrow pull;
Python holds only frozen handles; Runtime always connects to the store; no native state survives a
Python call (new session per `solve_*`).

- **Library ownership: partly.** pyo3 owns attach/detach, `Send + Sync` handle semantics, signal
  polling, `Py<T>` lifetimes; pyo3-async-runtimes owns asyncio↔tokio bridging with drop-cancellation;
  pyo3-arrow owns zero-copy export (Part A §1.6). The signal→cancel translation and owner-thread
  placement of `!Send` state stay project-owned.
- **Necessity.** *Target:* interactive value edit → re-solve must not rebuild structure; a live
  session is not prohibited by pyo3 (a `frozen` pyclass holding a sender to a session-owning thread is
  exactly the `NativeSession` shape). *Rule-imposed:* store connection and canonical admission on every
  call (§19, §21, scope table "Canonical store availability", ADR-0164 proposed — R1's responsibility);
  per-call session close (`run.rs::launch` opens and closes a `Staged` per run, W-A (a)). The budget
  singleton (ADR-0046) is a genuine process-envelope obligation.
- **Mapping consequences.** A Python-held session handle removes per-call native session and thread
  creation (W-A 1.15, 2.7) and enables warm starts and retained factorizations across value edits
  (R2/R4 own their numerical validity). pyo3 adds no new cost; it weakens nothing on the pyo3 side.
  The `worker_threads = pool_threads` sizing makes the I/O runtime as wide as the CPU budget although
  its CPU work (DataFusion VAL, W-A (b)) is outside the semaphore — a sizing coupling, not a library
  requirement.

---

## 2. Structure-level parallelism: where it could run and under which capacity allocation

Candidates and their independence premises (scientific independence is R2/R4's to confirm; this is
the execution side):

| Candidate | Independence available | Current state | Execution fit |
|---|---|---|---|
| Independent BTF blocks / SCCs at the same level of the block DAG | Blocks with no path between them in the condensation; a chain is sequential | One task (W-B J5.5) | Many small blocks: fork-join with grain control (`par_iter` over a level, `with_min_len`); one large SCC: its own factorization team instead |
| Multiple-shooting windows per NLP evaluation | Given node values, windows integrate independently | Sequential (W-B J4c) | Coarse, uniform-ish tasks; ideal for a fixed team; each window needs its own integrator/worker |
| Fit experiments per oracle call | Independent given θ; steady workers already per experiment (`workers[ei]`) | Sequential loop (`fitting/oracle.rs` l.137) | Coarse tasks with retained per-experiment workers; ordered reduction needed for deterministic addend order (DP-11; 10-11 inquiry D §4) |
| Transient fit experiments | Independent integrations | Sequential; integrator rebuilt per call (W-B J3.4b) | As shooting windows |
| Profile chains | Independent chains | `std::thread::scope` team, charged `threads × workers` | Already parallel; charge per phase |
| NASM / field-split subdomains (PETSc) | Subdomain solves | PETSc under a process mutex | In-process parallelism blocked by PETSc's process-global constraint; only A3 (processes) or PETSc's own MPI decomposition applies |
| FD Jacobian column groups, sensitivity parameters | Independent columns/parameters | Library-internal (IDAS staggered) or sequential | Fine-grained; library-owned where available |

**Thread-local prerequisites (Implemented, extra reads).** Structure tasks on another thread do not
inherit: the thread-local `COMPUTE` admission and `ORIGINAL` execution (`bn/execution.rs` l.53–57,
`OriginalScope` is deliberately `!Send`), the KINSOL inner-session cache and its per-thread budget
(`bn/implicit.rs` l.124, `budget_sessions` called once per session thread in `staged.rs`), POUNCE's
`ADMITTED` (`bn/pounce.rs` l.164), and TLS in `pse-math/src/{construction,typed_receipts}.rs`.
rayon propagates no context; a structure task must receive its scope explicitly, or the pool must set
it per thread (`spawn_handler`/`start_handler`, or `broadcast` on entry). Inner native teams inside a
structure task must be width 1, or each pool thread becomes an OpenMP master / HiGHS main with its own
persistent team (Part A §2).

**Capacity allocations (Proposed), from the libraries' nesting behaviour:**

- **S1. Per-job scoped team at admitted width.** The job (or session) acquires `w` core permits;
  structure work runs in `ThreadPoolBuilder::new().num_threads(w).build_scoped(...)` with the job's
  scope installed per thread; inner library teams forced to 1. This is POUNCE's existing pattern
  generalized. Under sixteen lanes on sixteen cores `w = 1` and it degenerates to today's sequential
  path; for one interactive case or one fit `w` = all cores. Discharged by rayon: fork-join, balance,
  join-before-return. Project: choosing `w`, TLS propagation, reserving `w` stacks for the scope
  lifetime (as F04 now does for POUNCE). Cost: a pool build per scope unless the scope is retained by
  a long-lived session; persistent idle team stacks while retained.
- **S2. Process pool shared by case- and structure-level CPU work** (Part A A1). Cases and structure
  tasks are all rayon jobs; work stealing redistributes cores from finished cases to wide structure
  levels automatically. Discharged: dynamic redistribution without a width decision. Lost: per-case CPU
  charge; isolation of long uninterruptible native calls (a pool worker blocked in a factorization);
  owner threads for retained `!Send` sessions must stay outside the pool and `install` into it, which
  blocks them (non-pool caller) — acceptable. Risk: cross-pool `install` from FERAL/POUNCE pools.
- **S3. Elastic width at phase boundaries.** S1 plus re-acquisition between phases (estimation at
  `threads`, profile at `threads × workers`, D-06 / Plan 36 fitting interface): FIFO `acquire_many`
  head-of-line blocking (Part A §1.3) means wide requests should be issued only at quiescent points
  and should accept partial width (`try_acquire_many` down a ladder) rather than queue for full width.

Interpretation: S1 fits the current owners with least change and covers both workload premises; S2
gives better utilization when cases are few and uneven, at the cost of per-case accounting. Neither
helps PETSc subdomains in-process.

---

## 3. Refined allocations against the ledgers

**A1 (rayon-centric process pool) → refined to S1 per-job scoped teams, with S2 as an option.**
Supported: no structure-level parallelism exists yet (W-B J5, J3, J4c), so nothing must be migrated;
POUNCE already uses the scoped pattern with charged stacks (W-B (b)); faer's `implicit.rs` site needs
only an ambient scoped pool to stop using the global pool. Undermined for case level: every durable
point is RPC-heavy (W-B J1b.7–14), and heartbeat/progress tasks are tokio tasks (W-B (a) background
tasks), so cases cannot be rayon jobs end to end; TLS-bound native state (§2) makes free migration of
work across pool threads unsafe without explicit scope transport.

**A2 (tokio for I/O and supervision; compute on owned threads under one core semaphore).** Supported:
the ledgers show most per-point obligations are store I/O and supervision (W-B J1b; W-A 1.14, 1.17);
lanes and `RunHandle`s already run as tokio tasks; drain is already "stop issuance, drain issued,
join" (W-B J1a.10, J2 C.4) — the `TaskTracker`+token shape. Weakly undermined: DataFusion VAL and
physical-admission queries execute on tokio workers (W-A (b)) and so sit outside the core semaphore;
Python's executor is sized to `pool_threads`. Refinement: long-lived owner threads (a fixed set per
process) serve sessions instead of a fresh thread per point (§1.2).

**A3 (process-partitioned lanes).** Supported: the managed primary and worker slots already are
processes with verified cgroup caps (W-B J2 C.1–3); PETSc/IBEX exclusion is per-process; worker
processes already reconstruct packages from canonical sources (W-B J1b facts). Undermined: that
reconstruction is repeated per process and per lane (W-B J1b facts, (c) "Modeling package in
workers"), so more processes means more recompilation unless products are shared; the interactive
path lives in the Python process (W-A). Refinement: A3 is the natural owner for process-global
libraries (PETSc NASM, IBEX) and for durable studies; A1/S1 + A2 inside a process for everything else.

---

## 4. Repeated or reconstructed work the libraries would remove by construction

1. **Per-point OS thread creation and join** (W-B J1a.8a, J1b.12; W-A 1.15, 2.7): a fixed set of
   long-lived owner threads (std threads or a pool with `spawn_handler`) removes it, together with one
   parked `spawn_blocking` thread per live session.
2. **Per-thread native library state rebuilt per point**: libgomp creates a team per new master thread
   and keeps it until that thread exits; HiGHS initializes a `thread_local` executor per new calling
   thread; the KINSOL inner-session cache is thread-local and starts empty on each new session thread
   (`bn/implicit.rs` l.124). A fresh thread per point discards all three every point; long-lived owner
   threads keep them warm by construction (the cache's reuse validity is already governed by its
   layout key).
3. **Per-call native session in the Python loop** (W-A 1.15, 2.7, (c) "Native session"): a
   pyo3 `frozen` handle owning a session sender keeps the session across value edits; pyo3 imposes
   nothing that prevents it.
4. **Global-pool dispatch for implicit factorizations** (W-B (b) Structure row): running inside a
   scoped pool makes the admitted width the actual width with no extra machinery.
5. **FERAL pool creation per solver under case concurrency**: `with_parallel(false)` gives the
   identical sequential result without building a pool that admission forbids using anyway.
6. **Separate team charging for profile phases** (W-B J3.6): S3's phase-boundary re-acquisition uses
   the existing semaphore; no new mechanism.

Not removed by any library here: per-point canonical premise re-establishment, publication volume,
result re-encoding, store reads per point (R1–R3's areas).

---

## 5. Open premises

| # | Premise that would change the mapping | Evidence that would settle it |
|---|---|---|
| P1 | Evaluators, providers and per-experiment workers are `Send` (needed for S1/S2 structure tasks). `CaseWorker` holds `Box<dyn Provider>`; its bound was not read | Static: the `Provider` trait bounds and a `Send` assertion on `CaseWorker`, `IntegratedExperiment` worker, block solver state |
| P2 | Complete inventory of thread-local state a structure task depends on (§2 list may be incomplete) | `rg 'thread_local!'` across `crates/*/src` with a per-site propagation decision |
| P3 | Which native libraries tolerate concurrent independent instances on different threads in one process (diffsol/IDAS/KINSOL private contexts: yes per prior review; Ipopt+MUMPS: guarded sections; PETSc: no) | Library thread-safety contracts per backend (R4) and the existing conformance parallel controls |
| P4 | HiGHS 1.15 executor is per OS thread, so the process-wide reset gate may be unnecessary except for same-thread reconfiguration and Uno's embedded HiGHS | Read `HighsTaskExecutor` lifetime with Uno's use; one small two-thread control if a decision depends on it |
| P5 | Desired split between case width and per-case team width (fixed `w = cores/lanes`, or elastic) | Maintainer workload premise: how often fewer than sixteen cases run while cores are idle |
| P6 | Deterministic reduction order for parallel experiments, windows and blocks (DP-11, `propagation_preserves_scalar_sum_bits`) | Existing determinism tests; ordered reduction design |
| P7 | Share of a durable lane's wall time spent in store I/O versus native solve (D-03) | A per-stage timing span in an existing study fixture (a measurement; not run) |
| P8 | libgomp spin cost with many concurrent OpenMP masters and `OMP_WAIT_POLICY` unset | Decide `OMP_WAIT_POLICY=passive` by rationale, or measure if contested |
| P9 | Whether the interactive Python loop must touch the store on every call (W-A (a)) | R1's contract analysis; it bounds how much a live session handle can skip, not whether pyo3 permits it |
| P10 | FERAL `Solver` lifetime in the POUNCE adapter (retained per role lease vs per solve) decides how many idle FERAL teams persist | Read `bn/pounce.rs` retention leases for the FERAL backend |
