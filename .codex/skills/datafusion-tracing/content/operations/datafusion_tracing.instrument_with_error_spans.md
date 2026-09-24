# `datafusion_tracing::instrument_with_error_spans`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrument_with_error_spans.json).

<a id="op-72d4757d5a2d42d9825db306"></a>
## instrument_with_error_spans

`macro` · `datafusion_tracing::instrument_with_error_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! instrument_with_error_spans
```

Source: `src/exec_instrument_macros.rs:483`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the error level.

This macro automatically sets the tracing level to `ERROR` and is a convenience wrapper around
[`instrument_with_spans!`] with the appropriate log level preset.

See [`instrument_with_spans!`] for details on instrumentation options and span fields.

# Examples

Basic usage with default options:
```rust
# use datafusion_tracing::{instrument_with_error_spans, InstrumentationOptions};
let instrument_rule = instrument_with_error_spans!(options: InstrumentationOptions::default());
```

Adding custom fields:
```rust
# use datafusion_tracing::{instrument_with_error_spans, InstrumentationOptions};
# use tracing::field;
# use std::collections::HashMap;
let custom_fields = HashMap::from([
    ("custom.key1".to_string(), "value1".to_string()),
]);
let options = InstrumentationOptions {
   custom_fields,
   ..Default::default()
};
let instrument_rule = instrument_with_error_spans!(
    options: options,
    datafusion.additional_info = "some info",
    custom.key1 = field::Empty,
);
```

[tracing_error_span]: https://docs.rs/tracing/latest/tracing/macro.error_span.html
[`error_span!`]: https://docs.rs/tracing/latest/tracing/macro.error_span.html
[`instrument_with_spans!`]: crate::instrument_with_spans!

<a id="op-50619facfba6088ed6e75ce1"></a>
## instrument_with_error_spans

`macro` · `datafusion_tracing::instrument_with_error_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
macro_rules! instrument_with_error_spans
```

Source: `src/exec_instrument_macros.rs:483`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the error level.

This macro automatically sets the tracing level to `ERROR` and is a convenience wrapper around
[`instrument_with_spans!`] with the appropriate log level preset.

See [`instrument_with_spans!`] for details on instrumentation options and span fields.

# Examples

Basic usage with default options:
```rust
# use datafusion_tracing::{instrument_with_error_spans, InstrumentationOptions};
let instrument_rule = instrument_with_error_spans!(options: InstrumentationOptions::default());
```

Adding custom fields:
```rust
# use datafusion_tracing::{instrument_with_error_spans, InstrumentationOptions};
# use tracing::field;
# use std::collections::HashMap;
let custom_fields = HashMap::from([
    ("custom.key1".to_string(), "value1".to_string()),
]);
let options = InstrumentationOptions {
   custom_fields,
   ..Default::default()
};
let instrument_rule = instrument_with_error_spans!(
    options: options,
    datafusion.additional_info = "some info",
    custom.key1 = field::Empty,
);
```

[tracing_error_span]: https://docs.rs/tracing/latest/tracing/macro.error_span.html
[`error_span!`]: https://docs.rs/tracing/latest/tracing/macro.error_span.html
[`instrument_with_spans!`]: crate::instrument_with_spans!
