# `datafusion_tracing::node::NodeRecordingStream`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.node.NodeRecordingStream.json).

<a id="op-4834b6f5d9bc45b5b7d94399"></a>
## NodeRecordingStream

`struct` · `datafusion_tracing::node::NodeRecordingStream` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct NodeRecordingStream
```

Source: `src/node.rs:64`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

A wrapper around a DataFusion stream that keeps a `NodeRecorder` alive
so the `datafusion.node` field is recorded only when the stream completes.

<a id="op-1a491e30cb8c3a05fbd24633"></a>
## Item

`assoc_type` · `datafusion_tracing::node::NodeRecordingStream::Item` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::node::NodeRecordingStream", "path": "NodeRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [79, 1], "end": [85, 2], "filename": "src/node.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/node.rs:80`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5ed623722175df74ad99da8b"></a>
## _recorder

`struct_field` · `datafusion_tracing::node::NodeRecordingStream::_recorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
_recorder: std::sync::Arc<NodeRecorder>
```

Source: `src/node.rs:67`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d02d9daf1cc0ce3eed6f356"></a>
## inner

`struct_field` · `datafusion_tracing::node::NodeRecordingStream::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: datafusion::execution::SendableRecordBatchStream
```

Source: `src/node.rs:66`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b7c4779e5290ea1a89a1d91"></a>
## new

`function` · `datafusion_tracing::node::NodeRecordingStream::new` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn new(inner: SendableRecordBatchStream, recorder: Arc<NodeRecorder>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::node::NodeRecordingStream", "path": "NodeRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [77, 2], "filename": "src/node.rs"}, "trait": null, "trait_path": null}`

Source: `src/node.rs:71`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b4544e57b0a8ad516c9eccbb"></a>
## poll_next

`function` · `datafusion_tracing::node::NodeRecordingStream::poll_next` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn poll_next(Pin<&mut self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::node::NodeRecordingStream", "path": "NodeRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [79, 1], "end": [85, 2], "filename": "src/node.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/node.rs:82`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-77189ba511585584d1ad349b"></a>
## schema

`function` · `datafusion_tracing::node::NodeRecordingStream::schema` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema(&self) -> SchemaRef
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::node::NodeRecordingStream", "path": "NodeRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [87, 1], "end": [91, 2], "filename": "src/node.rs"}, "trait": {"args": null, "id": "datafusion_execution::stream::RecordBatchStream", "path": "RecordBatchStream"}, "trait_path": "datafusion_execution::stream::RecordBatchStream"}`

Source: `src/node.rs:88`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
