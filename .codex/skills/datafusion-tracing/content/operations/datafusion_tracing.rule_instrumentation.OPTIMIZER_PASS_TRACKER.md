# `datafusion_tracing::rule_instrumentation::OPTIMIZER_PASS_TRACKER`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.OPTIMIZER_PASS_TRACKER.json).

<a id="op-fe7c4b8dcc803831646648e1"></a>
## OPTIMIZER_PASS_TRACKER

`constant` · `datafusion_tracing::rule_instrumentation::OPTIMIZER_PASS_TRACKER` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
const OPTIMIZER_PASS_TRACKER: thread::LocalKey<std::cell::RefCell<OptimizerPassTracker>> = _
```

Source: `src/rule_instrumentation.rs:168`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Tracks optimizer pass count and the parent span ID to detect new queries.
