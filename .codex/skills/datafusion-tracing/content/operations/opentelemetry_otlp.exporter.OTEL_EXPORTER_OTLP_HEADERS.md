# `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_HEADERS`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.OTEL_EXPORTER_OTLP_HEADERS.json).

<a id="op-f01a3aff9c55387f7cf20ba9"></a>
## OTEL_EXPORTER_OTLP_HEADERS

`constant` · `opentelemetry_otlp::exporter::OTEL_EXPORTER_OTLP_HEADERS` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const OTEL_EXPORTER_OTLP_HEADERS: &str = "OTEL_EXPORTER_OTLP_HEADERS"
```

Source: `src/exporter/mod.rs:26`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Key-value pairs to be used as headers associated with gRPC or HTTP requests
Example: `k1=v1,k2=v2`
Note: as of now, this is only supported for HTTP requests.
