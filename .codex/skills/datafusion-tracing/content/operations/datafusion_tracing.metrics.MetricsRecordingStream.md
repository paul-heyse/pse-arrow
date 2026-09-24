# `datafusion_tracing::metrics::MetricsRecordingStream`

Full upstream contracts; raw type trees and source locators in [structured records](datafusion_tracing.metrics.MetricsRecordingStream.json).

<a id="op-104120ad01ef35a964754737"></a>
## MetricsRecordingStream

`struct` · `datafusion_tracing::metrics::MetricsRecordingStream` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
struct MetricsRecordingStream
```

Source: `src/metrics.rs:71`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

A wrapper around a DataFusion stream that integrates automatic metrics recording.

It uses `MetricsRecorder` to safely handle metric aggregation and recording, avoiding concurrency issues.

<a id="op-ff99c231ed2d1e273024a12a"></a>
## Item

`assoc_type` · `datafusion_tracing::metrics::MetricsRecordingStream::Item` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::metrics::MetricsRecordingStream", "path": "MetricsRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [95, 1], "end": [102, 2], "filename": "src/metrics.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/metrics.rs:96`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7ff3624953f22ca9b10f0f05"></a>
## inner

`struct_field` · `datafusion_tracing::metrics::MetricsRecordingStream::inner` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
inner: datafusion::execution::SendableRecordBatchStream
```

Source: `src/metrics.rs:74`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

The underlying DataFusion stream providing record batches.

<a id="op-f3873b4bf482d85a3bcfb350"></a>
## metrics_recorder

`struct_field` · `datafusion_tracing::metrics::MetricsRecordingStream::metrics_recorder` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
metrics_recorder: std::sync::Arc<MetricsRecorder>
```

Source: `src/metrics.rs:77`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Shared recorder used for metric aggregation and logging, which will record the metrics when dropped.

<a id="op-d8716fce10dffa895b7c37c5"></a>
## new

`function` · `datafusion_tracing::metrics::MetricsRecordingStream::new` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn new(inner: SendableRecordBatchStream, metrics_recorder: Arc<MetricsRecorder>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::metrics::MetricsRecordingStream", "path": "MetricsRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [93, 2], "filename": "src/metrics.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics.rs:84`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Wraps a DataFusion stream with metrics recording capability.

The provided `MetricsRecorder` will handle logging the metrics via tracing when it is dropped.

<a id="op-b4f41276e970dd7f1ca74777"></a>
## poll_next

`function` · `datafusion_tracing::metrics::MetricsRecordingStream::poll_next` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn poll_next(Pin<&mut self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::metrics::MetricsRecordingStream", "path": "MetricsRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [95, 1], "end": [102, 2], "filename": "src/metrics.rs"}, "trait": {"args": null, "id": "futures_core::stream::Stream", "path": "Stream"}, "trait_path": "futures_core::stream::Stream"}`

Source: `src/metrics.rs:99`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Retrieves the next record batch from the underlying stream.

<a id="op-f27bf0ad92f6e6c2fa4bfb59"></a>
## schema

`function` · `datafusion_tracing::metrics::MetricsRecordingStream::schema` · datafusion-tracing 55.0.0
Reachability: `internal`.  Capture: private.

```rust
fn schema(&self) -> SchemaRef
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "datafusion_tracing::metrics::MetricsRecordingStream", "path": "MetricsRecordingStream"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [104, 1], "end": [109, 2], "filename": "src/metrics.rs"}, "trait": {"args": null, "id": "datafusion_execution::stream::RecordBatchStream", "path": "RecordBatchStream"}, "trait_path": "datafusion_execution::stream::RecordBatchStream"}`

Source: `src/metrics.rs:106`. [Exact documentation build](../../build/acquired/rustdoc/datafusion-tracing@55.0.0.private.json.zst).

Returns the schema for the record batches produced by the wrapped stream.
