# `opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_processor.BatchSpanProcessorBuilder.json).

<a id="op-648c91bb34917bcabbc46b85"></a>
## BatchSpanProcessorBuilder

`struct` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchSpanProcessorBuilder<E> where E: SpanExporter + Send + 'static
```

Source: `src/trace/span_processor.rs:707`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builder for `BatchSpanProcessorDedicatedThread`.

<a id="op-0f5b74b9cb881a6f58c6083a"></a>
## build

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> BatchSpanProcessor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder", "path": "BatchSpanProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [715, 1], "end": [728, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:725`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Build a new instance of `BatchSpanProcessor`.

<a id="op-48fa1778cbf82957d80738c3"></a>
## default

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> BatchSpanProcessorBuilder<E>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder", "path": "BatchSpanProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"outlives": "'static"}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::default::Default", "path": "$crate::default::Default"}}}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [706, 17], "end": [706, 24], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/span_processor.rs:706`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dd86a2484aa6625a556393ca"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder", "path": "BatchSpanProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"outlives": "'static"}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [706, 10], "end": [706, 15], "filename": "src/trace/span_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_processor.rs:706`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1c6e157bd6fc01b145f7ce35"></a>
## with_batch_config

`function` · `opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder::with_batch_config` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_batch_config(self, config: BatchConfig) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "E"}}], "constraints": []}}, "id": "opentelemetry_sdk::trace::span_processor::BatchSpanProcessorBuilder", "path": "BatchSpanProcessorBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}]}, "is_negative": false, "span": {"begin": [715, 1], "end": [728, 2], "filename": "src/trace/span_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_processor.rs:720`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the BatchConfig for [BatchSpanProcessorBuilder](../operations/opentelemetry_sdk.trace.span_processor.BatchSpanProcessorBuilder.md#op-648c91bb34917bcabbc46b85)
