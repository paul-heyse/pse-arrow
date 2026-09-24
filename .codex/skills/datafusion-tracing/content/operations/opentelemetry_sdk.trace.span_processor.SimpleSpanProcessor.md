# `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_processor.SimpleSpanProcessor.json).

<a id="op-251eecc5a13ec3fb5a757796"></a>
## SimpleSpanProcessor

`struct` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SimpleSpanProcessor<T: SpanExporter>
```

Source: `src/trace/span_processor.rs:118`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A [SpanProcessor](../operations/opentelemetry_sdk.trace.span_processor.SpanProcessor.md#op-d354c01a6b0b4f29c191ace7) that passes finished spans to the configured
`SpanExporter`, as soon as they are finished, without any batching. This is
typically useful for debugging and testing. For scenarios requiring higher
performance/throughput, consider using [BatchSpanProcessor](../operations/opentelemetry_sdk.trace.span_processor.BatchSpanProcessor.md#op-7b6dded2e02e9c652c240628).
Spans are exported synchronously
in the same thread that emits the log record.
When using this processor with the OTLP Exporter, the following exporter
features are supported:
- `grpc-tonic`: This requires TracerProvider to be created within a tokio
  runtime. Spans can be emitted from any thread, including tokio runtime
  threads.
- `reqwest-blocking-client`: TracerProvider may be created anywhere, but
  spans must be emitted from a non-tokio runtime thread.
- `reqwest-client`: TracerProvider may be created anywhere, but spans must be
  emitted from a tokio runtime thread.

<a id="op-ff55a07d6964d1a0d1a46c3c"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor", "path": "SimpleSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [117, 10], "end": [117, 15], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_processor.rs:117`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2bbe03e9a803ac3f7993a5c7"></a>
## force_flush

`function` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor", "path": "SimpleSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [131, 1], "end": [176, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:156`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-66e50a909d6b389799a1d28d"></a>
## new

`function` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(exporter: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor", "path": "SimpleSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [122, 1], "end": [129, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:124`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new [SimpleSpanProcessor](../operations/opentelemetry_sdk.trace.span_processor.SimpleSpanProcessor.md#op-251eecc5a13ec3fb5a757796) using the provided exporter.

<a id="op-7c2d1c8940c7654c52e08998"></a>
## on_end

`function` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor::on_end` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_end(&self, span: SpanData)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor", "path": "SimpleSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [131, 1], "end": [176, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:136`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-df64f5023210d936737cbb40"></a>
## on_start

`function` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor::on_start` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_start(&self, _span: &mut Span, _cx: &Context)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor", "path": "SimpleSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [131, 1], "end": [176, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:132`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e7bf34863ab64bab22499e31"></a>
## set_resource

`function` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor", "path": "SimpleSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [131, 1], "end": [176, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:171`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b13e478f5754af0374cedb2"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor", "path": "SimpleSpanProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [131, 1], "end": [176, 2], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::span_processor::SpanProcessor", "path": "SpanProcessor"}, "trait_path": "opentelemetry_sdk::trace::span_processor::SpanProcessor"}`

Source: `src/trace/span_processor.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
