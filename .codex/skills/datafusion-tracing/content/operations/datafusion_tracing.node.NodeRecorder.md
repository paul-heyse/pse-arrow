# `datafusion_tracing::node::NodeRecorder`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.node.NodeRecorder.json).

<a id="op-0a81ca8aba2058c85173a9a0"></a>
## NodeRecorder

`struct` · `datafusion_tracing::node::NodeRecorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct NodeRecorder
```

Source: `src/node.rs:34`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Records the `datafusion.node` span field once execution completes across all
partitions, when the value is fully qualified.

<a id="op-d98d337feef1c9b60e093318"></a>
## drop

`function` · `datafusion_tracing::node::NodeRecorder::drop` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::node::NodeRecorder", "path": "NodeRecorder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [52, 1], "end": [59, 2], "filename": "src/node.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/node.rs:53`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7a9966f74de6cda33043f6f"></a>
## execution_plan

`struct_field` · `datafusion_tracing::node::NodeRecorder::execution_plan` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
execution_plan: std::sync::Arc<dyn ExecutionPlan>
```

Source: `src/node.rs:35`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-08bda253ba12aca86677d7b8"></a>
## new

`function` · `datafusion_tracing::node::NodeRecorder::new` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn new(execution_plan: Arc<dyn ExecutionPlan>, span: Span) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::node::NodeRecorder", "path": "NodeRecorder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [39, 1], "end": [50, 2], "filename": "src/node.rs"}, "trait": null, "trait_path": null}`

Source: `src/node.rs:40`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7db8055474a25a9084d6dad9"></a>
## span

`struct_field` · `datafusion_tracing::node::NodeRecorder::span` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span: tracing::Span
```

Source: `src/node.rs:36`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
