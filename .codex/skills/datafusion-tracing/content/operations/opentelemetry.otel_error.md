# `opentelemetry::otel_error`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.otel_error.json).

<a id="op-e0cef892bef09d5ce13078ac"></a>
## otel_error

`macro` · `opentelemetry::otel_error` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! otel_error
```

Source: `src/global/internal_logging.rs:185`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Macro for logging error messages in OpenTelemetry.

# Fields:
- `name`: The operation or action being logged.
- Additional optional key-value pairs can be passed as attributes.

# Example:
```rust
use opentelemetry::otel_error;
otel_error!(name: "export_failure", error_code = 500, version = "1.0.0");
```
