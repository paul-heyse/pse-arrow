# `opentelemetry_sdk::metrics`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.json).

<a id="op-d7e2bc0dc5244417e035b9f2"></a>
## metrics

`module` · `opentelemetry_sdk::metrics` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod metrics
```

Source: `src/metrics/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The crust of the OpenTelemetry metrics SDK.

## Configuration

The metrics SDK configuration is stored with each [SdkMeterProvider](../operations/opentelemetry_sdk.metrics.meter_provider.SdkMeterProvider.md#op-d674cddddafa66c4223d1cb6).
Configuration for [Resource]s, views, and [ManualReader](../operations/opentelemetry_sdk.metrics.manual_reader.ManualReader.md#op-ae28e473e5d34d7697a88a62) or
[PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader.PeriodicReader.md#op-7af448cc206ead85a28fc452) instances can be specified.

### Example

```
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_sdk::{metrics::SdkMeterProvider, Resource};

// Generate SDK configuration, resource, views, etc
let resource = Resource::builder().build(); // default attributes about the current process

// Create a meter provider with the desired config
let meter_provider = SdkMeterProvider::builder().with_resource(resource).build();
global::set_meter_provider(meter_provider.clone());

// Use the meter provider to create meter instances
let meter = global::meter("my_app");

// Create instruments scoped to the meter
let counter = meter
    .u64_counter("power_consumption")
    .with_unit("kWh")
    .build();

// use instruments to record measurements
counter.add(10, &[KeyValue::new("rate", "standard")]);

// shutdown the provider at the end of the application to ensure any metrics not yet
// exported are flushed.
meter_provider.shutdown().unwrap();
```

[Resource]: crate::Resource
