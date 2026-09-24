# `opentelemetry_otlp::metric::MetricExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.metric.MetricExporterBuilder.json).

<a id="op-35d2e41d4737211d5d82da8e"></a>
## MetricExporterBuilder

`struct` · `opentelemetry_otlp::metric::MetricExporterBuilder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct MetricExporterBuilder<C>
```

Source: `src/metric.rs:42`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

A builder for creating a new [MetricExporter](../operations/opentelemetry_otlp.metric.MetricExporter.md#op-f1e874cefc2fb52cfa3fbce4).

<a id="op-b865a2ba4cc248f3b71eb493"></a>
## build

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::build` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<MetricExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [93, 1], "end": [99, 2], "filename": "src/metric.rs"}, "trait": null, "trait_path": null}`

Source: `src/metric.rs:95`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Build the [MetricExporter](../operations/opentelemetry_otlp.metric.MetricExporter.md#op-f1e874cefc2fb52cfa3fbce4) with the HTTP transport.

<a id="op-eef64ee6ce7ec593545c8fea"></a>
## build

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::build` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<MetricExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [83, 1], "end": [90, 2], "filename": "src/metric.rs"}, "trait": null, "trait_path": null}`

Source: `src/metric.rs:85`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Build the [MetricExporter](../operations/opentelemetry_otlp.metric.MetricExporter.md#op-f1e874cefc2fb52cfa3fbce4) with the gRPC Tonic transport.

<a id="op-e7b013b5034ef8997666c3b4"></a>
## clone

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::clone` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> MetricExporterBuilder<C>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 26], "end": [41, 31], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metric.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1231b2347b1510c16a4e8cc0"></a>
## default

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::default` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> MetricExporterBuilder<C>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::default::Default", "path": "$crate::default::Default"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 17], "end": [41, 24], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metric.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-56826135850b2036685c1078"></a>
## export_config

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [102, 1], "end": [106, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/metric.rs:103`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fb9d8c9c8461a6654bf3c26e"></a>
## export_config

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [109, 1], "end": [113, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/metric.rs:110`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-62b91d4e7e3e466c27e03c1b"></a>
## fmt

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 10], "end": [41, 15], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metric.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-496fbbb2589253be98373f98"></a>
## http_client_config

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::http_client_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn http_client_config(&mut self) -> &mut exporter::http::HttpConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [123, 1], "end": [127, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::http::HasHttpConfig", "path": "HasHttpConfig"}, "trait_path": "opentelemetry_otlp::exporter::http::HasHttpConfig"}`

Source: `src/metric.rs:124`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f73d173a1b755423a2713e05"></a>
## new

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::new` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::NoExporterBuilderSet", "path": "crate::NoExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [47, 1], "end": [52, 2], "filename": "src/metric.rs"}, "trait": null, "trait_path": null}`

Source: `src/metric.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Create a new [MetricExporterBuilder](../operations/opentelemetry_otlp.metric.MetricExporterBuilder.md#op-35d2e41d4737211d5d82da8e) with default settings.

<a id="op-2f5e58c8112447a50ca37275"></a>
## tonic_config

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::tonic_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tonic_config(&mut self) -> &mut TonicConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [116, 1], "end": [120, 2], "filename": "src/metric.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::HasTonicConfig", "path": "HasTonicConfig"}, "trait_path": "opentelemetry_otlp::exporter::tonic::HasTonicConfig"}`

Source: `src/metric.rs:117`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2047c8675ca8dd93bafa8f2d"></a>
## with_http

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::with_http` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_http(self) -> MetricExporterBuilder<HttpExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [80, 2], "filename": "src/metric.rs"}, "trait": null, "trait_path": null}`

Source: `src/metric.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

With the HTTP transport.

<a id="op-bf1fb9bc0677c79e314c183e"></a>
## with_temporality

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::with_temporality` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_temporality(self, temporality: Temporality) -> MetricExporterBuilder<C>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [80, 2], "filename": "src/metric.rs"}, "trait": null, "trait_path": null}`

Source: `src/metric.rs:74`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set the temporality for the metrics.

<a id="op-55d24e578462f7e853b234bc"></a>
## with_tonic

`function` · `opentelemetry_otlp::metric::MetricExporterBuilder::with_tonic` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_tonic(self) -> MetricExporterBuilder<TonicExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::metric::MetricExporterBuilder", "path": "MetricExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [54, 1], "end": [80, 2], "filename": "src/metric.rs"}, "trait": null, "trait_path": null}`

Source: `src/metric.rs:57`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

With the gRPC Tonic transport.
