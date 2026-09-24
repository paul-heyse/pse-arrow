# `opentelemetry::global::metrics::meter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.metrics.meter.json).

<a id="op-23e22146d4832f85a926eeb1"></a>
## meter

`function` · `opentelemetry::global::metrics::meter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn meter(name: &'static str) -> metrics::Meter
```

Source: `src/global/metrics.rs:53`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a named [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) via the currently configured global [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b).

This is a more convenient way of expressing `global::meter_provider().meter(name)`.

**NOTE:** Calls to [`meter()`](../operations/opentelemetry.global.metrics.meter.md#op-23e22146d4832f85a926eeb1) return a [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) backed by the global [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) configured during the method invocation.
If the global [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) is changed after getting [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) instances from these calls, the [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) instances returned will not reflect the change.
