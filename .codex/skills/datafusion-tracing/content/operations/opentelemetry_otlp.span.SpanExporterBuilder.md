# `opentelemetry_otlp::span::SpanExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.span.SpanExporterBuilder.json).

<a id="op-065d94718617452bd0b1b303"></a>
## SpanExporterBuilder

`struct` · `opentelemetry_otlp::span::SpanExporterBuilder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanExporterBuilder<C>
```

Source: `src/span.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

OTLP span exporter builder

<a id="op-39deaaa299ae4887a0997c21"></a>
## build

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::build` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<SpanExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [69, 1], "end": [76, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Build the [SpanExporter](../operations/opentelemetry_otlp.span.SpanExporter.md#op-9871902532b274e2c0786626) with the gRPC Tonic transport.

<a id="op-6299ea52db2014e6ba8cde27"></a>
## build

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::build` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> Result<SpanExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [79, 1], "end": [85, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:81`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Build the [SpanExporter](../operations/opentelemetry_otlp.span.SpanExporter.md#op-9871902532b274e2c0786626) with the HTTP transport.

<a id="op-3cac6170e16619d29dc997a0"></a>
## clone

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::clone` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanExporterBuilder<C>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [40, 26], "end": [40, 31], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/span.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-accfe50fa2ba2b70b020465f"></a>
## default

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::default` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> SpanExporterBuilder<C>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::default::Default", "path": "$crate::default::Default"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [40, 17], "end": [40, 24], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/span.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3b6beda90f76f6f98ea50f87"></a>
## export_config

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [95, 1], "end": [99, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/span.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cb0fd86b322e77715fffa760"></a>
## export_config

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [92, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/span.rs:89`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-92ace85eff414ccdb178a567"></a>
## fmt

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "C"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [40, 10], "end": [40, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-57f27be3d179691fb29af6b6"></a>
## http_client_config

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::http_client_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn http_client_config(&mut self) -> &mut exporter::http::HttpConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::HttpExporterBuilderSet", "path": "crate::HttpExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [109, 1], "end": [113, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::http::HasHttpConfig", "path": "HasHttpConfig"}, "trait_path": "opentelemetry_otlp::exporter::http::HasHttpConfig"}`

Source: `src/span.rs:110`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-28adef71e39416249541f4ce"></a>
## new

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::new` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::NoExporterBuilderSet", "path": "crate::NoExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 1], "end": [66, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Create a new [SpanExporterBuilder](../operations/opentelemetry_otlp.span.SpanExporterBuilder.md#op-065d94718617452bd0b1b303) with default settings.

<a id="op-653d75ac5f6b65c8ce4486fc"></a>
## tonic_config

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::tonic_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tonic_config(&mut self) -> &mut TonicConfig
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::TonicExporterBuilderSet", "path": "crate::TonicExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [102, 1], "end": [106, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::HasTonicConfig", "path": "HasTonicConfig"}, "trait_path": "opentelemetry_otlp::exporter::tonic::HasTonicConfig"}`

Source: `src/span.rs:103`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2a66a27a294bd8d508d1d65f"></a>
## with_http

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::with_http` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_http(self) -> SpanExporterBuilder<HttpExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::NoExporterBuilderSet", "path": "crate::NoExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 1], "end": [66, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:61`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

With the HTTP transport.

<a id="op-f2cdd8c6721634789a81b4ea"></a>
## with_tonic

`function` · `opentelemetry_otlp::span::SpanExporterBuilder::with_tonic` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_tonic(self) -> SpanExporterBuilder<TonicExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::NoExporterBuilderSet", "path": "crate::NoExporterBuilderSet"}}}], "constraints": []}}, "id": "opentelemetry_otlp::span::SpanExporterBuilder", "path": "SpanExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 1], "end": [66, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:53`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

With the gRPC Tonic transport.
