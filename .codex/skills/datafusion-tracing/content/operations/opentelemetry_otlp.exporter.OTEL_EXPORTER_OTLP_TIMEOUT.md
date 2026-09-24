# `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_TIMEOUT`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.OTEL_EXPORTER_OTLP_TIMEOUT.json).

<a id="op-ae15a1deec7b21ea3cbf609f"></a>
## OTEL_EXPORTER_OTLP_TIMEOUT

`constant` · `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_TIMEOUT` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_TIMEOUT: &str = "OTEL_EXPORTER_OTLP_TIMEOUT"
```

Source: `src/exporter/mod.rs:54`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Max waiting time for the backend to process each signal batch, defaults to 10 seconds.
