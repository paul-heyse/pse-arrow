> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R5 (library-research), Part A: capacity, parallelism and the Python boundary. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R5: capacity, parallelism and the Python boundary (Part A)

Inquiry R5 of the consolidated efficiency and solver target review. Supporting contribution only:
no verdict, finding identifiers or decision. Baseline `main` at `6c368c88a`, 2026-10-12, clean tree.
Read-only: no builds, tests, probes or services. One cheap static tool was run:
`cargo metadata --format-version 1 --offline` to read the *resolved* feature set of the relevant
crates (it changes nothing). Evidence labels follow principles §D: **Interface-checked** = pinned
source or documentation read; **Implemented** = current code path read; everything about
alternatives is **Proposed**.

Sources used:

- Pinned registry sources under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:
  `rayon-1.12.0`, `rayon-core-1.13.0`, `faer-0.24.4`, `tokio-1.53.2`, `tokio-util-0.7.19`,
  `pyo3-0.29.2` (CHANGELOG and `guide/src/{free-threading,parallelism}.md`),
  `pyo3-async-runtimes-0.29.0`, `symbolica-3.0.1`.
- POUNCE/FERAL at the pinned rev: `~/.cargo/git/checkouts/pounce-8316ac8e070882e5/ff63869/`
  (`vendor/feral/src/numeric/solver.rs`, FERAL 0.18.0).
- HiGHS 1.15.0 source through the `native-solver-libraries` skill corpus
  (`content/corpus/highs-sys@1.15.0/HiGHS/highs/parallel/`, `lp_data/Highs.cpp`).
- `symbolica-faer-oximo` skill index for faer `Par` / `get_global_parallelism` (0.24.4).
- Context7: `/websites/rs_rayon_rayon` (install/in_place_scope/yield), `/websites/rs_tokio_tokio`
  (`spawn_blocking`, `max_blocking_threads`), `/websites/pyo3_rs_main` (pyclass thread safety, abi3t).
  Context7 tracks `latest`; every claim below was re-checked against the pinned source where it matters.
- Prior evidence reused, not redone: the 2026-10-08 parallel-execution review (F01–F05 and its §8
  library table), the efficiency inquiry D file `multi-case-workflows-and-capacity.md` (D-01–D-08),
  Plan 36 `coordination.md` (admission/coexistence section), §18.8 of `numerical-execution.md`.
  Deltas against them are in §6.

---

## 1. Capability surface at the pinned versions (neutral)

### 1.1 rayon 1.12.0 / rayon-core 1.13.0 — work-stealing pools

| Aspect | Contract (Interface-checked) |
|---|---|
| What it does | Fork-join data parallelism (`join`, `scope`, `in_place_scope`, `par_iter`, `spawn`, `broadcast`) over a fixed-size *registry* of OS threads with per-thread deques and work stealing. |
| Pools | One lazily created **global** registry (size `RAYON_NUM_THREADS`, else `available_parallelism`; `build_global` once, fails if already initialized) plus any number of **local** `ThreadPool`s. Every pool owns its threads; pools do not share or lend threads. |
| Which pool runs work | Work runs on the *ambient* registry: the pool the current thread belongs to, else the global one. `ThreadPool::install(op)` moves `op` into a target pool. |
| Install from a non-pool thread | Caller blocks on a `LockLatch` (`registry.rs::in_worker_cold`); it does not help. |
| Install from another pool's worker | Caller spins on a `SpinLatch::cross` and **keeps executing its own pool's pending work** while waiting (`in_worker_cross`; docs "Warning: execution order"). Unrelated work can therefore run on that stack, inside any locks the caller holds. FERAL's source cites a "nested-rayon workspace-mutex self-deadlock" (its issue #102) from exactly this shape. |
| Lifetime | `ThreadPool::drop` calls `registry.terminate()` — threads exit **asynchronously, not joined**. `ThreadPoolBuilder::build_scoped(wrapper, with_pool)` runs inside `std::thread::scope`, so all pool threads are joined before it returns (deterministic teardown, TLS borrowable). |
| Thread hooks | `spawn_handler` (custom thread creation: name, stack, affinity, ICVs, permit acquisition), `start_handler`/`exit_handler`, `stack_size`, `thread_name`, `panic_handler`, `breadth_first`, `use_current_thread` (local registry is leaked by design). |
| Cancellation | None. Started jobs run to completion; cooperative stop must be polled by the job. Panics propagate to `join`/`scope`/`install` callers. |
| Introspection | `current_num_threads`, `current_thread_index`, `current_thread_has_pending_tasks`, `yield_now`/`yield_local`. No CPU budget, no metrics. |
| Nesting / oversubscription | Nested `join`/`par_iter` inside one pool never creates threads (it just enqueues). Oversubscription arises only from **multiple pools** (or rayon + other runtimes): thread count is the sum of pool sizes; rayon has no cross-pool or cross-library budget. |
| Newer release | 1.12.0 (2026-04-13) is the pinned and newest seen; changes since 1.11 are iterator additions/bug fixes, nothing material to capacity. |
| Simulator responsibility | Structure-level parallelism (independent BTF blocks/SCCs, Jacobian columns, multiple shooting segments, profile chains, batched members); library-internal parallelism (FERAL, faer, POUNCE convex batch, Symbolica streaming). Case-level parallelism only if cases are pure CPU work without blocking I/O. |

