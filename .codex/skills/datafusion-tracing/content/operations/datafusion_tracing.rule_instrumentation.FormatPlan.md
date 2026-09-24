# `datafusion_tracing::rule_instrumentation::FormatPlan`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.rule_instrumentation.FormatPlan.json).

<a id="op-ac5e0bac02dbf1e9a0ba9541"></a>
## FormatPlan

`trait` · `datafusion_tracing::rule_instrumentation::FormatPlan` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
trait FormatPlan
```

Source: `src/rule_instrumentation.rs:65`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Trait for formatting plans as strings for diffing and tracing.

This provides a unified interface for converting different plan types
(logical and physical) to their string representations.

<a id="op-96dce4851d6136a45e583610"></a>
## format_for_diff

`function` · `datafusion_tracing::rule_instrumentation::FormatPlan::format_for_diff` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn format_for_diff(&self) -> String
```

Source: `src/rule_instrumentation.rs:67`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Formats the plan as a string suitable for diff comparison.
