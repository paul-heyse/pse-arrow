# `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.periodic_reader_with_async_runtime.PeriodicReaderBuilder.json).

<a id="op-3d87b1432c36b05a86275636"></a>
## PeriodicReaderBuilder

`struct` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct PeriodicReaderBuilder<E, RT>
```

Source: `src/metrics/periodic_reader_with_async_runtime.rs:48`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configuration options for [PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader_with_async_runtime.PeriodicReader.md#op-2ff5056bea9eca1b7d916148).

A periodic reader is a [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) that collects and exports metric data
to the exporter at a defined interval.

By default, the returned [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) will collect and export data every
60 seconds, and will cancel export attempts that exceed 30 seconds. The
export time is not counted towards the interval between attempts.

The [collect] method of the returned [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) continues to gather and
return metric data to the user. It will not automatically send that data to
the exporter outside of the predefined interval.

[collect]: MetricReader::collect

<a id="op-0aa367fdab9aaae39d89f3f3"></a>
## build

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> PeriodicReader<E>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "RT"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder", "path": "PeriodicReaderBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "RT"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::Runtime", "path": "Runtime"}}}], "generic_params": [], "type": {"generic": "RT"}}}]}, "is_negative": false, "span": {"begin": [55, 1], "end": [149, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a [PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader_with_async_runtime.PeriodicReader.md#op-2ff5056bea9eca1b7d916148) with the given config.

<a id="op-5890117822fce41f71af85a7"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "RT"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder", "path": "PeriodicReaderBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "RT"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [47, 10], "end": [47, 15], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7f908a87c2b965829e7e82dd"></a>
## with_interval

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder::with_interval` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_interval(self, interval: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "RT"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder", "path": "PeriodicReaderBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "RT"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::Runtime", "path": "Runtime"}}}], "generic_params": [], "type": {"generic": "RT"}}}]}, "is_negative": false, "span": {"begin": [55, 1], "end": [149, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:85`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configures the intervening time between exports for a [PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader_with_async_runtime.PeriodicReader.md#op-2ff5056bea9eca1b7d916148).

This option overrides any value set for the `OTEL_METRIC_EXPORT_INTERVAL`
environment variable.

If this option is not used or `interval` is equal to zero, 60 seconds is
used as the default.

<a id="op-0b20f361870e9ee9bd9a20fc"></a>
## with_timeout

`function` · `opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder::with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_timeout(self, timeout: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "RT"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader_with_async_runtime::PeriodicReaderBuilder", "path": "PeriodicReaderBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "RT"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::Runtime", "path": "Runtime"}}}], "generic_params": [], "type": {"generic": "RT"}}}]}, "is_negative": false, "span": {"begin": [55, 1], "end": [149, 2], "filename": "src/metrics/periodic_reader_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/periodic_reader_with_async_runtime.rs:100`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configures the time a [PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader_with_async_runtime.PeriodicReader.md#op-2ff5056bea9eca1b7d916148) waits for an export to complete
before canceling it.

This option overrides any value set for the `OTEL_METRIC_EXPORT_TIMEOUT`
environment variable.

If this option is not used or `timeout` is equal to zero, 30 seconds is used
as the default.