### 1.2 faer 0.24.4 — `Par`

- `Par::Seq` or `Par::Rayon(NonZeroUsize)`. `Par::rayon(0)` = `rayon::current_num_threads()` of the
  *ambient* pool. **`Par::Rayon(n)` is a split hint, not a pool**: `utils::join_raw` recursively halves
  `n` and calls `rayon::join`, which runs on whichever registry the calling thread is in, else the
  global registry (Interface-checked, `faer-0.24.4/src/utils/mod.rs`, `src/lib.rs` ~l.940).
- Global setting: `set_global_parallelism` / `get_global_parallelism` / `disable_global_parallelism`
  (an `AtomicUsize`; default `Rayon(0)` when the `rayon` feature is on). Resolved features in this
  workspace include `rayon` (pulled by unification although the pin sets `default-features = false`),
  so a call that consults the global setting defaults to the whole global pool (cargo metadata).
- Scratch (`MemStack`) requirements are queried per `Par` — wider `Par` means larger scratch.
- Sparse LU/Cholesky choose simplicial vs supernodal by flop/nnz heuristics; `Par` matters only on
  supernodal/dense kernels (D-08 already recorded this).
- No cancellation inside a factorization. Responsibility: per-block factorization and dense kernels.

### 1.3 tokio 1.53.2 — runtime, blocking pool, sync, cooperative budget

Resolved features: `rt`, `rt-multi-thread`, `sync`, `macros`, `time`, `process`, `io-util`, plus
`fs`, `net`, `signal`, `io-std` from unification (cargo metadata).

| Aspect | Contract (Interface-checked) |
|---|---|
| Flavours | `current_thread` (one thread drives all tasks) and `multi_thread` (`worker_threads`, work stealing). |
| Blocking pool | `spawn_blocking` threads are separate from workers, default cap `max_blocking_threads = 512`, idle exit after `thread_keep_alive` (10 s). The queue applies **no backpressure** and may grow unbounded. A started blocking task **cannot be aborted** (`abort` is a no-op once running); runtime shutdown waits for them unless `shutdown_timeout`/`shutdown_background`. |
| `block_in_place` | Multi-thread only; hands the worker's core to another thread while the closure blocks. |
| `Handle::block_on` | Allowed from a non-runtime thread or a blocking-pool thread; panics on a runtime worker. |
| Cancellation | Dropping a future is cancellation at its next `.await`; `select!`, `timeout`, `JoinHandle::abort`, `JoinSet::abort_all`/drop all reduce to drop. No preemption of CPU-bound code. |
| Cooperative budget | Per-task budget (128 operations per poll) consumed by tokio resources; `task::coop::{consume_budget, has_budget_remaining, unconstrained, cooperative, poll_proceed}` are public. CPU work on a worker thread starves its siblings regardless of budget. |
| `Semaphore` | **Fair (FIFO)**, including `acquire_many`: a queued large request blocks later small ones (head-of-line). `try_acquire*` never queues. Owned permits (`OwnedSemaphorePermit`) move across tasks/threads; `close()` wakes all waiters with an error; `add_permits`/`forget` resize. Permits are counters only — they do not bind threads or memory. |
| Metrics | Stable: `num_workers`, `num_alive_tasks`, `global_queue_depth`; blocking-pool depth and per-worker metrics require `tokio_unstable`. |
| Newer release | 1.53.2 is the pinned and newest seen locally; no material change identified (search: local registry plus Context7 `latest`, not exhaustive). |
| Responsibility | I/O and RPC (SurrealDB WebSocket, DataFusion/object_store), supervision, timers/deadlines, bounded fan-out of futures; *not* a CPU scheduler. DataFusion executes its CPU operators on tokio worker threads. |

### 1.4 tokio-util 0.7.19 (features resolved: `rt`, `codec`, `compat`, `io`)

- `CancellationToken`: tree (`child_token`), `cancel`, `cancelled`/`cancelled_owned`,
  `run_until_cancelled[_owned]`, `drop_guard[_ref]`. Signal only; cancels nothing by itself.
- `TaskTracker`: `spawn`, `spawn_blocking`, `track_future`, `close`/`reopen`, `wait`, `len`, and
  **`token()`** — a `TaskTrackerToken` counted as a live task until dropped. This lets non-tokio work
  (for example a native OS thread holding the token until after join) participate in one drain wait.
- `AbortOnDropHandle`, `JoinQueue`, `PollSemaphore`, `PollSender`, `LocalPoolHandle`.
  `JoinMap` needs the `join-map` feature, not enabled.
- None of these provides CPU or memory brokerage, forced stop of blocking work, or priority.

### 1.5 Native library thread controls (pinned sources/builds)

