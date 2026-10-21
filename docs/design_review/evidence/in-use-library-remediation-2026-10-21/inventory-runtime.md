# Capability-gap inventory: runtime infrastructure and fitting (revised with the census)

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Capability-gap inventory: tokio/tokio-util, tracing, rayon and fitting libraries.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

Baseline: HEAD `0530dcaa1`, which is the source commit `a791c533e` plus the audit pages. Evidence came from Context7 (`/websites/rs_tokio-util`, `/websites/rs_tracing-subscriber`), the pinned sources under `~/.cargo/registry/src/…` and `~/.cargo/git/checkouts/levenberg-marquardt-*/e67066c`, and repository source. I took usage from census §6 (`…/scratchpad/review2/census.md:156-186`) and re-checked the parts that matter with `cargo tree --offline --locked -e features` and `rg`. Nothing was built or run, and I wrote no files.

**Bottom line.**
- **Fix now, as a one-liner (DA-012):** `driver.rs:142` should take a child of the caller's token instead of a fresh one. Today caller cancellation never reaches a running horizon point.
- **Biggest structural prize (DA-012):** one run supervisor built on the tokio-util cancellation tree. It removes the five lifecycle copies.
- **Second prize, needs a cost probe first (DA-020):** make `tracing` the real observation substrate and move the hub's aggregation into a `Layer`.
- **Small clear wins:** join the surrogate's rayon workers before its charge is released, and pin the global rayon pool so library parallelism cannot escape the CPU budget.

**Census corrections, verified.**
- **`env-filter` is enabled.** `tracing-subscriber` has `env-filter`, `fmt` and `registry` on through workspace unification (it comes in via a dev-dependency edge), so `EnvFilter` is available.
- **statrs' default features are on anyway.** `egobox-moe` turns on statrs' `default`, `nalgebra` and `rand`. So `nalgebra 0.33.3` resolves beside `0.35.0` (`Cargo.lock:4881`, `:4898`; `cargo tree -i nalgebra@0.33` → `statrs` ← `egobox-moe`). The pin comment at `Cargo.toml:287-288` ("default features off: its nalgebra 0.33 and rand would split the pinned graph") no longer matches reality. This is a fact for the coordinator; no type-universe invariant covers nalgebra.
- **tokio gains `fs`, `net`, `signal` and `io-std` through unification**, and `JoinSet` is used in `pse-operations` (for example `canonical_staging.rs:2393`, `canonical.rs:3629`).

## 1. Capability surface at the pins

**tokio-util 0.7.19 (`rt`)**
- `sync` is not feature-gated (`src/lib.rs:57`). `task` comes with `rt` (`:46-51`).
- `CancellationToken` (`src/sync/cancellation_token.rs`):
  - `child_token` (:204): parent cancellation reaches children; child cancellation never reaches the parent or siblings.
  - `cancel` (:220) traverses synchronously; once it returns, every descendant is cancelled.
  - **`is_cancelled` (:225) takes a mutex** (`tree_node.rs:82-84`). There is no atomic fast path.
  - `cancelled_owned` (:267), `drop_guard` / `drop_guard_ref` (:275, :283), `run_until_cancelled[_owned]` (:300, :330).
  - There is no cancel-callback hook. Propagating into a flat flag needs a future that waits on the token.
- `TaskTracker` (`src/task/task_tracker.rs:158`):
  - `spawn`, `spawn_blocking` (:452), `track_future` (:532), `token` (:549), `close` (:337), `wait` (:318).
  - `wait` returns only when the tracker is closed and empty, after the futures' destructors have run.
  - It does not refuse tokens after `close`, but it tracks them.
- Off: `join-map` and `time`.

**tokio 1.53.2**
- `JoinSet`, fair FIFO `Semaphore::acquire_many_owned`, `Notify`, `watch`, `task_local!`.
- Off, per the census: `taskdump` and `test-util`.

