# `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_processor.BatchSpanProcessor.json).

<a id="op-7b6dded2e02e9c652c240628"></a>
## BatchSpanProcessor

`struct` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchSpanProcessor
```

Source: `src/trace/span_processor.rs:285`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The `BatchSpanProcessor` collects finished spans in a buffer and exports them
in batches to the configured `SpanExporter`. This processor is ideal for
high-throughput environments, as it minimizes the overhead of exporting spans
individually. It uses a **dedicated background thread** to manage and export spans
asynchronously, ensuring that the application's main execution flow is not blocked.

This processor supports the following configurations:
- **Queue size**: Maximum number of spans that can be buffered.
- **Batch size**: Maximum number of spans to include in a single export.
- **Scheduled delay**: Frequency at which the batch is exported.

When using this processor with the OTLP Exporter, the following exporter
features are supported:
- `grpc-tonic`: Requires `TracerProvider` to be created within a tokio runtime.
- `reqwest-blocking-client`: Works with a regular `main` or `tokio::main`.

In other words, other clients like `reqwest` and `hyper` are not supported.

`BatchSpanProcessor` buffers spans in memory and exports them in batches. An
export is triggered when `max_export_batch_size` is reached or every
`scheduled_delay` milliseconds. Users can explicitly trigger an export using
the `force_flush` method. Shutdown also triggers an export of all buffered
spans and is recommended to be called before the application exits to ensure
all buffered spans are exported.

**Warning**: When using tokio's current-thread runtime, `shutdown()`, which
is a blocking call ,should not be called from your main thread. This can
cause deadlock. Instead, call `shutdown()` from a separate thread or use
tokio's `spawn_blocking`.


<a id="op-edc9caf472517cb9eba6d12f"></a>
## builder

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder<E>(exporter: E) -> BatchSpanProcessorBuilder<E> where E: SpanExporter + Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [297, 1], "end": [520, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:437`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

builder

<a id="op-5f784d50e5b1f6185dc94d6a"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [284, 10], "end": [284, 15], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_processor.rs:284`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1c19e34dcdbe88f66f1e5712"></a>
## force_flush

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [522, 1], "end": [703, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:596`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Flushes all pending spans.

<a id="op-da9df164a76bbf3a836cc1ff"></a>
## new

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new<E>(exporter: E, config: BatchConfig) -> Self where E: SpanExporter + Send + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [297, 1], "end": [520, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:299`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of `BatchSpanProcessor`.

<a id="op-947e699f0127b0a974d7ff4b"></a>
## on_end

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::on_end` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_end(&self, span: SpanData)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [522, 1], "end": [703, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:529`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Handles span end.

<a id="op-6865a056c97372a8f1055039"></a>
## on_start

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::on_start` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_start(&self, _span: &mut Span, _cx: &Context)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [522, 1], "end": [703, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:524`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Handles span start.

<a id="op-e29664da4c8ec86b2e000fdb"></a>
## set_resource

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [522, 1], "end": [703, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:697`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the resource for the processor.

<a id="op-cc4742c1e8302875533cae58"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessor::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [522, 1], "end": [703, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:633`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the processor.
