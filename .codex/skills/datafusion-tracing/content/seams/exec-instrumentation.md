# Instrumenting execution plans

instrument_with_*_spans! returns a physical optimizer rule that recursively wraps the plan it receives. Placing it after the relevant optimizer rewrites observes their resulting nodes; a later rule can preserve, add or replace wrappers. Planning instrumentation is separate.

## Entry points

| Item | Visibility | Methods | Reached via |
|---|---|---:|---|
| [`InstrumentationOptions`](../api/datafusion_tracing.options.md) | supported | 4 | — |
| [`InstrumentationOptionsBuilder`](../api/datafusion_tracing.options.md) | reachable-undocumented | 7 | InstrumentationOptions::builder() |

## Macros

| Macro | Accepted forms |
|---|---|
| `instrument_with_spans!` | `target: $target:expr, $lvl:expr, options: $options:expr, $($fields:tt)*`<br>`target: $target:expr, $lvl:expr, options: $options:expr`<br>`$lvl:expr, options: $options:expr, $($fields:tt)*`<br>`$lvl:expr, options: $options:expr` |
| `instrument_with_info_spans!` | `target: $target:expr, options: $options:expr, $($field:tt)*`<br>`options: $options:expr, $($field:tt)*`<br>`target: $target:expr, options: $options:expr`<br>`options: $options:expr` |
| `instrument_with_debug_spans!` | `target: $target:expr, options: $options:expr, $($field:tt)*`<br>`options: $options:expr, $($field:tt)*`<br>`target: $target:expr, options: $options:expr`<br>`options: $options:expr` |

Full table: [`catalogs/macros.md`](../catalogs/macros.md)

## Spans it emits

| Span | Target | Level | Fields | Evidence |
|---|---|---|---:|---|
| `InstrumentedExec` | `integration_utils` | INFO | 49 | recorded |

## What this seam cannot tell you

- The wrapped node's type, from this index. `InstrumentedExec` is private by design; `datafusion.node` on the span carries the display of the inner plan, and `ExecutionPlan` introspection still works through the wrapper.
- What a macro arm expands to. rustdoc emits no macro bodies; read `corpus/source/datafusion-tracing/exec_instrument_macros.rs`.
- Which DataFusion nodes exist, or what they cost. That is DataFusion's surface, not this one's.

## Read next

- [`catalogs/options.md`](../catalogs/options.md)
- [`catalogs/macros.md`](../catalogs/macros.md)
- [`catalogs/spans.md`](../catalogs/spans.md)
- [`content/corpus/source/datafusion-tracing/exec_instrument_rule.rs`](../corpus/source/datafusion-tracing/exec_instrument_rule.rs) — upstream, verbatim
- [`content/corpus/source/datafusion-tracing/instrumented_exec.rs`](../corpus/source/datafusion-tracing/instrumented_exec.rs) — upstream, verbatim
