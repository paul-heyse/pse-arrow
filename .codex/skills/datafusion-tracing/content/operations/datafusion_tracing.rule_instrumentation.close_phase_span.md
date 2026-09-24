# `datafusion_tracing::rule_instrumentation::close_phase_span`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.close_phase_span.json).

<a id="op-77f889a6d6bf8db52cd06c18"></a>
## close_phase_span

`function` · `datafusion_tracing::rule_instrumentation::close_phase_span` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn close_phase_span<P: FormatPlan>(ctx: PlanningContext, plan_after: &P)
```

Source: `src/rule_instrumentation.rs:204`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Closes a phase span, recording effective rules and plan diff if enabled.

The span is closed when `ctx` is dropped at the end of this function,
as `ctx._entered` holds the entered span guard.