**rayon-core 1.13.0**
- `build_scoped` (`lib.rs:322`, built on `std::thread::scope`, so workers join before it returns).
- `spawn_handler` (:433, caller-owned thread handles), `start_handler` / `exit_handler` (:638, :657), `thread_name`.
- `use_current_thread` (:547) leaks the registry for local pools.
- **`ThreadPool::drop` only calls `registry.terminate()`** (`thread_pool/mod.rs:399-402`). It does not join.
- `build_global` is never called anywhere in the workspace.

**tracing 0.1.44 / tracing-subscriber 0.3.23** (`env-filter`, `fmt` and `registry` enabled)
- `Layer` callbacks: `on_new_span`, `on_record`, `on_event`, `on_enter`, `on_exit`, `on_close`.
- Per-span storage through `LookupSpan` / `SpanRef::extensions_mut`.
- Per-layer filtering through `Layer::with_filter` → `Filtered`, with `filter::Targets`, `FilterFn` and `EnvFilter`.
- Composition: `Option<L>` and `Vec<L>` (`layer/mod.rs:1776`).
- Propagation: `Span::current`, `Instrument::instrument`, `WithSubscriber::with_subscriber`, `dispatcher::set_default` / `with_default`.

**tracing-perfetto-sdk-schema 0.13.1** provides the prost types only. **datafusion-tracing 55.0.0** provides the `instrument_with_info_spans!` and `instrument_rules_with_info_spans!` macros and `InstrumentationOptions`. **DataFusion** offers the `JoinSetTracer` / `set_join_set_tracer` hook.

**event-listener 5.4.2** offers blocking `wait` / `wait_deadline` with no runtime.

**blake3 1.8.7** (`std`)
- `keyed_hash` (`lib.rs:949`), `derive_key` (:1003), `Hasher::new_keyed` (:1097).
- `rayon` (`update_rayon`, :1497) and `mmap` are off.

**quick_cache 0.6.24**
- `Weighter`, `sync::Cache` / `unsync::Cache`.
- `Lifecycle::{is_pinned, before_evict, on_evict(_hot/_cold)}` (`lib.rs:170-215`).
- Placeholder guards give single-flight loading. `stats` is off.

**levenberg-marquardt e67066c**
- `with_ftol` (:154), `with_xtol` (:168), `with_gtol` (:194, cosine orthogonality, scale-invariant).
- `with_tol` (:207) sets `ftol = xtol = tol` and **`gtol = 0`**, and asserts `tol > 0`.
- Also `with_stepbound` (:227), `with_patience` (:241, asserts > 0), `with_scale_diag` (:248). `TerminationReason` is typed.
- No bounds support. `differentiate_numerically` exists and must stay unused.

**argmin 0.11** (`full` off): Gauss–Newton, BFGS/L-BFGS, trust region, Nelder–Mead, Brent. None is bound-constrained least squares.

**statrs 0.18**: `Normal`, `ChiSquared`, `StudentsT`, `FisherSnedecor`, with `ContinuousCDF::inverse_cdf`.

**allocation-counter 0.8.1** (dev dependency): `measure(..) -> AllocationInfo { bytes_max, bytes_current, … }` (`lib.rs:84-165`).

## 2. What the repository uses, verified

**There are two cancellation systems, by design.**
- **Hierarchical:** `pse_columnar::CancellationToken` wraps tokio-util, including `child_token` (`crates/pse-columnar/src/resource.rs:89-150`). `CancelSource` adds a deadline. Its child primitive `child_with_deadline` is tested (`crates/pse-runtime/src/cancel.rs:48-52`, `:86-133`).
- **Flat:** `pse_diagnostics::Cancellation` is `Arc<AtomicBool>` plus an `event_listener::Event` with blocking `wait_until` (`crates/pse-diagnostics/src/cancellation.rs:24-104`).
  - The native layer takes `Arc<AtomicBool>` (`crates/pse-backend-native/src/solve.rs:1199`, `implicit.rs:230`, `scip.rs:512`).
  - The flat leaf is justified, because tokio-util's `is_cancelled` takes a mutex.

