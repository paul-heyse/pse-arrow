# `opentelemetry::global::trace::ObjectSafeSpan`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.global.trace.ObjectSafeSpan.json).

<a id="op-774b90e87368f0e79025d926"></a>
## ObjectSafeSpan

`trait` · `opentelemetry::global::trace::ObjectSafeSpan` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait ObjectSafeSpan
```

Source: `src/global/trace.rs:11`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Allows a specific [`crate::trace::Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8) to be used generically by [`BoxedSpan`](../operations/opentelemetry.global.trace.BoxedSpan.md#op-d16cedc92e780c22627ef120)
instances by mirroring the interface and boxing the return types.

<a id="op-319fb73c3cf0b703cd51e48c"></a>
## add_event_with_timestamp

`function` · `opentelemetry::global::trace::ObjectSafeSpan::add_event_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event_with_timestamp(&mut self, name: Cow<'static, str>, timestamp: SystemTime, attributes: Vec<KeyValue>)
```

Source: `src/global/trace.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An API to record events at a specific time in the context of a given `Span`.

Events SHOULD preserve the order in which they're set. This will typically match
the ordering of the events' timestamps.

Note that the OpenTelemetry project documents certain ["standard event names and
keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
which have prescribed semantic meanings.

<a id="op-ecccd740bd392634a8a672e2"></a>
## add_link

`function` · `opentelemetry::global::trace::ObjectSafeSpan::add_link` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link(&mut self, span_context: SpanContext, attributes: Vec<KeyValue>)
```

Source: `src/global/trace.rs:89`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Adds a link to this span


<a id="op-9bce1bb82601bca73d965162"></a>
## end

`function` · `opentelemetry::global::trace::ObjectSafeSpan::end` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end(&mut self)
```

Source: `src/global/trace.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Finishes the `Span`.

Implementations MUST ignore all subsequent calls to `end` (there might be
exceptions when the tracer is streaming events and has no mutable state
associated with the Span).

Calls to `end` a Span MUST not have any effects on child `Span`s as they may
still be running and can be ended later.

This API MUST be non-blocking.

<a id="op-88789f6a782341617166a564"></a>
## end_with_timestamp

`function` · `opentelemetry::global::trace::ObjectSafeSpan::end_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end_with_timestamp(&mut self, timestamp: SystemTime)
```

Source: `src/global/trace.rs:110`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Finishes the `Span` with given timestamp

For more details, refer to [`Span::end`]

[`Span::end`]: trace::Span::end

<a id="op-f220c087ecffe0a26fb8a3df"></a>
## is_recording

`function` · `opentelemetry::global::trace::ObjectSafeSpan::is_recording` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_recording(&self) -> bool
```

Source: `src/global/trace.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns true if this `Span` is recording information like events with the `add_event`
operation, attributes using `set_attributes`, status with `set_status`, etc.

This flag SHOULD be used to avoid expensive computations of a `Span` attributes or events in
case when a `Span` is definitely not recorded. Note that any child span's recording is
determined independently from the value of this flag (typically based on the sampled flag of
a `TraceFlag` on `SpanContext`).

This flag may be true despite the entire trace being sampled out. This allows to record and
process information about the individual Span without sending it to the backend. An example
of this scenario may be recording and processing of all incoming requests for the processing
and building of SLA/SLO latency charts while sending only a subset - sampled spans - to the
backend. See also the sampling section of SDK design.

Users of the API should only access the `is_recording` property when instrumenting code and
never access `SampledFlag` unless used in context propagators.

<a id="op-56db1dfe186018cf8ac13ccd"></a>
## set_attribute

`function` · `opentelemetry::global::trace::ObjectSafeSpan::set_attribute` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attribute(&mut self, attribute: KeyValue)
```

Source: `src/global/trace.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An API to set a single `Attribute` where the attribute properties are passed
as arguments. To avoid extra allocations some implementations may offer a separate API for
each of the possible value types.

An `Attribute` is defined as a `KeyValue` pair.

Attributes SHOULD preserve the order in which they're set. Setting an attribute
with the same key as an existing attribute SHOULD overwrite the existing
attribute's value.

Note that the OpenTelemetry project documents certain ["standard
attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
that have prescribed semantic meanings.

<a id="op-854311e08af8f52f5d429934"></a>
## set_status

`function` · `opentelemetry::global::trace::ObjectSafeSpan::set_status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_status(&mut self, status: Status)
```

Source: `src/global/trace.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the status of the `Span`. `message` MUST be ignored when the status is `OK` or
`Unset`.

The order of status is `Ok` > `Error` > `Unset`. That's means set the status
to `Unset` will always be ignore, set the status to `Error` only works when current
status is `Unset`, set the status to `Ok` will be consider final and any further call
to this function will be ignore.

<a id="op-03ff163097b3bc948c93ba8a"></a>
## span_context

`function` · `opentelemetry::global::trace::ObjectSafeSpan::span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_context(&self) -> &SpanContext
```

Source: `src/global/trace.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the `SpanContext` for the given `Span`. The returned value may be used even after
the `Span is finished. The returned value MUST be the same for the entire `Span` lifetime.

<a id="op-788e25710e12244907c108c9"></a>
## update_name

`function` · `opentelemetry::global::trace::ObjectSafeSpan::update_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn update_name(&mut self, new_name: Cow<'static, str>)
```

Source: `src/global/trace.rs:85`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Updates the `Span`'s name. After this update, any sampling behavior based on the
name will depend on the implementation.

It is highly discouraged to update the name of a `Span` after its creation.
`Span` name is often used to group, filter and identify the logical groups of
spans. Often, filtering logic will be implemented before the `Span` creation
for performance reasons, and the name update may interfere with this logic.

The method name is called `update_name` to differentiate this method from the
regular property. It emphasizes that this operation signifies a
major change for a `Span` and may lead to re-calculation of sampling or
filtering decisions made previously depending on the implementation.
