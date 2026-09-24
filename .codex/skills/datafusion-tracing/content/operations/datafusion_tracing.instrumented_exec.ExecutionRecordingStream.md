# `datafusion_tracing::instrumented_exec::ExecutionRecordingStream`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.instrumented_exec.ExecutionRecordingStream.json).

<a id="op-1f87ce7ac12856c011a02163"></a>
## ExecutionRecordingStream

`struct` · `datafusion_tracing::instrumented_exec::ExecutionRecordingStream` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct ExecutionRecordingStream
```

Source: `src/instrumented_exec.rs:583`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-629e17a6caa634ea31a21d9b"></a>
## Item

`assoc_type` · `datafusion_tracing::instrumented_exec::ExecutionRecordingStream::Item` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::ExecutionRecordingStream", "path": "ExecutionRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [595, 1], "end": [601, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/instrumented_exec.rs:596`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b66e67d7ff6ccc34726025b6"></a>
## drop

`function` · `datafusion_tracing::instrumented_exec::ExecutionRecordingStream::drop` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::ExecutionRecordingStream", "path": "ExecutionRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [582, 15], "end": [582, 25], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/instrumented_exec.rs:582`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-00ffb07f3c0afebc6ac7fb66"></a>
## inner

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecordingStream::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: datafusion::execution::SendableRecordBatchStream
```

Source: `src/instrumented_exec.rs:585`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6e5ef5e8606b8f308c6a8ca7"></a>
## poll_next

`function` · `datafusion_tracing::instrumented_exec::ExecutionRecordingStream::poll_next` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn poll_next(Pin<&mut self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::ExecutionRecordingStream", "path": "ExecutionRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [595, 1], "end": [601, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/instrumented_exec.rs:598`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7d902acc74b29e39ae65681c"></a>
## recorders

`struct_field` · `datafusion_tracing::instrumented_exec::ExecutionRecordingStream::recorders` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
recorders: std::sync::Arc<ExecutionRecorders>
```

Source: `src/instrumented_exec.rs:586`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8f47730b54854e4c13a84281"></a>
## schema

`function` · `datafusion_tracing::instrumented_exec::ExecutionRecordingStream::schema` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema(&self) -> SchemaRef
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::instrumented_exec::ExecutionRecordingStream", "path": "ExecutionRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [611, 1], "end": [615, 2], "filename": "src/instrumented_exec.rs"}, "trait": {"args": null, "id": "datafusion_execution::stream::RecordBatchStream", "path": "RecordBatchStream"}, "trait_path": "datafusion_execution::stream::RecordBatchStream"}`

Source: `src/instrumented_exec.rs:612`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
