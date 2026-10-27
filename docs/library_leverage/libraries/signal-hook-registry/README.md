# signal-hook / signal-hook-registry

Realizations considered: upgrade route `signal-hook-registry` 1.4.8 (docs.rs latest at 2026-10-27);
not in the workspace lockfile (`ctrlc` 3.5.2 is present transitively).
Research date 2026-10-27. Sources: docs.rs `signal_hook_registry::register` (1.4.8).

## Types

### signal-hook/chained-native-signal-action
- **Need it serves:** observe a signal from Rust (set a flag, wake a waiter) without a Python
  handler involved.
- **Recognizable code shapes:** a Rust-side SIGINT observer in an embedded or extension context.
- **Mechanisms and APIs:** `signal_hook_registry::register(signal, action)` (unsafe, async-signal-safe
  action); `signal_hook::flag::register`, `low_level::pipe::register` (self-pipe wakeup).
  "If there was a previous signal handler for the given signal, it is chained" — the previous
  handler runs inside signal-hook's handler before the new actions.
- **Obligations discharged:** multiple actions per signal, chaining to the prior handler, self-pipe
  helpers.
- **Residual caller obligations:** registering before other threads install handlers (documented
  race); inside a Python process, any later `signal.signal()` call reinstalls CPython's own C handler
  and silently removes signal-hook's (the docs do not address later replacement).
- **Semantic conditions:** process-global; the action runs in signal context.
- **Limits and negative results:** no coordination with CPython's handler table.
- **Evidence:** Interface-checked: docs.rs `register` page (1.4.8).

## Current applicability

### [CBO-017](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-017) — Signal-aware blocking waits at the Python boundary
- **Types:** signal-hook/chained-native-signal-action.
- **Realization:** upgrade route (new dependency).
- **Conclusion:** ruled out here — a Python extension does not own the process signal table; CPython's
  wakeup-fd mechanism (cpython-signal-wakeup-fd.md) provides the same wakeup through the
  interpreter's own handler without the replacement hazard.
- **Evidence:** Interface-checked. Application Proposed (negative).
