# `opentelemetry_sdk::trace::id_generator::IdGenerator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.id_generator.IdGenerator.json).

<a id="op-cbc81e14419948e4abd3c6c2"></a>
## IdGenerator

`trait` · `opentelemetry_sdk::trace::id_generator::IdGenerator` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait IdGenerator: Send + Sync + fmt::Debug
```

Source: `src/trace/id_generator/mod.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Interface for generating IDs

<a id="op-68bc9f7eb4f5a83294668698"></a>
## new_span_id

`function` · `opentelemetry_sdk::trace::id_generator::IdGenerator::new_span_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new_span_id(&self) -> SpanId
```

Source: `src/trace/id_generator/mod.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Generate a new `SpanId`

<a id="op-0ef32587510d01301b6db7c5"></a>
## new_trace_id

`function` · `opentelemetry_sdk::trace::id_generator::IdGenerator::new_trace_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new_trace_id(&self) -> TraceId
```

Source: `src/trace/id_generator/mod.rs:9`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Generate a new `TraceId`
