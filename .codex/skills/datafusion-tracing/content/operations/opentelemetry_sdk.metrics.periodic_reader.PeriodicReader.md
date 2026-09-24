# `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.periodic_reader.PeriodicReader.json).

<a id="op-7af448cc206ead85a28fc452"></a>
## PeriodicReader

`struct` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct PeriodicReader<E: PushMetricExporter>
```

Source: `src/metrics/periodic_reader.rs:128`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A `MetricReader` that periodically collects and exports metrics at a configurable interval.

By default, [`PeriodicReader`] collects and exports metrics every **60 seconds**.
The time taken for export is **not** included in the interval. Use [`PeriodicReaderBuilder`]
to customize the interval.

[`PeriodicReader`] spawns a background thread to handle metric collection and export.
This thread remains active until [`shutdown()`] is called.

## Collection Process
"Collection" refers to gathering aggregated metrics from the SDK's internal storage.
During this phase, callbacks from observable instruments are also triggered.

[`PeriodicReader`] does **not** enforce a timeout for collection. If an
observable callback takes too long, it may delay the next collection cycle.
If a callback never returns, it **will stall** all metric collection (and exports)
indefinitely.

## Exporter Compatibility
When used with the [`OTLP Exporter`](https://docs.rs/opentelemetry-otlp), the following
transport options are supported:

- **`grpc-tonic`**: Requires [`MeterProvider`] to be initialized within a `tokio` runtime.
- **`reqwest-blocking-client`**: Works with both a standard (`main`) function and `tokio::main`.

[`PeriodicReader`] does **not** enforce a timeout for exports either. Instead,
the configured exporter is responsible for enforcing timeouts. If an export operation
never returns, [`PeriodicReader`] will **stop exporting new metrics**, stalling
metric collection.

## Manual Export & Shutdown
Users can manually trigger an export via [`force_flush()`]. Calling [`shutdown()`]
exports any remaining metrics and should be done before application exit to ensure
all data is sent.

**Warning**: If using **tokio’s current-thread runtime**, calling [`shutdown()`]
from the main thread may cause a deadlock. To prevent this, call [`shutdown()`]
from a separate thread or use tokio's `spawn_blocking`.

[`PeriodicReader`]: crate::metrics::PeriodicReader
[`PeriodicReaderBuilder`]: crate::metrics::PeriodicReaderBuilder
[`MeterProvider`]: crate::metrics::SdkMeterProvider
[`shutdown()`]: crate::metrics::SdkMeterProvider::shutdown
[`force_flush()`]: crate::metrics::SdkMeterProvider::force_flush

# Example

```no_run
use opentelemetry_sdk::metrics::PeriodicReader;
# fn example<E>(get_exporter: impl Fn() -> E)
# where
#     E: opentelemetry_sdk::metrics::exporter::PushMetricExporter,
# {

let exporter = get_exporter(); // set up a push exporter

let reader = PeriodicReader::builder(exporter).build();
# drop(reader);
# }
```

<a id="op-ce32b66f1ae55cdaa8fcfe60"></a>
## builder

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder(exporter: E) -> PeriodicReaderBuilder<E>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [140, 1], "end": [342, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/periodic_reader.rs:142`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configuration options for a periodic reader with own thread

<a id="op-ebaabef3303f33c280014bd8"></a>
## clone

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [132, 1], "end": [138, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/periodic_reader.rs:133`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8cab45851963d885ce3b159a"></a>
## collect

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::collect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [478, 1], "end": [509, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader.rs:483`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f359de19fa7fd0b8d4acbec3"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [344, 1], "end": [348, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/periodic_reader.rs:345`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-af424f7507389a1bc452df95"></a>
## force_flush

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [478, 1], "end": [509, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader.rs:487`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f7bb2a9388ad76ac765e278d"></a>
## register_pipeline

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::register_pipeline` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn register_pipeline(&self, pipeline: Weak<Pipeline>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [478, 1], "end": [509, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader.rs:479`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3b1efe919a1cdaf2e6b129ab"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [478, 1], "end": [509, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader.rs:495`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4144326e245450768b0e9d38"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReader::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self, kind: InstrumentKind) -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [478, 1], "end": [509, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader.rs:506`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

To construct a [MetricReader][metric-reader] when setting up an SDK,
The output temporality (optional), a function of instrument kind.
This function SHOULD be obtained from the exporter.

If not configured, the Cumulative temporality SHOULD be used.

[metric-reader]: https://github.com/open-telemetry/opentelemetry-specification/blob/0a78571045ca1dca48621c9648ec3c832c3c541c/specification/metrics/sdk.md#metricreader
