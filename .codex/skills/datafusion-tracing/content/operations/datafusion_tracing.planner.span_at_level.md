# `datafusion_tracing::planner::span_at_level`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.planner.span_at_level.json).

<a id="op-a21925464173bfe88d14de54"></a>
## span_at_level

`macro` · `datafusion_tracing::planner::span_at_level` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
macro_rules! span_at_level
```

Source: `src/planner.rs:34`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Creates a span at the specified tracing level with the given name and fields.

This macro eliminates the need for repetitive match blocks when creating spans
at different levels. The level must be a `tracing::Level` value.
