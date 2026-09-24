# `opentelemetry_otlp::span::OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.span.OTEL_EXPORTER_OTLP_TRACES_ENDPOINT.json).

<a id="op-24a3e002fb9db13b01814281"></a>
## OTEL_EXPORTER_OTLP_TRACES_ENDPOINT

`constant` · `opentelemetry_otlp::span::OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_TRACES_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT"
```

Source: `src/span.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Target to which the exporter is going to send spans, defaults to https://localhost:4317/v1/traces.
Learn about the relationship between this constant and default/metrics/logs at
<https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
