# `opentelemetry::metrics::instruments::AsyncInstrument`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.AsyncInstrument.json).

<a id="op-8954fcba997c984bc9c64c4e"></a>
## AsyncInstrument

`trait` · `opentelemetry::metrics::instruments::AsyncInstrument` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait AsyncInstrument<T>: Send + Sync
```

Source: `src/metrics/instruments/mod.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An SDK implemented instrument that records measurements via callback.

<a id="op-987601d76f6b76689552f350"></a>
## observe

`function` · `opentelemetry::metrics::instruments::AsyncInstrument::observe` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn observe(&self, measurement: T, attributes: &[KeyValue])
```

Source: `src/metrics/instruments/mod.rs:24`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Observes the state of the instrument.

It is only valid to call this within a callback.
