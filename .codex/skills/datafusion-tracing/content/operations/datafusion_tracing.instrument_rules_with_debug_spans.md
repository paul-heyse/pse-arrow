# `datafusion_tracing::instrument_rules_with_debug_spans`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrument_rules_with_debug_spans.json).

<a id="op-868a352ae499eb7c834e15dd"></a>
## instrument_rules_with_debug_spans

`macro` · `datafusion_tracing::instrument_rules_with_debug_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! instrument_rules_with_debug_spans
```

Source: `src/rule_instrumentation_macros.rs:158`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Instruments a `SessionState` with DEBUG-level tracing spans.

This is a convenience wrapper around [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-1c4efa69bf309db05ebcc5cf) that automatically
sets the tracing level to `DEBUG`.

See [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-1c4efa69bf309db05ebcc5cf) for detailed documentation.

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_debug_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full();
let session_state = instrument_rules_with_debug_spans!(
    options: options,
    state: session_state
);
```

<a id="op-8638e5773147f8ad52b46897"></a>
## instrument_rules_with_debug_spans

`macro` · `datafusion_tracing::instrument_rules_with_debug_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
macro_rules! instrument_rules_with_debug_spans
```

Source: `src/rule_instrumentation_macros.rs:158`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments a `SessionState` with DEBUG-level tracing spans.

This is a convenience wrapper around [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-dd96c18d1fb324be3624c5ff) that automatically
sets the tracing level to `DEBUG`.

See [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-dd96c18d1fb324be3624c5ff) for detailed documentation.

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_debug_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full();
let session_state = instrument_rules_with_debug_spans!(
    options: options,
    state: session_state
);
```
