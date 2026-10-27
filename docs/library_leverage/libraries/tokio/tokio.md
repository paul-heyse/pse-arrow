# Tokio

Realizations considered: installed `tokio` 1.53.2 (workspace declares `rt-multi-thread`, `sync`,
`macros`, `time`, `process`, `io-util`; the unified feature graph also carries `net`, `fs`,
`signal`, `io-std` through other dependents); upgrade route 1.54.1 (released 2026-10-21; no
change found to the capabilities below, adds `SemaphorePermit::semaphore()`, `Semaphore::blocking_acquire*`,
`watch::Receiver::is_closed`).
Research date 2026-10-27. Sources: installed source `tokio-1.53.2/src/time/timeout.rs`,
`src/io/mod.rs` (`cfg_net_unix!` gate of `AsyncFd`), `src/macros/cfg.rs`, `src/process/unix/pidfd_reaper.rs`;
tokio `CHANGELOG.md` (master, fetched 2026-10-27, entries 1.54.0 and 1.54.1).

## Types

### tokio/ordered-stop-race
- **Need it serves:** wait for one of several readiness sources (work, cancellation, deadline,
  capacity release, closure) with a defined winner when several are ready together, and a typed
  outcome per winner.
- **Recognizable code shapes:** `tokio::select! { biased; () = cancel => Err(Cancelled), () =
  sleep_until(at) => Err(Timeout), v = work => v }`; `timeout_at(deadline, fut)` wrappers;
  `Option<Instant>` deadlines turned into `pending()` when absent.
- **Mechanisms and APIs:** `tokio::select!` with `biased;` polls branches in source order, so the
  first listed ready branch wins; branch preconditions (`, if cond`) disable branches.
  `tokio::time::timeout_at(Instant, F)` / `timeout(Duration, F)` return `Result<F::Output, Elapsed>`.
  `sleep_until` for absolute deadlines.
- **Obligations discharged:** wake registration for all sources, timer management, cancellation
  safety of the dropped branches (each branch future is dropped when another wins).
- **Residual caller obligations:** ordering policy, mapping each winner to a domain outcome,
  minimum-of-enclosing-deadlines computation, and what happens to the losing work.
- **Semantic conditions:** `Timeout::poll` polls the inner future *first* and the delay second
  (`timeout.rs`, installed 1.53.2): a ready value wins over an elapsed deadline. A policy where a
  ready stop must win over ready work therefore cannot be expressed by `timeout_at` alone; it needs
  `select! { biased; … }` with the stop first. Every combinator here *drops* the losing future: it
  implements abandon-on-race, not "signal a stop and keep owning the work until it exits".
- **Version / build conditions:** `macros` and `time` features. Unchanged in 1.54.x per changelog.
- **Limits and negative results:** no built-in "deadline-carrying cancellation" type; no combinator
  that signals and then continues to await the same work (that is caller code: race, signal, then
  `.await` the retained future).
- **Evidence:** Interface-checked: `tokio-1.53.2/src/time/timeout.rs` (`Future for Timeout`
  polls `value` then `poll_delay`); select! `biased` documented semantics.

### tokio/change-notification
- **Need it serves:** wake in-process waiters when state owned elsewhere changes (a drain reaches
  zero, a release completes), instead of sleep-polling.
- **Recognizable code shapes:** `loop { if check() { break } sleep(10ms) }`; bounded retry loops in
  tests waiting for asynchronous release; `AtomicUsize` live counts with no wake.
- **Mechanisms and APIs:** `tokio::sync::watch` (`Sender::send_modify`, `send_if_modified`,
  `Receiver::wait_for(|v| pred)` which checks the current value before waiting, so no lost wake);
  `tokio::sync::Notify` (`notified()` + `Notified::enable()` to register before checking the
  predicate; `notify_waiters` wakes only registered waiters, `notify_one` stores one permit).
- **Obligations discharged:** waiter registration, wake fan-out, lost-wakeup avoidance (`watch`
  by value; `Notify` only when the register-then-check discipline is followed).
- **Residual caller obligations:** publishing the change at the true owner, choosing the
  predicate, keeping the authoritative check (a notification is only a hint when the state also
  lives in a durable store), fallback timing for cross-process changes.
- **Semantic conditions:** in-process only; `watch` retains only the latest value; `Notify` has no
  value. Neither crosses process boundaries.
