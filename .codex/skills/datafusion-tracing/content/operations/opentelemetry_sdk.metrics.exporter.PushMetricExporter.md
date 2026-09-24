# `opentelemetry_sdk::metrics::exporter::PushMetricExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.exporter.PushMetricExporter.json).

<a id="op-6d578ed5c0fd27cf6ab55b23"></a>
## PushMetricExporter

`trait` · `opentelemetry_sdk::metrics::exporter::PushMetricExporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait PushMetricExporter: Send + Sync + 'static
```

Source: `src/metrics/exporter.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Exporter handles the delivery of metric data to external receivers.

This is the final component in the metric push pipeline.

<a id="op-35542a6960cdcf6c4734ec03"></a>
## export

`function` · `opentelemetry_sdk::metrics::exporter::PushMetricExporter::export` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export(&self, metrics: &ResourceMetrics) -> impl std::future::Future<Output = OTelSdkResult> + Send
```

Source: `src/metrics/exporter.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Export serializes and transmits metric data to a receiver.

All retry logic must be contained in this function. The SDK does not
implement any retry logic. All errors returned by this function are
considered unrecoverable and will be logged.

<a id="op-03d9a8fdf315dfb87e77df7b"></a>
## force_flush

`function` · `opentelemetry_sdk::metrics::exporter::PushMetricExporter::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Source: `src/metrics/exporter.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Flushes any metric data held by an exporter.

<a id="op-f57b2a9d425200605154be87"></a>
## shutdown

`function` · `opentelemetry_sdk::metrics::exporter::PushMetricExporter::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Source: `src/metrics/exporter.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shutdown with the default timeout of 5 seconds.

<a id="op-aaa2194ae5ef9a40e259f449"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::metrics::exporter::PushMetricExporter::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Source: `src/metrics/exporter.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Releases any held computational resources.

After Shutdown is called, calls to Export will perform no operation and
instead will return an error indicating the shutdown state.

<a id="op-f7a9d0e0ad8372d18fe6b85e"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::exporter::PushMetricExporter::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self) -> Temporality
```

Source: `src/metrics/exporter.rs:39`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Access the [Temporality](../operations/opentelemetry_sdk.metrics.Temporality.md#op-dc2884d798040a5373238d65) of the MetricExporter.
