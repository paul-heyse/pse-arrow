# `datafusion_tracing::rule_instrumentation::record_modified_rule_in_context`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.record_modified_rule_in_context.json).

<a id="op-6d299c2b4065ac44927aef65"></a>
## record_modified_rule_in_context

`function` · `datafusion_tracing::rule_instrumentation::record_modified_rule_in_context` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn record_modified_rule_in_context(rule_name: &str)
```

Source: `src/rule_instrumentation.rs:176`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Records that a rule modified the plan in the current phase.
