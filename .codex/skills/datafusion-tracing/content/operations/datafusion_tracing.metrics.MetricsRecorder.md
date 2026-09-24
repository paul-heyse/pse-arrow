# `datafusion_tracing::metrics::MetricsRecorder`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.metrics.MetricsRecorder.json).

<a id="op-300c446f01aca0f9f2512062"></a>
## MetricsRecorder

`struct` · `datafusion_tracing::metrics::MetricsRecorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct MetricsRecorder
```

Source: `src/metrics.rs:34`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Records execution metrics and automatically logs them via tracing upon completion.

This struct is designed to handle metrics safely across concurrent partition executions.

<a id="op-9473bba8b1bd984b951e66da"></a>
## drop

`function` · `datafusion_tracing::metrics::MetricsRecorder::drop` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::metrics::MetricsRecorder", "path": "MetricsRecorder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 1], "end": [65, 2], "filename": "src/metrics.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/metrics.rs:54`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Aggregates and logs the collected metrics when the recorder goes out of scope,
which should only happen once all executed partition streams have completed.

<a id="op-0207a8a9624da2e392bd6a55"></a>
## execution_plan

`struct_field` · `datafusion_tracing::metrics::MetricsRecorder::execution_plan` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
execution_plan: std::sync::Arc<dyn ExecutionPlan>
```

Source: `src/metrics.rs:36`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Execution plan to retrieve metrics from once stream execution is finished.

<a id="op-a96c88cb38568916c83bf000"></a>
## new

`function` · `datafusion_tracing::metrics::MetricsRecorder::new` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn new(execution_plan: Arc<dyn ExecutionPlan>, span: Span) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::metrics::MetricsRecorder", "path": "MetricsRecorder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [42, 1], "end": [49, 2], "filename": "src/metrics.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics.rs:43`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-47cef435ce293f2d59d00c67"></a>
## span

`struct_field` · `datafusion_tracing::metrics::MetricsRecorder::span` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span: tracing::Span
```

Source: `src/metrics.rs:39`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Tracing span used to log metrics for observability.
