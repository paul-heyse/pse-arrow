# `datafusion_tracing::instrument_rules_with_spans`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrument_rules_with_spans.json).

<a id="op-1c4efa69bf309db05ebcc5cf"></a>
## instrument_rules_with_spans

`macro` · `datafusion_tracing::instrument_rules_with_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! instrument_rules_with_spans
```

Source: `src/rule_instrumentation_macros.rs:59`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Instruments a `SessionState` with tracing spans for all rule phases.

This macro wraps analyzer, optimizer, and physical optimizer rules with tracing spans
at the specified level. Rule spans are grouped under phase spans
("analyze_logical_plan", "optimize_logical_plan", "optimize_physical_plan").

When physical optimizer instrumentation is enabled, the query planner is also
automatically instrumented to trace physical plan creation.

# Syntax

```ignore
instrument_rules_with_spans!(
    $level,
    options: $options,
    state: $session_state,
    $($fields)*
)
```

# Arguments

* `$level` - The tracing level (e.g., `tracing::Level::INFO`)
* `options` - A `RuleInstrumentationOptions` instance
* `state` - A `SessionState` to instrument
* `$fields` - Optional additional span fields

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_info_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full().with_plan_diff();
let session_state = instrument_rules_with_info_spans!(
    options: options,
    state: session_state
);
```

<a id="op-dd96c18d1fb324be3624c5ff"></a>
## instrument_rules_with_spans

`macro` · `datafusion_tracing::instrument_rules_with_spans` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
macro_rules! instrument_rules_with_spans
```

Source: `src/rule_instrumentation_macros.rs:59`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments a `SessionState` with tracing spans for all rule phases.

This macro wraps analyzer, optimizer, and physical optimizer rules with tracing spans
at the specified level. Rule spans are grouped under phase spans
("analyze_logical_plan", "optimize_logical_plan", "optimize_physical_plan").

When physical optimizer instrumentation is enabled, the query planner is also
automatically instrumented to trace physical plan creation.

# Syntax

```ignore
instrument_rules_with_spans!(
    $level,
    options: $options,
    state: $session_state,
    $($fields)*
)
```

# Arguments

* `$level` - The tracing level (e.g., `tracing::Level::INFO`)
* `options` - A `RuleInstrumentationOptions` instance
* `state` - A `SessionState` to instrument
* `$fields` - Optional additional span fields

# Example

```rust,ignore
use datafusion_tracing::{instrument_rules_with_info_spans, RuleInstrumentationOptions};

let options = RuleInstrumentationOptions::full().with_plan_diff();
let session_state = instrument_rules_with_info_spans!(
    options: options,
    state: session_state
);
```
