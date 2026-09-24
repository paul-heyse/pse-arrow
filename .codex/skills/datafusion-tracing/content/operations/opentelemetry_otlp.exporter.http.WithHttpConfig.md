# `opentelemetry_otlp::exporter::http::WithHttpConfig`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.http.WithHttpConfig.json).

<a id="op-ea0ad5995876c4d86bba38bb"></a>
## WithHttpConfig

`trait` · `opentelemetry_otlp::exporter::http::WithHttpConfig` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait WithHttpConfig
```

Source: `src/exporter/http/mod.rs:520`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

This trait will be implemented for every struct that implemented [`HasHttpConfig`](../operations/opentelemetry_otlp.exporter.http.HasHttpConfig.md#op-a3a67b71029b24402184a807) trait.

## Examples
```
# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
# {
use crate::opentelemetry_otlp::WithHttpConfig;
let exporter_builder = opentelemetry_otlp::SpanExporter::builder()
    .with_http()
    .with_headers(std::collections::HashMap::new());
# }
```

<a id="op-9f34168f733a6be8e2db905e"></a>
## with_compression

`function` · `opentelemetry_otlp::exporter::http::WithHttpConfig::with_compression` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_compression(self, compression: Compression) -> Self
```

Source: `src/exporter/http/mod.rs:528`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set the compression algorithm to use when communicating with the collector.

<a id="op-14247c52f57a70e56e07d462"></a>
## with_headers

`function` · `opentelemetry_otlp::exporter::http::WithHttpConfig::with_headers` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_headers(self, headers: HashMap<String, String>) -> Self
```

Source: `src/exporter/http/mod.rs:525`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set additional headers to send to the collector.

<a id="op-3c0f548a95abbcb14e99a5b7"></a>
## with_http_client

`function` · `opentelemetry_otlp::exporter::http::WithHttpConfig::with_http_client` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_http_client<T: HttpClient + 'static>(self, client: T) -> Self
```

Source: `src/exporter/http/mod.rs:522`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Assign client implementation
