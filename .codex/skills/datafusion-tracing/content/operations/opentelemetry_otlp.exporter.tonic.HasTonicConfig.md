# `opentelemetry_otlp::exporter::tonic::HasTonicConfig`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.tonic.HasTonicConfig.json).

<a id="op-5549afd81c403aefae8d1efa"></a>
## HasTonicConfig

`trait` · `opentelemetry_otlp::exporter::tonic::HasTonicConfig` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait HasTonicConfig
```

Source: `src/exporter/tonic/mod.rs:344`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Expose interface for modifying [TonicConfig](../operations/opentelemetry_otlp.exporter.tonic.TonicConfig.md#op-a6cb94179d82f7fc38273c80) fields within the exporter builders.

<a id="op-0d665208f0a054ad0f5386f8"></a>
## tonic_config

`function` · `opentelemetry_otlp::exporter::tonic::HasTonicConfig::tonic_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tonic_config(&mut self) -> &mut TonicConfig
```

Source: `src/exporter/tonic/mod.rs:346`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Return a mutable reference to the export config within the exporter builders.
