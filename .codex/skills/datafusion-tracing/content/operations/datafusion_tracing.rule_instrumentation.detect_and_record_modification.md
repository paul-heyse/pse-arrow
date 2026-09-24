# `datafusion_tracing::rule_instrumentation::detect_and_record_modification`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.detect_and_record_modification.json).

<a id="op-3dca90dc0704d7166a415df5"></a>
## detect_and_record_modification

`function` · `datafusion_tracing::rule_instrumentation::detect_and_record_modification` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn detect_and_record_modification(before_str: &str, after_str: &str, span: &tracing::Span, record_diff: bool, rule_name: &str)
```

Source: `src/rule_instrumentation.rs:606`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Detects if a plan was modified and updates span attributes accordingly.

This function:
1. Compares plan string representations to detect actual changes
2. Generates and records a unified diff if `record_diff` is true
3. Updates the span's `otel.name` to indicate modification
4. Records the rule name in the planning context for effective_rules tracking
