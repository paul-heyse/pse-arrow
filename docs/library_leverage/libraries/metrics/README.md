# metrics and metrics-util

Realizations considered: not in the workspace. Upgrade route `metrics` 0.24.6 and `metrics-util`
0.20.4 (both released 2026-05-13).
Research date 2026-10-27. Sources: Context7 `/metrics-rs/metrics` (`_autodocs/index.md` architecture,
`api-reference/metrics-recorder.md` for `with_local_recorder`, `with_recorder`).

## Types

### metrics/facade-recorder
- **Need it serves:** decouple counting call sites from the storage and export backend.
- **Recognizable code shapes:** static counters plus a hand-written export bridge; several backends
  fed by the same increments.
- **Mechanisms and APIs:** `counter!`/`gauge!`/`histogram!` macros build a `Key` (name + labels) and
  call the effective recorder: thread-local (`with_local_recorder`, `set_default_local_recorder`),
  else global (`set_global_recorder`, once), else `NoopRecorder`. Handles are
  `Arc<dyn CounterFn>`. `metrics-util` supplies `Registry` with `AtomicStorage`, a
  `DebuggingRecorder`/`Snapshotter`, and recorder layers (fan-out, prefix, filter, router).
- **Obligations discharged:** key registry, handle caching, backend fan-out.
- **Residual caller obligations:** installing a recorder before the first count, saturation or
  overflow semantics, "unavailable" semantics, bridging to OpenTelemetry (separate crate).
- **Semantic conditions:** increments before a recorder is installed go to the no-op recorder and
  are lost; keys are dynamic name+label pairs rather than a closed enumeration.
- **Limits and negative results:** a second metric facade beside OpenTelemetry; no built-in
  saturating counter.
- **Evidence:** Interface-checked: Context7 autodocs.

## Current applicability

### [CBO-013](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-013) — `metrics` facade instead of static atomics
- **Types:** metrics/facade-recorder.
- **Realization:** upgrade route (new dependency pair).
- **Conclusion:** ruled out here — it reproduces the static atomic arrays with a dynamic registry,
  loses the closed event enumeration and the saturation flag unless a custom recorder is written,
  needs a recorder installed at process start to satisfy "counts start before any exporter", and
  adds a second facade beside OpenTelemetry without removing the observable-callback bridge.
- **Unsettled premises:** none consequential; would change only if several independent metric
  backends had to be fed.
- **Evidence:** Interface-checked; Proposed.
