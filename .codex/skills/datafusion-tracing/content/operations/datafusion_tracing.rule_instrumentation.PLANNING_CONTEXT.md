# `datafusion_tracing::rule_instrumentation::PLANNING_CONTEXT`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.PLANNING_CONTEXT.json).

<a id="op-18bd3974dee1397bfeccf5b9"></a>
## PLANNING_CONTEXT

`constant` · `datafusion_tracing::rule_instrumentation::PLANNING_CONTEXT` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
const PLANNING_CONTEXT: thread::LocalKey<std::cell::RefCell<Option<PlanningContext>>> = _
```

Source: `src/rule_instrumentation.rs:168`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The current planning context, if any phase is active.
