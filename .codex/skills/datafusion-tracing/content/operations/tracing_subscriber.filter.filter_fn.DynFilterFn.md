# `tracing_subscriber::filter::filter_fn::DynFilterFn`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.filter_fn.DynFilterFn.json).

<a id="op-f7f87b3a5d579fb3147cc3bf"></a>
## DynFilterFn

`struct` · `tracing_subscriber::filter::filter_fn::DynFilterFn` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct DynFilterFn<S, F = fn(&tracing_core::Metadata<'_>, &layer::Context<'_, S>) -> bool, R = fn(&'static tracing_core::Metadata<'static>) -> tracing_core::Interest>
```

Source: `src/filter/filter_fn.rs:46`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A filter implemented by a closure or function pointer that
determines whether a given span or event is enabled _dynamically_,
potentially based on the current [span context].

This type can be used for both [per-layer filtering][plf] (using its
[`Filter`] implementation) and [global filtering][global] (using its
[`Layer`] implementation).

See the [documentation on filtering with layers][filtering] for details.

[span context]: crate::layer::Context
[`Filter`]: crate::layer::Filter
[`Layer`]: crate::layer::Layer
[plf]: crate::layer#per-layer-filtering
[global]: crate::layer#global-filtering
[filtering]: crate::layer#filtering-with-layers

<a id="op-9a08b621799792baa0a39fb4"></a>
## callsite_enabled

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::callsite_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite_enabled(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "tracing_core::Interest"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [726, 5], "end": [742, 6], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/filter_fn.rs:735`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4f30664b8e6a05f23c5c465c"></a>
## clone

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [679, 1], "end": [692, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/filter_fn.rs:684`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2feb6fd8f54d38fd370c15c2"></a>
## enabled

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, cx: &Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "tracing_core::Interest"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [726, 5], "end": [742, 6], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/filter_fn.rs:731`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8d6a0472a7f34f8b6fa7310c"></a>
## enabled

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, cx: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "tracing_core::Interest"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "R"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [643, 1], "end": [660, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/filter_fn.rs:649`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4f9f9cbc4eca91346d552086"></a>
## fmt

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [662, 1], "end": [677, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/filter_fn.rs:663`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a22f4cdbfb3b46128d3b4416"></a>
## from

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from(f: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [694, 1], "end": [701, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/filter/filter_fn.rs:698`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-654470fa0fe6d9d80ef195fc"></a>
## max_level_hint

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "tracing_core::Interest"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [726, 5], "end": [742, 6], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/filter_fn.rs:739`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-36aee896474297987eea8c44"></a>
## new

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(enabled: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [362, 1], "end": [432, 2], "filename": "src/filter/filter_fn.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/filter_fn.rs:424`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Constructs a [`Filter`] from a function or closure that returns `true`
if a span or event should be enabled in the current [span
context][`Context`].

Unlike [`FilterFn`](../operations/tracing_subscriber.filter.filter_fn.FilterFn.md#op-eddd2f4a151c93eddd276ba9), a `DynFilterFn` is constructed from a closure or
function pointer that takes both the [`Metadata`] for a span or event
*and* the current [`Context`]. This means that a [`DynFilterFn`](../operations/tracing_subscriber.filter.filter_fn.DynFilterFn.md#op-f7f87b3a5d579fb3147cc3bf) can
choose whether to enable spans or events based on information about the
_current_ span (or its parents).

If this is *not* necessary, use [`FilterFn`](../operations/tracing_subscriber.filter.filter_fn.FilterFn.md#op-eddd2f4a151c93eddd276ba9) instead.

See the [documentation on per-layer filtering][plf] for details on using
[`Filter`]s.

[`Filter`]: crate::layer::Filter
[plf]: crate::layer#per-layer-filtering
[`Context`]: crate::layer::Context
[`Metadata`]: tracing_core::Metadata

# Examples

```
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    filter::DynFilterFn,
    util::SubscriberInitExt,
};

// Only enable spans or events within a span named "interesting_span".
let my_filter = DynFilterFn::new(|metadata, cx| {
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

<a id="op-ed02e12faeebca9988f2a0bc"></a>
## register_callsite

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}], "output": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::Interest", "path": "tracing_core::Interest"}}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "R"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [643, 1], "end": [660, 2], "filename": "src/filter/filter_fn.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/filter_fn.rs:653`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-47589132231a471d118a9a9d"></a>
## with_callsite_filter

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::with_callsite_filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_callsite_filter<R2>(self, callsite_enabled: R2) -> DynFilterFn<S, F, R2> where R2: Fn(&'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [434, 1], "end": [598, 2], "filename": "src/filter/filter_fn.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/filter_fn.rs:558`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Adds a function for filtering callsites to this filter.

When this filter's [`Filter::callsite_enabled`][cse] method is called,
the provided function will be used rather than the default.

By default, `DynFilterFn` assumes that, because the filter _may_ depend
dynamically on the current [span context], its result should never be
cached. However, some filtering strategies may require dynamic information
from the current span context in *some* cases, but are able to make
static filtering decisions from [`Metadata`] alone in others.

For example, consider the filter given in the example for
[`DynFilterFn::new`](../operations/tracing_subscriber.filter.filter_fn.DynFilterFn.md#op-36aee896474297987eea8c44). That filter enables all spans named
"interesting_span", and any events and spans that occur inside of an
interesting span. Since the span's name is part of its static
[`Metadata`], the "interesting_span" can be enabled in
[`callsite_enabled`][cse]:

```
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    filter::DynFilterFn,
    util::SubscriberInitExt,
};
use tracing_core::subscriber::Interest;

// Only enable spans or events within a span named "interesting_span".
let my_filter = DynFilterFn::new(|metadata, cx| {
    // If this *is* "interesting_span", make sure to enable it.
    if metadata.is_span() && metadata.name() == "interesting_span" {
        return true;
    }

    // Otherwise, are we in an interesting span?
    if let Some(current_span) = cx.lookup_current() {
        return current_span.name() == "interesting_span";
    }

    false
}).with_callsite_filter(|metadata| {
    // If this is an "interesting_span", we know we will always
    // enable it.
    if metadata.is_span() && metadata.name() == "interesting_span" {
        return Interest::always();
    }

    // Otherwise, it depends on whether or not we're in an interesting
    // span. You'll have to ask us again for each span/event!
    Interest::sometimes()
});

let my_layer = tracing_subscriber::fmt::layer();

tracing_subscriber::registry()
    .with(my_layer.with_filter(my_filter))
    .init();
```

[cse]: crate::layer::Filter::callsite_enabled
[`enabled`]: crate::layer::Filter::enabled
[`Metadata`]: tracing_core::Metadata
[span context]: crate::layer::Context

<a id="op-3f65ce32ab3ad5076203851a"></a>
## with_max_level_hint

`function` · `tracing_subscriber::filter::filter_fn::DynFilterFn::with_max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_level_hint(self, max_level_hint: impl Into<LevelFilter>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "F"}}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::filter::filter_fn::DynFilterFn", "path": "DynFilterFn"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"parenthesized": {"inputs": [{"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::metadata::Metadata", "path": "tracing_core::Metadata"}}}}, {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "crate::layer::Context"}}}}], "output": {"primitive": "bool"}}}, "id": "core::ops::function::Fn", "path": "Fn"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [434, 1], "end": [598, 2], "filename": "src/filter/filter_fn.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/filter_fn.rs:489`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the highest verbosity [`Level`] the filter function will enable.

The value passed to this method will be returned by this `DynFilterFn`'s
[`Filter::max_level_hint`] method.

If the provided function will not enable all levels, it is recommended
to call this method to configure it with the most verbose level it will
enable.

# Examples

```
use tracing_subscriber::{
    layer::{Layer, SubscriberExt},
    filter::{DynFilterFn, LevelFilter},
    util::SubscriberInitExt,
};
use tracing_core::Level;

// Only enable spans or events with levels at or below `INFO`, if
// we are inside a span called "interesting_span".
let my_filter = DynFilterFn::new(|metadata, cx| {
    // If the level is greater than INFO, disable it.
    if metadata.level() > &Level::INFO {
        return false;
    }

    // If any span in the current scope is named "interesting_span",
    // enable this span or event.
    for span in cx.lookup_current().iter().flat_map(|span| span.scope()) {
        if span.name() == "interesting_span" {
            return true;
         }
    }

    // Otherwise, disable it.
    false
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
