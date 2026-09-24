# `datafusion_tracing::exec_instrument_rule::SpanTracer`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.exec_instrument_rule.SpanTracer.json).

<a id="op-5edc3b97b64c10ccfbdcd352"></a>
## SpanTracer

`struct` · `datafusion_tracing::exec_instrument_rule::SpanTracer` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct SpanTracer
```

Source: `src/exec_instrument_rule.rs:99`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

A simple tracer that ensures any spawned task or blocking closure
inherits the current span via `in_current_span`.

<a id="op-1b0bf7f53e311d5a6662e823"></a>
## trace_block

`function` · `datafusion_tracing::exec_instrument_rule::SpanTracer::trace_block` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn trace_block(&self, f: Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>) -> Box<dyn FnOnce() -> Box<dyn Any + Send> + Send>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::exec_instrument_rule::SpanTracer", "path": "SpanTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [103, 1], "end": [116, 2], "filename": "src/exec_instrument_rule.rs"}, "trait": {"args": null, "id": "datafusion_common_runtime::trace_utils::JoinSetTracer", "path": "JoinSetTracer"}, "trait_path": "datafusion_common_runtime::trace_utils::JoinSetTracer"}`

Source: `src/exec_instrument_rule.rs:113`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments a boxed blocking closure by running it inside the
`Span::current()` context.

<a id="op-b201d0a499cfd174ff497569"></a>
## trace_future

`function` · `datafusion_tracing::exec_instrument_rule::SpanTracer::trace_future` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn trace_future(&self, fut: futures::future::BoxFuture<'static, Box<dyn Any + Send>>) -> futures::future::BoxFuture<'static, Box<dyn Any + Send>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::exec_instrument_rule::SpanTracer", "path": "SpanTracer"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [103, 1], "end": [116, 2], "filename": "src/exec_instrument_rule.rs"}, "trait": {"args": null, "id": "datafusion_common_runtime::trace_utils::JoinSetTracer", "path": "JoinSetTracer"}, "trait_path": "datafusion_common_runtime::trace_utils::JoinSetTracer"}`

Source: `src/exec_instrument_rule.rs:107`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Instruments a boxed future to run in the current span. The future's
return type is erased to `BoxedAny`, which we simply
run inside the `Span::current()` context.
