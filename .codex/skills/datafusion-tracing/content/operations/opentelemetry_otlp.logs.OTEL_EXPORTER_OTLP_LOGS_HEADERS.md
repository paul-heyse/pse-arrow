# `opentelemetry_otlp::logs::OTEL_EXPORTER_OTLP_LOGS_HEADERS`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.logs.OTEL_EXPORTER_OTLP_LOGS_HEADERS.json).

<a id="op-e40cca43cd6eeff4262022b8"></a>
## OTEL_EXPORTER_OTLP_LOGS_HEADERS

`constant` · `opentelemetry_otlp::logs::OTEL_EXPORTER_OTLP_LOGS_HEADERS` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_LOGS_HEADERS: &str = "OTEL_EXPORTER_OTLP_LOGS_HEADERS"
```

Source: `src/logs.rs:32`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Key-value pairs to be used as headers associated with gRPC or HTTP requests
for sending logs.
Example: `k1=v1,k2=v2`
Note: this is only supported for HTTP.
