# `opentelemetry_otlp::logs::LogExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.logs.LogExporterBuilder.json).

<a id="op-1bbe96b98f04329c66e7d345"></a>
## LogExporterBuilder

`struct` · `opentelemetry_otlp::logs::LogExporterBuilder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct LogExporterBuilder<C>
```

Source: `src/logs.rs:36`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Builder for creating a new [LogExporter](../operations/opentelemetry_otlp.logs.LogExporter.md#op-ab5d2897a246347ffe104c53).

<a id="op-64364f56c70741690a9851b2"></a>
## build

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::build` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<LogExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [82, 2], "filename": "src/logs.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Build the [LogExporter](../operations/opentelemetry_otlp.logs.LogExporter.md#op-ab5d2897a246347ffe104c53) with the HTTP transport.

<a id="op-65d3d2a062a164aff0a071d8"></a>
## build

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::build` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<LogExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [67, 1], "end": [74, 2], "filename": "src/logs.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Build the [LogExporter](../operations/opentelemetry_otlp.logs.LogExporter.md#op-ab5d2897a246347ffe104c53) with the gRPC Tonic transport.

<a id="op-8d91a78b9203fada1347a52d"></a>
## clone

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::clone` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> LogExporterBuilder<C>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [35, 26], "end": [35, 31], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-66f1c85add8af8bbd0c9ae53"></a>
## default

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::default` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> LogExporterBuilder<C>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::default::Default", "path": "$crate::default::Default"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [35, 17], "end": [35, 24], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/logs.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6e1f94baa200a7a0ab2ea9ef"></a>
## export_config

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [85, 1], "end": [89, 2], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/logs.rs:86`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dce5c852ff98518e0f01d306"></a>
## export_config

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [92, 1], "end": [96, 2], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/logs.rs:93`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9ab3541bf928348053c92e52"></a>
## fmt

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [35, 10], "end": [35, 15], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ad45027a6f6f69c645bf0915"></a>
## http_client_config

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::http_client_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn http_client_config(&mut self) -> &mut exporter::http::HttpConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [106, 1], "end": [110, 2], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::http::HasHttpConfig", "path": "HasHttpConfig"}, "trait_path": "opentelemetry_otlp::exporter::http::HasHttpConfig"}`

Source: `src/logs.rs:107`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fd4d74c779787b6b6811a247"></a>
## new

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::new` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::NoExporterBuilderSet", "path": "crate::NoExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 1], "end": [64, 2], "filename": "src/logs.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Create a new [LogExporterBuilder](../operations/opentelemetry_otlp.logs.LogExporterBuilder.md#op-1bbe96b98f04329c66e7d345) with default settings.

<a id="op-05ccd38418f22b3e3f28c91f"></a>
## tonic_config

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::tonic_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tonic_config(&mut self) -> &mut TonicConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [99, 1], "end": [103, 2], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::HasTonicConfig", "path": "HasTonicConfig"}, "trait_path": "opentelemetry_otlp::exporter::tonic::HasTonicConfig"}`

Source: `src/logs.rs:100`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c691d5fcee9781c2787b9112"></a>
## with_http

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::with_http` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_http(self) -> LogExporterBuilder<HttpExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::NoExporterBuilderSet", "path": "crate::NoExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 1], "end": [64, 2], "filename": "src/logs.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs.rs:58`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

With the HTTP transport.

<a id="op-57a73f8a59399b287944e1bc"></a>
## with_tonic

`function` · `opentelemetry_otlp::logs::LogExporterBuilder::with_tonic` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_tonic(self) -> LogExporterBuilder<TonicExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::NoExporterBuilderSet", "path": "crate::NoExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::logs::LogExporterBuilder", "path": "LogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 1], "end": [64, 2], "filename": "src/logs.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

With the gRPC Tonic transport.
