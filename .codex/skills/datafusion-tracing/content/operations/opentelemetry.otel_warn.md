# `opentelemetry::otel_warn`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.otel_warn.json).

<a id="op-e5ae390d4dad229053d8f802"></a>
## otel_warn

`macro` · `opentelemetry::otel_warn` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! otel_warn
```

Source: `src/global/internal_logging.rs:76`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Macro for logging warning messages in OpenTelemetry.

# Fields:
- `name`: The operation or action being logged.
- Additional optional key-value pairs can be passed as attributes.

# Example:
```rust
use opentelemetry::otel_warn;
otel_warn!(name: "export_warning", error_code = 404, version = "1.0.0");
```
