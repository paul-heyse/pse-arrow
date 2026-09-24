# `datafusion_tracing::rule_instrumentation::generate_plan_diff`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.generate_plan_diff.json).

<a id="op-21ade021ce0d5dc77748bdd9"></a>
## generate_plan_diff

`function` · `datafusion_tracing::rule_instrumentation::generate_plan_diff` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn generate_plan_diff(before: &str, after: &str) -> String
```

Source: `src/rule_instrumentation.rs:579`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Generates a unified diff between two plan strings.
