# `opentelemetry::global::metrics::meter_with_scope`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.metrics.meter_with_scope.json).

<a id="op-64230a73967f66d438df0c55"></a>
## meter_with_scope

`function` · `opentelemetry::global::metrics::meter_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn meter_with_scope(scope: InstrumentationScope) -> metrics::Meter
```

Source: `src/global/metrics.rs:80`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) with the given instrumentation scope.

This is a simpler alternative to `global::meter_provider().meter_with_scope(...)`

**NOTE:** Calls to [`meter_with_scope()`](../operations/opentelemetry.global.metrics.meter_with_scope.md#op-64230a73967f66d438df0c55) return a [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) backed by the global [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) configured during the method invocation.
If the global [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) is changed after getting [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) instances from these calls, the [`Meter`](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) instances returned will not reflect the change.

# Example

```
use std::sync::Arc;
use opentelemetry::global::meter_with_scope;
use opentelemetry::InstrumentationScope;
use opentelemetry::KeyValue;

let scope = InstrumentationScope::builder("io.opentelemetry")
    .with_version("0.17")
    .with_schema_url("https://opentelemetry.io/schema/1.2.0")
    .with_attributes(vec![(KeyValue::new("key", "value"))])
    .build();

let meter = meter_with_scope(scope);
```
