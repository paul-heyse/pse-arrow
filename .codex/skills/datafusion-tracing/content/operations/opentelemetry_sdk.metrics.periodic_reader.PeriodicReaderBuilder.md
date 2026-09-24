# `opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.periodic_reader.PeriodicReaderBuilder.json).

<a id="op-1ccb928c4d2c2ae19ce4569c"></a>
## PeriodicReaderBuilder

`struct` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct PeriodicReaderBuilder<E>
```

Source: `src/metrics/periodic_reader.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configuration options for [PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader.PeriodicReader.md#op-7af448cc206ead85a28fc452).

<a id="op-9a8cb8b42149339d684097f0"></a>
## build

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> PeriodicReader<E>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder", "path": "PeriodicReaderBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [35, 1], "end": [66, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/periodic_reader.rs:63`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a [PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader.PeriodicReader.md#op-7af448cc206ead85a28fc452) with the given config.

<a id="op-8a1bd459d0e50f66cd4694e6"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder", "path": "PeriodicReaderBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 10], "end": [29, 15], "filename": "src/metrics/periodic_reader.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/periodic_reader.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d3cd98d72e13d66a6ea2ab33"></a>
## with_interval

`function` · `opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder::with_interval` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_interval(self, interval: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::metrics::periodic_reader::PeriodicReaderBuilder", "path": "PeriodicReaderBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}}}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [35, 1], "end": [66, 2], "filename": "src/metrics/periodic_reader.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/periodic_reader.rs:55`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configures the intervening time between exports for a [PeriodicReader](../operations/opentelemetry_sdk.metrics.periodic_reader.PeriodicReader.md#op-7af448cc206ead85a28fc452).

This option overrides any value set for the `OTEL_METRIC_EXPORT_INTERVAL`
environment variable.

If this option is not used or `interval` is equal to zero, 60 seconds is
used as the default.