| Library | Thread mechanism and control | Coexistence and stop behaviour |
|---|---|---|
| Ipopt + MUMPS (seq) | Serial; Ipopt guards MUMPS sections (prior review §9). | Intermediate callback is the stop point; a factorization finishes first. |
| Ipopt + SPRAL | GNU OpenMP. `omp_set_num_threads` sets the **calling thread's** ICV, so concurrent masters each get their own team. Env (`OMP_CANCELLATION`, `OMP_PROC_BIND`, `OMP_PLACES`) is read once at libgomp start. `OMP_WAIT_POLICY` is unset in the image (`docker/solvers/Dockerfile`), so libgomp's default spin-then-sleep applies to idle team threads. | Idle team threads persist per master thread for the process lifetime (libgomp pool). |
| Ipopt / Clarabel + oneMKL Pardiso | `MKL_Set_Num_Threads_Local` (thread-local), `MKL_DYNAMIC` off, `MKL_CBWR` pinned. | Same GNU OpenMP runtime (`mkl-dynamic-lp64-gomp`). |
| HiGHS 1.15 | Work-stealing `HighsTaskExecutor`. Source shows the "global" executor handle is **`thread_local`** (`HighsTaskExecutor.cpp`: `static thread_local ... globalExecutorHandle`): the first `run` on an OS thread initializes an executor for that thread with `threads` workers (0 → `(hardware_concurrency()+1)/2`, which ignores affinity). A different `threads` value later on the same thread is an error until `resetGlobalScheduler`. | Interrupt callback for simplex, IPM, MIP; QP and PDLP do not poll (§18.8). |
| SCIP 10.0.2 (`TPI=tny`) | `SCIPsolveConcurrent` deterministic mode with min/max threads; `lp/threads=1`. | Event handler copies poll cancellation. |
| POUNCE | Local admitted rayon pool via `with_threads` (`build_scoped` → joined at scope end); POUNCE-convex batch uses `par_iter` on the ambient pool. | TNLP callbacks are the stop points. |
| FERAL 0.18.0 | **Per-`Solver` lazily built local rayon pool**, sized `RAYON_NUM_THREADS` else `available_parallelism`, reused for that solver's lifetime, released by `ThreadPool::drop` (unjoined). Not injectable; `with_parallel(false)` forces the bit-identical sequential driver; on pool-build failure it falls back to sequential, never to the global pool. | No cancellation inside a factorization. |
| Clarabel 0.11.1 | QDLDL serial; Pardiso via MKL threads (`max_threads`). | Termination callback. |
| Symbolica 3.0.1 | Evaluator optimizer uses `std::thread::scope` with `n_cores` OS threads, capped by `LicenseManager::max_threads` (unlicensed → 1). Streaming/`map_terms` build their own rayon pools. | `abort_check` closure. |
| PETSc 3.24 | Not thread-safe for concurrent lifecycle; project keeps a process phase mutex (`pse-backend-native/src/petsc.rs`). | — |
| SUNDIALS KINSOL/IDAS (7.1.1 via `sundials-sys` 0.6.2) | Serial vectors; private `SUNContext` per instance. | Evaluation / step boundaries. |
| egobox/linfa, ndarray (`rayon` feature resolved), blake3 `update_rayon` | Global rayon pool. | — |

### 1.6 pyo3 0.29.2, pyo3-async-runtimes 0.29.0, pyo3-arrow 0.19.0

- **Attach/detach.** `Python::detach(f)` releases the GIL (or detaches in free-threaded builds);
  `f` must be `Ungil` (`Send` on stable). The guide requires detaching around any code that spawns
  worker threads or may re-attach, to avoid hangs and deadlocks. `py.check_signals()` is the only
  signal hook; there is no built-in "cancel on KeyboardInterrupt".
- **Thread-safety of exposed objects.** `#[pyclass]` must be `Send + Sync` (since 0.23) unless
  `unsendable` (pinned to its creating Python thread; access from another thread panics).
  `frozen` removes the runtime borrow flag; interior mutability is the author's (`Mutex`, atomics).
  `PyOnceLock`, `MutexExt::lock_py_attached` and `OnceLockExt` detach while blocking.
- **Free-threaded Python.** Since 0.28, modules default to declaring free-threaded support
  (`gil_used = false`); 0.29 supports 3.14t+ and drops 3.13t. **`abi3`/`abi3-pyXX` is ignored with a
  warning on free-threaded interpreters**; the PEP 803 `abi3t`/`abi3t-py315` features (new in 0.29)
  target the 3.15+ stable free-threaded ABI. The resolved pyo3 features here are `abi3`,
  `abi3-py311`…`abi3-py315`, `extension-module`, `experimental-inspect` (cargo metadata).
- **Object lifetimes.** `Py<T>` is `Send + Sync` and can be held by Rust threads; dropping it without
  being attached defers the decref to a pending pool. Long-lived Rust state behind a `pyclass` lives
  as long as Python references it; interpreter finalization does not run Rust destructors of statics.
- **Async.** `future_into_py[_with_locals]` spawns the Rust future on the registered tokio runtime;
  cancelling the asyncio future **drops the Rust future** (oneshot cancel channel,
  `generic.rs` `Cancellable`). `into_future` goes the other way. `tokio::init_with_runtime` binds one
  process runtime.
