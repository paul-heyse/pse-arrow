# `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_PROTOCOL`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.OTEL_EXPORTER_OTLP_PROTOCOL.json).

<a id="op-211055e93d6aef1d18a0dac8"></a>
## OTEL_EXPORTER_OTLP_PROTOCOL

`constant` · `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_PROTOCOL` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_PROTOCOL: &str = "OTEL_EXPORTER_OTLP_PROTOCOL"
```

Source: `src/exporter/mod.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Protocol the exporter will use. Either `http/protobuf` or `grpc`.
