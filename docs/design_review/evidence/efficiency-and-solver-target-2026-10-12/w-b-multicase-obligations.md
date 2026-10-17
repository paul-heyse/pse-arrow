> Supporting evidence for [the efficiency and solver target review](https://github.com/paul-heyse/pse-arrow/blob/6af92d1dee1ffa0b80dca71e03a63b489f2d940c/docs/design_review/reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry W-B (code-mapper): multi-case and long-running obligations ledger, including the product-concurrency inventory. Its description of FERAL's thread pool as global is corrected by R5 and X1: FERAL 0.18 builds one pool per solver. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# W-B: multi-case and long-running work, obligations ledger

Baseline: `main` at `6c368c88a`, clean tree, 2026-10-12. Static reading only: no builds, tests,
probes or services. Paths are repository-relative; `rt/` = `crates/pse-runtime/src/`,
`py/` = `crates/pse-py/src/`, `bn/` = `crates/pse-backend-native/src/`, `ops/` =
`crates/pse-operations/src/`.

Evidence labels: **Implemented** means a code path exists and I read it. **Interface-checked**
means I inspected a signature, field or doc comment but did not follow the full body.
**Interpretation** marks my reading of what the facts imply. Nothing here is Tested or Measured.

Column key used in every stage table:
- **Owner** is `path::symbol`.
- **Obligations** use these tags: DUR durability, COORD coordination or exclusion, REUSE reuse or
  invalidation, ASSUR assurance or validation, OBS observation or telemetry, CAP capacity or
  admission, CANC cancellation or drain.
- **Crossing** uses these tags: PY↔RS, PROC process, RPC WebSocket to SurrealDB (each RPC is one
  explicit `BEGIN … COMMIT` SurrealQL transaction with an RPC-expiry fence; see
  `ops/canonical_studies.rs`), THR thread, FFI native.
- **Repeats / grows with** says what work repeats per point or iteration, and what it scales with.
- **Lib** names the library doing the scientific or infrastructure work, or says "bespoke".

---

## 0. Entry points and shared runtime facts

Python entry points (`python/pse/_workflow.py`, `_runs.py`, `_modeling.py`):

| Entry | What it does |
|---|---|
| `Runtime(settings, substrate=…, ephemeral=…)` | Always connects to the canonical SurrealDB store, even when ephemeral (`py/workflow.rs` `NativeRuntime::new`: `CanonicalStore::connect` then `open`). Ephemeral only changes `DurabilitySelection` (`rt/workflow/mod.rs:303`). |
| `ModelingPackage.admit_study(request)` → `StudyDefinition` | Calls `py/workflow/modeling.rs::admit_study`, then `admit_study_sources`. |
| `ModelingPackage.study(definition)` (no runtime argument) | Calls `rt/workflow/study_execution.rs::ModelingPackage::study`, which branches on the runtime's durability: `study_inner` when ephemeral, `study_durable` when durable. |
| `ModelingPackage.study(definition, runtime=rt)` → `StudyHandle` | Calls `rt/workflow/study.rs::Runtime::start_defined_study`. Execution needs a worker: the managed primary, `pse-worker` slots, or `Runtime.work()`. |
| `Runtime.work(maximum_actions=…)` | Calls `rt/workflow/worker.rs::Runtime::serve` with `until_idle`. The Python process itself acts as a worker. |
| `Runtime.start([PreparedOperation…])` / `PreparedOperation.start()` → `RunHandle` | Calls `rt/workflow/run.rs`. "Every run is supervised under its runtime's durability class" (module doc). A durable run registers an attempt in-process. |
| `ModelingPackage.prepare_fit` / `prepare_simulation` / `prepare_recycle` / `prepare_block_initialization` / `initialize` | Prepared operations started through `RunHandle`. |

Shared runtime facts:

- **One Python process tokio runtime.** `py/inspection/runtime.rs:19,47`: a static `EXECUTOR`,
  `new_multi_thread().worker_threads(pool_threads)`. Every blocking call runs
  `executor.block_on` inside `py.detach`, with a 100 ms poll for signals
  (`py/workflow.rs::blocking_on_with_recovery`). Arrow pulls spawn a scoped `pse-python-pull`
  OS thread per pull (`blocking_native_call`). (Implemented)
- **One accounting pool.** `rt/env.rs::SharedRuntime::build` builds a DataFusion
  `TrackConsumersPool<FairSpillPool>` and a `PeakRecordingPool`. Every runtime reservation
  (`math:native-job`, `study:*`, `canonical:ordinary-progress`, …) is a `MemoryConsumer` on
  that pool. (Implemented)
- **One CPU semaphore shared by compiler and native work.** `compiler_cpu =
  Semaphore(min(available_parallelism, pool_threads))` is passed to `MathService` as `cpu`
  (`rt/env.rs:62`, `rt/math.rs:316`). `MathService` adds two more semaphores, `jobs(policy.jobs)`
  and `active_jobs(policy.jobs)` (`rt/math.rs:655`). The default `MathPolicy.jobs` is 4
  (`rt/math.rs::MathPolicy::default`). (Implemented)
- **One OS thread per native job.** `rt/math/jobs.rs` (around line 1077) runs each admitted job
  on a fresh `std::thread::Builder("pse-math").stack_size(policy.stack_bytes)` thread. The
  thread is joined through `tokio::task::spawn_blocking(handle.join())`. CPU permits are
  released only after the join. Cancellation sets a flag the native callbacks poll, and
  ownership is retained until the join (`jobs.rs:1093–1130`). (Implemented)
- **Staged native session.** `rt/math/staged.rs` module doc: "A session owns one native thread
  for its whole life … Each step … takes its CPU permits for the step only … keeps its job slot,
  stack and worker share until it closes." The CPU allowance is paused and resumed around
  native-exclusion waits through `ComputeOwner` (`staged.rs:232`, `bn/execution.rs:29–118`).
  (Implemented)

---

## Journey 1a: thousand-point value sweep, ephemeral (in-process)

Path: `Runtime(…, ephemeral=True)` → `ModelingPackage.admit_study` → `ModelingPackage.study(definition)`
→ `study_inner`.

| # | Stage | Owner | Scientific work | Obligations (mechanism) | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|---|
| 1 | Study admission | `py/workflow/modeling.rs::admit_study` → `ModelingPackage::admit_study_sources` | Resolve binding paths to members; physical unit checks; operation-owned seed need | ASSUR (binding identity, unit conversion once, §19.3); CAP (decode bounded by `math.workspace_bytes`) | PY↔RS (JSON document) | Once per study; grows with points × assignments | bespoke + pse-quantity |
| 2 | Extent and binding checks | `rt/workflow/study_execution.rs::study_inner` (start) | None | ASSUR (`binding_hash == binding.identity()`, seed-role admission); CAP (`reserve("study:occurrence-outcomes", points×(…+4096))`) | none | Linear in points | bespoke |
| 3 | Pure policy setup | `pse_model::study_policy::AdmittedStudy::new`, dependents map, `ContinuationRetention` | Dependency DAG of occurrences | COORD (pure transition policy: Wait/Start/Refuse/Cancel) | none | Once; grows with points and edges | bespoke |
| 4 | Lane width | same | None | CAP (`width = min(math.cores(), budget.math.jobs)`; `reserve("study:held-input-lanes")`) | none | Once | bespoke |
| 5 | Per-point preparation on a lane | `study_operations.rs::StudyPreparationLane::prepare` → `ModelingPackage::prepare_bound_operation_with_admission` (`rt/workflow/bindings.rs:184`) | Validate binding against compiled model; `resolve_prepared_case` + `finish_case`, a value rebind over a shared structural view | REUSE (lane keeps `DeclaredStudyAdmission` and `HeldPreparationScope` while the key `(descriptor, paths, member set)` matches; `PreparationCounts` views/rebuilt/shared); ASSUR (`validate_binding`, route/procedure check); CAP (`HeldPreparationScope` allocation leases) | async task (`FuturesUnordered`) | Per point: rebind plus value-dependent products only when consumed values change (§19.3 `PreparationCounts`); structural prepare once per lane per key | Salsa compiler lanes (`rt/math/modeling.rs::CompilerLane`); Symbolica programs via `MathService::artifact` (DataFusion `DefaultCache` keyed by content hash) |
| 6 | Source protection while held | `rt/workflow/modeling/engagement.rs::HeldPreparationScope::renew` / `PreparationEngagement::admit` | None | DUR/COORD (canonical source read-selection lease `renew(SOURCE_LEASE=3600 s)`, heartbeat every 10 s while native work runs, through `joined`/`during`) | RPC | Per engagement admission and every 10 s per held view | SurrealDB |
| 7 | Batch grouping | `study_inner` grouping loop | None | COORD (only fresh, ready, coefficient-representation cases whose adapter has `capability().batch`: `rt/math/solves.rs::batch_backend`) | none | Per frontier pass | bespoke |
| 8a | Single execution | `execute_lane_inner` → `Staged::open_queued` → `execute` | Native solve (Ipopt/KINSOL/…) | CAP (`jobs`/`active_jobs`/`cpu` permits plus pool reservation, `admission_wait` deadline); CANC (cancel flag, join before release) | THR (new `pse-math` thread per Staged session), FFI | **New native session per independent point**; retained only for a single continuation successor (`retained` map) | Ipopt, KINSOL, PETSc, HiGHS, … |
| 8b | Batched execution | `Staged::batch(&preparations, Obligations::Final)` | One native bulk solve of N coefficient cases | as 8a | THR, FFI | One session per batch group | POUNCE convex batch (`bn/execution/pounce_convex.rs::execute_batch`) |
| 9 | Record outcome | `record_execution`, `RunResult::joined(...).finished(None, …)` | Completion and qualification (original-space assessment) | ASSUR (completion owns qualification); OBS (task-local `PreparationCounts`) | none | Per point; results stay in memory (`StudyOccurrenceResult::Ephemeral`) | bespoke |
| 10 | Drain and close | end of `study_inner` | None | CANC (stop issuance, drain every pending future, close lanes and `Staged`; fatal adapter error drains too) | THR join | Once | bespoke |

Facts:
- The ephemeral sweep writes no run, attempt or result rows. The store is still touched for
  source-selection lease renewals (stage 6). (Implemented)
- Continuation points with a single successor inherit the predecessor's `Staged` native session
  and its retained native state (`study_execution.rs`, "A single continuation successor may
  inherit private native state"). Fan-out successors get seeds but independent sessions.
  (Implemented)

## Journey 1b: thousand-point value sweep, durable (managed workers)

Path: `ModelingPackage.study(definition)` on a durable runtime → `study_durable`, or
`study(definition, runtime=rt)` → `start_defined_study`. Execution is served by the managed
primary (`pse-worker`), by `surreal_server.py worker` slots, or by `Runtime.work()`.

| # | Stage | Owner | Scientific work | Obligations (mechanism) | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|---|
| 1 | Extent and revision check | `study_execution.rs::study_durable` | None | ASSUR (exact modeling revision; `MAXIMUM_STUDY_POINTS=100_000`) | PY↔RS | Once | bespoke |
| 2 | Ensure managed primary | `rt/workflow/worker.rs::Runtime::ensure_managed_primary` | None | CAP (observer pool must fit `observer_memory_bytes`); COORD (supervisor `primary-admission.json` reservation, `fcntl` state lock, systemd unit); readiness JSON | PROC (spawns `python scripts/surreal_server.py ensure-primary --observer-pid …`; supervisor may start `pse-worker` as a systemd unit) | Per `study()` call | bespoke Python supervisor + systemd |
| 3 | Persist sources | `Operations::put_sources` / `sources` (`worker.rs:754,861`) | None | DUR (immutable source revisions) | RPC | Once per study; grows with source bytes | SurrealDB |
| 4 | Reopen revision and re-admit points | `start_defined_study` (`study.rs:402`) | Rebuild the `ModelingPackage` from the canonical revision; recompute each point's seed need; prepare explicit-seed targets | ASSUR (seed need must equal recorded policy; binding identity); REUSE (one `StudyPreparationLane` plus `admission` across points) | RPC (revision read), compile | Per point: `admit_binding_seed_need`; grows with points | Salsa/Symbolica |
| 5 | Create study | `CanonicalStore::create_study` (`ops/canonical_studies.rs:492`); `promote_seed`, `retain_study_input` | None | DUR (study header, occurrences, per-point `StoredRequest` run rows; bulk insertion "amortizes round trips while each transaction remains" bounded); COORD (cancel checkpoint; `abandon_study_inputs` loop on failure) | RPC (bounded batches) | Grows with points × descriptor size | SurrealDB |
| 6 | Waiter poll (local call only) | `study_durable` loop | None | OBS (`terminal_inventory` every 100 ms); CANC (cancel → `handle.cancel()` once) | RPC per poll | Grows with wall time; one RPC per 100 ms | bespoke |
| 7 | Worker discovery | `worker.rs::work_once_scoped`: `study_page` → `advance_study_readiness` → `study_candidates` → `study_scope` | Pure policy `scope.action(...)` | COORD (readiness notifications; `CandidateGroup` local turn per candidate key); recovery cursor (`recover_next_study`, `repair_study_readiness`, `study_assigned_page`) | RPC × several per candidate | Per action; grows with active studies × candidate pages | SurrealDB + bespoke |
| 8 | Candidate preparation in lane | `study.rs::prepare_study_candidate_in_lane` | Reconstruct package if `(revision, physical receipt)` changed; lane `prepare`; seed acquisition (`study_input` + `artifact_seed`, or `latest_seed` for continuation); prediction setup | REUSE (`StudyCandidateLane.source` per lane; `StudyPreparationLane` key match); ASSUR (seed compatibility `with_admitted_start`) | RPC (point, study header, seed reads), compile | Per point: canonical point read plus header read plus seed decode; package rebuilt per lane on source change | Arrow IPC seed decode, Salsa |
| 9 | Claim | `CanonicalStore::claim_study_point` | None | COORD (fenced claim under scope revision, generation and predecessors; definite-conflict rediscovery); DUR (attempt record, lease `policy.lease=30 s`) | RPC | Per point | SurrealDB |
| 10 | Attempt start and heartbeat | `rt/workflow/durable.rs::DurableAttempt::start`, `Heartbeat::spawn` | None | DUR/COORD (`renew_attempt` every `heartbeat=10 s`; heartbeat failure cancels native work) | tokio task + RPC | Every 10 s per running point | bespoke over SurrealDB |
| 11 | Mark started; prediction; renew | `mark_study_started`, `apply_study_prediction`, `renew_attempt` | Prediction sample from predecessor (`prediction_sample`) | ASSUR (prediction is start-only meaning, §19.3) | RPC × 2+ | Per point | bespoke |
| 12 | Native execution | `prepared.operation.start_attempt(self, cancel, attempt)` inside `HeldPreparationScope::during` | Native solve (fresh `Staged` per attempt) | CAP (as 1a stage 8a); OBS (progress `Tap` → `Streamer`, batch 64 / flush 200 ms); CANC (heartbeat-loss cancel, durable cancellation flag) | THR, FFI, RPC (progress appends) | Per point; **no batching** in the durable path (one candidate per action) | native libs |
| 13 | Finish and publish | `DurableAttempt::finish` → `store_tables` (`ResultAppendQueue`) → `store_seeds` → `copy_original_completion` → `terminate` | Completion copy | DUR (Arrow IPC chunks under fence; publication acks `PublicationAcks`; closed manifest; seed rows); ASSUR (demand-selected detail, ADR-0180) | RPC (chunked appends, close/seal) | Per point; grows with result rows × relations selected | Arrow IPC + SurrealDB |
| 14 | Record and finalize | `record_study_attempt`; `finalize_canonical_study` / `begin_study_finalization` / `conclude_study` | Study outcome summary | DUR (`runtime.study_outcomes` per occurrence); COORD (finalize turn `finalize:{study}` in `CandidateGroup`) | RPC | Per point plus once per study | SurrealDB |

Facts:
- In `pse-worker`, `resource_budget` sets `cache: CacheBudget::disabled(1024)` (engine cache
  service). The `MathPolicy` artifact retention (`artifact_bytes`) stays enabled
  (`xtask/src/bin/pse-worker.rs`, `resource_budget`). (Implemented)
- A worker process reconstructs the modeling package from canonical sources and recompiles in
  its own `MathService` and Salsa compiler lanes. Nothing is shared with the Python process that
  created the study. Interpretation: the first point on each worker repeats the
  revision→package→compile path that the creating process already ran, and so does each lane
  whose `StudyCandidateLane.source` is empty. Whether service-level single-flight
  (`MathService.selected_flights`, `basis_flights`, `modeling_cache`) dedupes those lanes is
  Interface-checked only (the fields exist; I did not follow the hit path).
- Continuation reuse differs by executor:
  - Local: in-memory `MinimalContinuation` values plus an inherited native session.
  - Durable: the seed is reread from the store (`Operations::latest_seed`, Arrow payload decode),
    plus optional prediction. Native session state never crosses durable points. (Implemented)

## Journey 2: sixteen independent cases concurrently

There are three product routes. They have different capacity owners.

| # | Route | Owner | Concurrency mechanism | Capacity charge | Notes |
|---|---|---|---|---|---|
| A | Ephemeral or local study, 16 points | `study_inner` | `FuturesUnordered` lanes, `width = min(math.cores(), math.jobs)` | Per native job: `jobs` permit + `active_jobs` + `cpu` permits (`cores`) + pool bytes (`reserve_active("math:native-job", cores, bytes)`) | Default `MathPolicy.jobs = 4` caps width at 4 unless `EngineSettings` raises it (Implemented). The reference conformance run sets `math_jobs=32`, `threads=16` (`packages/reference/conformance.toml`). |
| B | 16 separate `RunHandle`s from one Python runtime | `rt/workflow/run.rs` supervisors (`tokio::spawn` per run) | Independent supervisors; durable runs register attempts and heartbeat in-process | Same math semaphores; queue on `admission_wait` (30 s default); "an ephemeral run is admitted at once or refused" (`run.rs` module doc) | Each durable run owns its own `DurableAttempt`, `Heartbeat`, `Streamer` and RPCs. |
| C | Durable study served by the managed primary | `pse-worker` → `Runtime::serve` → `serve_owned_lanes` | `width` lanes (`maximum_in_flight = case_lanes`); refill on progress; poll otherwise | Worker-process `ResourceBudget` from the supervised execution profile (`pool_memory_bytes`, `cpu_threads`, `math_jobs`, `worker_bytes`); cgroup `memory.max` and `cpu.max` verified (`verify_placement_hierarchy`) | The primary requires `--maximum-in-flight == case_lanes` (`publish_ready`). The exclusive profile is `case_lanes=16, cpu_threads=16, math_jobs=32` (`.config/agent-capacity.toml`, harness-side declaration). |

Stage table for route C, per lane:

| # | Stage | Owner | Obligations | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|
| 1 | Worker launch | `scripts/surreal_server.py::worker` / `_ensure_primary`; `host_admission.py` | CAP (slot reservation files, `native_worker_memory_bytes` systemd scope cap, zero swap, 128 tasks); COORD (state lock; admission open/accepting_writes) | PROC, systemd | Once per worker | bespoke + systemd |
| 2 | Placement verification | `pse-worker::verify_placement` | ASSUR (cgroup memory/CPU quota ancestry; `available_parallelism ≥ cpu_threads`) | filesystem | Once | bespoke |
| 3 | OpenMP environment | `pse-worker::ensure_openmp_environment` | CAP (re-exec with `OMP_CANCELLATION`, `OMP_PROC_BIND`, `OMP_PLACES=sockets`) | PROC re-exec | Once | OpenMP |
| 4 | Lane serve loop | `worker.rs::serve_owned_lanes` | CAP (`reserve("study:worker-input-lanes")`); CANC (stop → drain each issued lane, then close lanes) | async | Per action | bespoke |
| 5–14 | As Journey 1b stages 7–14 | | | | | |

Facts:
- `CandidateGroup` is "Local preparation ownership only. Canonical claims remain the distributed
  authority" (`worker.rs:99`). Concurrent lanes and processes are made exclusive by the store's
  fenced claims. (Implemented)
- Lanes do not batch. The durable worker executes one candidate per action, so independent fresh
  points never share a POUNCE convex batch there. Route A can batch. (Implemented)

## Journey 3: parameter fitting, steady and transient

Path: `ModelingPackage.with_fit_declarations` → `prepare_fit(fit_id, request)` →
`PreparedOperation.start()` → `RunHandle`.

| # | Stage | Owner | Scientific work | Obligations | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|---|
| 1 | Fit preparation | `rt/workflow/fitting/modeling.rs`, `preparation.rs` ("Shared sparse fit assembly … after source interpretation") | Compile experiments with shared parameters; sparse derivative maps; measurement layout | REUSE ("Trial values never enter Salsa queries", `fitting.rs` doc); ASSUR (measurement and uncertainty admission) | compile | Once per fit; grows with experiments × observations | Salsa, Symbolica, faer (`fitting/sparse.rs`: "faer owns storage and Gram multiplication") |
| 2 | Run supervision | `rt/workflow/run.rs` | None | DUR (in-process attempt registration, heartbeat, progress); CAP (native job) | tokio, RPC | Once | bespoke |
| 3 | NLP solve | `fitting/oracle.rs::execute_profile` / `execute_inner` → `native::execution::nlp` | Ipopt (or another admitted NLP backend) on the fit oracle | CAP (one native job); CANC (`execution.stopped()` checked per evaluation) | THR, FFI | Once per fit | Ipopt / POUNCE |
| 4a | Steady experiment evaluation | `FitOracle::evaluate_demand`, `Experiment::Steady` | Constraints and Jacobian from a retained per-experiment `CaseWorker` | REUSE (`self.workers[ei]` retained across oracle calls; last-point cache by bitwise `x`) | in-thread | Per oracle call × steady experiments (sequential loop) | Symbolica evaluators |
| 4b | Transient experiment evaluation | `Experiment::Transient` → `IntegratedExperiment::integrate` (`rt/workflow/integrated.rs:199`) | Full integration per trial point, with forward sensitivity or adjoint gradient | REUSE (none at the integrator level: `self.program.worker(...)` is built per call, plus a new integrator via `integrate_with_progress_observed`); ASSUR (upgrade allowance check `admit_upgrade`) | in-thread FFI | **Per oracle call × transient experiments, sequential**; grows with horizon × states × (1 + sensitivities) | diffsol or SUNDIALS IDAS (`bn/dynamics.rs`) |
| 5 | Covariance and intervals | `fitting/covariance.rs`, `uncertainty.rs` | Wald covariance (KKT), propagation `Σ_y = J Σ_θ Jᵀ` | ASSUR (withheld vs certified) | in-thread | Once | faer |
| 6 | Profile-likelihood team | `fitting/profile.rs` (around line 751) | 2 chains per free parameter; pinned re-solves with secant/homotopy step control | CAP (`quiescent_phase(cores = threads × workers, bytes)` reacquires the active charge for the team; per-worker stacks plus adapter scope stacks); OBS (`actual_parallelism`, `scheduling_failures`) | THR (`std::thread::scope`, `workers` scoped threads, each inside `native::execution::scoped`) | Grows with parameters × points per chain | Ipopt via the one NLP runner |
| 7 | Publish | `DurableAttempt::finish` | Fit tables, `export_fit_parameters` | DUR (as Journey 1b stage 13) | RPC | Once | Arrow IPC |

Facts:
- Experiments are evaluated in a sequential loop per oracle call (`oracle.rs:136` `for (ei, e)
  in p.experiments`). I found no parallel experiment evaluation in `fitting/` (search:
  `spawn`, `par_iter`, `thread::scope`, `FuturesUnordered` across `rt/workflow/fitting/`; the only
  hit is `profile.rs`). (Implemented)
- Transient fits re-integrate from scratch at each trial point, with a fresh dynamics worker
  per call. Interpretation: IDAS or diffsol memory, symbolic factorization and the evaluator
  worker are not retained across Ipopt iterations.

## Journey 4: dynamics horizon (long trajectory with sensitivities; rolling horizon)

**4a. Single long simulation.** Path: `ModelingPackage.prepare_simulation(case, settings)` →
`.start()`, or `simulate(case, settings)`.

| # | Stage | Owner | Scientific work | Obligations | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|---|
| 1 | Prepare | `rt/workflow/dynamics.rs` ("only attempts own native state"), `ModelingPackage::declared_simulation` | Dynamic roles, consistent initialization | REUSE (shared typed compiler) | compile | Once | Salsa/Symbolica |
| 2 | Attempt | `run.rs` supervisor, `DurableAttempt` | None | DUR/COORD (register, heartbeat 10 s, lease 30 s) | tokio, RPC | Every 10 s | bespoke |
| 3 | Integrate | `bn/dynamics.rs::integrate_with_progress_observed` → `integrator::` (diffsol) or `idas::` | Integration with forward or adjoint sensitivities | CAP (native job); CANC (`Cancellation`); OBS (`Progress::new(256)` events → `Tap`) | THR, FFI | Grows with steps × states × sensitivity parameters | diffsol 0.16.2, SUNDIALS IDAS |
| 4 | Publish | `DurableAttempt::finish` → `store_tables` | `runtime.simulation_samples` and others | DUR (all advertised or selected relations appended **after** the run completes; chunked appends under the fence) | RPC | Grows with samples × columns | Arrow IPC + SurrealDB |

Observed: for a non-horizon run, result tables are appended in `finish` (`durable.rs:1159`),
not while the integration runs. The trajectory stays in the `RunResult` until then.
Interpretation: peak memory and interruption loss both scale with trajectory length. Only
progress events are durable mid-run.

**4b. Rolling horizon (MHE/NMPC).** Supported (`rt/workflow/horizon.rs`, `horizon/driver.rs`,
feature `solver-diffsol`).

| # | Stage | Owner | Scientific work | Obligations | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|---|
| 1 | Admission before effect | `horizon/driver.rs` | Canonical horizon assignments; estimator and controller structure prepared once | ASSUR (physical admission of inputs and priors, §19.3) | compile | Once | bespoke |
| 2 | One attempt, one session | driver + `Staged` (`rt/math/staged.rs`) | Steps strictly sequential on one worker-owned native session | REUSE (value-only overlays rebind; retained native state when coordinates and profile match, §17.6); CAP (per-step CPU permits, held job slot) | THR (one session thread for the run) | Per step: rebind, solve, apply | Ipopt (collocation), diffsol (plant) |
| 3 | Per-step publication | `DurableAttempt::begin_horizon_prefix` / `publish_horizon_step` (`durable.rs:935–1000`) | None | DUR (prefix appends per step; full or summary relations by `StudyOutputDemand`); conditional detail retention (`horizon/prefix.rs`) | RPC per step | Grows with steps × relations | Arrow IPC + SurrealDB |

**4c. Shooting** (`rt/workflow/shooting.rs`). Single and multiple shooting over
`IntegratedExperiment`. Multiple shooting integrates one window per node interval. I found no
thread or async fan-out in `shooting.rs` (same search as above). Interpretation: the windows
are integrated sequentially per NLP evaluation.

## Journey 5: recycle or conditional unit with staged initialization

Path: `ModelingPackage.prepare_recycle(case, selection, settings)` /
`prepare_block_initialization(...)` / `initialize(...)`.

| # | Stage | Owner | Scientific work | Obligations | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|---|
| 1 | Declared execution and analysis | `py/workflow/modeling.rs::prepare_recycle` → `declared_execution` | Structural analysis, incidence matching | CAP (dedicated `pse-matching` thread with `MATCHING_STACK`, `pse-structural/src/incidence.rs:306`) | THR | Once | bespoke matching / petgraph |
| 2 | Tear selection | `rt/math/flows.rs` ("Accounted pure flow preparation and native/library tear selection"); `bn/tears.rs` | Weighted tear selection as a HiGHS MILP; petgraph checks the witness | CAP (native job) | FFI | Once per structure | HiGHS, petgraph |
| 3 | Recycle iteration | `bn/recycle.rs` ("An explicit causal sweep is a KINSOL map; no recycle iteration algorithm lives here") | Fixed point / Anderson on the tear map | REUSE (`bn/implicit.rs`: KINSOL child sessions cached per worker thread and layout, KLU analysis refreshed, budget `inner_session_bytes`) | FFI | Per sweep; nested inner solves per evaluation | SUNDIALS KINSOL + KLU |
| 4 | Staged block initialization | `rt/math/initialization.rs` (stages as overlays; "a stage commits only when every block's candidate is a result") on `Staged` | Per block: solve step on one native session | REUSE (one session across blocks and stages; structure once, values rebound); ASSUR (stage commit only on full success; overlays never survive their step) | THR (session thread) | Per stage × blocks | KINSOL / Ipopt |
| 5 | Conditional blocks | `rt/math/solves/blocks.rs` ("Compiler-issued conditional equations executed in one original task") | Conditional equations | ASSUR | in-task | Per block | bespoke + library solvers |
| 6 | Publish | run supervisor / `DurableAttempt::finish` | Completion | DUR | RPC | Once | Arrow IPC |

Structure-level parallelism: I found no parallel solving of independent BTF blocks or SCCs.
Search: `par_iter`, `into_par_iter`, `rayon::scope`, `rayon::join`, `Par::rayon`,
`std::thread::scope`, `thread::spawn` and `JoinSet` across `crates/*/src`, excluding tests. The
only structure-level parallel site is `Par::rayon(scope.admitted_threads())` for the faer
factorization inside an implicit KINSOL child problem (`pse-math/src/implicit.rs:1046`).
Blocks run in one task. (Implemented as far as the search reaches. An empty search is not proof
of absence.)

## Journey 6: continuation, homotopy and path following

Supported in four forms:
- **Arclength path following.** `rt/math/solves/paths.rs` ("Prepared scalar paths. Library
  correctors own iteration") over `pse_math::continuation::ArclengthFamily`. The corrector is
  admitted only as `SolverSelection::Explicit(Backend::Ipopt)` (`paths.rs:969–980`). Optional
  event and turning-point localization comes with SVD probes (`EventPolicy`).
- **Finite supplied continuation in initialization.** `rt/math/initialization.rs`, on the
  `Staged` session.
- **Study continuation edges.** `StartPolicy::Continuation`:
  - in-process (Journey 1a): private native-state inheritance for a single successor;
  - durable (Journey 1b): `latest_seed` plus prediction.
- **Profile-chain homotopy-style step control** (Journey 3, stage 6).

| # | Stage | Owner | Obligations | Crossings | Repeats / grows with | Lib |
|---|---|---|---|---|---|---|
| 1 | Path preparation | `paths.rs::prepare_path`, `prepare_path_curvature`, `prepare_path_selection` | ASSUR (frozen original source verification of the curvature projection; connected-start requirement) | compile | Once | Symbolica (Second projection) |
| 2 | Path task | `paths.rs::path_task` inside `execution::adapter(Backend::Ipopt).scope(...)` | CAP (one native job); CANC | THR, FFI | Per path point: corrector solve; auxiliary points remain starts only | Ipopt |
| 3 | Original completion | completion owner | ASSUR (path points do not replace original completion) | in-task | Once | bespoke |

---

## (a) Product concurrency: writers and readers of shared state

### Product actors

| Actor | Process / crash domain | Shared state written | Shared state read | Mechanism |
|---|---|---|---|---|
| Python client (`Runtime`, `ModelingPackage`, `RunHandle`) | Python process; Rust in-process (tokio `EXECUTOR`) | Source revisions, physical sources; durable run, attempt, progress and result rows for in-process `RunHandle` runs; study creation and cancel; artifacts; history and retention operations | Results, manifests, progress, study status (poll) | RPC: SurrealDB WebSocket, finite SDK profile (`vendor/surrealdb/PATCH.md`: 32 app plus 2 control admissions, one session, ordered replay of setup only); two drivers `db` and `activation_db` (`ops/canonical.rs::with_access`) |
| Background tasks inside any runtime process | Same process | Attempt lease (`Heartbeat`, 10 s, `durable.rs:1962`); source-selection lease (`HeldPreparationScope`, 10 s, `engagement.rs`); progress batches (`Streamer`, 64 events / 200 ms) | n/a | `tokio::spawn` + RPC |
| Native job and session threads | Same process; a native crash takes the process | Process-global native state: PETSc `PROCESS`/`CHANGE` mutexes (`bn/petsc.rs:64`); HiGHS lifecycle mutex (`bn/highs_lifecycle.rs:18`); IBEX mutex (`bn/root_isolation.rs:45`); OpenMP and MKL thread counts (`bn/mkl.rs::Threads::enter`, scoped set and restore); thread-local compute admission (`bn/execution.rs` `COMPUTE`) | n/a | std mutex, thread-locals, `ComputeAdmission` pause/resume |
| Managed primary (`pse-worker`, `PSE_PRIMARY_NONCE`) | Separate process in a systemd unit/scope with a memory cap; launched on demand by `ensure_managed_primary` with `--observer-pid` | Claims, attempts, heartbeats, progress, result chunks, seeds, study outcomes, finalization | Study pages, scopes, sources, seeds | RPC; fenced claims |
| Managed worker slots (`surreal_server.py worker --worker-command pse-worker`) | Separate processes, one systemd scope per slot | Same as the primary | Same | RPC; supervisor slot files |
| `Runtime.work()` | Python process | Same as the primary | Same | RPC |
| Supervisor (`scripts/surreal_server.py`) | Short-lived Python subprocesses | State directory JSON: `primary-admission.json`, `worker-admission-N.json`, config `admission` / `accepting_writes`, receiver contexts | systemd and cgroup observations | `fcntl` state lock, atomic JSON publish, systemd user units |
| Host admission (`scripts/host_admission.py`) | Library used by the supervisor and by `scripts/pse-env` | Host allocation ledger | Same | File ledger. Used by the product deployment (services, receivers) **and** by agent tooling. |
| SurrealDB server | systemd unit (RocksDB; `SURREAL_RUNTIME_WORKER_THREADS=4`) | All canonical state | n/a | Server-side transactions; stored functions `fn::pse_execution_v1::fence/touch/register`, `fn::pse_history_v1::check_owner`, `fn::pse_artifact_v1::*` |

### Crash and interruption domains

- **Python process loss.** In-process durable runs lose their heartbeat, and their lease expires
  after 30 s. Recovery goes through `Operations::recover(run, operation)`
  ("recovery never invents scientific success", `durable.rs:76`) and study-point recovery
  (`recover_study_point`, `repair_study_readiness`). A study-creation interruption is covered by
  `abandon_study_inputs` under the study guard, or by `StudyCreationUnsettled`. Exports leave
  `.incomplete` files (`Runtime.export_results` doc).
- **Worker process loss.** Same lease expiry. The supervisor treats a slot as occupied until the
  scope's cgroup is proven empty (`surreal_server.py` worker docs, `surreal-substrate.md`
  "Managed native workers").
- **Store loss or disconnect.** Unknown writes "settle by original immutable identity". Socket
  closure "does not prove server abort or native drain" (§20.6, ADR-0164 text).
- **Native thread.** A thread is never abandoned. Cancellation is a flag, and capacity is
  released only after the join.

### Recovery obligations a product feature actually consumes

| Feature | Recovery obligation consumed | Requiring contract |
|---|---|---|
| Durable study survives process loss; retries per `PointPolicy.attempt_limit` | Lease expiry → reconcile the attempt; re-claim under a new fence; never blindly rerun a recorded attempt | §19.3 "Durable adapter"; §20.6; ADR-0164 (proposed) |
| Multiple workers or clients on one study | Fenced claims, generation and predecessor revision checks, definite-conflict rediscovery (`settle_current_study_candidate`) | §20.6 ("claim, start, append, heartbeat … register it before their decision"); ADR-0177 (proposed) |
| Multiple Python runtimes in one store | Caller, activity and physical lifetime separation; drain before physical close | §20.6; ADR-0181 (proposed) |
| Durable `RunHandle` single runs | Attempt registered before effect; terminal record before results reach waiters | `run.rs` module doc; §20.6 |
| Artifact creation interrupted | `recover_artifact_intent` ("never replay creation") | ADR-0182 (proposed) |
| Seed reuse across studies | Sealed eligibility witness, protected acquisition | ADR-0173 (proposed) |
| Quiesce, stop and backup | Close write admission, drain workers, check every registered receiver/observer/worker drained | `docs/dev/surreal-substrate.md` "Quiesce, stop and backup" |

Interpretation: the ephemeral in-process sweep (Journey 1a) consumes none of the run, attempt
or claim recovery obligations. It still holds source-selection leases (`SOURCE_LEASE`) through
the store.

### Harness and agent-tooling concurrency (kept separate)

- `.config/agent-capacity.toml` profiles (`functional`, `wide`, `timing`, `exclusive`,
  `exclusive-observer`) declare `case_lanes`, `cpu_threads` and `math_jobs` for test and agent
  slots. `scripts/pse-env` places commands in `pse.slice` scopes. The "sixteen case lanes" check
  in `pse-worker::verify_placement` uses the supervised execution profile, which these
  declarations feed.
- `#[cfg(feature = "canonical-tests")]` hooks in `pse-worker` (`qualification_native_entry`),
  `FinishExportGate`, and test-only `PreparationEffects` counters.
- `scripts/native_tests.py` and `scripts/validation_scope.py` manage `OMP_NUM_THREADS` and
  `MKL_NUM_THREADS` for test runs.

## (b) Parallelism and capacity

### Where parallelism is admitted

| Level | Site | Mechanism | Charged by |
|---|---|---|---|
| Case | Local study lanes (`study_inner`) | `FuturesUnordered`, width `min(cores, math.jobs)` | `jobs`, `active_jobs`, `cpu` semaphores + pool (per native job) |
| Case | Worker lanes (`serve_owned_lanes`) | width `maximum_in_flight` (≤ `min(cores, math.jobs)`) | same, inside the worker process budget; cgroup caps |
| Case | Worker processes | Supervisor slots (`native_workers` count) | systemd scope `memory.max`, CPU quota; host ledger |
| Case | Concurrent `RunHandle`s | tokio-spawned supervisors | math semaphores; `admission_wait` |
| Case (bulk) | POUNCE convex batch | `Staged::batch` (local study only) | one native job |
| Structure | Fit profile chains | `std::thread::scope` team | `quiescent_phase(threads × workers, bytes)` |
| Structure | Implicit KINSOL child factorization | faer `Par::rayon(admitted_threads)` | admitted thread count; **runs on rayon's global pool** (no `install` around it; `pse-math/src/implicit.rs:1049`) |
| Structure | BTF blocks, SCCs, multiple-shooting windows, fit experiments | none found | n/a |
| Library | POUNCE / POUNCE convex | Scoped local rayon pool (`bn/pounce.rs::with_threads`, `build_scoped`) | `scope_threads` stacks reserved (`execution::scope_stack_bytes`) |
| Library | FERAL (POUNCE linear solver) | Own global pool sized from `RAYON_NUM_THREADS` or `available_parallelism`, which cannot be injected; parallel only when admitted ≥ pool (`bn/pounce.rs::feral_threads`) | not separately charged; register R-41 |
| Library | Ipopt linear solvers | `threads > 1` only for SPRAL or Pardiso-MKL (`bn/ipopt/settings.rs:159`); MKL requires `MKL_DYNAMIC=FALSE` | `controls.threads` within the job's `cores` |
| Library | MKL / OpenMP | `mkl::Threads::enter(n)`: scoped `omp_set_num_threads`, `MKL_Set_Num_Threads_Local`; Uno and PETSc force 1 | job `cores` |
| Library | HiGHS | `threads` option, `parallel` on or off (`bn/highs.rs:623`) | job `cores` |
| Library | SCIP | `parallel/maxnthreads`; `lp/threads=1` (`bn/scip.rs:1121–1136`) | job `cores` |
| Library | Clarabel | `max_threads` 1 for QDLDL; Pardiso for more (`bn/conic.rs:439–531`) | job `cores` |
| Library | Surrogate fitting | Local rayon pool (`rt/math/surrogate/library.rs:761`) | options.threads |
| Compiler | Salsa compiler lanes; Symbolica optimization | shares the `cpu` semaphore (`compiler_cpu`); reference `optimization.cores = 1` | `cpu` permits |
| I/O | tokio runtime | `worker_threads = pool_threads`; default blocking pool | n/a (not charged) |

Capacity charging summary:
- **Memory.** One DataFusion `FairSpillPool` per process (`MemoryConsumer` reservations), the
  per-job `WorkerBudget` carved from the job reservation (`jobs.rs:404–540`), and per-process
  cgroup caps for managed workers.
- **CPU.** One tokio `Semaphore` per process (`cpu`), plus `jobs` (population, including idle
  sessions) and `active_jobs` (executing).
- **Exclusion.** Native-exclusion waits release CPU (`ComputeAdmission::pause` / `try_resume`).

Where nested pools meet (Implemented sites; Interpretation in parentheses):
1. A `pse-math` job thread enters adapter scopes in sequence (`execution::enter`). POUNCE creates
   a local rayon pool inside the job thread while FERAL runs its own global pool beside it
   (R-41).
2. The faer `Par::rayon(n)` used inside an implicit child dispatches to rayon's global pool
   (default size is the host's parallelism). (This pool is shared by every job in the process
   and sits outside the `cpu` semaphore's thread accounting; the admitted count only sets the
   split.)
3. OpenMP/MKL threads are set per thread, scoped. Process-wide OpenMP placement is set at
   `pse-worker` startup (`OMP_PROC_BIND`, `OMP_PLACES=sockets`).
4. Across processes, the Python process, the managed primary, worker slots and the SurrealDB
   server (4 tokio workers, RocksDB background threads) are bounded by separate cgroups or
   ledgers, not by one runtime semaphore.

## (c) Cross-point reuse

| Product | Persists across points or iterations | Rebuilt per point | Why (from code or docs) |
|---|---|---|---|
| Compiled programs (Symbolica `CompiledBody`) | `MathService.entries` (DataFusion `DefaultCache`, content-hash key, byte-weighted); `clear_program_cache` | Only on a miss | "callers cannot supply an independent cache key" (`rt/math/artifacts.rs`) |
| Salsa compiler workspace | `compiler_lanes`: one idle generation per lineage and context (`rt/math/modeling.rs:1258–1290`) | New lane per lineage/context change | — |
| Structural admission (`DeclaredExecution`) | `StudyPreparationLane.current.admission` while `(descriptor, paths, member set)` and source match | On key change | "Point values are never retained" (`study_operations.rs:381`) |
| Prepared view and basis (`PreparationEngagement`) | `HeldPreparationScope` per lane | When the lane closes | "never a cache entry" (`engagement.rs:21`) |
| Value-dependent products | Shared when consumed values are unchanged (`PreparationCounts.shared`) | Rebuilt when a consumed value changes (`rebuilt`) | §19.3, DP-09/DP-10 |
| Native session (Ipopt problem, KINSOL/KLU, HiGHS model) | Within one `Staged` sequence (horizon steps, initialization blocks, a local continuation chain with a single successor) | **Every independent study point** (new `Staged::open_queued`); every durable attempt | Session retention policy §17.6; fan-out gets independent sessions |
| KINSOL inner sessions | Per worker thread and layout (`bn/implicit.rs`, bounded by `inner_session_bytes`) | When the thread ends, which is per job | Plan 22 Y6 / I14 |
| HiGHS model | "retained native model updated in place for compatible layouts" (`bn/execution/highs.rs`) | On layout change | — |
| Starts | Local: in-memory `MinimalContinuation`. Durable: stored seeds (`latest_seed`, `artifact_seed`) plus prediction | Seed decode per durable point | ADR-0173 eligibility |
| Fit evaluators | Steady `CaseWorker`s per experiment, kept for the whole fit | Transient: dynamics worker and integrator per oracle call | `integrated.rs:211` |
| Modeling package in workers | `StudyCandidateLane.source` per lane while `(revision, physical receipt)` holds | Per lane on first use or source change | per-process `MathService` |
| Engine cache service | Python process: per `EngineSettings` | `pse-worker`: disabled (`CacheBudget::disabled`) | `pse-worker::resource_budget` |

## Material uncertainties and unfinished edges

- I did not verify whether `MathService.selected_flights` / `modeling_cache` dedupe package
  reconstruction across the 16 worker lanes (Interface-checked only).
- I did not trace `ResultAppendQueue` chunk sizing or the progress `Streamer` RPC cadence beyond
  the policy constants.
- How `Staged::batch` eligibility interacts with seeds was read only through
  `batch_backend` and `study_inner` grouping.
- `IntegratedExperiment::integrate` builds a new worker per call. I did not check whether
  diffsol or IDAS internally caches symbolic factorizations across those calls; that would be a
  library-side question for R2 or R4.
- The prior evidence `multi-case-workflows-and-capacity.md` (2026-10-11, D-01 to D-09) predates
  Plan 37. Several sites it names have changed, such as lane-held preparation and demand-selected
  detail. I took no position on those leads.
