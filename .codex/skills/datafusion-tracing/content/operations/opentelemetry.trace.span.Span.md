# `opentelemetry::trace::span::Span`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.span.Span.json).

<a id="op-7e58380e085740d966fa45c8"></a>
## Span

`trait` · `opentelemetry::trace::span::Span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait Span
```

Source: `src/trace/span.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The interface for a single operation within a trace.

Spans can be nested to form a trace tree. Each trace contains a root span,
which typically describes the entire operation and, optionally, one or more
sub-spans for its sub-operations.

The span `name` concisely identifies the work represented by the span, for
example, an RPC method name, a function name, or the name of a subtask or
stage within a larger computation. The span name should be the most general
string that identifies a (statistically) interesting class of spans, rather
than individual span instances while still being human-readable. That is,
`"get_user"` is a reasonable name, while `"get_user/314159"`, where `"314159"` is
a user ID, is not a good name due to its high cardinality. _Generality_
should be prioritized over _human-readability_.

For example, here are potential span names for an endpoint that gets a
hypothetical account information:

| Span Name         | Guidance     |
| ----------------- | ------------ |
| `get`             | Too general  |
| `get_account/42`  | Too specific |
| `get_account`     | Good, and account_id=42 would make a nice Span attribute |
| `get_account/{accountId}` | Also good (using the "HTTP route") |

The span's start and end timestamps reflect the elapsed real time of the
operation.

For example, if a span represents a request-response cycle (e.g. HTTP or an
RPC), the span should have a start time that corresponds to the start time
of the first sub-operation, and an end time of when the final sub-operation
is complete. This includes:

* receiving the data from the request
* parsing of the data (e.g. from a binary or json format)
* any middleware or additional processing logic
* business logic
* construction of the response
* sending of the response

Child spans (or in some cases events) may be created to represent
sub-operations which require more detailed observability. Child spans should
measure the timing of the respective sub-operation, and may add additional
attributes.

<a id="op-081e989fbc7405226872b7a0"></a>
## add_event

`function` · `opentelemetry::trace::span::Span::add_event` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event<T>(&mut self, name: T, attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Source: `src/trace/span.rs:59`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Record an event in the context this span.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-ceb29f172396181158adad93"></a>
## add_event_with_timestamp

`function` · `opentelemetry::trace::span::Span::add_event_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event_with_timestamp<T>(&mut self, name: T, timestamp: SystemTime, attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Source: `src/trace/span.rs:87`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Record an event with a timestamp in the context this span.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-27cd90da6e4e06cab00b2139"></a>
## add_link

`function` · `opentelemetry::trace::span::Span::add_link` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link(&mut self, span_context: SpanContext, attributes: Vec<KeyValue>)
```

Source: `src/trace/span.rs:171`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Adds [`Link`] to another [`SpanContext`](../operations/opentelemetry.trace.span_context.SpanContext.md#op-f21a0f1d9f45c44092215518).

This method allows linking the current span to another span, identified by its `SpanContext`. Links can be used
to connect spans from different traces or within the same trace. Attributes can be attached to the link to
provide additional context or metadata.

# Arguments

* `span_context` - The `SpanContext` of the span to link to. This represents the target span's unique identifiers
  and trace information.
* `attributes` - A vector of `KeyValue` pairs that describe additional attributes of the link. These attributes
  can include any contextual information relevant to the link between the spans.

[`Link`]: crate::trace::Link

<a id="op-9cdcfd7cbcf9418f60ab6b02"></a>
## end

`function` · `opentelemetry::trace::span::Span::end` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end(&mut self)
```

Source: `src/trace/span.rs:174`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Signals that the operation described by this span has now ended.

<a id="op-2ac4a8d0479b245c9a7e9a90"></a>
## end_with_timestamp

`function` · `opentelemetry::trace::span::Span::end_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end_with_timestamp(&mut self, timestamp: SystemTime)
```

Source: `src/trace/span.rs:179`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Signals that the operation described by this span ended at the given time.

<a id="op-010500dd840aee0d4261b86d"></a>
## is_recording

`function` · `opentelemetry::trace::span::Span::is_recording` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_recording(&self) -> bool
```

Source: `src/trace/span.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns `true` if this span is recording information.

Spans will not be recording information after they have ended.

This flag may be `true` despite the entire trace being sampled out. This
allows recording and processing of information about the individual
spans without sending it to the backend. An example of this scenario may
be recording and processing of all incoming requests for the processing
and building of SLA/SLO latency charts while sending only a subset -
sampled spans - to the backend.

<a id="op-a3a1a7ac4f08057418fcb4fd"></a>
## record_error

`function` · `opentelemetry::trace::span::Span::record_error` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn record_error(&mut self, err: &dyn Error)
```

Source: `src/trace/span.rs:72`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Record an error as an event for this span.

An additional call to [Span::set_status](../operations/opentelemetry.trace.span.Span.md#op-34e9986133a52903c09bfdb7) is required if the status of the
span should be set to error, as this method does not change the span status.

If this span is not being recorded then this method does nothing.

<a id="op-ac95a9373428b5d3c97bd374"></a>
## set_attribute

`function` · `opentelemetry::trace::span::Span::set_attribute` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attribute(&mut self, attribute: KeyValue)
```

Source: `src/trace/span.rs:122`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set an attribute of this span.

Setting an attribute with the same key as an existing attribute
results in both being stored as attribute, without any de-duplication
performed.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-cd3cad395d4f384410f02ab2"></a>
## set_attributes

`function` · `opentelemetry::trace::span::Span::set_attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attributes(&mut self, attributes: impl IntoIterator<Item = KeyValue>)
```

Source: `src/trace/span.rs:136`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Set multiple attributes of this span.

Setting an attribute with the same key as an existing attribute
results in both being stored as attribute, without any de-duplication
performed.

Note that the OpenTelemetry project documents certain "[standard
attributes]" that have prescribed semantic meanings and are available via
the [opentelemetry_semantic_conventions] crate.

[standard attributes]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/trace/semantic_conventions/README.md
[opentelemetry_semantic_conventions]: https://docs.rs/opentelemetry-semantic-conventions

<a id="op-34e9986133a52903c09bfdb7"></a>
## set_status

`function` · `opentelemetry::trace::span::Span::set_status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_status(&mut self, status: Status)
```

Source: `src/trace/span.rs:147`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the status of this `Span`.

If used, this will override the default span status, which is [`Status::Unset`](../operations/opentelemetry.trace.span.Status.md#op-52ed2013c1fc5d1bbf601a20).

<a id="op-df469898a74cf293da6c515d"></a>
## span_context

`function` · `opentelemetry::trace::span::Span::span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_context(&self) -> &SpanContext
```

Source: `src/trace/span.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A reference to the [`SpanContext`](../operations/opentelemetry.trace.span_context.SpanContext.md#op-f21a0f1d9f45c44092215518) for this span.

<a id="op-f5dc823eb60279e2e6f2260c"></a>
## update_name

`function` · `opentelemetry::trace::span::Span::update_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn update_name<T>(&mut self, new_name: T) where T: Into<Cow<'static, str>>
```

Source: `src/trace/span.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Updates the span's name.

After this update, any sampling behavior based on the name will depend on
the implementation.
