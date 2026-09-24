# `opentelemetry::global::metrics::meter_provider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.metrics.meter_provider.json).

<a id="op-99ee8b41c51ae0b81e26e55f"></a>
## meter_provider

`function` · `opentelemetry::global::metrics::meter_provider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn meter_provider() -> std::sync::Arc<dyn MeterProvider + Send + Sync>
```

Source: `src/global/metrics.rs:36`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns an instance of the currently configured global [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b).
