# `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_processor_with_async_runtime.BatchSpanProcessor.json).

<a id="op-29c7b9857e36b8dda7f68fbb"></a>
## BatchSpanProcessor

`struct` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchSpanProcessor<R: RuntimeChannel>
```

Source: `src/trace/span_processor_with_async_runtime.rs:85`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A [`SpanProcessor`](../operations/opentelemetry_sdk.trace.span_processor.SpanProcessor.md#op-d354c01a6b0b4f29c191ace7) that asynchronously buffers finished spans and reports
them at a preconfigured interval.

Batch span processors need to run a background task to collect and send
spans. Different runtimes need different ways to handle the background task.

Note: Configuring an opentelemetry `Runtime` that's not compatible with the
underlying runtime can cause deadlocks (see tokio section).

### Use with Tokio

Tokio currently offers two different schedulers. One is
`current_thread_scheduler`, the other is `multiple_thread_scheduler`. Both
of them default to use batch span processors to install span exporters.

Tokio's `current_thread_scheduler` can cause the program to hang forever if
blocking work is scheduled with other tasks in the same runtime. To avoid
this, be sure to enable the `rt-tokio-current-thread` feature in this crate
if you are using that runtime (e.g. users of actix-web), and blocking tasks
will then be scheduled on a different thread.

# Examples

This processor can be configured with an [`executor`] of your choice to
batch and upload spans asynchronously when they end. If you have added a
library like [`tokio`], you can pass in their respective
`spawn` and `interval` functions to have batching performed in those
contexts.

```
# #[cfg(feature="tokio")]
# {
use opentelemetry::global;
use opentelemetry_sdk::{runtime, testing::trace::NoopSpanExporter, trace};
use opentelemetry_sdk::trace::BatchConfigBuilder;
use std::time::Duration;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;

#[tokio::main]
async fn main() {
    // Configure your preferred exporter
    let exporter = NoopSpanExporter::new();

    // Create a batch span processor using an exporter and a runtime
    let batch = BatchSpanProcessor::builder(exporter, runtime::Tokio)
        .with_batch_config(BatchConfigBuilder::default().with_max_queue_size(4096).build())
        .build();

    // Then use the `with_batch_exporter` method to have the provider export spans in batches.
    let provider = trace::SdkTracerProvider::builder()
        .with_span_processor(batch)
        .build();

    let _ = global::set_tracer_provider(provider);
}
# }
```

[`executor`]: https://docs.rs/futures/0.3/futures/executor/index.html
[`tokio`]: https://tokio.rs

<a id="op-5ea49ea8381dcb1d8e06bf4b"></a>
## builder

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder<E>(exporter: E, runtime: R) -> BatchSpanProcessorBuilder<E, R> where E: SpanExporter
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [368, 1], "end": [419, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor_with_async_runtime.rs:409`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new batch processor builder

<a id="op-a98bac2d54513edae4cfed10"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [95, 1], "end": [101, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_processor_with_async_runtime.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2ddf3834d359830ac56ab3ad"></a>
## force_flush

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [103, 1], "end": [169, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor_with_async_runtime.rs:126`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-22a2a3367f103670f60deecb"></a>
## on_end

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::on_end` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_end(&self, span: SpanData)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [103, 1], "end": [169, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor_with_async_runtime.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d75b58080122e06b33f28684"></a>
## on_start

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::on_start` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_start(&self, _span: &mut Span, _cx: &Context)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [103, 1], "end": [169, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor_with_async_runtime.rs:104`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ad0618777f829a9ddca14ba3"></a>
## set_resource

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [103, 1], "end": [169, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor_with_async_runtime.rs:163`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-48f1211d63db6f89e26777e0"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor", "path": "BatchSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [103, 1], "end": [169, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor_with_async_runtime.rs:139`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
