# `opentelemetry_otlp::span::OTEL_EXPORTER_OTLP_TRACES_TIMEOUT`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.span.OTEL_EXPORTER_OTLP_TRACES_TIMEOUT.json).

<a id="op-50b0b8a65c93f8bed1d39dc2"></a>
## OTEL_EXPORTER_OTLP_TRACES_TIMEOUT

`constant` · `opentelemetry_otlp::span::OTEL_EXPORTER_OTLP_TRACES_TIMEOUT` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_TRACES_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT"
```

Source: `src/span.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Max waiting time for the backend to process each spans batch, defaults to 10s.
