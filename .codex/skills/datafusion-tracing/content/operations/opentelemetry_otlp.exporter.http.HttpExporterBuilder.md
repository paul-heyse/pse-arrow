# `opentelemetry_otlp::exporter::http::HttpExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.http.HttpExporterBuilder.json).

<a id="op-335af994c198d146a5909322"></a>
## HttpExporterBuilder

`struct` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct HttpExporterBuilder
```

Source: `src/exporter/http/mod.rs:85`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Configuration for the OTLP HTTP exporter.

## Examples

```no_run
# #[cfg(feature="metrics")]
use opentelemetry_sdk::metrics::Temporality;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create a span exporter you can use when configuring tracer providers
# #[cfg(feature="trace")]
let span_exporter = opentelemetry_otlp::SpanExporter::builder().with_http().build()?;

// Create a metrics exporter you can use when configuring meter providers
# #[cfg(feature="metrics")]
let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
    .with_http()
    .with_temporality(Temporality::default())
    .build()?;

// Create a log exporter you can use when configuring logger providers
# #[cfg(feature="logs")]
let log_exporter = opentelemetry_otlp::LogExporter::builder().with_http().build()?;
# Ok(())
# }
```


<a id="op-a3ba690f115854f37cbb3685"></a>
## build_log_exporter

`function` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder::build_log_exporter` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build_log_exporter(self) -> Result<LogExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::http::HttpExporterBuilder", "path": "HttpExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [295, 2], "filename": "src/exporter/http/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/exporter/http/mod.rs:257`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Create a log exporter with the current configuration

<a id="op-105d984be54c0ce5b64decf9"></a>
## build_metrics_exporter

`function` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder::build_metrics_exporter` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build_metrics_exporter(self, temporality: opentelemetry_sdk::metrics::Temporality) -> Result<MetricExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::http::HttpExporterBuilder", "path": "HttpExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [295, 2], "filename": "src/exporter/http/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/exporter/http/mod.rs:276`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Create a metrics exporter with the current configuration

<a id="op-4e4668cb611c0e2b45fc56a7"></a>
## build_span_exporter

`function` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder::build_span_exporter` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build_span_exporter(self) -> Result<SpanExporter, ExporterBuildError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::http::HttpExporterBuilder", "path": "HttpExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [295, 2], "filename": "src/exporter/http/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/exporter/http/mod.rs:238`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Create a log exporter with the current configuration

<a id="op-fbbfe24881c9581f6e0dd7d9"></a>
## default

`function` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder::default` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::http::HttpExporterBuilder", "path": "HttpExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [90, 1], "end": [103, 2], "filename": "src/exporter/http/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/exporter/http/mod.rs:91`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8922a8632d17b69537ab7904"></a>
## export_config

`function` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::http::HttpExporterBuilder", "path": "crate::exporter::http::HttpExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [230, 1], "end": [234, 2], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::HasExportConfig", "path": "HasExportConfig"}, "trait_path": "opentelemetry_otlp::exporter::HasExportConfig"}`

Source: `src/exporter/mod.rs:231`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-574d32d74964b5d1315e1cf9"></a>
## fmt

`function` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::http::HttpExporterBuilder", "path": "HttpExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [84, 10], "end": [84, 15], "filename": "src/exporter/http/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/exporter/http/mod.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-18c6e5161e8c248bdbc1636a"></a>
## http_client_config

`function` · `opentelemetry_otlp::exporter::http::HttpExporterBuilder::http_client_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn http_client_config(&mut self) -> &mut HttpConfig
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::http::HttpExporterBuilder", "path": "HttpExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [502, 1], "end": [506, 2], "filename": "src/exporter/http/mod.rs"}, "trait": {"args": null, "id": "opentelemetry_otlp::exporter::http::HasHttpConfig", "path": "HasHttpConfig"}, "trait_path": "opentelemetry_otlp::exporter::http::HasHttpConfig"}`

Source: `src/exporter/http/mod.rs:503`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
