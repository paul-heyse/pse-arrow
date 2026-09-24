# `tracing_opentelemetry::layer::layer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_opentelemetry.layer.layer.json).

<a id="op-4afec9087dca8bb3ed230ac2"></a>
## layer

`function` · `tracing_opentelemetry::layer::layer` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn layer<S>() -> OpenTelemetryLayer<S, noop::NoopTracer> where S: Subscriber + for<'span> LookupSpan<'span>
```

Source: `src/layer.rs:76`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Construct a layer to track spans via [OpenTelemetry].

[OpenTelemetry]: https://opentelemetry.io

# Examples

```rust,no_run
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;

// Use the tracing subscriber `Registry`, or any other subscriber
// that impls `LookupSpan`
let subscriber = Registry::default().with(tracing_opentelemetry::layer());
# drop(subscriber);
```
