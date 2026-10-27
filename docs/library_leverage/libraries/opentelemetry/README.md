# OpenTelemetry Rust (API and SDK)

Realizations considered: installed `opentelemetry` / `opentelemetry_sdk` 0.33.1 (features
`trace`, `logs`, `metrics`; SDK also `internal-logs`); latest released is 0.33.1 (2026-10-09).
Research date 2026-10-27. Sources: Context7 `/open-telemetry/opentelemetry-rust` —
`docs/metrics.md` (synchronous vs asynchronous instruments) and `docs/design/observability.md`
(global meter snapshot behaviour); repository use read at baseline for context only.

## Types

### opentelemetry/observable-instruments-over-owned-state
- **Need it serves:** export counts or gauges whose authoritative value is maintained elsewhere
  (atomics, registries, runtime metrics) on the SDK's collection schedule.
- **Recognizable code shapes:** static atomics read by `with_callback` closures; snapshots taken in
  callbacks.
- **Mechanisms and APIs:** `meter.u64_observable_counter(name).with_description(..).with_unit(..)
  .with_callback(|observer| observer.observe(v, attrs)).build()` and the gauge / f64 / up-down
  variants; callbacks run per collection.
- **Obligations discharged:** collection timing, aggregation, temporality, export.
- **Residual caller obligations:** the source of truth, saturation and "unavailable" handling
  (a callback can skip observing), attribute vocabularies.
- **Semantic conditions:** the upstream guide recommends `ObservableCounter` exactly "when you
  already have a variable tracking a count".
- **Limits:** builder calls are imperative per instrument; no declarative instrument table.
- **Evidence:** Interface-checked: `docs/metrics.md` (Context7).

### opentelemetry/sync-instruments-and-provider-order (negative)
- **Need it serves:** count at the point of occurrence with `Counter::add`.
- **Mechanisms and APIs:** `Counter<u64>::add`, `Histogram::record`, `UpDownCounter`, `Gauge`.
- **Semantic conditions:** `global::meter()` returns a snapshot that does not upgrade when a provider
  is installed later; instruments created before the provider are permanently no-op
  (`docs/design/observability.md`). Values are readable in-process only through a reader
  collection, not as plain numbers.
- **Limits and negative results:** cannot provide counts that begin before any exporter/provider
  exists, nor a cheap in-process read for passive checkpoints.
- **Evidence:** Interface-checked: upstream design note (Context7).

## Current applicability

### [CBO-013](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-013) — Could synchronous instruments replace atomics read by observable callbacks?
- **Types:** both above.
- **Realization:** installed 0.33.1.
- **Conclusion:** ruled out here — counts must start before any exporter and are also read by
  passive checkpoints; observable callbacks over owned atomics are the upstream-recommended shape
  and match the current design.
- **What remains local:** the atomics, saturation flag, snapshots. The repetitive builder code in
  `telemetry.rs::install_runtime_metrics` could be driven from `MetricDeclaration.kind`
  (`observable_counter`/`observable_gauge`, u64/f64) by a small local table — no library feature.
- **Evidence:** Interface-checked; Proposed.
