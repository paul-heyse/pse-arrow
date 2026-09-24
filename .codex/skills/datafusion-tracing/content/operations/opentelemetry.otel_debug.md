# `opentelemetry::otel_debug`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.otel_debug.json).

<a id="op-b8b8f21ffbff842f043e26a8"></a>
## otel_debug

`macro` · `opentelemetry::otel_debug` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! otel_debug
```

Source: `src/global/internal_logging.rs:134`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Macro for logging debug messages in OpenTelemetry.

# Fields:
- `name`: The operation or action being logged.
- Additional optional key-value pairs can be passed as attributes.

# Example:
```rust
use opentelemetry::otel_debug;
otel_debug!(name: "debug_operation", debug_level = "high", version = "1.0.0");
```
