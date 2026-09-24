# `opentelemetry_otlp::exporter::http::HasHttpConfig`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.http.HasHttpConfig.json).

<a id="op-a3a67b71029b24402184a807"></a>
## HasHttpConfig

`trait` · `opentelemetry_otlp::exporter::http::HasHttpConfig` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait HasHttpConfig
```

Source: `src/exporter/http/mod.rs:496`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Expose interface for modifying builder config.

<a id="op-3c457a6f683521f7a6b2540b"></a>
## http_client_config

`function` · `opentelemetry_otlp::exporter::http::HasHttpConfig::http_client_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn http_client_config(&mut self) -> &mut HttpConfig
```

Source: `src/exporter/http/mod.rs:498`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Return a mutable reference to the config within the exporter builders.
