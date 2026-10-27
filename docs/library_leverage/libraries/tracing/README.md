# tracing, tracing-subscriber and tracing-opentelemetry

Realizations considered: installed `tracing` 0.1.44, `tracing-core` 0.1.36, `tracing-subscriber`
0.3.23 (features `env-filter`, `registry`; latest released, 2026-03-13), `tracing-opentelemetry`
0.34.0 (default features `tracing-log`, `metrics` active). No upgrade route examined beyond these.
Research date 2026-10-27. Sources: installed source `tracing-0.1.44/src/span.rs` (`Inner`
holds the span's `Dispatch`; `do_enter`; `Span::with_subscriber`), `src/instrument.rs`
(`in_current_span`, `WithSubscriber::with_current_subscriber`), `tracing-subscriber-0.3.23/src/registry/mod.rs`
(`SpanRef::extensions`, `extensions_mut`), `registry/extensions.rs` (`insert`, `get`, `get_mut`),
`tracing-opentelemetry-0.34.0/Cargo.toml` and `src/metrics.rs` (`MetricsLayer`, field prefixes).

## Types

### tracing/span-carried-context
- **Need it serves:** attach operation-scoped values to the current span so that whatever enters
  the span can retrieve them, instead of separate thread- or task-locals.
- **Recognizable code shapes:** several `X::current()` captures beside `Span::current()`; layers
  that look up per-span state.
- **Mechanisms and APIs:** a `Layer` stores values with `ctx.span(id).extensions_mut().insert(T)`
  (`T: Send + Sync + 'static`, one value per type); application code reaches them through
  `Span::with_subscriber(|(id, dispatch)| dispatch.downcast_ref::<Registry>()…span(id).extensions().get::<T>())`
  (the route `tracing-opentelemetry`'s span extension trait uses). A `Span` value carries its own
  `Dispatch`, so `span.enter()` on another thread notifies that span's subscriber.
- **Obligations discharged:** per-span storage lifetime (dropped at span close), lookup through
  the span tree, parent/child relationships.
- **Residual caller obligations:** the value only exists if the span is enabled; events and *new*
  spans on another thread still go to that thread's default dispatcher, so a thread with a scoped
  (non-global) dispatcher must still `set_default`.
- **Semantic conditions:** with no subscriber, or with a span disabled by every filter,
  `Span::current()` is `Span::none()` and carries nothing. Extensions are keyed by type, not by
  role.
- **Version / build conditions:** `registry` feature (installed).
- **Limits and negative results:** not a carrier for state that must exist without a subscriber;
  does not cross foreign C callbacks automatically.
- **Evidence:** Interface-checked: sources listed above.

### tracing/future-dispatch-propagation
- **Need it serves:** move the current span and dispatcher into a spawned future.
- **Recognizable code shapes:** `fut.instrument(Span::current()).with_subscriber(get_default(Clone::clone))`.
- **Mechanisms and APIs:** `Instrument::in_current_span`, `WithSubscriber::with_current_subscriber`
  (captures the current default dispatcher), `Instrument::instrument(span)`.
- **Obligations discharged:** re-entering span and dispatcher on every poll.
- **Residual caller obligations:** applying it at each spawn; OS threads need explicit
  `set_default` + `span.enter()`.
- **Limits:** futures only.
- **Evidence:** Interface-checked: `instrument.rs` at 0.1.44.

### tracing-subscriber/lifecycle-layer-fanout
- **Need it serves:** derive several observation products (timelines, counters, recorders) from one
  stream of span/event lifecycle callbacks.
- **Recognizable code shapes:** the same boundary reported to several sinks by hand at each call
  site; custom `Layer` implementations that already see `on_new_span`, `on_enter`, `on_exit`,
  `on_close`, `on_event`.
- **Mechanisms and APIs:** `Layer` callbacks; per-layer filters (`Layer::with_filter`) so an
  unfiltered recording layer sees spans that an export layer drops; `Layered` composition;
  span extensions for per-span timing state.
- **Obligations discharged:** callsite registration, filtering, span-tree bookkeeping, ordering of
  enter/exit per thread.
- **Residual caller obligations:** sink semantics (bounds, "unavailable" vs zero), and anything that
  must happen without a subscriber.
- **Semantic conditions:** callbacks run synchronously on the emitting thread; a layer must stay
  bounded and non-blocking. Sampling in an OpenTelemetry layer does not remove spans from other
  layers; filters do.
- **Limits and negative results:** nothing happens when no subscriber is installed (default
  no-op dispatcher); a layer-based fan-out cannot be the only record for a contract that requires
  observation without a subscriber.
- **Evidence:** Interface-checked: tracing-subscriber 0.3.23 `Layer` API.

### tracing-opentelemetry/event-metrics-layer
- **Need it serves:** turn tracing events into OpenTelemetry instruments without separate metric
  calls.
- **Recognizable code shapes:** an event and a counter increment emitted together.
- **Mechanisms and APIs:** `MetricsLayer::new(meter_provider)`; event fields prefixed
  `monotonic_counter.`, `counter.`, `histogram.` become synchronous instruments.
- **Obligations discharged:** instrument creation and recording per event.
- **Residual caller obligations:** provider lifetime, attribute cardinality, naming.
- **Semantic conditions:** requires an installed subscriber and a provider at layer construction;
  events before installation are not counted.
- **Version / build conditions:** `metrics` feature (default, active at 0.34.0).
- **Evidence:** Interface-checked: `tracing-opentelemetry-0.34.0/src/metrics.rs`, `Cargo.toml`.

## Current applicability

### [CBO-013](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-013) — One lifecycle event fanned out to several hand-fed sinks
- **Types:** tracing-subscriber/lifecycle-layer-fanout; tracing-opentelemetry/event-metrics-layer.
- **Realization:** installed pins.
- **Conclusion:** fits with conditions for the *subscriber-present* sinks only — the capture layer
  (`tooling/capture/layer.rs`) already demonstrates the route; recorder intervals and operational
  counts cannot move behind a layer because the hub and the counts must work, and counts must start,
  without any subscriber.
- **What it would discharge / what remains local:** with a subscriber, one span could drive capture,
  OTel export and a per-span interval; the hub (`pse-diagnostics`, which does not depend on
  `tracing`), the invocation recorder and the static atomics remain direct sinks. The library-shaped
  remedy is therefore a local lifecycle-event value that calls the direct sinks and opens the span,
  not a layer.
- **Semantic gaps:** "observation without a subscriber", "counts before any exporter", "unavailable
  distinguishable from zero" (a layer sees nothing when absent, indistinguishable from zero).
- **Unsettled premises:** whether the recorder's bounded intervals (64 intervals, 128-byte names)
  could be derived from the hub's phases instead (local consolidation, not library).
- **Evidence:** Interface-checked (`jobs.rs`, `invocation.rs`, `capture/layer.rs`, `pse-diagnostics/Cargo.toml`); Proposed.

### [CBO-014](../../../library_leverage/codebase/execution-lifecycle-and-observation.md#cbo-014) — Execution context re-established at each boundary
- **Types:** tracing/span-carried-context; tracing/future-dispatch-propagation.
- **Conclusion:** ruled out as the carrier for recorder and observation handles — they must survive
  without a subscriber and under filters, and the recorder must be the exact `Arc` even when no span
  exists. Fits only as a simplification of the tracing pair itself: `telemetry::inherit` equals
  `in_current_span().with_current_subscriber()`.
- **What remains local:** a single context value (runtime handle, recorder context, observation
  context, dispatch, span, execution scope) with one `capture()` and one `enter()` returning a
  composite guard; it also serves the staged session, where the context travels inside each
  request message to a long-lived thread rather than at thread spawn.
- **Sharpening:** production installs a global default dispatcher (`install_global` uses
  `set_global_default`), so the dispatch carrier is needed for scoped compositions (Python host,
  tests) rather than for the global path.
- **Evidence:** Interface-checked; Proposed.
