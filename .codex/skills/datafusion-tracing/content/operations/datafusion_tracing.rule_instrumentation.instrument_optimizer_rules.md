# `datafusion_tracing::rule_instrumentation::instrument_optimizer_rules`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.instrument_optimizer_rules.json).

<a id="op-22b604ea97836ca80661e6b1"></a>
## instrument_optimizer_rules

`function` · `datafusion_tracing::rule_instrumentation::instrument_optimizer_rules` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn instrument_optimizer_rules(rules: Vec<std::sync::Arc<dyn OptimizerRule + Send + Sync>>, options: &rule_options::RuleInstrumentationOptions, span_create_fn: &std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>, phase_span_create_fn: &std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>) -> Vec<std::sync::Arc<dyn OptimizerRule + Send + Sync>>
```

Source: `src/rule_instrumentation.rs:1040`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments optimizer rules with phase sentinel and optional rule-level spans.
