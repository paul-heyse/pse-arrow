# `opentelemetry_otlp::metric::OTEL_EXPORTER_OTLP_METRICS_HEADERS`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.metric.OTEL_EXPORTER_OTLP_METRICS_HEADERS.json).

<a id="op-f39c53616e6b85394cf01c98"></a>
## OTEL_EXPORTER_OTLP_METRICS_HEADERS

`constant` · `opentelemetry_otlp::metric::OTEL_EXPORTER_OTLP_METRICS_HEADERS` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_METRICS_HEADERS: &str = "OTEL_EXPORTER_OTLP_METRICS_HEADERS"
```

Source: `src/metric.rs:38`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Key-value pairs to be used as headers associated with gRPC or HTTP requests
for sending metrics.
Example: `k1=v1,k2=v2`
Note: this is only supported for HTTP.
