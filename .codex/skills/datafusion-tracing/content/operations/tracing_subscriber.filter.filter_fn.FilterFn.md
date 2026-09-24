# `tracing_subscriber::filter::filter_fn::FilterFn`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.filter_fn.FilterFn.json).

<a id="op-eddd2f4a151c93eddd276ba9"></a>
## FilterFn

`struct` · `tracing_subscriber::filter::filter_fn::FilterFn` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct FilterFn<F = fn(&tracing_core::Metadata<'_>) -> bool>
```

Source: `src/filter/filter_fn.rs:25`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A filter implemented by a closure or function pointer that
determines whether a given span or event is enabled, based on its
[`Metadata`].

This type can be used for both [per-layer filtering][plf] (using its
[`Filter`] implementation) and [global filtering][global] (using its
[`Layer`] implementation).

See the [documentation on filtering with layers][filtering] for details.

[`Metadata`]: tracing_core::Metadata
[`Filter`]: crate::layer::Filter
[`Layer`]: crate::layer::Layer
[plf]: crate::layer#per-layer-filtering
[global]: crate::layer#global-filtering
[filtering]: crate::layer#filtering-with-layers

<a id="op-22810fdfd1dc9c5eb4da1d83"></a>
## callsite_enabled

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::callsite_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite_enabled(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [709, 5], "end": [724, 6], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/filter_fn.rs:717`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a03f861f79e2e652303578c2"></a>
## clone

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> FilterFn<F>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [24, 10], "end": [24, 15], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/filter_fn.rs:24`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-30ecf91be863ed225bb728b8"></a>
## enabled

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [324, 1], "end": [340, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/filter_fn.rs:329`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-84a1ca769717c0757454b021"></a>
## enabled

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, _: &Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [709, 5], "end": [724, 6], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/filter_fn.rs:713`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-77219b4be5435470da1b9a39"></a>
## fmt

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [351, 1], "end": [358, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/filter_fn.rs:352`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d248f8e25a791ac51d17ae1d"></a>
## from

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from(enabled: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [342, 1], "end": [349, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/filter/filter_fn.rs:346`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-47404af9b639da8f63fdd6ab"></a>
## max_level_hint

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [709, 5], "end": [724, 6], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/filter_fn.rs:721`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-79218dfdbf7d920b8a86d941"></a>
## new

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(enabled: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [182, 1], "end": [322, 2], "filename": "src/filter/filter_fn.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/filter_fn.rs:225`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Constructs a [`FilterFn`](../operations/tracing_subscriber.filter.filter_fn.FilterFn.md#op-eddd2f4a151c93eddd276ba9) from a function or closure that returns `true`
if a span or event should be enabled, based on its [`Metadata`].

If determining whether a span or event should be enabled also requires
information about the current span context, use [`DynFilterFn`](../operations/tracing_subscriber.filter.filter_fn.DynFilterFn.md#op-f7f87b3a5d579fb3147cc3bf) instead.

See the [documentation on per-layer filtering][plf] for details on using
[`Filter`]s.

[`Filter`]: crate::layer::Filter
[plf]: crate::layer#per-layer-filtering
[`Metadata`]: tracing_core::Metadata

# Examples

```
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    filter::FilterFn,
    util::SubscriberInitExt,
};

let my_filter = FilterFn::new(|metadata| {
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

<a id="op-22edb30b1464c7eb7db44ea3"></a>
## register_callsite

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [324, 1], "end": [340, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/filter_fn.rs:333`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5e9adf70086b870c0917951a"></a>
## with_max_level_hint

`function` · `tracing_subscriber::filter::filter_fn::FilterFn::with_max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_level_hint(self, max_level_hint: impl Into<LevelFilter>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::FilterFn", "path": "FilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [182, 1], "end": [322, 2], "filename": "src/filter/filter_fn.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/filter_fn.rs:269`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the highest verbosity [`Level`] the filter function will enable.

The value passed to this method will be returned by this `FilterFn`'s
[`Filter::max_level_hint`] method.

If the provided function will not enable all levels, it is recommended
to call this method to configure it with the most verbose level it will
enable.

# Examples

```
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    filter::{filter_fn, LevelFilter},
    util::SubscriberInitExt,
};
use tracing_core::Level;

let my_filter = filter_fn(|metadata| {
    // Only enable spans or events with targets starting with `my_crate`
    // and levels at or below `INFO`.
    metadata.level() <= &Level::INFO && metadata.target().starts_with("my_crate")
})
    // Since the filter closure will only enable the `INFO` level and
    // below, set the max level hint
    .with_max_level_hint(LevelFilter::INFO);

let my_layer = tracing_subscriber::fmt::layer();

tracing_subscriber::registry()
    .with(my_layer.with_filter(my_filter))
    .init();
```

[`Level`]: tracing_core::Level
[`Filter::max_level_hint`]: crate::layer::Filter::max_level_hint
