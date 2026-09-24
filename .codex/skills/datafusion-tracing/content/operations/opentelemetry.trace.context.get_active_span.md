# `opentelemetry::trace::context::get_active_span`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.context.get_active_span.json).

<a id="op-99c7c9f3027b0f6e1eceb0da"></a>
## get_active_span

`function` · `opentelemetry::trace::context::get_active_span` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get_active_span<F, T>(f: F) -> T where F: FnOnce(SpanRef<'_>) -> T
```

Source: `src/trace/context.rs:386`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Executes a closure with a reference to this thread's current span.

# Examples

```
use opentelemetry::{global, trace::{Span, Tracer}, KeyValue};
use opentelemetry::trace::get_active_span;

fn my_function() {
    // start an active span in one function
    global::tracer("my-component").in_span("span-name", |_cx| {
        // anything happening in functions we call can still access the active span...
        my_other_function();
    })
}

fn my_other_function() {
    // call methods on the current span from
    get_active_span(|span| {
        span.add_event("An event!", vec![KeyValue::new("happened", true)]);
    })
}
```
