# `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.in_memory_exporter.InMemoryMetricExporter.json).

<a id="op-bce1568ad3e395f17ac0d53a"></a>
## InMemoryMetricExporter

`struct` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InMemoryMetricExporter
```

Source: `src/metrics/in_memory_exporter.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

 An in-memory metrics exporter that stores metrics data in memory.

 This exporter is useful for testing and debugging purposes. It stores
 metric data in a `VecDeque<ResourceMetrics>`. Metrics can be retrieved
 using the `get_finished_metrics` method.

 # Panics

 This exporter may panic
 - if there's an issue with locking the `metrics` Mutex, such as if the Mutex is poisoned.
 - the data point recorded is not one of [i64, u64, f64]. This shouldn't happen if used with OpenTelemetry API.

 # Example

 ```
# use opentelemetry_sdk::metrics;
# use opentelemetry::{KeyValue};
# use opentelemetry::metrics::MeterProvider;
# use opentelemetry_sdk::metrics::InMemoryMetricExporter;
# use opentelemetry_sdk::metrics::PeriodicReader;

# #[tokio::main]
# async fn main() {
 // Create an InMemoryMetricExporter
  let exporter = InMemoryMetricExporter::default();

  // Create a MeterProvider and register the exporter
  let meter_provider = metrics::SdkMeterProvider::builder()
      .with_reader(PeriodicReader::builder(exporter.clone()).build())
      .build();

  // Create and record metrics using the MeterProvider
  let meter = meter_provider.meter("example");
  let counter = meter.u64_counter("my_counter").build();
  counter.add(1, &[KeyValue::new("key", "value")]);

  meter_provider.force_flush().unwrap();

  // Retrieve the finished metrics from the exporter
  let finished_metrics = exporter.get_finished_metrics().unwrap();

  // Print the finished metrics
 for resource_metrics in finished_metrics {
      println!("{:?}", resource_metrics);
  }
# }
 ```

<a id="op-a24f2f46eda514f9f65bf34b"></a>
## clone

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [67, 1], "end": [74, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/in_memory_exporter.rs:68`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cd0685521a6b3d141d615089"></a>
## default

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [82, 1], "end": [86, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/in_memory_exporter.rs:83`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d1e54bd1ef9e593d992dbd2"></a>
## export

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::export` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [264, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metrics/in_memory_exporter.rs:240`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-12674ffd96146c414ed9234a"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [76, 1], "end": [80, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/in_memory_exporter.rs:77`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-45462c965d991709df202f7d"></a>
## force_flush

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [264, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metrics/in_memory_exporter.rs:249`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-30dbe07a9ad88fdc876499f8"></a>
## get_finished_metrics

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::get_finished_metrics` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get_finished_metrics(&self) -> Result<Vec<ResourceMetrics>, InMemoryExporterError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [134, 1], "end": [237, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/in_memory_exporter.rs:149`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the finished metrics as a vector of `ResourceMetrics`.

# Errors

Returns a `MetricError` if the internal lock cannot be acquired.

# Example

```
# use opentelemetry_sdk::metrics::InMemoryMetricExporter;

let exporter = InMemoryMetricExporter::default();
let finished_metrics = exporter.get_finished_metrics().unwrap();
```

<a id="op-16db5090fb72bb3b1c06f181"></a>
## reset

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::reset` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn reset(&self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [134, 1], "end": [237, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/in_memory_exporter.rs:168`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Clears the internal storage of finished metrics.

# Example

```
# use opentelemetry_sdk::metrics::InMemoryMetricExporter;

let exporter = InMemoryMetricExporter::default();
exporter.reset();
```

<a id="op-6a6b61a10ad4fff150c53b07"></a>
## shutdown

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [264, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metrics/in_memory_exporter.rs:253`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c5d2abd49adebf21cabb5ec2"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [264, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metrics/in_memory_exporter.rs:257`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4e0e3830ccb6d47703f5c667"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self) -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::in_memory_exporter::InMemoryMetricExporter", "path": "InMemoryMetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [264, 2], "filename": "src/metrics/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metrics/in_memory_exporter.rs:261`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
