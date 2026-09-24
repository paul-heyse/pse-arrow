# `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_ENDPOINT`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.OTEL_EXPORTER_OTLP_ENDPOINT.json).

<a id="op-c5ab1ad1fe1f2a78e1277102"></a>
## OTEL_EXPORTER_OTLP_ENDPOINT

`constant` · `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_ENDPOINT` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_ENDPOINT"
```

Source: `src/exporter/mod.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Target to which the exporter is going to send signals, defaults to https://localhost:4317.
Learn about the relationship between this constant and metrics/spans/logs at
<https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
