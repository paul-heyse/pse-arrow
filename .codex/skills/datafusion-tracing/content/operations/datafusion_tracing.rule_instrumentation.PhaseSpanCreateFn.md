# `datafusion_tracing::rule_instrumentation::PhaseSpanCreateFn`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.PhaseSpanCreateFn.json).

<a id="op-d8451753a982aec1daa91d2d"></a>
## PhaseSpanCreateFn

`type_alias` · `datafusion_tracing::rule_instrumentation::PhaseSpanCreateFn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
type PhaseSpanCreateFn = dyn Fn(&str) -> tracing::Span + Send + Sync
```

Source: `src/rule_instrumentation.rs:84`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Type alias for a function that creates a phase span given the phase name.
Example phase names: "analyze_logical_plan", "optimize_logical_plan", "optimize_physical_plan".
