# `opentelemetry::trace::context::mark_span_as_active`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.context.mark_span_as_active.json).

<a id="op-da8fd29a0da93e7e5eb4d6d6"></a>
## mark_span_as_active

`function` · `opentelemetry::trace::context::mark_span_as_active` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn mark_span_as_active<T: trace::Span + Send + Sync + 'static>(span: T) -> ContextGuard
```

Source: `src/trace/context.rs:358`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Mark a given `Span` as active.

The `Tracer` MUST provide a way to update its active `Span`, and MAY provide convenience
methods to manage a `Span`'s lifetime and the scope in which a `Span` is active. When an
active `Span` is made inactive, the previously-active `Span` SHOULD be made active. A `Span`
maybe finished (i.e. have a non-null end time) but still be active. A `Span` may be active
on one thread after it has been made inactive on another.

# Examples

```
use opentelemetry::{global, trace::{Span, Tracer}, KeyValue};
use opentelemetry::trace::{get_active_span, mark_span_as_active};

fn my_function() {
    let tracer = global::tracer("my-component-a");
    // start an active span in one function
    let span = tracer.start("span-name");
    let _guard = mark_span_as_active(span);
    // anything happening in functions we call can still access the active span...
    my_other_function();
}

fn my_other_function() {
    // call methods on the current span from
    get_active_span(|span| {
        span.add_event("An event!".to_string(), vec![KeyValue::new("happened", true)]);
    });
}
```
