# `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_processor_with_async_runtime.BatchSpanProcessorBuilder.json).

<a id="op-cc9c4783ad12e1b5376dc7f9"></a>
## BatchSpanProcessorBuilder

`struct` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchSpanProcessorBuilder<E, R>
```

Source: `src/trace/span_processor_with_async_runtime.rs:424`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A builder for creating [`BatchSpanProcessor`](../operations/opentelemetry_sdk.trace.span_processor_with_async_runtime.BatchSpanProcessor.md#op-29c7b9857e36b8dda7f68fbb) instances.


<a id="op-6aa1b7139f1e0132b9ce0d69"></a>
## build

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> BatchSpanProcessor<R>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder", "path": "BatchSpanProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [430, 1], "end": [444, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor_with_async_runtime.rs:441`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Build a batch processor

<a id="op-eaf36ab83fc8ab1aec696a3d"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder", "path": "BatchSpanProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [423, 10], "end": [423, 15], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_processor_with_async_runtime.rs:423`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b4d4d374db181aaa6253ecbf"></a>
## with_batch_config

`function` · `opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder::with_batch_config` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_batch_config(self, config: BatchConfig) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessorBuilder", "path": "BatchSpanProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::runtime::RuntimeChannel", "path": "RuntimeChannel"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [430, 1], "end": [444, 2], "filename": "src/trace/span_processor_with_async_runtime.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor_with_async_runtime.rs:436`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the BatchConfig for [BatchSpanProcessorBuilder](../operations/opentelemetry_sdk.trace.span_processor_with_async_runtime.BatchSpanProcessorBuilder.md#op-cc9c4783ad12e1b5376dc7f9)