- **Version / build conditions:** `sync` feature; 1.54.0 changes `Notify` to acquire/release
  orderings (changelog #8325).
- **Limits and negative results:** cannot replace waits on state owned by another process or a
  database; there the poll remains (or a store/OS notification route is needed).
- **Evidence:** Interface-checked: tokio sync docs; changelog 1.54.0.

### tokio/fd-readiness-await
- **Need it serves:** await readiness of an arbitrary Linux file descriptor (for example a pidfd)
  on the runtime's reactor, without a blocking thread.
- **Recognizable code shapes:** `spawn_blocking(|| blocking_wait())` used only to wait on an
  OS object; poll loops on `/proc` or `is_finished()`.
- **Mechanisms and APIs:** `tokio::io::unix::AsyncFd<OwnedFd>`, `readable().await` /
  `AsyncFdReadyGuard::clear_ready`. Tokio itself uses a pidfd for child-process reaping
  (`process/unix/pidfd_reaper.rs`).
- **Obligations discharged:** reactor registration and wakeups for edge readiness.
- **Residual caller obligations:** creating and owning the fd, meaning of readiness, nonblocking
  mode, fallback when the kernel lacks the facility.
- **Semantic conditions:** readiness only; no data is read from a pidfd.
- **Version / build conditions:** `AsyncFd` requires tokio feature `net` on unix
  (`cfg_net_unix!`) and a runtime with the IO driver enabled.
- **Limits and negative results:** tokio offers no awaitable join for `std::thread` threads; the
  supported bridge is `spawn_blocking(move || handle.join())`.
- **Evidence:** Interface-checked: `tokio-1.53.2/src/io/mod.rs`, `src/macros/cfg.rs`,
  `src/process/unix/pidfd_reaper.rs`.

### tokio/task-local-context
- **Need it serves:** carry per-operation context (deadline, recorder, activity owner) through
  async calls without parameters.
- **Recognizable code shapes:** `tokio::task_local! { static X: T }`, `X.scope(v, fut)`,
  `X.sync_scope(v, f)`, `X.try_with(...)` with thread-local fallbacks.
- **Mechanisms and APIs:** `LocalKey::scope`, `sync_scope`, `try_with`, `get`.
- **Obligations discharged:** install-on-poll and restore semantics for one future tree.
- **Residual caller obligations:** re-scoping at every `tokio::spawn` and every `std::thread`
  boundary; values are not inherited by spawned tasks or threads.
- **Semantic conditions:** the scope applies while the scoped future is polled; futures created
  *inside* that poll (for example by a retry combinator's factory closure) see the value.
- **Limits and negative results:** no propagation into spawned tasks, blocking-pool closures or OS
  threads; tokio's (unstable) task hooks expose spawn/poll events but no user-data inheritance.
- **Evidence:** Interface-checked: tokio task-local docs; changelog 1.54.0 "Unstable" section.

## Current applicability

### [CBO-015](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-015) — Cancellation, deadline and lease renewal composed separately at each site
- **Types:** tokio/ordered-stop-race; tokio-util/cancellation-tree (see tokio-util.md).
- **Realization:** installed 1.53.2.
- **Conclusion:** fits with conditions — the admission waits (`math/jobs.rs` retry waits and
  compiler admission) are abandon-on-race waits that a local helper over `select! { biased; … }`
  can fold; `timeout_at` alone is ruled out where "a ready stop wins" is the contract.
- **What it would discharge / what remains local:** timer and wake plumbing are already library;
  what repeats locally is the ordering (cancel > closed > deadline > wake) and the outcome
  mapping. A local `wait_admission(cancel, deadline, fut)` helper would own that once.
- **Prerequisites, configuration or extension:** none beyond current features.
- **Semantic gaps against the CBO's preservation constraints:** the native join site
  (`admitted_storage`) is *signal-and-drain*: on cancel or deadline it signals and then keeps
  awaiting the join. No tokio combinator expresses that; dropping the join future would violate
  "cancellation never releases work".
- **Unsettled premises and what would settle them:** whether every admission site needs stop-first
  ordering (read each site's contract).
- **Evidence:** Interface-checked (tokio source, repo `jobs.rs` select sites at baseline); application Proposed.

### [CBO-015](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-015) — Lease renewal (two implementations)
- **Types:** tokio `time::interval` + `MissedTickBehavior::Skip`; tokio-util/task-supervision.
- **Realization:** installed.
- **Conclusion:** fits with conditions — a renewal task owned by a guard (interval, stop token,
  `JoinHandle<Result<…>>` instead of a failure `Mutex`) can serve both sites, but the two sites
  differ in cadence (`policy.heartbeat` vs `lease/3`) and concurrency: the study site renews
  *inline*, so its `work` future is not polled while `renew_attempt` awaits.
- **What it would discharge / what remains local:** library: timer, task, stop wake. Local: lease
  policy, failure classification, cancelling the guarded work on loss.
- **Semantic gaps:** moving study renewal onto a concurrent task changes behaviour (work progresses
  during renewal). Whether the pause is intentional is unsettled.
- **Evidence:** Interface-checked (`durable.rs` `Heartbeat`, `study.rs` `admit_study_candidate`); Proposed.

### [CBO-016](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-016) (in-process side) — Polling for release or drain owned in-process
- **Types:** tokio/change-notification.
- **Realization:** installed.
- **Conclusion:** fits with conditions — an in-process "release settled" signal (a `watch` live
  count with `wait_for(|n| *n == 0)`, or the repository's existing event-listener `WakeSignal`)
  can replace the 32–64-iteration retry loops that wait for an asynchronously spawned protection
  release.
- **What it would discharge / what remains local:** the wait and wake; the authoritative check
  (`forget_run_results` / `lifecycle_release` against the store) remains and must still run after
  the wake.
- **Prerequisites:** the release task (spawned by `ProtectionOwner::drop` and tracked by the
  drain's `TaskTracker`) must publish completion to the signal.
- **Semantic gaps:** none for in-process waiters if the notification stays a hint ("correctness
  never depends on notification delivery").
- **Unsettled premises:** the observed retry loops are in tests; production `forget_run_results`
  refuses rather than waits. Whether a production waiter needs this is not established.
- **Evidence:** Interface-checked (`canonical.rs` `ProtectionOwner`, `canonical_results.rs`
  `ResultReadDrain`, test loops in `canonical_result_retention.rs`, `canonical_lifecycle.rs`); Proposed.

**Reviewer correction (2026-10-27).** The unsettled premise is answered: every production wait in scope is cross-process, and in-process wake types already exist where producer and waiter share a process. The comparative review retains the current waits, with one optional fix: race `wait_study_inventory`'s sleep with cancellation. See the [comparative review](../../reviews/execution-lifecycle-and-observation.md#cbo-016).

### [CBO-018](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-018) — Two executor threads per native job to wait for a join
- **Types:** tokio/fd-readiness-await; linux-pidfd/thread-exit-readiness (see linux-pidfd.md).
- **Realization:** installed 1.53.2; `AsyncFd` needs `net`, present in the unified feature graph
  but not declared by the workspace entry (declaring it is a manifest change, no version move).
- **Conclusion:** fits with conditions — awaiting a thread pidfd through `AsyncFd` removes the
  blocking-pool slot; ruled out on kernels before 6.9.
- **What remains local:** opening the pidfd inside the thread, ordering, fallback to
  `spawn_blocking(join)`, panic-to-typed-error mapping, the final (non-blocking) `join()`.
- **Semantic gaps:** must still witness all thread-exit destructors; see linux-pidfd.md.
- **Evidence:** Interface-checked; Proposed. Probe requested (README).

**Reviewer correction (2026-10-27).** Deferred rather than adopted: the review keeps
`spawn_blocking(join)` inside one native-thread owner and swaps to `AsyncFd` readiness only if
blocking-pool pressure is measured. The workspace tokio pin enables `rt-multi-thread`, `sync`,
`macros`, `time`, `process` and `io-util` but not `net`, so adoption would declare `net`, which a
process-pidfd or `cgroup.events` wait would share. See the
[comparative review](../../reviews/execution-lifecycle-and-observation.md#cbo-018).

### [CBO-014](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-014) — Execution context re-established by hand
- **Types:** tokio/task-local-context (negative).
- **Conclusion:** ruled out here as a propagation mechanism — task-locals do not cross
  `tokio::spawn` or OS threads, which is why the capture/enter blocks exist.
- **Evidence:** Interface-checked.

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — I/O gate and load admission
- **Types:** `tokio::sync::Semaphore` (already used); object-store/request-limit (see object-store.md).
- **Conclusion:** current use is idiomatic; no further tokio capability found that adds the
  nested-scope re-entrancy (`ACTIVE_IO` lease chain) — that remains local.
- **Evidence:** Interface-checked (`stores/admission.rs`).
