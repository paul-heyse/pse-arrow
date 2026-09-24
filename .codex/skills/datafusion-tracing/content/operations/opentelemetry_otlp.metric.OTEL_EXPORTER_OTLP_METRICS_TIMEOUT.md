# `opentelemetry_otlp::metric::OTEL_EXPORTER_OTLP_METRICS_TIMEOUT`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.metric.OTEL_EXPORTER_OTLP_METRICS_TIMEOUT.json).

<a id="op-23b2c2de0ba04b13f7374503"></a>
## OTEL_EXPORTER_OTLP_METRICS_TIMEOUT

`constant` · `opentelemetry_otlp::metric::OTEL_EXPORTER_OTLP_METRICS_TIMEOUT` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_METRICS_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_METRICS_TIMEOUT"
```

Source: `src/metric.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Max waiting time for the backend to process each metrics batch, defaults to 10s.
