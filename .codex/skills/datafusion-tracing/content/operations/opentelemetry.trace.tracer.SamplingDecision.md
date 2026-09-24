# `opentelemetry::trace::tracer::SamplingDecision`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.tracer.SamplingDecision.json).

<a id="op-29fe2075c223853678046a1f"></a>
## SamplingDecision

`enum` · `opentelemetry::trace::tracer::SamplingDecision` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum SamplingDecision
```

Source: `src/trace/tracer.rs:399`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Decision about whether or not to sample

<a id="op-7f7f55262ce2ff5b1f087a11"></a>
## Drop

`variant` · `opentelemetry::trace::tracer::SamplingDecision::Drop` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Drop
```

Source: `src/trace/tracer.rs:401`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span will not be recorded and all events and attributes will be dropped.

<a id="op-6ee0bd57efba59c96052ce67"></a>
## RecordAndSample

`variant` · `opentelemetry::trace::tracer::SamplingDecision::RecordAndSample` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
RecordAndSample
```

Source: `src/trace/tracer.rs:407`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span data will be recorded and exported.

<a id="op-5d7b7abcfa6e5cbf9bc57cc3"></a>
## RecordOnly

`variant` · `opentelemetry::trace::tracer::SamplingDecision::RecordOnly` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
RecordOnly
```

Source: `src/trace/tracer.rs:404`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span data wil be recorded, but not exported.

<a id="op-8fdb6f165557300ec41d59ef"></a>
## clone

`function` · `opentelemetry::trace::tracer::SamplingDecision::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SamplingDecision
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SamplingDecision", "path": "SamplingDecision"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [398, 10], "end": [398, 15], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/tracer.rs:398`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0bb1bf7e3f0a5b713c7f91d9"></a>
## eq

`function` · `opentelemetry::trace::tracer::SamplingDecision::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SamplingDecision) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SamplingDecision", "path": "SamplingDecision"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [398, 24], "end": [398, 33], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/tracer.rs:398`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cc9d5bb8f395644af0cb6de7"></a>
## fmt

`function` · `opentelemetry::trace::tracer::SamplingDecision::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SamplingDecision", "path": "SamplingDecision"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [398, 17], "end": [398, 22], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/tracer.rs:398`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
