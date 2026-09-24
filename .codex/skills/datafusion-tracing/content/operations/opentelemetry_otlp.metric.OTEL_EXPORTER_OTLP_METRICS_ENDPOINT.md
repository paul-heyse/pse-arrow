# `opentelemetry_otlp::metric::OTEL_EXPORTER_OTLP_METRICS_ENDPOINT`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.metric.OTEL_EXPORTER_OTLP_METRICS_ENDPOINT.json).

<a id="op-8edbc60dca511a4d006992b5"></a>
## OTEL_EXPORTER_OTLP_METRICS_ENDPOINT

`constant` · `opentelemetry_otlp::metric::OTEL_EXPORTER_OTLP_METRICS_ENDPOINT` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_METRICS_ENDPOINT: &str = "OTEL_EXPORTER_OTLP_METRICS_ENDPOINT"
```

Source: `src/metric.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Target to which the exporter is going to send metrics, defaults to https://localhost:4317/v1/metrics.
Learn about the relationship between this constant and default/spans/logs at
<https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/protocol/exporter.md#endpoint-urls-for-otlphttp>
