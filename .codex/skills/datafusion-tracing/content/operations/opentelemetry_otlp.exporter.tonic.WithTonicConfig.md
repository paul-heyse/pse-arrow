# `opentelemetry_otlp::exporter::tonic::WithTonicConfig`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.tonic.WithTonicConfig.json).

<a id="op-112358fcec71a29e158eaf2a"></a>
## WithTonicConfig

`trait` · `opentelemetry_otlp::exporter::tonic::WithTonicConfig` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait WithTonicConfig
```

Source: `src/exporter/tonic/mod.rs:370`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Expose methods to override [TonicConfig](../operations/opentelemetry_otlp.exporter.tonic.TonicConfig.md#op-a6cb94179d82f7fc38273c80).

This trait will be implemented for every struct that implemented [`HasTonicConfig`](../operations/opentelemetry_otlp.exporter.tonic.HasTonicConfig.md#op-5549afd81c403aefae8d1efa) trait.

## Examples
```
# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
# {
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
let exporter_builder = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .with_compression(opentelemetry_otlp::Compression::Gzip);
# }
```

<a id="op-8798185098b579081a338f76"></a>
## with_channel

`function` · `opentelemetry_otlp::exporter::tonic::WithTonicConfig::with_channel` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_channel(self, channel: tonic::transport::Channel) -> Self
```

Source: `src/exporter/tonic/mod.rs:413`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Use `channel` as tonic's transport channel.
this will override tls config and should only be used
when working with non-HTTP transports.

Users MUST make sure the [`ExportConfig::timeout`](../operations/opentelemetry_otlp.exporter.ExportConfig.md#op-3c9ce635c145057f395c35e9) is
the same as the channel's timeout.

<a id="op-d91dc5967e804ec995f5b198"></a>
## with_compression

`function` · `opentelemetry_otlp::exporter::tonic::WithTonicConfig::with_compression` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_compression(self, compression: Compression) -> Self
```

Source: `src/exporter/tonic/mod.rs:405`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set the compression algorithm to use when communicating with the collector.

<a id="op-58152aac58a1c14ff7417476"></a>
## with_interceptor

`function` · `opentelemetry_otlp::exporter::tonic::WithTonicConfig::with_interceptor` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_interceptor<I>(self, interceptor: I) -> Self where I: tonic::service::Interceptor + Clone + Send + Sync + 'static
```

Source: `src/exporter/tonic/mod.rs:475`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Use a custom `interceptor` to modify each outbound request.
This can be used to modify the gRPC metadata, for example
to inject auth tokens.

**Note**: Calling this method multiple times will replace the previous
interceptor. If you need multiple interceptors, chain them together
before passing to this method.

# Examples

## Single interceptor
```no_run
# #[cfg(feature = "grpc-tonic")]
# {
use tonic::{Request, Status};
use opentelemetry_otlp::WithTonicConfig;

fn auth_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    req.metadata_mut().insert("authorization", "Bearer token".parse().unwrap());
    Ok(req)
}

let exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .with_interceptor(auth_interceptor)
    .build()?;
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Multiple interceptors (chaining)
```no_run
# #[cfg(feature = "grpc-tonic")]
# {
use tonic::{Request, Status};
use opentelemetry_otlp::WithTonicConfig;

fn auth_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    req.metadata_mut().insert("authorization", "Bearer token".parse().unwrap());
    Ok(req)
}

fn logging_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    println!("Sending gRPC request with metadata: {:?}", req.metadata());
    Ok(req)
}

// Chain interceptors by wrapping them
fn combined_interceptor(req: Request<()>) -> Result<Request<()>, Status> {
    let req = logging_interceptor(req)?;
    auth_interceptor(req)
}

let exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .with_interceptor(combined_interceptor)
    .build()?;
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

<a id="op-a2faee71e1150dce52a6de3d"></a>
## with_metadata

`function` · `opentelemetry_otlp::exporter::tonic::WithTonicConfig::with_metadata` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_metadata(self, metadata: MetadataMap) -> Self
```

Source: `src/exporter/tonic/mod.rs:402`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set custom metadata entries to send to the collector.

**Note**: This method is additive - calling it multiple times will merge
the metadata entries. If the same key is provided in multiple calls,
the last value will override previous ones.

# Example
```no_run
# #[cfg(feature = "grpc-tonic")]
# {
use tonic::metadata::MetadataMap;
use opentelemetry_otlp::WithTonicConfig;

let mut metadata1 = MetadataMap::new();
metadata1.insert("key1", "value1".parse().unwrap());

let mut metadata2 = MetadataMap::new();
metadata2.insert("key2", "value2".parse().unwrap());

let exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .with_metadata(metadata1)  // Adds key1=value1
    .with_metadata(metadata2)  // Adds key2=value2 (both are present)
    .build()?;
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

<a id="op-9e0397dde5ac189cae9e9e74"></a>
## with_tls_config

`function` · `opentelemetry_otlp::exporter::tonic::WithTonicConfig::with_tls_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_tls_config(self, tls_config: ClientTlsConfig) -> Self
```

Source: `src/exporter/tonic/mod.rs:373`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Set the TLS settings for the collector endpoint.
