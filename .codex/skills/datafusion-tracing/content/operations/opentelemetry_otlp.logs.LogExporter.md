# `opentelemetry_otlp::logs::LogExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.logs.LogExporter.json).

<a id="op-ab5d2897a246347ffe104c53"></a>
## LogExporter

`struct` · `opentelemetry_otlp::logs::LogExporter` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct LogExporter
```

Source: `src/logs.rs:114`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

OTLP exporter that sends log data

<a id="op-9505de08786b4fefc77ac298"></a>
## builder

`function` · `opentelemetry_otlp::logs::LogExporter::builder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> LogExporterBuilder<NoExporterBuilderSet>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::logs::LogExporter", "path": "LogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [126, 1], "end": [145, 2], "filename": "src/logs.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs.rs:128`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Obtain a builder to configure a [LogExporter](../operations/opentelemetry_otlp.logs.LogExporter.md#op-ab5d2897a246347ffe104c53).

<a id="op-650924e411a153257d718b9c"></a>
## export

`function` · `opentelemetry_otlp::logs::LogExporter::export` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::logs::LogExporter", "path": "LogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 1], "end": [174, 2], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}, "trait_path": "opentelemetry_sdk::logs::export::LogExporter"}`

Source: `src/logs.rs:148`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-54dfd1edde820d88d57d13d1"></a>
## fmt

`function` · `opentelemetry_otlp::logs::LogExporter::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::logs::LogExporter", "path": "LogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [113, 10], "end": [113, 15], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs.rs:113`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-82a2045c7c61bafa2e08bf5c"></a>
## set_resource

`function` · `opentelemetry_otlp::logs::LogExporter::set_resource` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &opentelemetry_sdk::Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::logs::LogExporter", "path": "LogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 1], "end": [174, 2], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}, "trait_path": "opentelemetry_sdk::logs::export::LogExporter"}`

Source: `src/logs.rs:157`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-909363780c196ac8e0b64f70"></a>
## shutdown_with_timeout

`function` · `opentelemetry_otlp::logs::LogExporter::shutdown_with_timeout` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: time::Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::logs::LogExporter", "path": "LogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [147, 1], "end": [174, 2], "filename": "src/logs.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}, "trait_path": "opentelemetry_sdk::logs::export::LogExporter"}`

Source: `src/logs.rs:166`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
