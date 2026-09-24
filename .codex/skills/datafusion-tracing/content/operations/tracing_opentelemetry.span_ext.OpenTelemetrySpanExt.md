# `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_opentelemetry.span_ext.OpenTelemetrySpanExt.json).

<a id="op-0b43e4d9637e605ccf00e66e"></a>
## OpenTelemetrySpanExt

`trait` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
trait OpenTelemetrySpanExt
```

Source: `src/span_ext.rs:17`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Utility functions to allow tracing [`Span`]s to accept and return
[OpenTelemetry] [`Context`]s.

[`Span`]: tracing::Span
[OpenTelemetry]: https://opentelemetry.io
[`Context`]: opentelemetry::Context

<a id="op-71a6f14002acf5aef837264a"></a>
## add_event

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::add_event` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event(&self, name: impl Into<Cow<'static, str>>, attributes: Vec<KeyValue>)
```

Source: `src/span_ext.rs:199`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Adds an OpenTelemetry event directly to this span, bypassing `tracing::event!`.
This allows for adding events with dynamic attribute keys, similar to `set_attribute` for span attributes.
Events are added with the current timestamp.

# Examples

```rust
use opentelemetry::{KeyValue};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing::Span;

let app_root = tracing::span!(tracing::Level::INFO, "processing_request");

let dynamic_attrs = vec![
    KeyValue::new("job_id", "job-123"),
    KeyValue::new("user.id", "user-xyz"),
];

// Add event using the extension method
app_root.add_event("job_started".to_string(), dynamic_attrs);

// ... perform work ...

app_root.add_event("job_completed", vec![KeyValue::new("status", "success")]);
```

<a id="op-512c21d5a86c168fcc1e076e"></a>
## add_event_with_timestamp

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::add_event_with_timestamp` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event_with_timestamp(&self, name: impl Into<Cow<'static, str>>, timestamp: SystemTime, attributes: Vec<KeyValue>)
```

Source: `src/span_ext.rs:221`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Adds an OpenTelemetry event with a specific timestamp directly to this span.
Similar to `add_event`, but allows overriding the event timestamp.

# Examples

```rust
use opentelemetry::{KeyValue};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing::Span;
use std::time::{Duration, SystemTime};
use std::borrow::Cow;

let app_root = tracing::span!(tracing::Level::INFO, "historical_event_processing");

let event_time = SystemTime::now() - Duration::from_secs(60);
let event_attrs = vec![KeyValue::new("record_id", "rec-456")];
let event_name: Cow<'static, str> = "event_from_past".into();

app_root.add_event_with_timestamp(event_name, event_time, event_attrs);
```

<a id="op-266ccbf4eefdf92d340f1de9"></a>
## add_link

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::add_link` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link(&self, cx: SpanContext)
```

Source: `src/span_ext.rs:99`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Associates `self` with a given OpenTelemetry trace, using the provided
followed span [`SpanContext`].

[`SpanContext`]: opentelemetry::trace::SpanContext

# Examples

```rust
use opentelemetry::{propagation::TextMapPropagator, trace::TraceContextExt};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use std::collections::HashMap;
use tracing::Span;

// Example carrier, could be a framework header map that impls otel's `Extractor`.
let mut carrier = HashMap::new();

// Propagator can be swapped with b3 propagator, jaeger propagator, etc.
let propagator = TraceContextPropagator::new();

// Extract otel context of linked span via the chosen propagator
let linked_span_otel_context = propagator.extract(&carrier);

// Extract the linked span context from the otel context
let linked_span_context = linked_span_otel_context.span().span_context().clone();

// Generate a tracing span as usual
let app_root = tracing::span!(tracing::Level::INFO, "app_start");

// Assign linked trace from external context
app_root.add_link(linked_span_context);

// Or if the current span has been created elsewhere:
let linked_span_context = linked_span_otel_context.span().span_context().clone();
Span::current().add_link(linked_span_context);
```

<a id="op-760f9bfe4eda9c3451079166"></a>
## add_link_with_attributes

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::add_link_with_attributes` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link_with_attributes(&self, cx: SpanContext, attributes: Vec<KeyValue>)
```

Source: `src/span_ext.rs:105`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Associates `self` with a given OpenTelemetry trace, using the provided
followed span [`SpanContext`] and attributes.

[`SpanContext`]: opentelemetry::trace::SpanContext

<a id="op-c26a88b60df82a3795be8466"></a>
## context

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::context` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn context(&self) -> Context
```

Source: `src/span_ext.rs:134`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Extracts an OpenTelemetry [`Context`] from `self`.

[`Context`]: opentelemetry::Context

# Examples

```rust
use opentelemetry::Context;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing::Span;

fn make_request(cx: Context) {
    // perform external request after injecting context
    // e.g. if the request's headers impl `opentelemetry::propagation::Injector`
    // then `propagator.inject_context(cx, request.headers_mut())`
}

// Generate a tracing span as usual
let app_root = tracing::span!(tracing::Level::INFO, "app_start");

// To include tracing context in client requests from _this_ app,
// extract the current OpenTelemetry context.
make_request(app_root.context());

// Or if the current span has been created elsewhere:
make_request(Span::current().context())
```

<a id="op-68c7c385aa30042cebe748d0"></a>
## set_attribute

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::set_attribute` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attribute(&self, key: impl Into<Key>, value: impl Into<Value>)
```

Source: `src/span_ext.rs:153`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets an OpenTelemetry attribute directly for this span, bypassing `tracing`.
If fields set here conflict with `tracing` fields, the `tracing` fields will supersede fields set with `set_attribute`.
This allows for more than 32 fields.

# Examples

```rust
use opentelemetry::Context;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing::Span;

// Generate a tracing span as usual
let app_root = tracing::span!(tracing::Level::INFO, "app_start");

// Set the `http.request.header.x_forwarded_for` attribute to `example`.
app_root.set_attribute("http.request.header.x_forwarded_for", "example");
```

<a id="op-5cabc5950fe9bbd7a9d2a2bf"></a>
## set_parent

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::set_parent` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_parent(&self, cx: Context) -> Result<(), SetParentError>
```

Source: `src/span_ext.rs:61`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Associates `self` with a given OpenTelemetry trace, using the provided
parent [`Context`].

This method exists primarily to make it possible to inject a _distributed_ incoming
context, e.g. span IDs, etc.

A span's parent should only be set _once_, for the purpose described above.
Additionally, once a span has been fully built - and the SpanBuilder has been
consumed - the parent _cannot_ be mutated.

This method provides error handling for cases where the span context
cannot be set, such as when the OpenTelemetry layer is not present
or when the span has already been started.

[`Context`]: opentelemetry::Context

# Examples

```rust
use opentelemetry::{propagation::TextMapPropagator, trace::TraceContextExt};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use std::collections::HashMap;
use tracing::Span;

// Example carrier, could be a framework header map that impls otel's `Extractor`.
let mut carrier = HashMap::new();

// Propagator can be swapped with b3 propagator, jaeger propagator, etc.
let propagator = TraceContextPropagator::new();

// Extract otel parent context via the chosen propagator
let parent_context = propagator.extract(&carrier);

// Generate a tracing span as usual
let app_root = tracing::span!(tracing::Level::INFO, "app_start");

// Assign parent trace from external context
let _ = app_root.set_parent(parent_context.clone());

// Or if the current span has been created elsewhere:
let _ = Span::current().set_parent(parent_context);
```

<a id="op-b2331a746389a9cac01f771b"></a>
## set_status

`function` · `tracing_opentelemetry::span_ext::OpenTelemetrySpanExt::set_status` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_status(&self, status: Status)
```

Source: `src/span_ext.rs:172`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets an OpenTelemetry status for this span.
This is useful for setting the status of a span that was created by a library that does not declare
the otel.status_code field of the span in advance.

# Examples

```rust
use opentelemetry::trace::Status;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing::Span;

/// // Generate a tracing span as usual
let app_root = tracing::span!(tracing::Level::INFO, "app_start");

// Set the Status of the span to `Status::Ok`.
app_root.set_status(Status::Ok);
```            
