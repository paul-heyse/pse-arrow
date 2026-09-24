# `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.periodic_reader_with_async_runtime.PeriodicReader.json).

<a id="op-2ff5056bea9eca1b7d916148"></a>
## PeriodicReader

`struct` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct PeriodicReader<E: PushMetricExporter>
```

Source: `src/metrics/periodic_reader_with_async_runtime.rs:188`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) that continuously collects and exports metric data at a set
interval.

By default it will collect and export data every 60 seconds, and will cancel
export attempts that exceed 30 seconds. The export time is not counted
towards the interval between attempts.

The [collect] method of the returned continues to gather and
return metric data to the user. It will not automatically send that data to
the exporter outside of the predefined interval.

The [runtime] can be selected based on feature flags set for this crate.

The exporter can be any exporter that implements [PushMetricExporter](../operations/opentelemetry_sdk.metrics.exporter.PushMetricExporter.md#op-6d578ed5c0fd27cf6ab55b23) such
as [opentelemetry-otlp].

[collect]: MetricReader::collect
[runtime]: crate::runtime
[opentelemetry-otlp]: https://docs.rs/opentelemetry-otlp/latest/opentelemetry_otlp/

# Example

```no_run
use opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader;
# fn example<E, R>(get_exporter: impl Fn() -> E, get_runtime: impl Fn() -> R)
# where
#     E: opentelemetry_sdk::metrics::exporter::PushMetricExporter,
#     R: opentelemetry_sdk::runtime::Runtime,
# {

let exporter = get_exporter(); // set up a push exporter like OTLP
let runtime = get_runtime(); // select runtime: e.g. opentelemetry_sdk:runtime::Tokio

let reader = PeriodicReader::builder(exporter, runtime).build();
# drop(reader);
# }
```

<a id="op-caf6eb45cfb3e43269754d4f"></a>
## builder

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder<RT>(exporter: E, runtime: RT) -> PeriodicReaderBuilder<E, RT> where RT: Runtime
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [202, 1], "end": [210, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:204`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configuration options for a periodic reader

<a id="op-942cfd4bf245be0bcc15daa4"></a>
## clone

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [193, 1], "end": [200, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:194`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2f191de28cfa6edaca50298b"></a>
## collect

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::collect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn collect(&self, rm: &mut ResourceMetrics) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [335, 1], "end": [440, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:356`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d4a31cc937542940c0ff822f"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [212, 1], "end": [216, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:213`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-80f165d1254099fd3f1ecb9b"></a>
## force_flush

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [335, 1], "end": [440, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:380`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9b89e351eb1620109ab4b55c"></a>
## register_pipeline

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::register_pipeline` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn register_pipeline(&self, pipeline: Weak<Pipeline>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [335, 1], "end": [440, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:336`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1c0ec25abecde9efa345cf3b"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [335, 1], "end": [440, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:401`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f21331cc0128a436772b1bee"></a>
## temporality

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader::temporality` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self, kind: InstrumentKind) -> super::Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReader", "path": "PeriodicReader"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [335, 1], "end": [440, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::reader::MetricReader", "path": "MetricReader"}, "trait_path": "opentelemetry_sdk::metrics::reader::MetricReader"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:437`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

To construct a [MetricReader][metric-reader] when setting up an SDK,
The output temporality (optional), a function of instrument kind.
This function SHOULD be obtained from the exporter.

If not configured, the Cumulative temporality SHOULD be used.
  
[metric-reader]: https://github.com/open-telemetry/opentelemetry-specification/blob/0a78571045ca1dca48621c9648ec3c832c3c541c/specification/metrics/sdk.md#metricreader
