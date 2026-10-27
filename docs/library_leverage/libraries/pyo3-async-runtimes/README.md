# pyo3-async-runtimes

Realizations considered: installed `pyo3-async-runtimes` =0.29.0 with `tokio-runtime`.
Research date 2026-10-27. Sources: pyo3-async-runtimes 0.29.0 source (`src/generic.rs`
`future_into_py*`, `Cancellable`, `PyDoneCallback`; `src/lib.rs` module docs).

## Types

### pyo3-async-runtimes/asyncio-cancellation-bridge
- **Need it serves:** let Python's event loop own interruption: a Rust future exposed as an asyncio
  awaitable is cancelled when the Python future is cancelled (e.g. `asyncio.run` cancelling the main
  task on KeyboardInterrupt).
- **Recognizable code shapes:** `future_into_py_with_locals(py, locals, async move {...})` plus a
  guard type whose `Drop` cancels native work.
- **Mechanisms and APIs:** `tokio::future_into_py`, `future_into_py_with_locals`; on Python-side
  cancellation, a done-callback sends on a oneshot (`cancel_tx`) and the `Cancellable` wrapper stops
  polling the Rust future (documented: "If the asyncio.Future returned by this conversion is
  cancelled via asyncio.Future.cancel, the Rust future will be cancelled as well").
- **Obligations discharged:** propagation of asyncio cancellation into Rust; the event loop's own
  signal handling (the loop installs a wakeup fd; in 3.11+ `asyncio.Runner` turns SIGINT into task
  cancellation).
- **Residual caller obligations:** cancellation is by **dropping** the Rust future — cooperative
  native cancel must be triggered from `Drop` and a drain cannot be awaited there; raising with the
  diagnostic as `__cause__` is not provided.
- **Semantic conditions:** requires an event loop on the calling thread; a synchronous API built on
  `asyncio.run` would nest-fail inside a running loop.
- **Limits and negative results:** no synchronous signal-woken wait helper (search scope: `src/*.rs`
  for `signal`: only module-level docs about main-thread signal ownership).
- **Evidence:** Interface-checked: 0.29.0 source.

## Current applicability

### [CBO-017](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-017) — Signal-aware blocking waits at the Python boundary
- **Types:** pyo3-async-runtimes/asyncio-cancellation-bridge.
- **Realization:** installed 0.29.0.
- **Conclusion:** ruled out here for the synchronous calls — routing sync waits through asyncio
  would trade polling for drop-based cancellation that cannot express "cancel, then drain, then raise
  with cause", and would break callers already inside an event loop; it remains the right route for
  the existing `wait_async` path.
- **Evidence:** Interface-checked. Application Proposed (negative).