- **pyo3-arrow 0.19.** `PyArray`, `PyRecordBatch`, `PyRecordBatchReader`, `PyTable`, `PySchema`, with
  the Arrow PyCapsule interface (`__arrow_c_array__`, `__arrow_c_stream__`): zero-copy export/import of
  buffers through the C Data/Stream interfaces; a stream is consumed once. `buffer_protocol` feature is
  resolved.
- Responsibility: the Python boundary for an interactive session (edit → re-solve), result access,
  async waiting and interruption.

### 1.7 OS-level controls (Linux)

- `std::thread::available_parallelism` honours the affinity mask and cgroup v2 CPU quota; libgomp
  default team size honours affinity; C++ `hardware_concurrency` (HiGHS default) does not.
- The managed primary is placed with `taskset --cpu-list` and a systemd `CPUQuota`
  (`scripts/surreal_server.py::role_command`, `resources`), so in-process defaults derived from
  `available_parallelism` see the placement width.
- Thread stack sizes are explicit per spawn (`std::thread::Builder::stack_size`, rayon `stack_size`).

---

## 2. Nesting and oversubscription behaviour across these libraries

Interface-checked facts about what happens when one mechanism runs inside another; interpretation is
labelled.

| Outer → inner | Behaviour |
|---|---|
| rayon pool worker → `install` into another pool | Outer worker steals its own pool's work while waiting; cross-pool interleaving inside held locks. |
| non-pool thread (native session thread) → `faer::Par::Rayon(n)` | Runs on the **global** rayon pool (all sessions share it); `n` bounds splitting, not threads. |
| POUNCE `with_threads(k)` scope → faer/convex `par_iter` | Runs on the local `k`-thread pool. |
| any thread → FERAL parallel factor | FERAL's own per-solver pool (`available_parallelism` threads), outside any outer pool. |
| rayon/tokio thread → OpenMP (SPRAL/Pardiso) | Each calling OS thread becomes an OpenMP master with its own team; teams persist per master. |
| any thread → HiGHS | Per-OS-thread executor with `threads` workers, persistent on that thread until reset. |
| tokio worker → CPU-heavy code (DataFusion operators, synchronous preparation) | Worker blocked; tokio budget does not preempt. |
| tokio → `spawn_blocking` | New OS thread up to 512; unbounded queue; uncancellable once started. |
| Python thread → Rust | GIL held unless `detach`; a Rust thread that needs `attach` while the caller holds the GIL deadlocks. |

**Interpretation.** No library here offers a cross-library CPU budget or a hook through which another
library's pool could borrow threads. Every library-internal team is either *sized by the caller at
entry* (OpenMP ICV, MKL local, HiGHS `threads`, SCIP thread count, rayon local pool size, Symbolica
`n_cores`, faer `Par`) or *self-sized from the process environment* (FERAL, rayon global, HiGHS with 0,
libgomp defaults). Oversubscription is therefore governed by two things the libraries do provide: the
entry-time count, and the process environment/affinity. Idle persistent teams (libgomp, HiGHS
executors, FERAL per-solver pools, the rayon global pool) cost threads and stacks but, apart from
libgomp's spin phase, not CPU.

---

## 3. Current use (pointers, no judgment)