**How caller cancellation is linked today.**
- Simulation, shooting, fit and modeling sequences each create `Cancellation::default()` and pass `move || forwarding.cancel()` into the shared `supervised_with_cancel`. That function bridges by `select!` on `parent.cancelled()` → `stop(); work.await` (`crates/pse-runtime/src/workflow/run.rs:843-870`; copies at `:969-970`, `:1221`, `:1462`, `:1824`).
- **Horizon:**
  - It checks the caller only during admission.
  - Then `let cancel = crate::CancelSource::new();` (`workflow/horizon/driver.rs:142`) shadows the caller's token.
  - `RunHandle::staged` holds only that new source (`run.rs:154-175`).
  - Confirmed: caller cancellation after start never reaches a horizon run (DA-012).
- **The public lease** is a hand-written drop guard that also records `run.stop_requested` (`run.rs:45-67`).
- **The lane server** creates `let drain = crate::CancelSource::new()` (`workflow/worker.rs:377`) and links it to `stop` only through the event-loop `select!`.

**Activity drain is bespoke counters.**
- `RuntimeActivity`: a mutex counter, closing stores when it reaches zero (`workflow/mod.rs:726-800`).
- Canonical `ConnectionActivity` / `PhysicalLifetime`: a counter, an idle `Notify`, phases and close rounds (`crates/pse-operations/src/canonical_lifetime.rs:25-140`).
- Task-local continuations re-propagated by hand on every spawn (`mod.rs:881-890`; `math/jobs.rs:1381-1384`).
- `TaskTracker` appears only once, in the result-release drain (`crates/pse-operations/src/canonical_results.rs:36`, `:192`).

**Native drain.**
- In `math/jobs.rs:1381-1490`, admission is a try-all-or-nothing loop over two `ReleaseNotifyingSemaphore`s (thin tokio `Semaphore` wrappers, `crates/pse-engine/src/resources.rs:20-80`) plus a DataFusion reservation (`jobs.rs:295-370`).
- The named `pse-math` thread is joined via `spawn_blocking(handle.join())`, and `drop(compute)` happens only after the join (`:1457-1490`).

**Observation is two parallel systems.**
- The hub: `crates/pse-diagnostics/src/observation.rs` (3,881 lines), `producer.rs` (1,216), `inspection/performance.rs` (1,237), `thread_cpu.rs`.
- The hub's own propagation:
  - a `thread_local! CURRENT` (`observation.rs:2081`);
  - re-entering `scope` / `inherit` futures (`:2094-2124`);
  - manual `enter` in rayon wrappers (`crates/pse-backend-native/src/pounce.rs:203-222`) and in each `par_iter` member (`execution/pounce_convex.rs:530-545`).
- About 1,350 observation mentions against about 60 `tracing::` mentions. `pse-runtime` emits essentially no spans or events, and production installs no subscriber.
- **Both systems are propagated side by side** at every boundary:
  - native job threads capture `observation::current()` *and* `dispatcher::get_default` plus `Span::current()` (`jobs.rs:1422-1440`);
  - DataFusion's `JoinSetTracer` does `future.instrument(span).with_subscriber(dispatch)` *wrapped in* `observation::inherit` (`crates/pse-engine/src/session/assurance.rs:46-84`).
- `datafusion-tracing` emits operator spans (`assurance.rs:99-122`) that are outside the hub's tree.
- Capture is a `tracing_subscriber::Layer` that correlates spans by reading `observation::current()` (`crates/pse-runtime/src/capture/layer.rs:55-80`). `capture/timeline.rs` writes Perfetto through the schema crate and prost.
- The public reader is `RunHandle::observations() -> OperationReader` (`run.rs:82`; `crates/pse-py/src/diagnostics.rs:361`).

**rayon.**
- Scoped, joining per-call pools for POUNCE and FERAL (`pounce.rs:213-224`).
- A **non-scoped** pool per retained surrogate (`math/surrogate/library.rs:830-835`). Its `Drop` waits for a "state destroyed" reply and then drops the pool unjoined (`:807-815`).
- `pse-math` sizes parallel work only when already on a rayon worker (`crates/pse-math/src/implicit.rs:1918-1928`).
- Global-pool-capable dependents: faer, gemm, numerica, symbolica, rustworkx-core, ndarray and egobox (`cargo tree -i rayon`).

