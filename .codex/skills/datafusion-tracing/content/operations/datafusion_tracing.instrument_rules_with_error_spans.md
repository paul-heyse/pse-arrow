# `datafusion_tracing::instrument_rules_with_error_spans`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrument_rules_with_error_spans.json).

<a id="op-6a4984b9c0f01ba6838dd215"></a>
## instrument_rules_with_error_spans

`macro` · `datafusion_tracing::instrument_rules_with_error_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! instrument_rules_with_error_spans
```

Source: `src/rule_instrumentation_macros.rs:278`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Instruments a `SessionState` with ERROR-level tracing spans.

This is a convenience wrapper around [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-1c4efa69bf309db05ebcc5cf) that automatically
sets the tracing level to `ERROR`.

See [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-1c4efa69bf309db05ebcc5cf) for detailed documentation.

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_error_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full();
let session_state = instrument_rules_with_error_spans!(
    options: options,
    state: session_state
);
```

<a id="op-4d1acc622d71c17d6c907822"></a>
## instrument_rules_with_error_spans

`macro` · `datafusion_tracing::instrument_rules_with_error_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
macro_rules! instrument_rules_with_error_spans
```

Source: `src/rule_instrumentation_macros.rs:278`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments a `SessionState` with ERROR-level tracing spans.

This is a convenience wrapper around [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-dd96c18d1fb324be3624c5ff) that automatically
sets the tracing level to `ERROR`.

See [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-dd96c18d1fb324be3624c5ff) for detailed documentation.

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_error_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full();
let session_state = instrument_rules_with_error_spans!(
    options: options,
    state: session_state
);
```
