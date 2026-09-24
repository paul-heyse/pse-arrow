# Instrumenting planning phases

instrument_rules_with_*_spans! accepts state and returns an instrumented SessionState. Phase-only, full and selected-phase configurations differ in detail. The amount of trace output depends on the query and rule set.

## Entry points

| Item | Visibility | Methods | Reached via |
|---|---|---:|---|
| [`RuleInstrumentationOptions`](../api/datafusion_tracing.rule_options.md) | supported | 7 | — |
| [`RuleInstrumentationOptionsBuilder`](../api/datafusion_tracing.rule_options.md) | reachable-undocumented | 11 | RuleInstrumentationOptions::builder() |

## Macros

| Macro | Accepted forms |
|---|---|
| `instrument_rules_with_spans!` | `target: $target:expr, $lvl:expr, options: $options:expr, state: $state:expr, $($fields:tt)*`<br>`target: $target:expr, $lvl:expr, options: $options:expr, state: $state:expr`<br>`$lvl:expr, options: $options:expr, state: $state:expr, $($fields:tt)*`<br>`$lvl:expr, options: $options:expr, state: $state:expr` |
| `instrument_rules_with_info_spans!` | `target: $target:expr, options: $options:expr, state: $state:expr, $($field:tt)*`<br>`options: $options:expr, state: $state:expr, $($field:tt)*`<br>`target: $target:expr, options: $options:expr, state: $state:expr`<br>`options: $options:expr, state: $state:expr` |

Full table: [`catalogs/macros.md`](../catalogs/macros.md)

## Spans it emits

| Span | Target | Level | Fields | Evidence |
|---|---|---|---:|---|
| `Phase` | `integration_utils` | INFO | 4 | recorded |
| `Rule` | `integration_utils` | INFO | 1 | recorded |
| `create_physical_plan` | `datafusion_tracing::planner` | INFO | 2 | recorded |

## What this seam cannot tell you

- A documented way to instrument one phase only. `full()` and `phase_only()` are the whole documented surface; the ten per-phase selectors are on the undocumented builder.
- A complete semantic explanation of a rule from its name alone. With plan-diff recording enabled, the source records datafusion.plan_diff on applicable rule spans; otherwise the snapshot field set is narrower.
- Plan diffs, by default. `with_plan_diff()` is opt-in, and upstream's own integration tests disable it because the output is not deterministic.

## Read next

- [`catalogs/options.md`](../catalogs/options.md)
- [`catalogs/macros.md`](../catalogs/macros.md)
- [`content/corpus/source/datafusion-tracing/rule_instrumentation.rs`](../corpus/source/datafusion-tracing/rule_instrumentation.rs) — upstream, verbatim
- [`content/corpus/source/datafusion-tracing/rule_options.rs`](../corpus/source/datafusion-tracing/rule_options.rs) — upstream, verbatim
