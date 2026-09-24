# `datafusion_tracing::preview::PreviewFn`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.preview.PreviewFn.json).

<a id="op-790aee9983a988cb21acbadc"></a>
## PreviewFn

`type_alias` · `datafusion_tracing::preview::PreviewFn` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
type PreviewFn = dyn Fn(&datafusion::arrow::record_batch::RecordBatch) -> Result<String, datafusion::arrow::error::ArrowError> + Send + Sync
```

Source: `src/preview.rs:36`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.
