# `tracing_subscriber::filter::filter_fn::dynamic_filter_fn`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.filter_fn.dynamic_filter_fn.json).

<a id="op-0350fe07950fac7bea247a51"></a>
## dynamic_filter_fn

`function` · `tracing_subscriber::filter::filter_fn::dynamic_filter_fn` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn dynamic_filter_fn<S, F>(f: F) -> DynFilterFn<S, F> where F: Fn(&tracing_core::Metadata<'_>, &layer::Context<'_, S>) -> bool
```

Source: `src/filter/filter_fn.rs:175`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Constructs a [`DynFilterFn`](../operations/tracing_subscriber.filter.filter_fn.DynFilterFn.md#op-f7f87b3a5d579fb3147cc3bf) from a function or closure that returns `true`
if a span or event should be enabled within a particular [span context][`Context`].

This is equivalent to calling [`DynFilterFn::new`](../operations/tracing_subscriber.filter.filter_fn.DynFilterFn.md#op-36aee896474297987eea8c44).

Unlike [`filter_fn`](../operations/tracing_subscriber.filter.filter_fn.filter_fn.md#op-d60a8bd9283e60182f495969), this function takes a closure or function pointer
taking the [`Metadata`] for a span or event *and* the current [`Context`].
This means that a [`DynFilterFn`](../operations/tracing_subscriber.filter.filter_fn.DynFilterFn.md#op-f7f87b3a5d579fb3147cc3bf) can choose whether to enable spans or
events based on information about the _current_ span (or its parents).

If this is *not* necessary, use [`filter_fn`](../operations/tracing_subscriber.filter.filter_fn.filter_fn.md#op-d60a8bd9283e60182f495969) instead.

The returned [`DynFilterFn`](../operations/tracing_subscriber.filter.filter_fn.DynFilterFn.md#op-f7f87b3a5d579fb3147cc3bf) can be used for both [per-layer filtering][plf]
(using its [`Filter`] implementation) and [global filtering][global] (using
its  [`Layer`] implementation).

See the [documentation on filtering with layers][filtering] for details.

# Examples

```
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    filter,
    util::SubscriberInitExt,
};

// Only enable spans or events within a span named "interesting_span".
let my_filter = filter::dynamic_filter_fn(|metadata, cx| {
    // If this *is* "interesting_span", make sure to enable it.
    if metadata.is_span() && metadata.name() == "interesting_span" {
        return true;
    }

    // Otherwise, are we in an interesting span?
    if let Some(current_span) = cx.lookup_current() {
        return current_span.name() == "interesting_span";
    }

    false
});

let my_layer = tracing_subscriber::fmt::layer();

tracing_subscriber::registry()
    .with(my_layer.with_filter(my_filter))
    .init();

// This event will not be enabled.
tracing::info!("something happened");

tracing::info_span!("interesting_span").in_scope(|| {
    // This event will be enabled.
    tracing::debug!("something else happened");
});
```

[`Filter`]: crate::layer::Filter
[`Layer`]: crate::layer::Layer
[plf]: crate::layer#per-layer-filtering
[global]: crate::layer#global-filtering
[filtering]: crate::layer#filtering-with-layers
[`Context`]: crate::layer::Context
[`Metadata`]: tracing_core::Metadata
