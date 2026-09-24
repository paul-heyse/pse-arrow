# `datafusion_tracing::instrument_with_warn_spans`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrument_with_warn_spans.json).

<a id="op-098dfcf75cd1b23106666c64"></a>
## instrument_with_warn_spans

`macro` · `datafusion_tracing::instrument_with_warn_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! instrument_with_warn_spans
```

Source: `src/exec_instrument_macros.rs:425`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the warn level.

This macro automatically sets the tracing level to `WARN` and is a convenience wrapper around
[`instrument_with_spans!`] with the appropriate log level preset.

See [`instrument_with_spans!`] for details on instrumentation options and span fields.

# Examples

Basic usage with default options:
```rust
# use datafusion_tracing::{instrument_with_warn_spans, InstrumentationOptions};
let instrument_rule = instrument_with_warn_spans!(options: InstrumentationOptions::default());
```

Adding custom fields:
```rust
# use datafusion_tracing::{instrument_with_warn_spans, InstrumentationOptions};
# use tracing::field;
# use std::collections::HashMap;
let custom_fields = HashMap::from([
    ("custom.key1".to_string(), "value1".to_string()),
]);
let options = InstrumentationOptions {
   custom_fields,
   ..Default::default()
};
let instrument_rule = instrument_with_warn_spans!(
    options: options,
    datafusion.additional_info = "some info",
    custom.key1 = field::Empty,
);
```

[tracing_warn_span]: https://docs.rs/tracing/latest/tracing/macro.warn_span.html
[`warn_span!`]: https://docs.rs/tracing/latest/tracing/macro.warn_span.html
[`instrument_with_spans!`]: crate::instrument_with_spans!

<a id="op-4eb8276a60f2a24e181bb5de"></a>
## instrument_with_warn_spans

`macro` · `datafusion_tracing::instrument_with_warn_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
macro_rules! instrument_with_warn_spans
```

Source: `src/exec_instrument_macros.rs:425`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the warn level.

This macro automatically sets the tracing level to `WARN` and is a convenience wrapper around
[`instrument_with_spans!`] with the appropriate log level preset.

See [`instrument_with_spans!`] for details on instrumentation options and span fields.

# Examples

Basic usage with default options:
```rust
# use datafusion_tracing::{instrument_with_warn_spans, InstrumentationOptions};
let instrument_rule = instrument_with_warn_spans!(options: InstrumentationOptions::default());
```

Adding custom fields:
```rust
# use datafusion_tracing::{instrument_with_warn_spans, InstrumentationOptions};
# use tracing::field;
# use std::collections::HashMap;
let custom_fields = HashMap::from([
    ("custom.key1".to_string(), "value1".to_string()),
]);
let options = InstrumentationOptions {
   custom_fields,
   ..Default::default()
};
let instrument_rule = instrument_with_warn_spans!(
    options: options,
    datafusion.additional_info = "some info",
    custom.key1 = field::Empty,
);
```

[tracing_warn_span]: https://docs.rs/tracing/latest/tracing/macro.warn_span.html
[`warn_span!`]: https://docs.rs/tracing/latest/tracing/macro.warn_span.html
[`instrument_with_spans!`]: crate::instrument_with_spans!
