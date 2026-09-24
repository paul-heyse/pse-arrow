# `datafusion_tracing::instrument_rules_with_trace_spans`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrument_rules_with_trace_spans.json).

<a id="op-e4108b3f6bfa17d523d847cb"></a>
## instrument_rules_with_trace_spans

`macro` · `datafusion_tracing::instrument_rules_with_trace_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! instrument_rules_with_trace_spans
```

Source: `src/rule_instrumentation_macros.rs:118`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Instruments a `SessionState` with TRACE-level tracing spans.

This is a convenience wrapper around [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-1c4efa69bf309db05ebcc5cf) that automatically
sets the tracing level to `TRACE`.

See [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-1c4efa69bf309db05ebcc5cf) for detailed documentation.

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_trace_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full();
let session_state = instrument_rules_with_trace_spans!(
    options: options,
    state: session_state
);
```

<a id="op-27790040199c6c6422f91fcf"></a>
## instrument_rules_with_trace_spans

`macro` · `datafusion_tracing::instrument_rules_with_trace_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
macro_rules! instrument_rules_with_trace_spans
```

Source: `src/rule_instrumentation_macros.rs:118`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments a `SessionState` with TRACE-level tracing spans.

This is a convenience wrapper around [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-dd96c18d1fb324be3624c5ff) that automatically
sets the tracing level to `TRACE`.

See [`instrument_rules_with_spans!`](../operations/datafusion_tracing.instrument_rules_with_spans.md#op-dd96c18d1fb324be3624c5ff) for detailed documentation.

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_trace_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full();
let session_state = instrument_rules_with_trace_spans!(
    options: options,
    state: session_state
);
```
