# `opentelemetry::trace::context::SpanRef`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.context.SpanRef.json).

<a id="op-26afa4e0847e74cb9d170fdd"></a>
## SpanRef

`struct` · `opentelemetry::trace::context::SpanRef` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanRef<'a>
```

Source: `src/trace/context.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A reference to the currently active span in this context.

<a id="op-4ed37ec194c9baf3732a7c0c"></a>
## add_event

`function` · `opentelemetry::trace::context::SpanRef::add_event` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event<T>(&self, name: T, attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Record an event in the context this span.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-5fb3f49102f48dbf73d05ba0"></a>
## add_event_with_timestamp

`function` · `opentelemetry::trace::context::SpanRef::add_event_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event_with_timestamp<T>(&self, name: T, timestamp: std::time::SystemTime, attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:104`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Record an event with a timestamp in the context this span.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-17cce267d7aae905a73c5800"></a>
## add_link

`function` · `opentelemetry::trace::context::SpanRef::add_link` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link(&self, span_context: SpanContext, attributes: Vec<KeyValue>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:208`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Adds a [`Link`] to another [`SpanContext`](../operations/opentelemetry.trace.span_context.SpanContext.md#op-f21a0f1d9f45c44092215518).

This method allows linking the current span to another span, identified by
its `SpanContext`. Links can be used to connect spans from different traces
or within the same trace. Attributes can be attached to the link to provide
additional context or metadata.

# Arguments

* `span_context` - The `SpanContext` of the span to link to. This represents
  the target span's unique identifiers and trace information.
* `attributes` - A vector of `KeyValue` pairs that describe additional
  attributes of the link. These attributes can include any contextual
  information relevant to the link between the spans.

Note - Any [`Link`] added via this mechanism is not accessible to a `Sampler`.
It is recommended to add Links at [`Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8) creation time, rather than adding
them afterwards.

[`Link`]: crate::trace::Link

<a id="op-790ba092b0a26c1d4851dff1"></a>
## end

`function` · `opentelemetry::trace::context::SpanRef::end` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end(&self)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:213`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Signals that the operation described by this span has now ended.

<a id="op-3583b79a8cd0996b7efd82e8"></a>
## end_with_timestamp

`function` · `opentelemetry::trace::context::SpanRef::end_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end_with_timestamp(&self, timestamp: std::time::SystemTime)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:218`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Signals that the operation described by this span ended at the given time.

<a id="op-f7d491e9ad85f3549cd71fbd"></a>
## fmt

`function` · `opentelemetry::trace::context::SpanRef::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 10], "end": [18, 15], "filename": "src/trace/context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/context.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-08f0845f335f97fb2537c49c"></a>
## is_recording

`function` · `opentelemetry::trace::context::SpanRef::is_recording` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_recording(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:132`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns `true` if this span is recording information.

Spans will not be recording information after they have ended.

This flag may be `true` despite the entire trace being sampled out. This
allows recording and processing of information about the individual
spans without sending it to the backend. An example of this scenario may
be recording and processing of all incoming requests for the processing
and building of SLA/SLO latency charts while sending only a subset -
sampled spans - to the backend.

<a id="op-d32b9946b8f2d24248e0a2a2"></a>
## record_error

`function` · `opentelemetry::trace::context::SpanRef::record_error` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn record_error(&self, err: &dyn Error)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:92`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Record an error as an event for this span.

An additional call to [Span::set_status](../operations/opentelemetry.trace.span.Span.md#op-34e9986133a52903c09bfdb7) is required if the status of the
span should be set to error, as this method does not change the span status.

If this span is not being recorded then this method does nothing.

<a id="op-f3569ad4c8d6e5063bff8f79"></a>
## set_attribute

`function` · `opentelemetry::trace::context::SpanRef::set_attribute` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attribute(&self, attribute: KeyValue)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:151`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set an attribute of this span.

Setting an attribute with the same key as an existing attribute
generally overwrites the existing attribute's value.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-4d322bcbd75999e09c5612f7"></a>
## set_attributes

`function` · `opentelemetry::trace::context::SpanRef::set_attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attributes(&self, attributes: impl IntoIterator<Item = KeyValue>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:166`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set multiple attributes of this span.

Setting an attribute with the same key as an existing attribute
generally overwrites the existing attribute's value.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-6b0d85da658ac64d09ad407e"></a>
## set_status

`function` · `opentelemetry::trace::context::SpanRef::set_status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_status(&self, status: Status)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:173`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the status of this `Span`.

If used, this will override the default span status, which is [`Status::Unset`](../operations/opentelemetry.trace.span.Status.md#op-52ed2013c1fc5d1bbf601a20).

<a id="op-11fa10725af62dd5d81864ad"></a>
## span_context

`function` · `opentelemetry::trace::context::SpanRef::span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_context(&self) -> &SpanContext
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:118`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A reference to the [`SpanContext`](../operations/opentelemetry.trace.span_context.SpanContext.md#op-f21a0f1d9f45c44092215518) for this span.

<a id="op-9606d612f275f913acf0506b"></a>
## update_name

`function` · `opentelemetry::trace::context::SpanRef::update_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn update_name<T>(&self, new_name: T) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "opentelemetry::trace::context::SpanRef", "path": "SpanRef"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [221, 2], "filename": "src/trace/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/context.rs:181`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Updates the span's name.

After this update, any sampling behavior based on the name will depend on
the implementation.
