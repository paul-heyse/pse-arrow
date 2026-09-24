# Macro invocation grammar

13 macros, 49 accepted invocation forms. Read an arm as the literal shape you type: `options:` is a **keyword**, not a positional argument, and omitting it is a macro expansion error that names nothing useful.

## Two families

| Family | Macros | Wraps | Returns |
|---|---:|---|---|
| `instrument_with_*_spans!` | 6 | every `ExecutionPlan` node, at run time | a `PhysicalOptimizerRule` you register |
| `instrument_rules_with_*_spans!` | 6 | the analyzer, optimizer and physical-optimizer passes, at plan time | a `SessionState` |

The second takes `state:` as well as `options:`, which is the shape difference: one produces a rule you add to a builder, the other consumes and returns the state.

## Arms

### `instrument_rules_with_debug_spans!`

Instruments a `SessionState` with DEBUG-level tracing spans.

Level: **DEBUG**

- `instrument_rules_with_debug_spans!(target: $target:expr, options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_debug_spans!(options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_debug_spans!(target: $target:expr, options: $options:expr, state: $state:expr)`
- `instrument_rules_with_debug_spans!(options: $options:expr, state: $state:expr)`

### `instrument_rules_with_error_spans!`

Instruments a `SessionState` with ERROR-level tracing spans.

Level: **ERROR**

- `instrument_rules_with_error_spans!(target: $target:expr, options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_error_spans!(options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_error_spans!(target: $target:expr, options: $options:expr, state: $state:expr)`
- `instrument_rules_with_error_spans!(options: $options:expr, state: $state:expr)`

### `instrument_rules_with_info_spans!`

Instruments a `SessionState` with INFO-level tracing spans.

Level: **INFO**

- `instrument_rules_with_info_spans!(target: $target:expr, options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_info_spans!(options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_info_spans!(target: $target:expr, options: $options:expr, state: $state:expr)`
- `instrument_rules_with_info_spans!(options: $options:expr, state: $state:expr)`

### `instrument_rules_with_spans!`

Instruments a `SessionState` with tracing spans for all rule phases.

Level: **caller-supplied**

- `instrument_rules_with_spans!(target: $target:expr, $lvl:expr, options: $options:expr, state: $state:expr, $($fields:tt)*)`
- `instrument_rules_with_spans!(target: $target:expr, $lvl:expr, options: $options:expr, state: $state:expr)`
- `instrument_rules_with_spans!($lvl:expr, options: $options:expr, state: $state:expr, $($fields:tt)*)`
- `instrument_rules_with_spans!($lvl:expr, options: $options:expr, state: $state:expr)`

### `instrument_rules_with_trace_spans!`

Instruments a `SessionState` with TRACE-level tracing spans.

Level: **TRACE**

- `instrument_rules_with_trace_spans!(target: $target:expr, options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_trace_spans!(options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_trace_spans!(target: $target:expr, options: $options:expr, state: $state:expr)`
- `instrument_rules_with_trace_spans!(options: $options:expr, state: $state:expr)`

### `instrument_rules_with_warn_spans!`

Instruments a `SessionState` with WARN-level tracing spans.

Level: **WARN**

- `instrument_rules_with_warn_spans!(target: $target:expr, options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_warn_spans!(options: $options:expr, state: $state:expr, $($field:tt)*)`
- `instrument_rules_with_warn_spans!(target: $target:expr, options: $options:expr, state: $state:expr)`
- `instrument_rules_with_warn_spans!(options: $options:expr, state: $state:expr)`

### `instrument_with_debug_spans!`

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the debug level.

Level: **DEBUG**

- `instrument_with_debug_spans!(target: $target:expr, options: $options:expr, $($field:tt)*)`
- `instrument_with_debug_spans!(options: $options:expr, $($field:tt)*)`
- `instrument_with_debug_spans!(target: $target:expr, options: $options:expr)`
- `instrument_with_debug_spans!(options: $options:expr)`

### `instrument_with_error_spans!`

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the error level.

Level: **ERROR**

- `instrument_with_error_spans!(target: $target:expr, options: $options:expr, $($field:tt)*)`
- `instrument_with_error_spans!(options: $options:expr, $($field:tt)*)`
- `instrument_with_error_spans!(target: $target:expr, options: $options:expr)`
- `instrument_with_error_spans!(options: $options:expr)`

### `instrument_with_info_spans!`

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the info level.

Level: **INFO**

- `instrument_with_info_spans!(target: $target:expr, options: $options:expr, $($field:tt)*)`
- `instrument_with_info_spans!(options: $options:expr, $($field:tt)*)`
- `instrument_with_info_spans!(target: $target:expr, options: $options:expr)`
- `instrument_with_info_spans!(options: $options:expr)`

### `instrument_with_spans!`

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan`.

Level: **caller-supplied**

- `instrument_with_spans!(target: $target:expr, $lvl:expr, options: $options:expr, $($fields:tt)*)`
- `instrument_with_spans!(target: $target:expr, $lvl:expr, options: $options:expr)`
- `instrument_with_spans!($lvl:expr, options: $options:expr, $($fields:tt)*)`
- `instrument_with_spans!($lvl:expr, options: $options:expr)`

### `instrument_with_trace_spans!`

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the trace level.

Level: **TRACE**

- `instrument_with_trace_spans!(target: $target:expr, options: $options:expr, $($field:tt)*)`
- `instrument_with_trace_spans!(options: $options:expr, $($field:tt)*)`
- `instrument_with_trace_spans!(target: $target:expr, options: $options:expr)`
- `instrument_with_trace_spans!(options: $options:expr)`

### `instrument_with_warn_spans!`

Constructs a new instrumentation `PhysicalOptimizerRule` for a DataFusion `ExecutionPlan` at the warn level.

Level: **WARN**

- `instrument_with_warn_spans!(target: $target:expr, options: $options:expr, $($field:tt)*)`
- `instrument_with_warn_spans!(options: $options:expr, $($field:tt)*)`
- `instrument_with_warn_spans!(target: $target:expr, options: $options:expr)`
- `instrument_with_warn_spans!(options: $options:expr)`

### `span_at_level!`

Creates a span at the specified tracing level with the given name and fields.

Level: **caller-supplied**

- `span_at_level!($level:expr, $name:expr, $($field:tt)*)`

## The custom-field coupling

`InstrumentationOptions::builder().add_custom_field("env", "production")` sets a value. It does **not** create a field. A `tracing` span's field set is fixed when the span is created, so the macro must declare the key too:

```rust
let rule = instrument_with_info_spans!(
    options: options,
    env = field::Empty,      // declared here
    region = field::Empty,
);
```

Upstream says the same thing in `corpus/examples/integration-utils/src/lib.rs`, in a comment beside those two lines: *custom fields keys must be defined at compile time*. Declare without setting and the field is absent from the span; set without declaring and the value goes nowhere. Neither case is an error at compile time or at run time.

## What an arm expands to is not in this index

rustdoc emits no macro bodies, at any format version -- every arm above ends `=> { ... }` in the source document too. The expansion is in `corpus/source/datafusion-tracing/exec_instrument_macros.rs` and `corpus/source/datafusion-tracing/rule_instrumentation_macros.rs`, and what it does at run time is in `index/behaviors.tsv`. Do not infer it from the arm.
