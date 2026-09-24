# `opentelemetry::otel_info`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.otel_info.json).

<a id="op-d872f52f096df3fb8e3d9cf3"></a>
## otel_info

`macro` · `opentelemetry::otel_info` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! otel_info
```

Source: `src/global/internal_logging.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).


**Note**: These macros (`otel_info!`, `otel_warn!`, `otel_debug!`, and `otel_error!`) are intended to be used
**internally within OpenTelemetry code** or for **custom exporters, processors and other plugins**. They are not designed
for general application logging and should not be used for that purpose.

When running tests with `--nocapture`, these macros will print their output to stdout. This is useful for debugging
test failures and understanding the flow of operations during testing.

Macro for logging informational messages in OpenTelemetry.

# Fields:
- `name`: The operation or action being logged.
- Additional optional key-value pairs can be passed as attributes.

# Example:
```rust
use opentelemetry::otel_info;
otel_info!(name: "sdk_start", version = "1.0.0", schema_url = "http://example.com");
```