| Family | Where (paths under `crates/`) |
|---|---|
| tokio runtime construction | Python: `pse-py/src/inspection/runtime.rs::acquire` (static `EXECUTOR`, `multi_thread`, `worker_threads = budget.threads.pool_threads`, default blocking pool) and `pse-py/src/workflow/modeling.rs` (~l.1831); worker: `xtask/src/bin/pse-worker.rs` (`worker_threads(2)`, ctrl-c `select!`). |
| tokio `Semaphore` as capacity | `pse-runtime/src/env.rs::SharedRuntime::build` (`compiler_cpu` = min(`available_parallelism`, `pool_threads`)), `pse-runtime/src/math.rs` (`cpu`, `jobs`, `active_jobs`), `math/jobs.rs` (`admit_entry`, `try_acquire_owned`, `acquire_many_owned`), `math/staged.rs` (session slot, per-step CPU), `pse-engine/src/cache_service/mod.rs` (loads/queries/outputs), `pse-engine/src/stores.rs`. |
| Memory admission | DataFusion `FairSpillPool` + `TrackConsumersPool` + `PeakRecordingPool` (`env.rs`), `MathPolicy` (`math.rs`), §18.8. |
| `spawn_blocking` / `block_on` | `math/staged.rs` (`spawn_blocking(thread.join)` per session), `math/jobs.rs`, `workflow/durable.rs`, `workflow/modeling/pure.rs`; `pse-operations/src/canonical.rs` `Handle::block_on` inside preparation jobs; `pse-py/src/workflow.rs::blocking_on_with_recovery`. |
| tokio-util | `CancellationToken` in `pse-columnar` (`flight.rs`, `resource.rs`, `owned_buffer.rs`, canon) and `pse-testkit`; `TaskTracker` only in `pse-operations/src/canonical_results.rs`; `JoinSet` in `pse-operations` canonical modules and `pse-engine/src/session/assurance.rs`; `FuturesUnordered` in `workflow/{worker,study_execution}.rs`, `modeling/conformance.rs`. |
| Dedicated OS threads | Native session `pse-math` threads (`math/staged.rs::session_on_reserved`), job threads (`math/jobs.rs` ~l.1077), `pse-python-pull` (`pse-py/src/workflow.rs::blocking_native_call`), structural matching stacks (`pse-structural/src/incidence.rs`), `pse-compiler/.../executable/grouped.rs`, `pse-relations/src/validate/prepared/consolidation_unit.rs`, `math/surrogate.rs`. |
| rayon local pools | `pse-backend-native/src/pounce.rs::with_threads` (`build_scoped`), `pse-runtime/src/math/surrogate/library.rs` (`build()`, egobox); FERAL internal. |
| rayon ambient/global | `pse-backend-native/src/execution/pounce_convex.rs` (`par_iter`); faer `Par::rayon(admitted)` in `pse-math/src/implicit.rs` (~l.1046); `Par::Seq` elsewhere (`pse-math/src/sparse.rs`, `dynamics/linear.rs`, `fitting/oracle.rs`); blake3 `update_rayon` gated by `ResourceBudget::hashing_may_use_pool`. No `set_global_parallelism`/`build_global` call found (rg over `crates/`). |
| Native thread controls | `pse-backend-native/src/mkl.rs::Threads` (OpenMP + MKL local, restored on drop), `ipopt/settings.rs`, `conic.rs`, `scip.rs`, `highs.rs` + `highs_lifecycle.rs` (process `RwLock` around reset), `petsc.rs` (process phase mutex), `root_isolation.rs` (IBEX mutex), `pounce.rs::feral_threads` (FERAL parallel only when admitted ≥ its pool size). |
| Compute pause/resume | `pse-backend-native/src/execution.rs` (`ComputeAdmission` trait, `compute_scoped`, `pause_compute`, `try_resume_compute`). |
| Symbolica cores | `pse-math/src/library.rs::Optimization { cores: 1 default, ≤ 64 }`. |
| pyo3 | `py.detach` around Arrow table/cursor production (`pse-py/src/workflow/modeling.rs`), 100 ms poll loops with `check_signals` and cancel callback (`pse-py/src/workflow.rs`), `future_into_py_with_locals` for `NativeRunHandle::wait_async`, all `pyclass(frozen)`, module without explicit `gil_used`; pyo3-arrow `PyRecordBatchReader`/`__arrow_c_stream__` in `pse-py/src/inspection/stream.rs`; Python package built `abi3` (`python/pse/_native.abi3.so`, interpreter pin 3.14.7). |
| Deployment placement | `scripts/surreal_server.py` (`taskset`, `CPUQuota`, `cpu_threads`, `math_jobs`, `--maximum-in-flight`); `pse-worker` re-executes with OpenMP env. |

---

## 4. Responsibility questions (answered from library behaviour)

### 4.1 Single capacity model, or bounded independent pools with oversubscription rules?

Facts that decide it:

1. **CPU.** Every parallel library either takes an entry-time thread count or self-sizes from the
   environment (§2). None can lend threads to, or borrow from, another. A process-wide count of
   *active computing threads* can therefore only be enforced at the points where the project passes a
   thread count in; it cannot be enforced inside the libraries.
2. **Waiting.** A thread blocked in a library guard (PETSc phase, HiGHS reset gate, IBEX, MUMPS
   sections), on a rayon `LockLatch`, or on I/O does not compute. A counter that is held across those
   waits measures *ownership*, not CPU use (prior F03; Plan 36 C8 `try_resume`).
3. **Memory.** Only DataFusion's pool offers cooperative reservation; native libraries allocate opaquely.
   No library supplies a wait queue for memory (DataFusion pools refuse; tokio semaphores queue but count
   nothing physical).
