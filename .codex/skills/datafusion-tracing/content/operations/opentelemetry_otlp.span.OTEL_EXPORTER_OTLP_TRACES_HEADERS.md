# `opentelemetry_otlp::span::OTEL_EXPORTER_OTLP_TRACES_HEADERS`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.span.OTEL_EXPORTER_OTLP_TRACES_HEADERS.json).

<a id="op-7370a9579750ec3b57e9275c"></a>
## OTEL_EXPORTER_OTLP_TRACES_HEADERS

`constant` · `opentelemetry_otlp::span::OTEL_EXPORTER_OTLP_TRACES_HEADERS` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_TRACES_HEADERS: &str = "OTEL_EXPORTER_OTLP_TRACES_HEADERS"
```

Source: `src/span.rs:37`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Key-value pairs to be used as headers associated with gRPC or HTTP requests
for sending spans.
Example: `k1=v1,k2=v2`
Note: this is only supported for HTTP.
