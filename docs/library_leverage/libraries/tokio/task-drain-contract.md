# Task tracking, admission and drain

Reusable type: `tokio-util/task-tracking-drain-contract`.
Research date 2026-11-02. **Interface-checked** against tokio-util 0.7.19 source.
No new runtime probe was run.

`TaskTracker` (the `rt` feature) tracks task lifetimes. `wait()` completes when the tracker is
both closed and empty. `close()` allows waiters eventually to complete; it **does not prevent
new tasks from being spawned**. The application needs its own admission rule if shutdown must
exclude new work. Calling `close()` on an empty tracker while another party may still submit
work is therefore not proof that the system has settled.

`CancellationToken::cancel()` requests cooperative cancellation. A worker must observe it and
settle its effects. The token is not an error/result collector, and cancellation does not wait
for task completion. A useful composition is: stop admission under the owner's synchronization,
request cancellation, close tracking, then await tracked work. The exact effect boundary and
failure aggregation remain caller policy.

A tracker does not retain task outputs. Keep `JoinHandle`s or another result route when errors
must be collected; `JoinSet` may be the better complete option for that need. Dropping a handle
does not itself establish settlement. Blocking/native work requires its own cooperation and
retention guarantees; canceling an async delivery future does not prove its underlying CPU or
external I/O has stopped.

`CancellationToken::run_until_cancelled` is an abandonment combinator: cancellation drops the
wrapped future. Its later polling favors future completion over cancellation. It cannot be
silently substituted for an operation requiring ready-stop precedence or signal-and-drain.

Sources: [TaskTracker API](https://docs.rs/tokio-util/0.7.19/tokio_util/task/struct.TaskTracker.html),
[exact implementation](https://docs.rs/tokio-util/0.7.19/src/tokio_util/task/task_tracker.rs.html),
[CancellationToken API](https://docs.rs/tokio-util/0.7.19/tokio_util/sync/struct.CancellationToken.html),
[graceful shutdown guide](https://tokio.rs/tokio/topics/shutdown).
