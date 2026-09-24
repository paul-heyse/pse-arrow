# `datafusion_tracing::rule_instrumentation::drop_planning_context`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.drop_planning_context.json).

<a id="op-25e27c00bed0df9d505ce9f9"></a>
## drop_planning_context

`function` · `datafusion_tracing::rule_instrumentation::drop_planning_context` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn drop_planning_context()
```

Source: `src/rule_instrumentation.rs:187`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Drops the current planning context on a fatal rule error, closing the active
phase span and resetting `OPTIMIZER_PASS_TRACKER` for `Optimizer` and
`PhysicalOptimizer` phases so the next query starts at pass 0.
