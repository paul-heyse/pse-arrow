# `opentelemetry_otlp::span::SpanExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.span.SpanExporter.json).

<a id="op-9871902532b274e2c0786626"></a>
## SpanExporter

`struct` · `opentelemetry_otlp::span::SpanExporter` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanExporter
```

Source: `src/span.rs:117`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

OTLP exporter that sends tracing data

<a id="op-72dfdf8f60dc2fdebaf4c76a"></a>
## builder

`function` · `opentelemetry_otlp::span::SpanExporter::builder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> SpanExporterBuilder<NoExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::span::SpanExporter", "path": "SpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [129, 1], "end": [148, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:131`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Obtain a builder to configure a [SpanExporter](../operations/opentelemetry_otlp.span.SpanExporter.md#op-9871902532b274e2c0786626).

<a id="op-b816b3d8e3555db60b2a2e81"></a>
## export

`function` · `opentelemetry_otlp::span::SpanExporter::export` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::span::SpanExporter", "path": "SpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [150, 1], "end": [168, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}, "trait_path": "opentelemetry_sdk::trace::export::SpanExporter"}`

Source: `src/span.rs:151`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2ab9b2d06e9411a323b3d72e"></a>
## fmt

`function` · `opentelemetry_otlp::span::SpanExporter::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::span::SpanExporter", "path": "SpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [116, 10], "end": [116, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:116`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0cd3475ffb42375d8fc00c1b"></a>
## set_resource

`function` · `opentelemetry_otlp::span::SpanExporter::set_resource` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::span::SpanExporter", "path": "SpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [150, 1], "end": [168, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}, "trait_path": "opentelemetry_sdk::trace::export::SpanExporter"}`

Source: `src/span.rs:160`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
