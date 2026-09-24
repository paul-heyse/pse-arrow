# `datafusion_tracing::instrumented_exec::SpanCreateFn`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrumented_exec.SpanCreateFn.json).

<a id="op-46e93b15b8af72ed297201b3"></a>
## SpanCreateFn

`type_alias` · `datafusion_tracing::instrumented_exec::SpanCreateFn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
type SpanCreateFn = dyn Fn() -> tracing::Span + Send + Sync
```

Source: `src/instrumented_exec.rs:66`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Type alias for a function that creates a tracing span.
