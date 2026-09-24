# `tracing_subscriber::filter::filter_fn::filter_fn`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.filter_fn.filter_fn.json).

<a id="op-d60a8bd9283e60182f495969"></a>
## filter_fn

`function` · `tracing_subscriber::filter::filter_fn::filter_fn` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn filter_fn<F>(f: F) -> FilterFn<F> where F: Fn(&tracing_core::Metadata<'_>) -> bool
```

Source: `src/filter/filter_fn.rs:104`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Constructs a [`FilterFn`](../operations/tracing_subscriber.filter.filter_fn.FilterFn.md#op-eddd2f4a151c93eddd276ba9), from a function or closure that returns `true` if
a span or event should be enabled, based on its [`Metadata`].

The returned [`FilterFn`](../operations/tracing_subscriber.filter.filter_fn.FilterFn.md#op-eddd2f4a151c93eddd276ba9) can be used for both [per-layer filtering][plf]
(using its [`Filter`] implementation) and [global filtering][global] (using
its  [`Layer`] implementation).

See the [documentation on filtering with layers][filtering] for details.

This is equivalent to calling [`FilterFn::new`](../operations/tracing_subscriber.filter.filter_fn.FilterFn.md#op-79218dfdbf7d920b8a86d941).

[`Metadata`]: tracing_core::Metadata
[`Filter`]: crate::layer::Filter
[`Layer`]: crate::layer::Layer
[plf]: crate::layer#per-layer-filtering
[global]: crate::layer#global-filtering
[filtering]: crate::layer#filtering-with-layers

# Examples

```
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    filter,
    util::SubscriberInitExt,
};

let my_filter = filter::filter_fn(|metadata| {
    // Only enable spans or events with the target "interesting_things"
    metadata.target() == "interesting_things"
});

let my_layer = tracing_subscriber::fmt::layer();

tracing_subscriber::registry()
    .with(my_layer.with_filter(my_filter))
    .init();

// This event will not be enabled.
tracing::warn!("something important but uninteresting happened!");

// This event will be enabled.
tracing::debug!(target: "interesting_things", "an interesting minor detail...");
```