**Fitting.**
- LM call: `.with_tol(p.accuracy.stationarity).with_scale_diag(..).with_patience(update_limit)` (`workflow/fitting/oracle/residual.rs:356-360`).
- Eligibility is a field-by-field conjunction (`:9-58`).
- Stationarity is re-checked independently (`:380`; `results.rs:471`).
- The Gauss–Newton covariance is over `W = R·S/σ`, which assumes known σ (`covariance.rs:10-12`, `:311-331`).
- Wald intervals use `Normal` (`:355`) and profile thresholds use `ChiSquared(1)` (`profile.rs:906-911`). Both are correct for known σ, so the statrs use is sound.

**blake3** is `derive_key` frames in `pse-ids` only, under governance ownership (`crates/pse-ids/src/derive.rs:8`). **quick_cache** is `unsync::Cache` behind a `Mutex` with `shards(1)`, `hot_allocation(1.0)` and `ghost_allocation(0.0)` (`math/retention/weighted.rs:55-70`).

## 3. Unused and under-used capabilities

### C1. One run supervisor on the tokio-util cancellation tree (DA-012) — Proposed; APIs Interface-checked

**Mechanism.**
1. A generic `RunSupervisor` holds the skeleton: mint, attempt, register, dispatch, admission wait, join, publish. Each operation kind supplies its native closure and finish.
2. The run token is `parent.child_with_deadline(deadline)`, or `CancelSource::new()` with no parent.
3. The lease holds `run_token.drop_guard()`, wrapped so that `run.stop_requested` is still recorded.
4. Exactly one bridge inside the supervisor future projects the tree onto the flat native leaf: `select!{ work, run_token.cancelled() => { flag.cancel(); work.await } }`.
5. Horizon becomes one kind of the same supervisor.

**Immediate minimal correction:** `driver.rs:142` → `let cancel = cancel.child_with_deadline(None);`. It must be a child, not the caller's token: the lease cancels `checks` on drop, and that must not cancel the caller.

**(i) What it owns.**
- DA-012's divergence: one linking point for every kind, including horizon, and horizon replay through the same entry.
- The copies at `run.rs:949-1171`, `:1209-1388`, `:1444-1628`, `:1803-1945` and `horizon/driver.rs:87-271`.
- A lifecycle change is then made once.
- caller → study → lane → run becomes a structural tree.
- `worker.rs:377`'s `drain` becomes `stop.child_with_deadline(None)`, keeping the event-loop arm.

**(ii) Removed.** Four forwarding closures, two lease constructors, the shadow source and the manual `Lease::drop`.

**(iii) Limits.**
- The flat leaf stays, because of the mutex and the native callbacks. "Single token" means one *authority* with one derived leaf per job.
- If the supervisor future is never polled after a `DropGuard` fires during runtime shutdown, the native job runs to completion. That is a correct drain, just slower, and the same as today.

**(iv) Reverse check.**
- The drain is preserved: the bridge cancels, then awaits; nothing aborts.
- One process budget is untouched.
- Lane parallelism is unchanged: lanes keep in-task `FuturesUnordered` draining (`study_execution.rs:802-830`). Do not use `JoinSet` or `AbortOnDropHandle` for lanes, because abort-on-drop breaks "no return drops running lanes".
- For durable horizon lanes, fences and "cancellation wins" still decide the seal; linking only adds a prompt native stop.

### C2. `tracing` as the observation substrate, with hub aggregation as a `Layer` (DA-020) — Proposed; feasibility partly unresolved

