# `opentelemetry::trace::tracer::SpanBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.tracer.SpanBuilder.json).

<a id="op-02f76db2795a61efb093c941"></a>
## SpanBuilder

`struct` · `opentelemetry::trace::tracer::SpanBuilder` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanBuilder
```

Source: `src/trace/tracer.rs:242`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

`SpanBuilder` allows span attributes to be configured before the span
has started.

```
use opentelemetry::{
    global,
    trace::{TracerProvider, SpanBuilder, SpanKind, Tracer},
};

let tracer = global::tracer("example-tracer");

// The builder can be used to create a span directly with the tracer
let _span = tracer.build(SpanBuilder {
    name: "example-span-name".into(),
    span_kind: Some(SpanKind::Server),
    ..Default::default()
});

// Or used with builder pattern
let _span = tracer
    .span_builder("example-span-name")
    .with_kind(SpanKind::Server)
    .start(&tracer);
```

<a id="op-1851d970ddea9bc0c54b78ec"></a>
## attributes

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
attributes: Option<Vec<KeyValue>>
```

Source: `src/trace/tracer.rs:265`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span attributes that are provided at the span creation time.
More attributes can be added afterwards.
Providing duplicate keys will result in multiple attributes
with the same key, as there is no de-duplication performed.

<a id="op-32289cc10335542a0d21de91"></a>
## clone

`function` · `opentelemetry::trace::tracer::SpanBuilder::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [241, 10], "end": [241, 15], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/tracer.rs:241`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6d8e787f7e114684c6a5e1eb"></a>
## default

`function` · `opentelemetry::trace::tracer::SpanBuilder::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> SpanBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [241, 24], "end": [241, 31], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/tracer.rs:241`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-39e940c8a20cdcd99a6deda1"></a>
## end_time

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::end_time` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
end_time: Option<std::time::SystemTime>
```

Source: `src/trace/tracer.rs:259`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span end time

<a id="op-1bc1e4e8b98af49778120b44"></a>
## events

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::events` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
events: Option<Vec<trace::Event>>
```

Source: `src/trace/tracer.rs:268`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span events

<a id="op-c8fb81aa5ad61e037d041326"></a>
## fmt

`function` · `opentelemetry::trace::tracer::SpanBuilder::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [241, 17], "end": [241, 22], "filename": "src/trace/tracer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/tracer.rs:241`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8bfe023943363f6ffea20e6e"></a>
## from_name

`function` · `opentelemetry::trace::tracer::SpanBuilder::from_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_name<T: Into<Cow<'static, str>>>(name: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:283`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new span builder from a span name

<a id="op-03bd47e034f01a75154e9d39"></a>
## links

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::links` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
links: Option<Vec<trace::Link>>
```

Source: `src/trace/tracer.rs:271`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span Links

<a id="op-115ca10ecac610f266f06883"></a>
## name

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
name: std::borrow::Cow<'static, str>
```

Source: `src/trace/tracer.rs:253`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span name

<a id="op-7f97a072643cbfbfbeedfb46"></a>
## sampling_result

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::sampling_result` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
sampling_result: Option<SamplingResult>
```

Source: `src/trace/tracer.rs:277`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sampling result

<a id="op-7878fccad83151e35b697bbe"></a>
## span_id

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::span_id` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
span_id: Option<SpanId>
```

Source: `src/trace/tracer.rs:247`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span id, useful for integrations with external tracing systems.

<a id="op-02bd64d2cdf5b188a91e8615"></a>
## span_kind

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::span_kind` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
span_kind: Option<trace::SpanKind>
```

Source: `src/trace/tracer.rs:250`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span kind

<a id="op-7812462adec34fa9f68aa93d"></a>
## start

`function` · `opentelemetry::trace::tracer::SpanBuilder::start` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn start<T: Tracer>(self, tracer: &T) -> T::Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:374`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Builds a span with the given tracer from this configuration.

<a id="op-adb1298bedb26626c820f201"></a>
## start_time

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::start_time` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
start_time: Option<std::time::SystemTime>
```

Source: `src/trace/tracer.rs:256`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span start time

<a id="op-ab11b8235f375513724d83b1"></a>
## start_with_context

`function` · `opentelemetry::trace::tracer::SpanBuilder::start_with_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn start_with_context<T: Tracer>(self, tracer: &T, parent_cx: &Context) -> T::Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:379`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Builds a span with the given tracer from this configuration and parent.

<a id="op-32cdab0310899b8371f70b75"></a>
## status

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
status: trace::Status
```

Source: `src/trace/tracer.rs:274`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Span status

<a id="op-0366ba416774a0699f3b09a6"></a>
## trace_id

`struct_field` · `opentelemetry::trace::tracer::SpanBuilder::trace_id` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trace_id: Option<TraceId>
```

Source: `src/trace/tracer.rs:244`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Trace id, useful for integrations with external tracing systems.

<a id="op-b0757997eade5ddb51cea2bb"></a>
## with_attributes

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_attributes<I>(self, attributes: I) -> Self where I: IntoIterator<Item = KeyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:333`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign span attributes from an iterable.
Providing duplicate keys will result in multiple attributes
with the same key, as there is no de-duplication performed.    

<a id="op-6884efa2fff2964a4fcb4a10"></a>
## with_end_time

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_end_time` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_end_time<T: Into<SystemTime>>(self, end_time: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:323`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign span end time

<a id="op-a9d81d24d9d1914e923c4e3c"></a>
## with_events

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_events` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_events(self, events: Vec<Event>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:344`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign events

<a id="op-affb6b2b14d6cfb787bebea2"></a>
## with_kind

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_kind` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_kind(self, span_kind: SpanKind) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:307`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign span kind

<a id="op-9a67d73624c56a431d832c05"></a>
## with_links

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_links` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_links(self, links: Vec<Link>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:352`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign links

<a id="op-5861ead2d3632238f73996f5"></a>
## with_sampling_result

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_sampling_result` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_sampling_result(self, sampling_result: SamplingResult) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:366`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign sampling result

<a id="op-1b274c7efa60bee9d0f368f4"></a>
## with_span_id

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_span_id` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_id(self, span_id: SpanId) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:299`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign span id

<a id="op-06d9b01f4d044acd315bb72a"></a>
## with_start_time

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_start_time` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_start_time<T: Into<SystemTime>>(self, start_time: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:315`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign span start time

<a id="op-c46cb579048f8ad586abc281"></a>
## with_status

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_status(self, status: Status) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:361`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Assign status code

<a id="op-e85e81adca69032c9c606fbf"></a>
## with_trace_id

`function` · `opentelemetry::trace::tracer::SpanBuilder::with_trace_id` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_trace_id(self, trace_id: TraceId) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::tracer::SpanBuilder", "path": "SpanBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [281, 1], "end": [382, 2], "filename": "src/trace/tracer.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/tracer.rs:291`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Specify trace id to use if no parent context exists