4. **Persistent teams.** libgomp, HiGHS executors, FERAL per-solver pools and the rayon global pool
   persist after the call that created them; their *threads and stacks* are a retained lifetime distinct
   from their *CPU* use (prior F04 dealt with POUNCE's).

**Interpretation (neutral).** CPU and memory behave differently:

- For **CPU**, bounded independent pools are adequate *if* the entry-time counts are derived from one
  per-process number and summed only over concurrently computing teams. Oversubscription by idle teams
  is harmless except for libgomp spinning (controlled by `OMP_WAIT_POLICY=passive` or
  `GOMP_SPINCOUNT`, which the image does not set). The libraries' own contracts therefore favour
  *one CPU number per process, split at entry*, rather than a scheduler that spans libraries. A
  semaphore of cores, held only while actually computing, is the library-consistent realization; the
  libraries give nothing finer.
- For **memory**, a single accounting authority is needed because the libraries offer none, but it can
  only be conservative admission (§18.8 already states this).
- **Thread count and stack memory** of persistent teams are a third quantity; they are bounded only by
  how many distinct OS threads become OpenMP masters, HiGHS mains or FERAL solver owners. Long-lived,
  few, reused owning threads keep this finite; many short-lived threads do not (libgomp keeps a pool
  per master until the master exits).
- Whether one *model* covers all three is a design choice; the libraries neither need nor support a
  unified scheduler.

### 4.2 Which admission, cancellation and drain obligations do the libraries already discharge?

| Obligation | Discharged by a library | Genuinely project-owned, and why |
|---|---|---|
| Bounded concurrency of futures | `Semaphore`, `FuturesUnordered`, `JoinSet`, `buffer_unordered` | What a permit *means* (cores, jobs, sessions) and its lifetime. |
| Fair queueing | tokio `Semaphore` is FIFO | Head-of-line blocking by `acquire_many` (D-06) is a consequence of that fairness; avoiding it needs request shaping, not a different primitive. |
| Cancellation signalling | `CancellationToken` tree, drop-cancellation of futures, pyo3-async-runtimes cancel→drop | Mapping the signal to **native checkpoints** (Ipopt intermediate callback, HiGHS interrupt, SCIP event handler, KINSOL evaluation, Symbolica `abort_check`): libraries expose a polling hook, the project must poll the token in it. Native factorizations and HiGHS QP/PDLP cannot be interrupted. |
| Drain / teardown witness | `TaskTracker` (`close`+`wait`, `token()` for non-task work), `JoinHandle`, rayon `build_scoped`, `std::thread::scope` | Native objects are `!Send` and must be destroyed on their owning thread; only `join` witnesses TLS destruction. `ThreadPool::drop` (FERAL, `surrogate/library.rs`) does **not** join. |
| Deadlines | `tokio::time::timeout`, Ipopt `max_wall_time`, HiGHS `time_limit`, SCIP `limits/time`, Clarabel `time_limit` | Carrying one original clock across retries and stages. |
| Blocking isolation | `spawn_blocking`, `block_in_place` | `spawn_blocking` cannot stop started work and its queue is unbounded, so it is not admission. |
| Thread counts for native teams | Each library's entry setting | Deriving all of them from one process budget; reading back env-fixed settings (OpenMP). |
| Process-global native state | none | PETSc lifecycle, HiGHS reset, IBEX, MUMPS: exclusion is library-*required*, its implementation is necessarily project code (prior F03). |
| Memory limits | DataFusion pools; SCIP `limits/memory`; FERAL bounded-storage profile | Everything else (opaque native heaps). |
| GIL / interpreter | `detach`, `check_signals`, `PyOnceLock`/`MutexExt`, `Py<T>` deferred decref | Polling signals during long native work and translating them into cancellation; owning-thread placement for `!Send` state. |

### 4.3 The Python boundary: an in-process, long-lived Python session over a live Rust model

What pyo3 requires and offers (Interface-checked), with the current shape (Implemented):

- **Ownership.** A live model holding native solver objects is `!Send` (§18.8). A `pyclass` must be
  `Send + Sync` unless `unsendable`, and `unsendable` would pin it to one Python thread and panic
  elsewhere. Library-consistent shape: the `pyclass` holds a `Send + Sync` handle (`Arc`, channel
  sender) to a Rust-owned thread that owns the live model — which is what `NativeSession` already is
  (`math/staged.rs`: one `pse-math` thread, `mpsc` requests, join-witnessed close). Values that are
  pure data (evaluators, prepared structure) can sit behind `Arc` and `Mutex`/`RwLock` in a `frozen`
  pyclass.
- **GIL.** Every call that waits on Rust must `detach`; the current poll loop
  (`blocking_on_with_recovery`: `detach(block_on(timeout(100ms)))` then `check_signals`) follows the
  guide. Nothing on the Rust side may `attach` while a Python caller waits attached; the code attaches
  only in `future_into_py` completion.
- **Cancellation.** Signals are visible only through `check_signals` polling; asyncio cancellation
  arrives as a dropped future. Both must be turned into the session's own cancel flag, because dropping
  a future does not stop a native step (§18.8; tokio `spawn_blocking` contract).
- **Zero copy.** pyo3-arrow exports Arrow buffers through the PyCapsule C interfaces without copying;
  inbound Arrow (for example parameter vectors from NumPy/pyarrow) can be imported the same way, and
  the pyo3 buffer protocol (`PyBuffer<f64>`) reads contiguous NumPy memory. No `numpy` crate is pinned.
  Zero copy into a solver needs the Rust side to accept borrowed slices for the duration of a call or to
  copy once into session-owned storage (values-only edits are small).
- **Runtime lifetime.** One process tokio runtime (`static EXECUTOR`) and one `PROCESS_RUNTIME` are
  never shut down; at interpreter exit, statics are not dropped, so in-flight native threads are not
  joined by Rust (Interface-checked for statics; exit behaviour not exercised).
- **Free-threading.** With `abi3` resolved, a free-threaded interpreter build ignores abi3; a
  free-threaded wheel would need a separate non-abi3 build (3.14t) or `abi3t` (3.15+). The module does
  not set `gil_used`, so under 0.29 it declares free-threaded support by default; `frozen` pyclasses with
  `Sync` contents are already the required shape. Free-threading would let several Python threads drive
  independent sessions concurrently without serializing on the GIL; it does not change native
  thread-safety constraints.

---

## 5. Library-shaped allocations

Each sketch is **Proposed**. "Makes unnecessary" is at responsibility level.

### A1. One rayon-centric compute pool per process; library thread counts derived from it

**Shape.** One process `rayon::ThreadPool` sized to the placement width (`available_parallelism`
under taskset/cgroup) built with `build_scoped` or kept for process life, with `spawn_handler`
setting names, stacks and OpenMP/MKL ICVs per worker. Case-level and structure-level work are rayon
jobs (`scope`/`par_iter`/`spawn`), so nested parallelism (cases → blocks → Jacobian columns → faer
`Par::rayon(0)`) is work-stolen inside one fixed thread set. Libraries with their own teams receive
counts derived from the same width: FERAL `with_parallel(false)` inside the pool (or only at top
level), OpenMP/MKL/HiGHS/SCIP/Symbolica at 1 when nested under case parallelism, >1 only for a single
large case. Native session state lives on long-lived dedicated threads outside the pool (they are
`!Send` owners), and those threads `install` into the pool for structure-level parallel phases.

- **Discharged by the library:** thread creation and reuse, load balancing across uneven blocks and
  cases, nested parallelism without new threads, join of scoped work, panic propagation.
- **Remains the project's:** memory admission (rayon has none); cooperative cancellation polling;
  `!Send` owner threads and their join; process-global native exclusion (PETSc, HiGHS reset, IBEX);
  deriving every non-rayon library count; I/O must stay off the pool (a rayon worker blocked on a
  SurrealDB RPC removes a core).
- **Lost or weakened:** an explicit per-case CPU charge (work stealing makes per-case CPU use
  emergent); deterministic placement of a case on a fixed team; FERAL/OpenMP/HiGHS parallelism under
  case-level concurrency (they cannot join the rayon pool, so they run serial or oversubscribe);
  isolation — a long uninterruptible factorization occupies a pool worker; cross-pool `install` from
  pool workers steals unrelated work inside held locks (FERAL #102 hazard), so native owner threads
  must stay outside the pool.
- **Cost:** moderate integration (case driver becomes rayon-shaped; async preparation/publication must
  hand off to tokio), low upgrade cost (rayon is stable), testing must cover nested-install and
  long-job starvation.
- **Makes unnecessary:** per-job CPU-permit arithmetic for structure-level teams; POUNCE/surrogate
  ad hoc local pools (replaced by the process pool); faer global-pool ambiguity (D-08 seam) because the
  ambient pool is always the process pool; team-stack reservations per scope (pool stacks are fixed and
  pre-charged once).

### A2. tokio for I/O and supervision only; compute on owned threads with one core semaphore

**Shape.** tokio (multi-thread, small `worker_threads`, bounded `max_blocking_threads`) owns RPC,
DataFusion queries, timers, progress streams and supervision via `CancellationToken` trees and one
`TaskTracker` per runtime/study (native owner threads hold a `TaskTrackerToken` until after `join`).
All CPU work runs on long-lived owned threads: native session threads (existing shape) plus one
process rayon pool for structure-level parallelism. CPU capacity is a single `Semaphore` of cores,
acquired with the team width a step will actually use and released whenever the step blocks on a guard
or I/O (the `try_resume` pattern). Library thread counts are set from the acquired width at entry.

- **Discharged by the library:** I/O concurrency and backpressure (`Semaphore`, bounded channels),
  structured cancellation (`CancellationToken`), drain accounting (`TaskTracker` including native
  tokens), deadlines (`timeout`), FIFO fairness, idle blocking-thread reaping.
- **Remains the project's:** the meaning and lifetime of the core count; memory admission;
  `!Send` owner threads; native checkpoint polling; process-global exclusion; release-while-waiting at
  native guards; preventing tokio workers from doing CPU work (DataFusion operators run on tokio workers
  — either a separate DataFusion runtime or accepting that query CPU is outside the core semaphore).
- **Lost or weakened:** work stealing across cases (static team widths per step); FIFO
  head-of-line blocking for wide requests unless widths are shaped; one more thread hop at every
  async↔compute boundary.
- **Cost:** low for the I/O side (much already exists); moderate to separate DataFusion CPU from
  supervision workers; testing focuses on release-while-waiting and drain.
- **Makes unnecessary:** bespoke population/drain bookkeeping where `TaskTracker` + tokens can witness
  completion; `spawn_blocking(thread.join)` per session (a tracker token dropped after join plus a
  oneshot gives the same witness without a parked blocking thread); separate job-slot semaphores whose
  only role is to bound threads rather than memory.

### A3. Process-partitioned lanes: the OS as the capacity model

**Shape.** Case-level parallelism is realized by N worker processes, each pinned (taskset/cgroup) to a
disjoint CPU set and memory slice; inside a process everything is serial or uses the whole local slice.
Library defaults then become correct: `available_parallelism`, libgomp, FERAL, rayon global and MKL all
self-size to the slice (HiGHS with `threads=0` does not, because `hardware_concurrency` ignores
affinity, so it still needs an explicit count). Process-global native state (PETSc, HiGHS scheduler,
IBEX, MUMPS) is naturally exclusive per process.

- **Discharged by the OS/libraries:** CPU partitioning and isolation (cgroups/affinity), memory
  ceilings (`MemoryMax`), crash isolation, native process-global constraints, library thread defaults.
- **Remains the project's:** a work queue and claim protocol between processes (today the SurrealDB
  study claims), artifact sharing across processes (re-preparation or a shared on-disk/IPC product
  form), progress and result transport, supervision and drain of processes, the interactive in-process
  path (Python cannot cheaply cross a process for every edit).
- **Lost or weakened:** shared immutable preparation in memory across cases (each process holds its own
  compiled evaluators unless products are shared via files/mmap); dynamic rebalancing of cores between a
  wide single case and many narrow ones; fast interactive edit loops when they cross a process.
- **Cost:** low library integration, high supervision and transport cost (much of which exists in the
  managed-worker deployment); testing shifts to process orchestration.
- **Makes unnecessary:** in-process cross-backend exclusion strategy for process-global libraries
  (prior F03), per-library thread-count derivation under case concurrency, FERAL/OpenMP oversubscription
  concerns, much of in-process CPU-permit accounting for case lanes.

**Cross-cutting (interpretation).** The three are combinable: A2's I/O/compute split is compatible with
A1 inside a process and with A3 across processes. None removes the need for project-owned memory
admission, native checkpoint polling, owning-thread teardown, or process-global native exclusion; those
obligations come from the native libraries' contracts, not from the choice of executor.

---

## 6. Deltas against prior evidence

- **FERAL pool (R-41, §18.8).** At the pinned rev FERAL 0.18.0's pool is **per `Solver`**, lazily
  built, sized from `RAYON_NUM_THREADS`/`available_parallelism`, persisting for the solver's lifetime
  and dropped without join — not a single process-global pool (Interface-checked,
  `vendor/feral/src/numeric/solver.rs` ~l.425–720). The "cannot be injected" premise holds;
  `with_parallel(false)` gives the bit-identical sequential driver. Consequence for retained sessions:
  each retained parallel FERAL solver keeps its own idle team alive.
- **faer `Par::rayon(n)` (D-08 follow-up).** D-08's `get_global_parallelism` call is gone;
  `pse-math/src/implicit.rs` now passes `Par::rayon(admitted)`. Because `Par::Rayon(n)` runs on the
  *ambient* registry, a call from a native session thread outside a POUNCE scope still executes on the
  **global** rayon pool, shared by all sessions; `n` bounds splitting, not which threads run it
  (Interface-checked; whether this path is reached with `admitted > 1` was not traced).
- **HiGHS scheduler.** The 1.15 source makes the executor handle `thread_local` (per initializing OS
  thread). The prior review described a process scheduler lifecycle; the project's process-wide
  `RwLock` around `resetGlobalScheduler` (`highs_lifecycle.rs`) may be broader than the library's
  per-thread state requires. Not verified at runtime; the reason the project guard exists (Uno's HiGHS
  subproblems, reset semantics with in-flight workers) was not re-traced. Hand to R4/Part B.
- **pyo3 free-threading.** New since prior evidence: 0.28 made `gil_used = false` the default, 0.29
  added `abi3t` and dropped 3.13t; the project's `abi3` build cannot produce a free-threaded wheel.
- **OpenMP wait policy.** `OMP_WAIT_POLICY`/`GOMP_SPINCOUNT` are not set by the image or `pse-worker`
  (rg over `docker/`, `crates/`, `scripts/`, `.config/`); idle SPRAL/Pardiso teams under concurrent
  masters spin for libgomp's default period before sleeping. Not previously recorded.
- **Unjoined rayon pool drops.** `math/surrogate/library.rs` uses `ThreadPoolBuilder::build()` (drop
  does not join); POUNCE uses `build_scoped` (joined). The teardown-witness contract in §18.8 is met only
  by the latter (Interface-checked; effect not measured).
- Prior parallel review F04 (team stacks) is recorded resolved by D; F05 dispatch resolved by D;
  nothing here contradicts those.

## 7. Material uncertainties and unfinished items

- No runtime check of any nesting behaviour; all thread-count consequences are from source.
- Whether libgomp teams under the project's affinity/`OMP_PLACES=sockets` placement actually bind as
  the image README expects under many concurrent masters is unverified.
- HiGHS thread-local executor: semantics read from source only; interaction with Uno's embedded HiGHS
  not traced.
- The resolved cargo features come from `cargo metadata` over the whole workspace; a single-crate
  build under ADR-0122 unification is expected to match but was not checked per crate.
- Newer-release claims cover the local registry and Context7 `latest` only; upstream release lists were
  not fetched.
- Part B (mapping W-A/W-B obligations onto these libraries) is pending.