**Mechanism.**
- **Emission:** spans (`info_span!(target: "pse::obs", …)`) for operations and phases. Facts become events with static callsite names (`event!(target: "pse::obs", name: "native.stop_requested", value = true)`), plus `unit` and `value` fields.
- **Propagation:** only the tracing route the code already uses at `assurance.rs:52-68` and `jobs.rs:1427-1440`: `Instrument`, `WithSubscriber`, `dispatcher::with_default` and `span.in_scope`.
- **Aggregation:** the hub's value-adding part (bounded budget, catalogue, windows and buckets, coherent records, producers) becomes `ObservationLayer: Layer<Registry>`. It keeps per-span state in `extensions_mut()` and assigns its own stable `OperationId`, because Registry span IDs are recycled. `OperationReader` reads a shared `Arc`.
- **Per-runtime routing:** a hub reference rides on the root span and is inherited in `on_new_span`.
- **Filtering:** `with_filter(filter::Targets…)`, or `EnvFilter`, which is now known to be available. Disabled policy means no layer, so callsites cache `Interest::never`.
- **Capture and Perfetto** become separate-crate layers that the host composes, with no `observation::current()` correlation.
- **datafusion-tracing operator spans** join the operation tree.

**(i) What it owns.**
- DA-020's foundation-crate telemetry runtime: `pse-diagnostics` keeps vocabulary only.
- Toolkit placement outside the production root becomes a composition choice, because `Layer` is the seam.
- It removes the parallel hub propagation at every boundary, including the doubled `JoinSetTracer` wrapping.

**(ii) Removed.** `CURRENT`, `scope` / `inherit`, the `EnterGuard`s, and capture-to-hub correlation. Aggregation code moves but stays.

**(iii) Conditions and limits.**
1. A library must not set the global default. The runtime scopes its own `Dispatch` (Registry + hub layer + host-supplied `Vec<Box<dyn Layer<_>>>`). A host's global subscriber then sees pse spans only through that API. This is an explicit API decision.
2. Span creation allocates a slab entry. Per-iteration producers must be events folded into the parent's extension. This is unmeasured.
3. Layer callbacks must be panic-free, because native callbacks unwind behind `catch_unwind`.
4. Dynamic metric names must become field values or static `name:` literals.

**(iv) Reverse check.**
- Observation still cannot change outcomes, and readers stay passive.
- No cancellation, budget or SurrealDB effect.
- The risk of a missing `enter` on some path is the same as the hub's today.
- **Remedy open** pending P2. The fallback is placement only (a hub crate outside foundation and root) that ingests tracing events.

### C3. `TaskTracker` for the activity drain — Proposed; partial fit, low value alone

It can replace the counter plus `Notify` in `RuntimeActivityOwner` and `PhysicalLifetime`: `close(); spawn(wait().then(close_stores))`, with the closed check and `token()` kept under the existing mutex so refusal stays atomic. It can also track the detached native-lane joiners (`math/staged.rs:1013-1020`) and job supervisors (`jobs.rs:1381`) through `spawn` / `spawn_blocking`.

Limits: the canonical phases, close rounds and validation and observation fields stay. It is neutral on every strength. Do it only as part of C1.

### C4. Rayon pool lifetime and the global pool — Proposed; clear wins

**(a) Join the surrogate pool's workers** using `spawn_handler` with retained `std::thread` `JoinHandle`s that are joined after `drop(pool)`, or `build_scoped` if the actor shape allows. This puts the surrogate under the drain rule ("compute released only after join").

**(b) Pin the global pool at deployment construction:** `ThreadPoolBuilder::new().num_threads(1).thread_name(..).build_global()`, recording a typed "foreign pool already initialised" fact if it fails. `native_child.rs:481` already does the equivalent for the child process.

Both strengthen the drain and the single budget. Whether any production path actually reaches the global pool is unresolved (P3).

### C5. LM tolerance mapping (fitting; next to DA-016) — Proposed; needs a probe

`with_tol(stationarity)` turns off the library's stationarity-like test (`gtol = 0`, `lm.rs:207-216`), so relative-reduction and relative-step tests stand in for a stationarity request. The test at `lm_inquiry.rs:989` notes a scaled-stationarity mismatch.

