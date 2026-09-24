# `datafusion_tracing::preview::PreviewRecorderBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.preview.PreviewRecorderBuilder.json).

<a id="op-52bf5222076ab569704b82c4"></a>
## PreviewRecorderBuilder

`struct` · `datafusion_tracing::preview::PreviewRecorderBuilder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct PreviewRecorderBuilder
```

Source: `src/preview.rs:183`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Builder for [`PreviewRecorder`](../operations/datafusion_tracing.preview.PreviewRecorder.md#op-af2cf65f8ccc09c18ce9aca6).

<a id="op-909e385367f50cdd2dcab4c9"></a>
## build

`function` · `datafusion_tracing::preview::PreviewRecorderBuilder::build` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn build(self) -> PreviewRecorder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecorderBuilder", "path": "PreviewRecorderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [190, 1], "end": [217, 2], "filename": "src/preview.rs"}, "trait": null, "trait_path": null}`

Source: `src/preview.rs:207`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Build the `PreviewRecorder`, returning a `DataFusionResult`
which is `Ok` if all required fields were set or an `Err` if any are missing.

<a id="op-3709efa133bfb346d2df0517"></a>
## limit

`function` · `datafusion_tracing::preview::PreviewRecorderBuilder::limit` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn limit(self, limit: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecorderBuilder", "path": "PreviewRecorderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [190, 1], "end": [217, 2], "filename": "src/preview.rs"}, "trait": null, "trait_path": null}`

Source: `src/preview.rs:191`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-87f68e132405bef801486565"></a>
## limit

`struct_field` · `datafusion_tracing::preview::PreviewRecorderBuilder::limit` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
limit: Option<usize>
```

Source: `src/preview.rs:186`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4ccd2388bca2c074d28de0d9"></a>
## partition_count

`struct_field` · `datafusion_tracing::preview::PreviewRecorderBuilder::partition_count` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
partition_count: usize
```

Source: `src/preview.rs:185`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-50f86e5b0dfb4cff3937b0b1"></a>
## preview_fn

`function` · `datafusion_tracing::preview::PreviewRecorderBuilder::preview_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn preview_fn(self, preview_fn: Option<Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::preview::PreviewRecorderBuilder", "path": "PreviewRecorderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [190, 1], "end": [217, 2], "filename": "src/preview.rs"}, "trait": null, "trait_path": null}`

Source: `src/preview.rs:196`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c880817e2e01577cd93c4e55"></a>
## preview_fn

`struct_field` · `datafusion_tracing::preview::PreviewRecorderBuilder::preview_fn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
preview_fn: std::sync::Arc<dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync>
```

Source: `src/preview.rs:187`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e7996e79b5b495842062c6ad"></a>
## span

`struct_field` · `datafusion_tracing::preview::PreviewRecorderBuilder::span` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
span: tracing::Span
```

Source: `src/preview.rs:184`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
