# `opentelemetry::trace::context::TraceContextExt`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.context.TraceContextExt.json).

<a id="op-dc23835cc5d2a3b3f83ec729"></a>
## TraceContextExt

`trait` · `opentelemetry::trace::context::TraceContextExt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait TraceContextExt
```

Source: `src/trace/context.rs:226`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Methods for storing and retrieving trace data in a [`Context`](../operations/opentelemetry.context.Context.md#op-ca59114006a0744de26919a1).

See [`Context`](../operations/opentelemetry.context.Context.md#op-ca59114006a0744de26919a1) for examples of setting and retrieving the current context.

<a id="op-2e96479ee0fd23c3c992b5a9"></a>
## current_with_span

`function` · `opentelemetry::trace::context::TraceContextExt::current_with_span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn current_with_span<T: trace::Span + Send + Sync + 'static>(span: T) -> Self
```

Source: `src/trace/context.rs:246`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a clone of the current context with the included [`Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8).

# Examples

```
use opentelemetry::{global, trace::{TraceContextExt, Tracer}, Context};

let tracer = global::tracer("example");

// build a span
let span = tracer.start("parent_span");

// create a new context from the currently active context that includes this span
let cx = Context::current_with_span(span);

// create a child span by explicitly specifying the parent context
let child = tracer.start_with_context("child_span", &cx);
# drop(child)
```

<a id="op-18e37763cef0802167b57219"></a>
## has_active_span

`function` · `opentelemetry::trace::context::TraceContextExt::has_active_span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn has_active_span(&self) -> bool
```

Source: `src/trace/context.rs:293`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns whether or not an active span has been set.

# Examples

```
use opentelemetry::{trace::TraceContextExt, Context};

assert!(!Context::map_current(|cx| cx.has_active_span()));
```

<a id="op-0f2672d0e15f49de49acf25d"></a>
## span

`function` · `opentelemetry::trace::context::TraceContextExt::span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span(&self) -> SpanRef<'_>
```

Source: `src/trace/context.rs:282`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a reference to this context's span, or the default no-op span if
none has been set.

# Examples

```
use opentelemetry::{trace::TraceContextExt, Context};

// Add an event to the currently active span
Context::map_current(|cx| cx.span().add_event("An event!", vec![]));
```

<a id="op-77be189c28a5907c9705b3f7"></a>
## with_remote_span_context

`function` · `opentelemetry::trace::context::TraceContextExt::with_remote_span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_remote_span_context(&self, span_context: trace::SpanContext) -> Self
```

Source: `src/trace/context.rs:298`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a copy of this context with the span context included.

This is useful for building propagators.

<a id="op-38bc49a4a016431f743b3d1b"></a>
## with_span

`function` · `opentelemetry::trace::context::TraceContextExt::with_span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span<T: trace::Span + Send + Sync + 'static>(&self, span: T) -> Self
```

Source: `src/trace/context.rs:269`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a clone of this context with the included span.

# Examples

```
use opentelemetry::{global, trace::{TraceContextExt, Tracer}, Context};

fn fn_with_passed_in_context(cx: &Context) {
    let tracer = global::tracer("example");

    // build a span
    let span = tracer.start("parent_span");

    // create a new context from the given context that includes the span
    let cx_with_parent = cx.with_span(span);

    // create a child span by explicitly specifying the parent context
    let child = tracer.start_with_context("child_span", &cx_with_parent);
    # drop(child)
}

