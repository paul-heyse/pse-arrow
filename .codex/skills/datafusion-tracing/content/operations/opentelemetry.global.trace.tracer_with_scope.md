# `opentelemetry::global::trace::tracer_with_scope`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.tracer_with_scope.json).

<a id="op-6cab60b60532289cc7111c48"></a>
## tracer_with_scope

`function` · `opentelemetry::global::trace::tracer_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer_with_scope(scope: InstrumentationScope) -> BoxedTracer
```

Source: `src/global/trace.rs:419`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a [`Tracer`] with the given instrumentation scope
via the configured [`GlobalTracerProvider`](../operations/opentelemetry.global.trace.GlobalTracerProvider.md#op-41d9d444ae131d3115b9c6a7).

This is a simpler alternative to `global::tracer_provider().tracer_with_scope(...)`

# Example

```
use std::sync::Arc;
use opentelemetry::global::tracer_with_scope;
use opentelemetry::InstrumentationScope;
use opentelemetry::KeyValue;

let scope = InstrumentationScope::builder("io.opentelemetry")
    .with_version("0.17")
    .with_schema_url("https://opentelemetry.io/schema/1.2.0")
    .with_attributes(vec![(KeyValue::new("key", "value"))])
    .build();

let tracer = tracer_with_scope(scope);
```

[`Tracer`]: crate::trace::Tracer
