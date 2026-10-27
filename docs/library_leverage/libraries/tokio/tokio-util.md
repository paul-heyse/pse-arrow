# tokio-util

Realizations considered: installed `tokio-util` 0.7.19 (workspace features: `rt` only;
`CancellationToken`, `DropGuard`, `DropGuardRef` are unconditional; `TaskTracker`,
`AbortOnDrop(Handle)`, `JoinQueue` need `rt`); upgrade route 0.7.20 (released 2026-10-09; fix
"notify `CancellationToken` descendants after releasing locks" (#8556), MSRV 1.85; no new
cancellation or tracking API).
Research date 2026-10-27. Sources: installed source `tokio-util-0.7.19/src/sync/cancellation_token.rs`
(`run_until_cancelled`, `RunUntilCancelledFuture::poll`), `sync/cancellation_token/guard.rs`,
`task/mod.rs`, `task/task_tracker.rs`, `task/abort_on_drop.rs`, `Cargo.toml` features;
tokio-util `CHANGELOG.md` (master, fetched 2026-10-27).

## Types

### tokio-util/cancellation-tree
- **Need it serves:** one cooperative stop request reaching every async waiter in an operation
  tree, with children cancellable without affecting parents or siblings.
- **Recognizable code shapes:** `Arc<AtomicBool>` stop flags plus a wake mechanism; hand-written
  parent/child stop propagation; "cancel on drop" guard structs.
- **Mechanisms and APIs:** `CancellationToken::{new, child_token, cancel, is_cancelled,
  cancelled, cancelled_owned, drop_guard, drop_guard_ref, run_until_cancelled,
  run_until_cancelled_owned}`; `DropGuard::{token, disarm}` (`token` added 0.7.19).
- **Obligations discharged:** tree propagation, waker registration before the state check
  (cancellation before the first poll is observed), idempotent cancel, cancel-on-drop.
- **Residual caller obligations:** deadlines (none in the type), stop/timeout distinction,
  synchronous or foreign readers (a C callback cannot await; it needs a plain atomic it can read
  without a runtime), bridging to other stop types.
- **Semantic conditions:** `run_until_cancelled` returns `None` immediately if already cancelled,
  but on later polls it polls the future *first* and the cancellation second — "biased towards the
  future completion" (source and doc comment). It drops the future on cancellation (abandon
  semantics). No public accessor exposes an `AtomicBool` for foreign code.
- **Version / build conditions:** no feature needed; 0.7.20 fixes a descendant-notification
  locking issue.
- **Limits and negative results:** no deadline-carrying token; no "signal and keep awaiting"
  combinator; no FFI-readable flag.
- **Evidence:** Interface-checked: `cancellation_token.rs` lines for `run_until_cancelled` and
  `RunUntilCancelledFuture::poll` at 0.7.19.

### tokio-util/task-supervision
- **Need it serves:** own background tasks (renewal, release, producers) so they can be stopped,
  awaited at shutdown, or aborted when their owner disappears.
- **Recognizable code shapes:** `struct X { stop: oneshot::Sender, task: JoinHandle, failure:
  Arc<Mutex<Option<E>>> }`; live counters plus "closing" flags for drains.
- **Mechanisms and APIs:** `TaskTracker::{spawn, spawn_blocking, track_future, token, close,
  reopen, wait, len, is_empty}` (wait resolves only when closed *and* empty);
  `AbortOnDropHandle<T>` (abort on drop, `detach`); `CancellationToken` + `DropGuard` for
  cooperative stop; `JoinHandle<Result<_, E>>` to return a failure instead of storing it.
- **Obligations discharged:** drain waiting, abort-on-drop, cooperative stop wiring.
- **Residual caller obligations:** failure policy, what stop means for an in-flight effect
  (abort cancels an RPC midway; a token stop waits for the next check).
- **Semantic conditions:** `TaskTracker::wait` cannot be used for "wait until the currently
  tracked tasks finish" while the tracker stays open; abort is not cooperative.
- **Version / build conditions:** `rt` feature (declared).
- **Limits and negative results:** no lease/heartbeat helper; no keyed task registry beyond
  `JoinMap` (`join-map` feature, not declared).
- **Evidence:** Interface-checked: `task/task_tracker.rs`, `task/abort_on_drop.rs` at 0.7.19.

## Current applicability

### [CBO-015](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-015) — Two cancellation types bridged by hand
- **Types:** tokio-util/cancellation-tree.
- **Realization:** installed 0.7.19; 0.7.20 adds only the notification fix.
- **Conclusion:** ruled out here as a replacement of the native stop type — the FFI flag must be
  readable from C callbacks without a runtime, and `CancellationToken` exposes no atomic. The
  `CancelSource::join_native` bridge (≈15 lines) is the residual and is already small.
- **What remains local:** the bridge, the native `Cancellation` (atomic plus event-listener
  `Event`), minimum-deadline composition (`CancelSource::with_deadline`).
- **Semantic gaps:** `run_until_cancelled` would invert the "ready stop wins" rule and drop the
  work; it is unsuitable for signal-and-drain sites. Note: the native job path in `jobs.rs`
  awaits `pse_diagnostics::Cancellation::cancelled()` (event-listener), not the tokio-util token.
- **Evidence:** Interface-checked (`cancel.rs`, `pse-diagnostics/src/cancellation.rs`,
  tokio-util source); Proposed.

### [CBO-015](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-015) — Deadline folded into a token
- **Types:** tokio-util/cancellation-tree.
- **Conclusion:** ruled out — a timer task that cancels a token at the deadline merges two outcomes
  the code keeps distinct (`Cancelled` vs `admission_timeout`, and `source_owns_deadline`).
- **Evidence:** Interface-checked; Proposed.

### [CBO-015](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-015) — One `Lease` guard for both renewal sites
- **Types:** tokio-util/task-supervision.
- **Conclusion:** fits with conditions — a guard holding a child `CancellationToken` (stop),
  a `JoinHandle<Result<(), E>>` (failure return) and a link to the guarded work's token (cancel on
  loss) covers both `Heartbeat` and the inline study renewal; `AbortOnDropHandle` is available if
  abandoning an in-flight renew RPC is acceptable.
- **What remains local:** cadence policy, renewal RPC, error attribution.
- **Semantic gaps:** current `Heartbeat::stop` waits for an in-flight renewal to finish; abort
  would not. Study renewal currently pauses work (see tokio.md).
- **Evidence:** Interface-checked; Proposed.

### [CBO-016](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-016) (in-process side)
- **Types:** tokio-util/task-supervision.
- **Conclusion:** partially already adopted — `ResultReadDrain` uses `TaskTracker` tokens for
  release tasks. `TaskTracker::wait` cannot signal "release settled" while the store stays open, so
  the in-process waiter needs a separate signal (tokio.md, tokio/change-notification).
- **Evidence:** Interface-checked (`canonical_results.rs`).

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — Flight producers
- **Types:** tokio-util/task-supervision.
- **Conclusion:** fits as a peripheral aid only — `TaskTracker::spawn` could own detached flight
  producers for shutdown join; it does not supply single-flight semantics.
- **Evidence:** Interface-checked; Proposed.
