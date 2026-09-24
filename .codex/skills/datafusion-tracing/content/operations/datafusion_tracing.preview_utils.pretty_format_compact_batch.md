# `datafusion_tracing::preview_utils::pretty_format_compact_batch`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.preview_utils.pretty_format_compact_batch.json).

<a id="op-cc2a4e58ce947d96244e03f2"></a>
## pretty_format_compact_batch

`function` · `datafusion_tracing::preview_utils::pretty_format_compact_batch` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: hosted.

```rust
fn pretty_format_compact_batch(batch: &datafusion::arrow::array::RecordBatch, max_width: usize, max_row_height: usize, min_compacted_col_width: usize) -> Result<impl Display, datafusion::arrow::error::ArrowError>
```

Source: `src/preview_utils.rs:36`. [Exact documentation build](https://docs.rs/crate/datafusion-tracing/55.0.0/json).

Formats a `RecordBatch` as a neatly aligned ASCII table,
constraining the total width to `max_width`. Columns are
dynamically resized or truncated, and columns that cannot
fit within the given width may be dropped.

<a id="op-5a8fa15e2bb92e83eed1b7b8"></a>
## pretty_format_compact_batch

`function` · `datafusion_tracing::preview_utils::pretty_format_compact_batch` · datafusion-tracing 55.0.0
Reachability: `supported`.  Capture: private.

```rust
fn pretty_format_compact_batch(batch: &datafusion::arrow::array::RecordBatch, max_width: usize, max_row_height: usize, min_compacted_col_width: usize) -> Result<impl Display, datafusion::arrow::error::ArrowError>
```

Source: `src/preview_utils.rs:36`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Formats a `RecordBatch` as a neatly aligned ASCII table,
constraining the total width to `max_width`. Columns are
dynamically resized or truncated, and columns that cannot
fit within the given width may be dropped.
