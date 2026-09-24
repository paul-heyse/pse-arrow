# `datafusion_tracing::preview::PreviewRecorder`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.preview.PreviewRecorder.json).

<a id="op-af2cf65f8ccc09c18ce9aca6"></a>
## PreviewRecorder

`struct` · `datafusion_tracing::preview::PreviewRecorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct PreviewRecorder
```

Source: `src/preview.rs:39`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Stores up to the first `limit` rows for preview.

<a id="op-bf56baee26de780cbdcce54c"></a>
## builder

`function` · `datafusion_tracing::preview::PreviewRecorder::builder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn builder(span: Span, partition_count: usize) -> PreviewRecorderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecorder", "path": "PreviewRecorder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [171, 1], "end": [181, 2], "filename": "src/preview.rs"}, "trait": null, "trait_path": null}`

Source: `src/preview.rs:173`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Returns a new builder for creating a `PreviewRecorder`.

<a id="op-38a91c140519f918721b94b9"></a>
## drop

`function` · `datafusion_tracing::preview::PreviewRecorder::drop` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecorder", "path": "PreviewRecorder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [46, 1], "end": [78, 2], "filename": "src/preview.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/preview.rs:47`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-83ba318709117ebf3514bb0f"></a>
## limit

`struct_field` · `datafusion_tracing::preview::PreviewRecorder::limit` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
limit: usize
```

Source: `src/preview.rs:41`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b9ea89dfe5b0c73c14189d75"></a>
## partition_previews

`struct_field` · `datafusion_tracing::preview::PreviewRecorder::partition_previews` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
partition_previews: Vec<std::sync::OnceLock<datafusion::arrow::record_batch::RecordBatch>>
```

Source: `src/preview.rs:42`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b7bb6acdfe631ea6e470631"></a>
## preview_fn

`struct_field` · `datafusion_tracing::preview::PreviewRecorder::preview_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
preview_fn: std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>
```

Source: `src/preview.rs:43`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d03f55d52d0db1cffa7e71ce"></a>
## span

`struct_field` · `datafusion_tracing::preview::PreviewRecorder::span` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span: tracing::Span
```

Source: `src/preview.rs:40`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
