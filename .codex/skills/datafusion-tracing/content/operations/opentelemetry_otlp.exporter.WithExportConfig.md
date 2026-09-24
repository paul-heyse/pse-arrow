# `opentelemetry_otlp::exporter::WithExportConfig`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.WithExportConfig.json).

<a id="op-0bdf07713b47cb866690df94"></a>
## WithExportConfig

`trait` · `opentelemetry_otlp::exporter::WithExportConfig` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait WithExportConfig
```

Source: `src/exporter/mod.rs:250`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Expose methods to override [ExportConfig](../operations/opentelemetry_otlp.exporter.ExportConfig.md#op-cdfcaa8d369d011331bae7f7).

This trait will be implemented for every struct that implemented [`HasExportConfig`](../operations/opentelemetry_otlp.exporter.HasExportConfig.md#op-3fee529c9e1df0847979bb0a) trait.

## Examples
```
# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
# {
use crate::opentelemetry_otlp::WithExportConfig;
let exporter_builder = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .with_endpoint("http://localhost:7201");
# }
```

<a id="op-c3ad5bc6d28d7ee0abfd5fd2"></a>
## with_endpoint

`function` · `opentelemetry_otlp::exporter::WithExportConfig::with_endpoint` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_endpoint<T: Into<String>>(self, endpoint: T) -> Self
```

Source: `src/exporter/mod.rs:254`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set the address of the OTLP collector. If not set or set to empty string, the default address is used.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-35ecc00dad482b1b9b42504f"></a>
## with_export_config

`function` · `opentelemetry_otlp::exporter::WithExportConfig::with_export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_export_config(self, export_config: ExportConfig) -> Self
```

Source: `src/exporter/mod.rs:270`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set export config. This will override all previous configurations.

Note: Programmatically setting this will override any value set via environment variables.

<a id="op-46693ba38bfc7e7c338b4d7b"></a>
## with_protocol

`function` · `opentelemetry_otlp::exporter::WithExportConfig::with_protocol` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_protocol(self, protocol: Protocol) -> Self
```

Source: `src/exporter/mod.rs:262`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set the protocol to use when communicating with the collector.

Note that protocols that are not supported by exporters will be ignored. The exporter
will use default protocol in this case.

## Note
All exporters in this crate only support one protocol, thus choosing the protocol is a no-op at the moment.

<a id="op-965ea09ecbe24c13b7acd651"></a>
## with_timeout

`function` · `opentelemetry_otlp::exporter::WithExportConfig::with_timeout` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_timeout(self, timeout: Duration) -> Self
```

Source: `src/exporter/mod.rs:266`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set the timeout to the collector.

Note: Programmatically setting this will override any value set via the environment variable.
