# `datafusion_tracing::rule_instrumentation::RuleSpanCreateFn`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.RuleSpanCreateFn.json).

<a id="op-bc3bcf5240e7b08c5c7e5dbb"></a>
## RuleSpanCreateFn

`type_alias` · `datafusion_tracing::rule_instrumentation::RuleSpanCreateFn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
type RuleSpanCreateFn = dyn Fn(&str) -> tracing::Span + Send + Sync
```

Source: `src/rule_instrumentation.rs:631`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Function type that creates a span for a rule, given the rule name.
