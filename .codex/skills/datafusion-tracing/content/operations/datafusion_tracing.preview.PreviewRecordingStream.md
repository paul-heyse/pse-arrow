# `datafusion_tracing::preview::PreviewRecordingStream`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.preview.PreviewRecordingStream.json).

<a id="op-294d35d24acf7be9999cab4c"></a>
## PreviewRecordingStream

`struct` · `datafusion_tracing::preview::PreviewRecordingStream` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct PreviewRecordingStream
```

Source: `src/preview.rs:81`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-154222f8d72528362d13b8ae"></a>
## Item

`assoc_type` · `datafusion_tracing::preview::PreviewRecordingStream::Item` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecordingStream", "path": "PreviewRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [108, 1], "end": [140, 2], "filename": "src/preview.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/preview.rs:109`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-528499289bdc0568f92cbc59"></a>
## drop

`function` · `datafusion_tracing::preview::PreviewRecordingStream::drop` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecordingStream", "path": "PreviewRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 15], "end": [80, 25], "filename": "src/preview.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/preview.rs:80`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5917d9df04b27e53d2353286"></a>
## inner

`struct_field` · `datafusion_tracing::preview::PreviewRecordingStream::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: datafusion::execution::SendableRecordBatchStream
```

Source: `src/preview.rs:83`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5a3328fde7664bb64adb4f9b"></a>
## limit

`struct_field` · `datafusion_tracing::preview::PreviewRecordingStream::limit` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
limit: usize
```

Source: `src/preview.rs:86`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c46247f1622211dbb373696c"></a>
## new

`function` · `datafusion_tracing::preview::PreviewRecordingStream::new` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn new(inner: SendableRecordBatchStream, preview_recorder: Arc<PreviewRecorder>, partition: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecordingStream", "path": "PreviewRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 1], "end": [106, 2], "filename": "src/preview.rs"}, "trait": null, "trait_path": null}`

Source: `src/preview.rs:92`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-07b657ab950343ced4c61ee9"></a>
## partition

`struct_field` · `datafusion_tracing::preview::PreviewRecordingStream::partition` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
partition: usize
```

Source: `src/preview.rs:84`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-68f0e92c9f2b2c6997e4bbba"></a>
## poll_next

`function` · `datafusion_tracing::preview::PreviewRecordingStream::poll_next` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn poll_next(Pin<&mut self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecordingStream", "path": "PreviewRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [108, 1], "end": [140, 2], "filename": "src/preview.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/preview.rs:111`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c165593934770eafc18e9e48"></a>
## preview_batch

`struct_field` · `datafusion_tracing::preview::PreviewRecordingStream::preview_batch` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
preview_batch: Option<datafusion::arrow::record_batch::RecordBatch>
```

Source: `src/preview.rs:87`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7cedaedf04da8a6d8c9b8c68"></a>
## preview_recorder

`struct_field` · `datafusion_tracing::preview::PreviewRecordingStream::preview_recorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
preview_recorder: std::sync::Arc<PreviewRecorder>
```

Source: `src/preview.rs:88`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-972d492c7055c6919f6332db"></a>
## schema

`function` · `datafusion_tracing::preview::PreviewRecordingStream::schema` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema(&self) -> SchemaRef
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecordingStream", "path": "PreviewRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [161, 1], "end": [165, 2], "filename": "src/preview.rs"}, "trait": {"args": null, "id": "datafusion_execution::stream::RecordBatchStream", "path": "RecordBatchStream"}, "trait_path": "datafusion_execution::stream::RecordBatchStream"}`

Source: `src/preview.rs:162`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-af04a8f5e1355bf979e9037c"></a>
## stored_rows

`struct_field` · `datafusion_tracing::preview::PreviewRecordingStream::stored_rows` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
stored_rows: usize
```

Source: `src/preview.rs:85`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
