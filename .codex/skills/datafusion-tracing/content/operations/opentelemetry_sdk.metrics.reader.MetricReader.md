# `opentelemetry_sdk::metrics::reader::MetricReader`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.reader.MetricReader.json).

<a id="op-50fae5ec1b547de01d0084a6"></a>
## MetricReader

`trait` · `opentelemetry_sdk::metrics::reader::MetricReader` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait MetricReader: fmt::Debug + Send + Sync + 'static
```

Source: `src/metrics/reader.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The interface used between the SDK and an exporter.

Control flow is bi-directional through the `MetricReader`, since the SDK
initiates `force_flush` and `shutdown` while the reader initiates
collection. The `register_pipeline` method here informs the metric reader
that it can begin reading, signaling the start of bi-directional control
flow.

Typically, push-based exporters that are periodic will implement
`MetricExporter` themselves and construct a `PeriodicReader` to satisfy this
interface.

Pull-based exporters will typically implement `MetricReader` themselves,
since they read on demand.

<a id="op-6af6e8e8effb8a7bcc01e0a5"></a>
## collect

`function` · `opentelemetry_sdk::metrics::reader::MetricReader::collect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult
```

Source: `src/metrics/reader.rs:33`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Gathers and returns all metric data related to the [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) from the
SDK and stores it in the provided [ResourceMetrics](../operations/opentelemetry_sdk.metrics.data.ResourceMetrics.md#op-779be1786eb4f37fa53a44d6) reference.

An error is returned if this is called after shutdown.

<a id="op-1d6e84fc80ea68b4858a3e7d"></a>
## force_flush

`function` · `opentelemetry_sdk::metrics::reader::MetricReader::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Source: `src/metrics/reader.rs:39`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Flushes all metric measurements held in an export pipeline.

There is no guaranteed that all telemetry be flushed or all resources have
been released on error.

<a id="op-86d8f51e89ba576a4b39fa84"></a>
## register_pipeline

`function` · `opentelemetry_sdk::metrics::reader::MetricReader::register_pipeline` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn register_pipeline(&self, pipeline: Weak<Pipeline>)
```

Source: `src/metrics/reader.rs:27`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Registers a [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) with a [Pipeline].

The pipeline argument allows the `MetricReader` to signal the sdk to collect
and send aggregated metric measurements.

Unresolved upstream links (retained, not inferred): `Pipeline`.

<a id="op-b78f65f4a42d8c7f7a9c56c2"></a>
## shutdown

`function` · `opentelemetry_sdk::metrics::reader::MetricReader::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Source: `src/metrics/reader.rs:52`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

shutdown with default timeout

<a id="op-e34c46d66c21addf2f595e35"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::metrics::reader::MetricReader::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Source: `src/metrics/reader.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Flushes all metric measurements held in an export pipeline and releases any
held computational resources.

There is no guaranteed that all telemetry be flushed or all resources have
been released on error.

After `shutdown` is called, calls to `collect` will perform no operation and
instead will return an error indicating the shutdown state.

<a id="op-4fd31a42412cde3a1fd27f6e"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::reader::MetricReader::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self, kind: InstrumentKind) -> Temporality
```

Source: `src/metrics/reader.rs:60`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The output temporality, a function of instrument kind.
This SHOULD be obtained from the exporter.

If not configured, the Cumulative temporality SHOULD be used.
