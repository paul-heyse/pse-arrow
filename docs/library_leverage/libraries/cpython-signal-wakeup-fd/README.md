# CPython signal wakeup fd (native capability)

Realizations considered: CPython standard library `signal.set_wakeup_fd` (3.7+ for
`warn_on_full_buffer`); applies to whatever interpreter the extension runs in.
Research date 2026-10-27. Sources: docs.python.org/3/library/signal.html (fetched 2026-10-27).

## Types

### cpython/signal-wakeup-fd
- **Need it serves:** wake a thread blocked outside the interpreter promptly when a signal arrives,
  instead of polling `PyErr_CheckSignals` on an interval.
- **Recognizable code shapes:** detached waits with short timeouts followed by signal checks;
  event loops multiplexing I/O with interruption.
- **Mechanisms and APIs:** `signal.set_wakeup_fd(fd, *, warn_on_full_buffer=True)`: on receipt of a
  signal **for which a Python handler is registered** (SIGINT has `default_int_handler` by default),
  the low-level C handler writes the signal number as one byte into `fd`, then sets the flag that
  `PyErr_CheckSignals` later services. Returns the previous fd (−1 if none); `fd` must be
  non-blocking; the reader must drain bytes. A native waiter polls `{completion fd, wakeup fd}`
  (e.g. eventfd/pipe + `poll`), and on wakeup reattaches and calls `check_signals`.
- **Obligations discharged:** prompt, event-driven detection of signal arrival.
- **Residual caller obligations:**
  - Only the **main thread of the main interpreter** may call it (else `ValueError`), so the setup
    happens while attached on the main thread.
  - It is a single process-global slot: asyncio (and other libraries) install their own; the waiter
    must save and restore the previous fd and must not steal wakeups another owner relies on
    (forwarding bytes or restoring before returning).
  - Use `warn_on_full_buffer=False` when using the fd for wakeup only.
  - The Python handler still runs only at the next `check_signals` on the main thread.
  - Windows requires a socket handle.
- **Semantic conditions:** signals with no Python handler (or `SIG_IGN`/`SIG_DFL`) do not write;
  coalesced signals may lose bytes when the buffer fills.
- **Limits and negative results:** non-main-thread callers cannot install it; it does not help
  threads other than the waiting main thread.
- **Evidence:** Interface-checked: Python signal module docs (quoted text on main-thread rule,
  non-blocking fd, return value, handler requirement, `warn_on_full_buffer`).

## Current applicability

### [CBO-017](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-017) — Signal-aware blocking waits at the Python boundary
- **Types:** cpython/signal-wakeup-fd.
- **Realization:** available in every supported CPython; no dependency.
- **Conclusion:** fits with conditions as an optional refinement inside one consolidated wait helper —
  it can replace interval polling with a wakeup for main-thread waits, provided the helper saves and
  restores the previous wakeup fd and falls back to polling off the main thread.
- **What it would discharge / what remains local:** discharges latency-by-interval; local: the poll
  set, fd lifetime, restore-on-exit, fallback path, cancel→drain→raise ordering.
- **Prerequisites, configuration or extension:** a completion signal that is an fd (eventfd/pipe) or a
  waiter thread bridging the channel to an fd.
- **Semantic gaps against the CBO's preservation constraints:** none to GIL release or ordering; must
  not attach Python from worker callbacks (setup/teardown happen on the calling thread).
- **Unsettled premises and what would settle them:** whether interaction with an active asyncio loop
  (or IPython/Jupyter, which manage SIGINT) is acceptable — a small probe in each host would settle it.
- **Evidence:** Interface-checked. Application Proposed.

**Reviewer correction (2026-10-27).** The comparative review does not recommend the wakeup fd. It recommends one local `pse-py` `interrupt::wait` helper with one interval and a per-site `OnSignal::{Drain, Return}` policy, leaving cause and recovery keys at each site; PyO3 still offers no wakeable `check_signals` (re-confirmed through Context7). Sequence the helper with any change that makes the native methods public types, so it is consolidated once. See the [comparative review](../../reviews/execution-lifecycle-and-observation.md#cbo-017).
