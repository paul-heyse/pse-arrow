# `opentelemetry::global::metrics::set_meter_provider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.metrics.set_meter_provider.json).

<a id="op-6b3a1e871bc2b067a0f4aa75"></a>
## set_meter_provider

`function` · `opentelemetry::global::metrics::set_meter_provider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_meter_provider<P>(new_provider: P) where P: metrics::MeterProvider + Send + Sync + 'static
```

Source: `src/global/metrics.rs:21`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the given [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) instance as the current global meter
provider.
Libraries should NOT call this function. It is intended for applications/executables.

**NOTE:** This function should be called before getting [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) instances via [`meter()`](../operations/opentelemetry.global.metrics.meter.md#op-23e22146d4832f85a926eeb1) or [`meter_with_scope()`](../operations/opentelemetry.global.metrics.meter_with_scope.md#op-64230a73967f66d438df0c55). Otherwise, you could get no-op [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) instances.