Use `with_gtol` for the stationarity request, and set `with_ftol` / `with_xtol` explicitly. The independent original-space check stays the qualification authority, so nothing regresses.

DA-016's LM predicate defect has no library remedy in this family; it is a destructuring fix. argmin offers no bounded least squares, so bounded fits rightly stay on the NLP backends.

### C6. Budget and charging (DA-022) — Proposed; partial

- **quick_cache `Lifecycle::on_evict` / `before_evict`.** An entry could own its reservation and release it at eviction, making capacity and charge one fact instead of the `entry_weight` estimate (`weighted.rs:36-41`).
- **allocation-counter `measure`** gives the measuring-allocator drift test DA-022 says is missing, for the project's own Rust structures. It only sees the Rust allocator, not SUNDIALS or FERAL `malloc`.
- One budget is preserved.

### C7. Covariance provenance (DA-010) — blake3 is the weaker route

A `keyed_hash` stamp, under a per-deployment key over a `pse_ids` frame, would let the runtime tell its own unaltered fit covariance from an external one without a lookup. It guards against accidents only, and governance confines blake3 to `pse-ids`.

The better remedy is outside this family: resolve `run_id` against the durable or in-process record, and label the result external or uncertified otherwise. statrs needs no change.

### No in-family library remedy

- **DA-013:** a policy move into `pse-model::study_policy`.
- **DA-021:** use explicit inputs, for example `foreign_allowance` as a value to LM eligibility (`residual.rs:37-57`). Never use ambient tracing or task-local context as a service locator.

## 4. Clear wins no DA names

1. The surrogate pool join (C4a).
2. The global rayon pin (C4b).
3. `worker.rs:377` `drain` as a child token.
4. LM `gtol` (C5), pending P4.
5. Coordinator note: statrs default features are unified on through `egobox-moe`, which resolves `nalgebra 0.33.3`. The comment at `Cargo.toml:287-288` is stale, and no in-family fix exists short of an egobox feature change.

Rejected:
- `use_current_thread`: leaks the registry.
- blake3 `rayon` / `mmap`: use the uncounted global pool.
- `JoinSet`, `JoinMap` or `AbortOnDropHandle` for lanes: abort-on-drop breaks the drain.
- `DelayQueue`: no need.

## 5. Unresolved questions and probes

- **P1 (makes DA-012 Tested).** Add a test next to `horizon_tests.rs:2405`: `start_with_cancel(&caller)`, cancel `caller` after admission, then assert a cancelled `RunResult` and that pool `reserved()` returns to its baseline only after the native join. Expected: it fails today and passes after the `:142` fix.
- **P2 (decides C2).** A criterion bench on a steady solve and a fit comparing three arms:
  1. the hub;
  2. tracing events, a Registry and a prototype aggregation layer;
  3. no layer.

  Measure per-callback wall time and allocations (allocation-counter). Adopt C2 if (2) is within noise of (1).
- **P3.** A diagnostic `build_global` with a counting `start_handler` across the qualification journeys, or `/proc/<pid>/task/*/comm` sampling. Any start means uncounted parallelism.
- **P4.** `lm_inquiry.rs` cases with `with_gtol` added: compare `TerminationReason`, evaluations and scaled stationarity.
- **P5.** Admission fairness: the try-all loop (`jobs.rs:316-368`) bypasses `Semaphore`'s FIFO order. Run a full-width job against a continuous stream of 16 one-core jobs and record its wait. If it is unbounded, queuing on `acquire_many_owned` while holding nothing else is an option, but it risks holding CPU while memory waits: **remedy open**.
- **P6.** Confirm whether allocation-counter counts per thread or globally (`lib.rs:165`).
- **P7.** Split the hub's roughly 6,300 lines into propagation (which C2 deletes) and aggregation (which it keeps). Unresolved.

Evidence labels:
- *Interface-checked*: library APIs at the pins.
- *Implemented*: repository facts and the census items I re-verified.
- *Proposed*: every recommendation.

Nothing is *Tested* or *Measured*.