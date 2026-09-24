# `opentelemetry_otlp::metric::MetricExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.metric.MetricExporter.json).

<a id="op-f1e874cefc2fb52cfa3fbce4"></a>
## MetricExporter

`struct` · `opentelemetry_otlp::metric::MetricExporter` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct MetricExporter
```

Source: `src/metric.rs:139`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Export metrics in OTEL format.

<a id="op-64286b6029ff8c3301a61ea4"></a>
## builder

`function` · `opentelemetry_otlp::metric::MetricExporter::builder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> MetricExporterBuilder<NoExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::metric::MetricExporter", "path": "MetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [191, 1], "end": [218, 2], "filename": "src/metric.rs"}, "trait": null, "trait_path": null}`

Source: `src/metric.rs:193`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Obtain a builder to configure a [MetricExporter](../operations/opentelemetry_otlp.metric.MetricExporter.md#op-f1e874cefc2fb52cfa3fbce4).

<a id="op-000dcecdfd543bfee5cc47cf"></a>
## export

`function` · `opentelemetry_otlp::metric::MetricExporter::export` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
async fn export(&self, metrics: &ResourceMetrics) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::metric::MetricExporter", "path": "MetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [158, 1], "end": [189, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metric.rs:159`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-44e2fc7da25eb12699324633"></a>
## fmt

`function` · `opentelemetry_otlp::metric::MetricExporter::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::metric::MetricExporter", "path": "MetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [152, 1], "end": [156, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metric.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-618f37004fd406996d3c5c41"></a>
## force_flush

`function` · `opentelemetry_otlp::metric::MetricExporter::force_flush` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::metric::MetricExporter", "path": "MetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [158, 1], "end": [189, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metric.rs:168`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1ce1f8553fdca1f155702162"></a>
## shutdown

`function` · `opentelemetry_otlp::metric::MetricExporter::shutdown` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::metric::MetricExporter", "path": "MetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [158, 1], "end": [189, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metric.rs:173`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a96c4a9aaba2d659f7850918"></a>
## shutdown_with_timeout

`function` · `opentelemetry_otlp::metric::MetricExporter::shutdown_with_timeout` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: std::time::Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::metric::MetricExporter", "path": "MetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [158, 1], "end": [189, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metric.rs:177`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-779bfce0a758346911783cdf"></a>
## temporality

`function` · `opentelemetry_otlp::metric::MetricExporter::temporality` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn temporality(&self) -> Temporality
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::metric::MetricExporter", "path": "MetricExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [158, 1], "end": [189, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::metrics::exporter::PushMetricExporter", "path": "PushMetricExporter"}, "trait_path": "opentelemetry_sdk::metrics::exporter::PushMetricExporter"}`

Source: `src/metric.rs:186`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
