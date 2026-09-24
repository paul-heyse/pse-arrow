# `opentelemetry_otlp::exporter::HasExportConfig`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.HasExportConfig.json).

<a id="op-3fee529c9e1df0847979bb0a"></a>
## HasExportConfig

`trait` · `opentelemetry_otlp::exporter::HasExportConfig` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait HasExportConfig
```

Source: `src/exporter/mod.rs:215`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Provide access to the [ExportConfig](../operations/opentelemetry_otlp.exporter.ExportConfig.md#op-cdfcaa8d369d011331bae7f7) field within the exporter builders.

<a id="op-168a9dcf791dd6d592f10167"></a>
## export_config

`function` · `opentelemetry_otlp::exporter::HasExportConfig::export_config` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export_config(&mut self) -> &mut ExportConfig
```

Source: `src/exporter/mod.rs:217`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Return a mutable reference to the [ExportConfig](../operations/opentelemetry_otlp.exporter.ExportConfig.md#op-cdfcaa8d369d011331bae7f7) within the exporter builders.
