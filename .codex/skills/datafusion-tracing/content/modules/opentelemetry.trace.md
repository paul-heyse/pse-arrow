# `opentelemetry::trace`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.json).

<a id="op-860b44a9d0353bc997e5d841"></a>
## trace

`module` · `opentelemetry::trace` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod trace
```

Source: `src/trace/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

API for tracing applications and libraries.

The `trace` module includes types for tracking the progression of a single
request while it is handled by services that make up an application. A trace
is a tree of [`Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8)s which are objects that represent the work being done
by individual services or components involved in a request as it flows
through a system. This module implements the OpenTelemetry [trace
specification].

[trace specification]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/api.md

## Getting Started

In application code:

```
use opentelemetry::trace::{Tracer, noop::NoopTracerProvider};
use opentelemetry::global;

fn init_tracer() {
    // Swap this no-op provider for your tracing service of choice (jaeger, zipkin, etc)
    let provider = NoopTracerProvider::new();

    // Configure the global `TracerProvider` singleton when your app starts
    // (there is a no-op default if this is not set by your application)
    let _ = global::set_tracer_provider(provider);
}

fn do_something_tracked() {
    // Then you can get a named tracer instance anywhere in your codebase.
    let tracer = global::tracer("my-component");

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });
}

// in main or other app start
init_tracer();
do_something_tracked();
```

In library code:

```
use opentelemetry::{global, trace::{Span, Tracer, TracerProvider}};
use opentelemetry::InstrumentationScope;
use std::sync::Arc;

fn my_library_function() {
    // Use the global tracer provider to get access to the user-specified
    // tracer configuration
    let tracer_provider = global::tracer_provider();

    // Get a tracer for this library
    let scope = InstrumentationScope::builder("my_name")
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
        .build();

    let tracer = tracer_provider.tracer_with_scope(scope);

    // Create spans
    let mut span = tracer.start("doing_work");

    // Do work...

    // End the span
    span.end();
}
```

## Overview

The tracing API consists of a three main traits:

* [`TracerProvider`](../operations/opentelemetry.trace.tracer_provider.TracerProvider.md#op-3d14c743f48c20c76241bcf3)s are the entry point of the API. They provide access to
  `Tracer`s.
* [`Tracer`](../operations/opentelemetry.trace.tracer.Tracer.md#op-a87fa61076cf2b7384423932)s are types responsible for creating `Span`s.
* [`Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8)s provide the API to trace an operation.

## Working with Async Runtimes

Exporting spans often involves sending data over a network or performing
other I/O tasks. OpenTelemetry allows you to schedule these tasks using
whichever runtime you are already using such as [Tokio].
When using an async runtime it's best to use the batch span processor
where the spans will be sent in batches as opposed to being sent once ended,
which often ends up being more efficient.

[Tokio]: https://tokio.rs

## Managing Active Spans

Spans can be marked as "active" for a given [`Context`], and all newly
created spans will automatically be children of the currently active span.

The active span for a given thread can be managed via [`get_active_span`](../operations/opentelemetry.trace.context.get_active_span.md#op-99c7c9f3027b0f6e1eceb0da)
and [`mark_span_as_active`](../operations/opentelemetry.trace.context.mark_span_as_active.md#op-da8fd29a0da93e7e5eb4d6d6).

[`Context`]: crate::Context

```
use opentelemetry::{global, trace::{self, Span, Status, Tracer, TracerProvider}};

fn may_error(rand: f32) {
    if rand < 0.5 {
        // Get the currently active span to record additional attributes,
        // status, etc.
        trace::get_active_span(|span| {
            span.set_status(Status::error("value too small"));
        });
    }
}

// Get a tracer
let tracer = global::tracer("my_tracer");

// Create a span
let span = tracer.start("parent_span");

// Mark the span as active
let active = trace::mark_span_as_active(span);

// Any span created here will be a child of `parent_span`...

// Drop the guard and the span will no longer be active
drop(active)
```

Additionally [`Tracer::in_span`](../operations/opentelemetry.trace.tracer.Tracer.md#op-bdb2045ac638a69960430837) can be used as shorthand to simplify
managing the parent context.

```
use opentelemetry::{global, trace::Tracer};

// Get a tracer
let tracer = global::tracer("my_tracer");

// Use `in_span` to create a new span and mark it as the parent, dropping it
// at the end of the block.
tracer.in_span("parent_span", |cx| {
    // spans created here will be children of `parent_span`
});
```

#### Async active spans

Async spans can be propagated with [`TraceContextExt`](../operations/opentelemetry.trace.context.TraceContextExt.md#op-dc23835cc5d2a3b3f83ec729) and [`FutureExt`](../operations/opentelemetry.context.future_ext.FutureExt.md#op-0b23d57a653ef8543662473d).

```
use opentelemetry::{Context, global, trace::{FutureExt, TraceContextExt, Tracer}};

async fn some_work() { }
# async fn in_an_async_context() {

// Get a tracer
let tracer = global::tracer("my_tracer");

// Start a span
let span = tracer.start("my_span");

// Perform some async work with this span as the currently active parent.
some_work().with_context(Context::current_with_span(span)).await;
# }
```
