# `opentelemetry::metrics::instruments::Callback`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.instruments.Callback.json).

<a id="op-0a99651bc566904de66489b6"></a>
## Callback

`type_alias` · `opentelemetry::metrics::instruments::Callback` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
type Callback<T> = Box<dyn Fn(&dyn AsyncInstrument<T>) + Send + Sync>
```

Source: `src/metrics/instruments/mod.rs:241`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A function registered with a [Meter](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) that makes observations for the
instruments it is registered with.

The async instrument parameter is used to record measurement observations
for these instruments.

The function needs to complete in a finite amount of time.
