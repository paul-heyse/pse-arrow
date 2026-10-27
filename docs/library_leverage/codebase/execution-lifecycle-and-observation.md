# Execution lifecycle, concurrency and observation

This page covers `pse-runtime` sessions, native jobs, durable workers and studies, cancellation,
observation and telemetry, and the `pse-py` blocking boundary. Contracts are owned by
[§18](../../authoritative_design/sections/numerical-execution.md#section-18),
[§21](../../authoritative_design/sections/workflows-and-results.md#section-21) and
[§23](../../authoritative_design/sections/operations-and-validation.md#section-23). The baseline is on
the [entry page](README.md).

**System picture.** The runtime owns a process-wide `SharedRuntime`. That includes a DataFusion
`FairSpillPool`, a shared resource budget, job and CPU semaphores, and a math service that runs
native jobs on dedicated OS threads. Those threads are joined, together with thread-local foreign
destructors, before ownership is released.

Library use is idiomatic and good:
- `tokio_util::CancellationToken` with child tokens and drop guards, plus `CancelSource`, which
  adds a minimum deadline.
- `tokio::sync::Semaphore` and `Notify`, `FuturesUnordered`, `select!`, and `tokio::process`
  with `kill_on_drop`.
- `tokio_metrics` task monitors.
- The OpenTelemetry SDK `BatchSpanProcessor`.
- `rustix` pidfd calls.
- `thiserror` and `miette`.

The following are legitimately custom:
- the paired CPU/memory permit protocol, under which no native owner waits for memory;
- joining before release;
- claim fences and leases on the canonical store;
- the closed host-operation catalogue;
- the same-host, no-daemon diagnostics model.

<a id="cbo-013"></a>

## CBO-013 — One lifecycle event fanned out to several hand-fed observation sinks

- **Operation and consumers.** Every native lifecycle boundary is reported. The boundaries are
  admission wait, dispatch, cancel, deadline and join. Consumers are operators, the Python host,
  review and capture tooling, and evidence documents.
- **Current mechanism.** Around each boundary, `math/jobs.rs` calls five sinks:
  - `pse_diagnostics::observation::phase`, the `ObservationHub` (about 5k lines in
    `pse-diagnostics/src/observation*`);
  - `tracing` spans and events (`tooling/telemetry.rs`, about 2.3k lines);
  - `crate::invocation::stage` for the per-invocation recorder, with hand caps on intervals and
    name lengths;
  - static atomic arrays in `tooling/operational_metrics.rs`;
  - the `tooling/capture/layer.rs` recording layer.

  About 81 call sites keep `stage`, `phase` and the operational metrics in step by hand. Metric
  meaning is declared in `tooling/metric_declarations.rs`. Each instrument is then installed
  through repeated observable builders that re-read a snapshot.
- **Why it matters.**
  - A new or renamed boundary needs edits in several forms.
  - The hub and the recorder partly duplicate the tracing span tree.
- **Patterns.** LOP-09 (primary), LOP-02, LOP-06.
- **Footprint.**
  - Inspected: `jobs.rs`, `invocation.rs` and `telemetry.rs`.
  - Suspected: about 18k lines under `tooling/` and about 6k in `pse-diagnostics`.
- **Preservation constraints.**
  - Recording stays passive and bounded, with no scientific effect.
  - No daemon; checkpoints stay on the same host.
  - "Unavailable" stays distinguishable from zero.
  - Evidence documents keep citing exact intervals.
  - Counts start before any exporter exists, which is the reason for the static atomics.
- **Counterevidence.**
  - The hub deliberately works without a tracing subscriber.
  - Spans can be sampled or filtered, so they cannot be the only record.
  - The right unification may be one lifecycle event type with fan-out, not one library.
- **Status.** Observation is a documented subsystem (current promise); consolidating it is a
  proposed addition.
- **Research questions.**
  - Can one `tracing_subscriber::Layer` (the repo already writes one) feed the hub, the recorder
    and the metrics from a single span or event?
  - Could OpenTelemetry synchronous instruments, or the `metrics` facade, replace atomics read by
    observable callbacks?
  - Could `MetricDeclaration` drive instrument installation directly?
- **Library assessment correction (2026-10-27).** `pse-diagnostics` has no `tracing` dependency, so the hub working without a subscriber is a structural property, not an incidental one. See [tracing](../libraries/tracing/README.md) and [opentelemetry](../libraries/opentelemetry/README.md).

<a id="cbo-014"></a>

## CBO-014 — Execution context re-established by hand at each thread or task boundary

- **Operation and consumers.** Native workers, staged solves, shooting window teams and the
  Python pull thread all need the runtime handle, invocation recorder, observation context,
  tracing dispatcher and current span re-entered on the new thread.
- **Current mechanism.**
  - `math/jobs.rs` (the native job spawn) captures `Handle::current()`,
    `observation::current()`, `invocation::TransferredContext::capture()`,
    `dispatcher::get_default` and `Span::current()`. Inside the spawned closure it enters five
    guards plus `native_execution_scope()`.
  - The same dance repeats in `math/staged.rs`, `workflow/shooting/window_team.rs` (three
    places), `workflow/run.rs` and `pse-py/src/workflow.rs`.
  - `invocation.rs` keeps a thread-local and a task-local with fallback logic.
  - `PopulationScope::capture` handles a separate `task_local!`.
  - `telemetry::inherit` and `observation::inherit` add further wrappers.
- **Why it matters.**
  - Every new spawn site must remember every guard, in order.
  - A missed guard silently loses attribution or population scope on that path.
- **Patterns.** [LOP-X002](../assessment-patterns.md#lop-x002) (primary), LOP-02, LOP-09.
- **Footprint.** Six or more spawn sites were inspected; the mixed task-local and thread-local
  use extends across `workflow/`.
- **Preservation constraints.**
  - The recorder must be the exact `Arc`.
  - Nested invocations still start through `start_scope`.
  - Native threads keep their explicit stack size and name.
  - Join must witness foreign destructors.
- **Counterevidence.**
  - `TransferredContext` is already half of a single context value.
  - The remedy may be one local `spawn_native` helper rather than a library.
- **Status.** Proposed addition.
- **Research questions.**
  - Can span extensions carry the recorder and observation handles, so that one `Span::enter`
    restores everything?
  - Does any executor or tracing facility propagate this set to foreign OS threads?
- **Library assessment correction (2026-10-27).** The native job spawn passes through five more wrappers before `tokio::spawn`, including `inherit_activity`. There are more than 20 `task_local!` carriers. `staged.rs` passes context inside each request message to a long-lived thread, so a spawn helper alone would not cover it. Production installs a global default dispatcher, so the dispatcher carrier matters only for scoped setups. Symbolica's thread-local `CURRENT_ABORT` is a further carrier. See [tracing](../libraries/tracing/README.md).

<a id="cbo-015"></a>

## CBO-015 — Cancellation, deadline and lease renewal composed separately at each site

- **Operation and consumers.** Cooperative cancellation has to reach async waiters, CPU loops and
  C callbacks. Separately, durable claim leases are renewed while work runs. Consumers are
  native jobs, durable runs and study admission.
- **Current mechanism.**
  - **Two cancellation types, bridged by hand.** `tokio_util::CancellationToken` and
    `pse_diagnostics::Cancellation` (an `Arc<AtomicBool>` plus an `event_listener::Event`) are
    bridged in `cancel.rs` (`CancelSource::join_native`).
  - **Deadlines raced per wait.** An absolute deadline travels beside the token and is raced in
    each wait: `sleep_until` arms in several `select!` sites in `math/jobs.rs`.
  - **Lease renewal written twice.**
    - A `Heartbeat` task (interval, oneshot stop, failure `Mutex`) in `workflow/durable.rs`.
    - An inline `select!` with `sleep(renewal)` in `workflow/study.rs`
      (`claim_study_admission`).
- **Why it matters.** Each encoding of "cancel on deadline or lease loss" is another place where
  a wake can be missed or a task leaked.
- **Patterns.** LOP-08 (primary), LOP-09.
- **Footprint.** Cancellation types appear in about 290 files. Two renewal implementations were
  read.
- **Preservation constraints.**
  - The FFI flag stays readable from C callbacks without a runtime.
  - Cancellation never releases work, so a ready stop wins over a ready candidate.
  - The deadline is the minimum over enclosing clocks.
- **Counterevidence.**
  - The FFI flag is a hard requirement and the bridge is small.
  - The clearer gaps are the duplicated renewal and the per-site deadline races.
- **Status.** Current promise.
- **Research questions.**
  - Could one `Lease` guard, built from a `TaskTracker` or `JoinHandle` with a `DropGuard` and
    linked to a token, serve both renewal sites?
  - Could `timeout_at` or `run_until_cancelled` composed into a deadline-carrying token fold the
    repeated races?
- **Library assessment correction (2026-10-27).** The native join wait waits on the native `Cancellation` type, signals a stop, and then keeps waiting for the thread. The admission waits instead drop the losing side, so the two need different treatment ([LOP-X005](../assessment-patterns.md#lop-x005)). Study renewal pauses the guarded work while the renewal call runs. See [tokio](../libraries/tokio/README.md).

<a id="cbo-016"></a>

## CBO-016 — Waiting by polling for state owned elsewhere

- **Operation and consumers.** Several waits observe state owned by someone else:
  - workers discover claimable attempts;
  - study waiters observe inventory;
  - the runtime observes the primary receiver's phase;
  - callers wait for asynchronous protection or reader drain in the canonical store.
- **Current mechanism.**
  - Worker discovery polls every 500 ms, with a 30 s recovery pass (`workflow/worker.rs`).
  - `wait_study_inventory` (`study_execution.rs`) loops on a 100 ms sleep. At every observation
    interval it spawns the Python supervisor to run `observe-primary`.
  - `StudyWaitControls.poll_seconds` defaults to 0.5 s, and `study_observation.rs` polls on a
    timer.
  - In `pse-operations`, staging, artifacts and file locks sleep-poll.
  - Tests retry retire or release 32–64 times because a returned buffer releases its protection
    asynchronously.
  - No live query or changefeed is used anywhere.
- **Why it matters.**
  - Latency and idle cost grow with the number of waiters.
  - Spawning an interpreter to read state is heavy.
  - An in-process drop notification, or an OS or store notification, could replace some timers.
- **Patterns.** LOP-08, LOP-12, LOP-04.
- **Footprint.** About ten production sites and many test loops.
- **Preservation constraints.**
  - Canonical claims remain the distributed authority.
  - Observation grants no release authority.
  - No daemon.
  - Correctness never depends on notification delivery: a missed notification falls back to
    polling.
  - Bounded requests.
- **Counterevidence.**
  - Polling with leases is robust.
  - A live query needs a long-lived connection, which conflicts with the bounded-request
    profile.
  - Some sites intentionally return "not yet".
- **Status.** Proposed addition. The current behaviour is a documented polling contract.
- **Research questions.**
  - Does the bounded gRPC profile support `LIVE SELECT` or changefeeds?
  - Would `tokio::sync::Notify` on the in-process reader-drop path suffice?
  - Can `cgroup.events`, `pidfd` or systemd D-Bus state replace the `observe-primary`
    subprocess?
- **Library assessment correction (2026-10-27).** `ResultReadDrain` already uses `TaskTracker`, and a `WakeSignal` notification type already exists. Production `forget_run_results` refuses rather than waits, so most drain polling loops are in tests. LIVE SELECT works over the Rust SDK's gRPC `Subscribe` stream; the open question is the bounded-request profile. See [surrealdb](../libraries/surrealdb/README.md) and [tokio](../libraries/tokio/README.md).

<a id="cbo-017"></a>

## CBO-017 — Signal-aware blocking waits at the Python boundary written per call

- **Operation and consumers.** A synchronous Python call must stay interruptible (Ctrl-C) while
  native work runs detached from the GIL. Consumers are workflow pulls, capture joins and waits,
  and inspection streams.
- **Current mechanism.**
  - `pse-py/src/workflow.rs` (`blocking_native_call` and its neighbours) spawns a scoped worker
    thread and loops `py.detach(recv_timeout(100 ms))`, then `check_signals`, then cancel, then
    drain.
  - `pse-py/src/diagnostics.rs` polls every 20 ms with `thread::sleep`.
  - `inspection/stream.rs` has its own loop.
  - There are at least eight `check_signals` sites, with different intervals and orderings.
  - A separate async path (`wait_async` with `CancelWait`) exists alongside these.
  - Each loop also re-captures context ([CBO-014](#cbo-014)).
- **Why it matters.** Several bespoke wait–signal–cancel–join loops lead to inconsistent latency
  and ordering for one user-visible behaviour.
- **Patterns.** LOP-09, LOP-08.
- **Footprint.** Eight or more sites. Shared helpers already exist for part of this.
- **Preservation constraints.**
  - The GIL is released while waiting.
  - On a signal: native cancel, then drain, then raise with the diagnostic as the cause.
  - Panics after the drain are reported.
  - No Python attach from worker callbacks.
- **Counterevidence.** `check_signals` has no wakeable form, so some polling loop is
  unavoidable. The gain is consolidation and possibly signal-driven wakeup.
- **Status.** Current promise.
- **Research questions.**
  - Is there a PyO3 or `pyo3-async-runtimes` pattern for signal-woken waits, such as a self-pipe
    via `signal.set_wakeup_fd` or `signal-hook`?
  - Can every loop use one helper?
- **Library assessment correction (2026-10-27).** There are 10 `check_signals` sites with 100 ms and 20 ms intervals and differing orderings. See [cpython-signal-wakeup-fd](../libraries/cpython-signal-wakeup-fd/README.md).

<a id="cbo-018"></a>

## CBO-018 — Two executor threads per native job to wait for a join

- **Operation and consumers.** Run a native solve on a large-stack thread, wait for its join,
  cancel it, and enforce its deadline. Consumers are all native jobs.
- **Current mechanism.** `math/jobs.rs` spawns a named `std::thread` with a configured stack, then
  uses `tokio::task::spawn_blocking` only to call `handle.join()`, then waits in a `select!` on
  cancel, deadline and join. The same shape appears in `staged.rs` and `window_team.rs`.
  Native-thread accounting is hand-counted.
- **Why it matters.** A blocking-pool slot is occupied per job just to wait for a join. The join
  owner and the accounting are coded separately at each site.
- **Patterns.** LOP-08, LOP-01.
- **Footprint.** One central site and two similar sites.
- **Preservation constraints.**
  - Completion is signalled only after the thread exits, because thread-local destructors must
    finish, and a guard dropped inside the closure is not exit.
  - Large stack.
  - A panic maps to a typed error.
- **Counterevidence.**
  - This is a deliberate design.
  - The cost is probably negligible against solve time.
  - This is a weak opportunity.
- **Status.** Current promise.
- **Research questions.**
  - Is there a supported way to await OS-thread exit, for example via pidfd on the thread ID or
    via a dedicated runtime with `thread_stack_size`?

<a id="cbo-019"></a>

## CBO-019 — Linux process, cgroup and pressure facts parsed by hand in Rust and Python

- **Operation and consumers.**
  - Placement admission verifies that a worker's cgroup memory and CPU limits admit its profile.
  - Peak RSS and process identity feed diagnostic checkpoints.
  - Tooling reads process identity and pressure stall information (PSI).
- **Current mechanism.**
  - Rust:
    - `pse-runtime/src/bin/pse-worker.rs` parses `/proc/self/cgroup`, `memory.max`, `cpu.max`
      and `cgroup.controllers` while walking ancestor cgroups.
    - `peak.rs` parses `VmHWM`.
    - `observation_target.rs` parses `/proc/<pid>/stat` and `exe`.
    - `pse-math/src/native_compiled.rs` parses `/proc` stat fields ([CBO-004](cross-cutting-mechanisms.md#cbo-004)).
  - Python: `scripts/process_observation.py` and `pressure_observation.py`.
- **Why it matters.**
  - The format knowledge is held twice, in two languages.
  - Each parser needs its own unit and overflow tests.
- **Patterns.** LOP-01, LOP-09.
- **Footprint.** About six small parsers.
- **Preservation constraints.**
  - Fail closed.
  - Exact equality of the cap to the profile.
  - The walk continues to the root.
  - Start-tick identity against PID reuse.
  - Inherited and absent `cpu.max` remain distinct.
- **Counterevidence.**
  - Each parser is small and fit for purpose.
  - Library strictness is uncertain.
  - Low priority.
- **Status.** Current promise.
- **Research questions.**
  - Do `procfs` / `cgroups-rs` (Rust) and `psutil` (Python) expose the same strict semantics,
    including PSI?
- **Library assessment correction (2026-10-27).** The footprint is larger than estimated: 8 Rust readers of `/proc/*/stat` and 4 of `/proc/self/status`. The canonical store's owner identity (boot ID, start ticks, pid namespace and cgroup) is a missed manifestation. The host runs systemd 255 and kernel 7.0. See [LOP-X008](../assessment-patterns.md#lop-x008) and [procfs](../libraries/procfs/README.md).

<a id="cbo-020"></a>

## CBO-020 — Primary-receiver supervision split between the Rust runtime and a Python script

- **Operation and consumers.** The runtime and the `pse-worker` binary start, observe and stop
  the SurrealDB primary and the native worker units. This covers cgroup caps, a nonce, the scope
  inode and readiness receipts.
- **Current mechanism.**
  - `workflow/worker.rs` builds `tokio::process::Command`s for `surreal_server.py ensure-primary`
    and `observe-primary`, bounds their output, and exchanges JSON checked by a `PrimaryOwner`
    identity.
  - `workflow/native_deployment.rs` pins the script and worker by SHA-256.
  - `scripts/surreal_server.py` (about 11.6k lines) drives `systemd-run` and `systemctl --user`.
  - Lifecycle decisions are made and re-verified on both sides: hash, JSON identity and cgroup
    readback.
- **Why it matters.**
  - The start, readiness, owner-change, drain and release lifecycle is held in two languages.
    That doubles the independent decisions per lifecycle step.
  - Every observation pays for an interpreter start.
- **Patterns.** LOP-09, LOP-08.
- **Footprint.** About 800 Rust lines. The script's internals are recorded under
  [CBO-048](developer-tooling.md#cbo-048).
- **Preservation constraints.**
  - systemd user units are the placement authority.
  - Ownership fails closed.
  - Nothing is released without a drain.
  - Host-local over portable.
- **Counterevidence.**
  - Deliberate deployment ownership.
  - A local operator script is acceptable for a personal project.
- **Status.** Current promise.
- **Research questions.**
  - Could a systemd D-Bus client (`zbus` with `StartTransientUnit` and unit properties) let the
    runtime observe or own units directly?
  - Which script operations are generic supervision and which are policy?
