# `opentelemetry_otlp::exporter::tonic::TonicExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.tonic.TonicExporterBuilder.json).

<a id="op-1dc0eafef051ed5e96a1a2a9"></a>
## TonicExporterBuilder

`struct` · `opentelemetry_otlp::exporter::tonic::TonicExporterBuilder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TonicExporterBuilder
```

Source: `src/exporter/tonic/mod.rs:103`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Configuration for the [tonic] OTLP GRPC exporter.

It allows you to
- add additional metadata
- set tls config (via the  `tls` feature)
- specify custom [channel]s

[tonic]: <https://github.com/hyperium/tonic>
[channel]: tonic::transport::Channel

## Examples

```no_run
# #[cfg(feature="metrics")]
use opentelemetry_sdk::metrics::Temporality;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create a span exporter you can use to when configuring tracer providers
# #[cfg(feature="trace")]
let span_exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic().build()?;

// Create a metric exporter you can use when configuring meter providers
# #[cfg(feature="metrics")]
let metric_exporter = opentelemetry_otlp::MetricExporter::builder()
    .with_tonic()
    .with_temporality(Temporality::default())
    .build()?;

// Create a log exporter you can use when configuring logger providers
# #[cfg(feature="logs")]
let log_exporter = opentelemetry_otlp::LogExporter::builder().with_tonic().build()?;
# Ok(())
# }
```

Unresolved upstream links (retained, not inferred): `tonic::transport::Channel`.

<a id="op-e9245f7b87851f27cbb97f7b"></a>
## default

`function` · `opentelemetry_otlp::exporter::tonic::TonicExporterBuilder::default` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::TonicExporterBuilder", "path": "TonicExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [142, 2], "filename": "src/exporter/tonic/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/exporter/tonic/mod.rs:122`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b58fd68db830fc90c77d7214"></a>
## export_config

`function` · `opentelemetry_otlp::exporter::tonic::TonicExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::TonicExporterBuilder", "path": "crate::exporter::tonic::TonicExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [222, 1], "end": [226, 2], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/exporter/mod.rs:223`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-96b47a8a30ad70e6057c73ac"></a>
## fmt

`function` · `opentelemetry_otlp::exporter::tonic::TonicExporterBuilder::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::TonicExporterBuilder", "path": "TonicExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [102, 10], "end": [102, 15], "filename": "src/exporter/tonic/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/exporter/tonic/mod.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5c8ab91d8c194a6efc423fdc"></a>
## tonic_config

`function` · `opentelemetry_otlp::exporter::tonic::TonicExporterBuilder::tonic_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tonic_config(&mut self) -> &mut TonicConfig
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::TonicExporterBuilder", "path": "TonicExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [350, 1], "end": [354, 2], "filename": "src/exporter/tonic/mod.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::tonic::HasTonicConfig", "path": "HasTonicConfig"}, "trait_path": "opentelemetry_otlp::exporter::tonic::HasTonicConfig"}`

Source: `src/exporter/tonic/mod.rs:351`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
