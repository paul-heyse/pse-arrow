# `opentelemetry::metrics::meter::MeterProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.metrics.meter.MeterProvider.json).

<a id="op-8ef66c51648972f9d178f50b"></a>
## MeterProvider

`trait` · `opentelemetry::metrics::meter::MeterProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait MeterProvider
```

Source: `src/metrics/meter.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Provides access to named [Meter](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) instances, for instrumenting an application
or crate.

<a id="op-cbd9b6cb05d16873221d7b1d"></a>
## meter

`function` · `opentelemetry::metrics::meter::MeterProvider::meter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn meter(&self, name: &'static str) -> Meter
```

Source: `src/metrics/meter.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new [Meter](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) with the provided name and default configuration.

A [Meter](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) should be scoped at most to a single application or crate. The
name needs to be unique so it does not collide with other names used by
an application, nor other applications.


# Examples

```
use opentelemetry::{global, metrics::MeterProvider};
use opentelemetry::KeyValue;

let provider = global::meter_provider();

// meter used in applications
let meter = provider.meter("my_app");
```

<a id="op-1eafb16e855754cf70e00322"></a>
## meter_with_scope

`function` · `opentelemetry::metrics::meter::MeterProvider::meter_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn meter_with_scope(&self, scope: InstrumentationScope) -> Meter
```

Source: `src/metrics/meter.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new [Meter](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a) with the given instrumentation scope.

# Examples

```
use std::sync::Arc;
use opentelemetry::InstrumentationScope;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;

let provider = SdkMeterProvider::default();

// meter used in applications/binaries
let meter = provider.meter("my_app");

// meter used in libraries/crates that optionally includes version and schema url
let scope = InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
    .with_version(env!("CARGO_PKG_VERSION"))
    .with_schema_url("https://opentelemetry.io/schema/1.0.0")
    .build();

let meter = provider.meter_with_scope(scope);
```
