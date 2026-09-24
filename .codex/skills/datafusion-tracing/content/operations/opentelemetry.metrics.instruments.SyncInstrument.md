# `opentelemetry::metrics::instruments::SyncInstrument`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.SyncInstrument.json).

<a id="op-8182f71cc08d2a261c1e7364"></a>
## SyncInstrument

`trait` · `opentelemetry::metrics::instruments::SyncInstrument` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait SyncInstrument<T>: Send + Sync
```

Source: `src/metrics/instruments/mod.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An SDK implemented instrument that records measurements synchronously.

<a id="op-3bbfe272558e5ccbe8e79409"></a>
## measure

`function` · `opentelemetry::metrics::instruments::SyncInstrument::measure` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn measure(&self, measurement: T, attributes: &[KeyValue])
```

Source: `src/metrics/instruments/mod.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Records a measurement synchronously.
