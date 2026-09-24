# `opentelemetry::trace::tracer::SamplingResult`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.tracer.SamplingResult.json).

<a id="op-e423c1725dcb165dcc01f8f4"></a>
## SamplingResult

`struct` · `opentelemetry::trace::tracer::SamplingResult` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SamplingResult
```

Source: `src/trace/tracer.rs:386`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The result of sampling logic for a given span.

<a id="op-ddb6ae299c42652337985b46"></a>
## attributes

`struct_field` · `opentelemetry::trace::tracer::SamplingResult::attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
attributes: Vec<KeyValue>
```

Source: `src/trace/tracer.rs:391`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Extra attributes to be added to the span by the sampler

<a id="op-d9509ecd1a3fd87d8f4d100c"></a>
## clone

`function` · `opentelemetry::trace::tracer::SamplingResult::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SamplingResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SamplingResult", "path": "SamplingResult"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [385, 10], "end": [385, 15], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/tracer.rs:385`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d9b030cccb3449b16c618619"></a>
## decision

`struct_field` · `opentelemetry::trace::tracer::SamplingResult::decision` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
decision: SamplingDecision
```

Source: `src/trace/tracer.rs:388`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The decision about whether or not to sample.

<a id="op-8ae9f781f530891057d541d2"></a>
## eq

`function` · `opentelemetry::trace::tracer::SamplingResult::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SamplingResult) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SamplingResult", "path": "SamplingResult"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [385, 24], "end": [385, 33], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/tracer.rs:385`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b37d22a4e3ef4bd817fb678"></a>
## fmt

`function` · `opentelemetry::trace::tracer::SamplingResult::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SamplingResult", "path": "SamplingResult"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [385, 17], "end": [385, 22], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/tracer.rs:385`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-953d0429801a6769796e45a3"></a>
## trace_state

`struct_field` · `opentelemetry::trace::tracer::SamplingResult::trace_state` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trace_state: trace::TraceState
```

Source: `src/trace/tracer.rs:394`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Trace state from parent context, may be modified by samplers.
