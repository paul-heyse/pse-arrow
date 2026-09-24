# `opentelemetry::global::trace::BoxedSpan`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.BoxedSpan.json).

<a id="op-d16cedc92e780c22627ef120"></a>
## BoxedSpan

`struct` · `opentelemetry::global::trace::BoxedSpan` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BoxedSpan
```

Source: `src/global/trace.rs:156`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Wraps the [`BoxedTracer`](../operations/opentelemetry.global.trace.BoxedTracer.md#op-716a226937c40d7602b7e516)'s [`Span`] so it can be used generically by
applications without knowing the underlying type.

[`Span`]: crate::trace::Span

<a id="op-ddd7e91224b7c686b7df16aa"></a>
## add_event_with_timestamp

`function` · `opentelemetry::global::trace::BoxedSpan::add_event_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event_with_timestamp<T>(&mut self, name: T, timestamp: SystemTime, attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:179`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Records events at a specific time in the context of a given `Span`.

Note that the OpenTelemetry project documents certain ["standard event names and
keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
which have prescribed semantic meanings.

<a id="op-7d99af8c5ebe35781fa85d0d"></a>
## add_link

`function` · `opentelemetry::global::trace::BoxedSpan::add_link` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link(&mut self, span_context: trace::SpanContext, attributes: Vec<KeyValue>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:227`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Adds a link to this span


<a id="op-66129486da936db589be021e"></a>
## end_with_timestamp

`function` · `opentelemetry::global::trace::BoxedSpan::end_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end_with_timestamp(&mut self, timestamp: SystemTime)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:232`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Finishes the span with given timestamp.

<a id="op-8a94b4cea95a7f04b3fa85b8"></a>
## fmt

`function` · `opentelemetry::global::trace::BoxedSpan::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [167, 1], "end": [171, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/global/trace.rs:168`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-907e80b6a908f271cb21d140"></a>
## is_recording

`function` · `opentelemetry::global::trace::BoxedSpan::is_recording` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_recording(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:198`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns true if this `Span` is recording information like events with the `add_event`
operation, attributes using `set_attributes`, status with `set_status`, etc.

<a id="op-5f5ab6ee0dea15348eb7cacb"></a>
## set_attribute

`function` · `opentelemetry::global::trace::BoxedSpan::set_attribute` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attribute(&mut self, attribute: KeyValue)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:207`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets a single `Attribute` where the attribute properties are passed as arguments.

Note that the OpenTelemetry project documents certain ["standard
attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
that have prescribed semantic meanings.

<a id="op-cb32ee756698f8bbb0435d92"></a>
## set_status

`function` · `opentelemetry::global::trace::BoxedSpan::set_status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_status(&mut self, status: trace::Status)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:213`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the status of the `Span`. If used, this will override the default `Span`
status, which is `Unset`.

<a id="op-13e7d5c6178fd6fd048dfbd7"></a>
## span_context

`function` · `opentelemetry::global::trace::BoxedSpan::span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_context(&self) -> &trace::SpanContext
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:192`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the `SpanContext` for the given `Span`.

<a id="op-565114d5ebb18b057d0b0b7d"></a>
## update_name

`function` · `opentelemetry::global::trace::BoxedSpan::update_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn update_name<T>(&mut self, new_name: T) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::global::trace::BoxedSpan", "path": "BoxedSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [173, 1], "end": [235, 2], "filename": "src/global/trace.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/global/trace.rs:218`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Updates the `Span`'s name.
