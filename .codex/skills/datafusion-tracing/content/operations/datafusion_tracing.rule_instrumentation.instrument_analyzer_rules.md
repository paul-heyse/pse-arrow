# `datafusion_tracing::rule_instrumentation::instrument_analyzer_rules`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.instrument_analyzer_rules.json).

<a id="op-48337857af43d6ebae0ff583"></a>
## instrument_analyzer_rules

`function` · `datafusion_tracing::rule_instrumentation::instrument_analyzer_rules` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn instrument_analyzer_rules(rules: Vec<std::sync::Arc<dyn AnalyzerRule + Send + Sync>>, options: &rule_options::RuleInstrumentationOptions, span_create_fn: &std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>, phase_span_create_fn: &std::sync::Arc<dyn Fn(&str) -> tracing::Span + Send + Sync>) -> Vec<std::sync::Arc<dyn AnalyzerRule + Send + Sync>>
```

Source: `src/rule_instrumentation.rs:995`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments analyzer rules with phase sentinel and optional rule-level spans.
