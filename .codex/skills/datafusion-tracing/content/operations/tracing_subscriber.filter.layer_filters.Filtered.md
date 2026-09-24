# `tracing_subscriber::filter::layer_filters::Filtered`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.layer_filters.Filtered.json).

<a id="op-54bb3c78606034b0e2ea655e"></a>
## Filtered

`struct` · `tracing_subscriber::filter::layer_filters::Filtered` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Filtered<L, F, S>
```

Source: `src/filter/layer_filters/mod.rs:60`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) that wraps an inner [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) and adds a [`Filter`] which
controls what spans and events are enabled for that layer.

This is returned by the [`Layer::with_filter`](../operations/tracing_subscriber.layer.Layer.md#op-7bd3ef10607f7c7b6ba70fdd) method. See the
[documentation on per-layer filtering][plf] for details.

[`Filter`]: crate::layer::Filter
[plf]: crate::layer#per-layer-filtering

<a id="op-c6de3321bdd4ab646b6de9aa"></a>
## clone

`function` · `tracing_subscriber::filter::layer_filters::Filtered::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Filtered<L, F, S>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [59, 10], "end": [59, 15], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/layer_filters/mod.rs:59`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-98bbb1d9ae805d788b17976f"></a>
## enabled

`function` · `tracing_subscriber::filter::layer_filters::Filtered::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, cx: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:766`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d2020eeb696fe298a9823c94"></a>
## event_enabled

`function` · `tracing_subscriber::filter::layer_filters::Filtered::event_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, event: &Event<'_>, cx: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:821`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-df4056f92443d4cfe0a16fa1"></a>
## filter

`function` · `tracing_subscriber::filter::layer_filters::Filtered::filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn filter(&self) -> &F
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [612, 1], "end": [716, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/layer_filters/mod.rs:646`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Borrows the [`Filter`](crate::layer::Filter) used by this layer.

<a id="op-35f92873fc823a5a9808a09b"></a>
## filter_mut

`function` · `tracing_subscriber::filter::layer_filters::Filtered::filter_mut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn filter_mut(&mut self) -> &mut F
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [612, 1], "end": [716, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/layer_filters/mod.rs:675`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Mutably borrows the [`Filter`](crate::layer::Filter) used by this layer.

When this layer can be mutably borrowed, this may be used to mutate the filter.
Generally, this will primarily be used with the
[`reload::Handle::modify`](crate::reload::Handle::modify) method.

# Examples

```
# use tracing::info;
# use tracing_subscriber::{filter,fmt,reload,Registry,prelude::*};
# fn main() {
let filtered_layer = fmt::Layer::default().with_filter(filter::LevelFilter::WARN);
let (filtered_layer, reload_handle) = reload::Layer::new(filtered_layer);
#
# // specifying the Registry type is required
# let _: &reload::Handle<filter::Filtered<fmt::Layer<Registry>,
# filter::LevelFilter, Registry>,Registry>
# = &reload_handle;
#
info!("This will be ignored");
reload_handle.modify(|layer| *layer.filter_mut() = filter::LevelFilter::INFO);
info!("This will be logged");
# }
```

<a id="op-4cddda321b16bb43ad5847d9"></a>
## fmt

`function` · `tracing_subscriber::filter::layer_filters::Filtered::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [886, 1], "end": [898, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/layer_filters/mod.rs:891`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-943f41ad1e381ab61220a790"></a>
## inner

`function` · `tracing_subscriber::filter::layer_filters::Filtered::inner` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn inner(&self) -> &L
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [612, 1], "end": [716, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/layer_filters/mod.rs:680`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Borrows the inner [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) wrapped by this `Filtered` layer.

<a id="op-138ce17105f17cb2f400cb6e"></a>
## inner_mut

`function` · `tracing_subscriber::filter::layer_filters::Filtered::inner_mut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_mut(&mut self) -> &mut L
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [612, 1], "end": [716, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/layer_filters/mod.rs:713`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Mutably borrows the inner [`Layer`] wrapped by this `Filtered` layer.

This method is primarily expected to be used with the
[`reload::Handle::modify`](crate::reload::Handle::modify) method.

# Examples

```
# use tracing::info;
# use tracing_subscriber::{filter,fmt,reload,Registry,prelude::*};
# fn non_blocking<T: std::io::Write>(writer: T) -> (fn() -> std::io::Stdout) {
#   std::io::stdout
# }
# fn main() {
let filtered_layer = fmt::layer().with_writer(non_blocking(std::io::stderr())).with_filter(filter::LevelFilter::INFO);
let (filtered_layer, reload_handle) = reload::Layer::new(filtered_layer);
#
# // specifying the Registry type is required
# let _: &reload::Handle<filter::Filtered<fmt::Layer<Registry, _, _, fn() -> std::io::Stdout>,
# filter::LevelFilter, Registry>, Registry>
# = &reload_handle;
#
info!("This will be logged to stderr");
reload_handle.modify(|layer| *layer.inner_mut().writer_mut() = non_blocking(std::io::stdout()));
info!("This will be logged to stdout");
# }
```

[`Layer`]: crate::layer::Layer

<a id="op-f7ed9407717b5be7e55152fd"></a>
## new

`function` · `tracing_subscriber::filter::layer_filters::Filtered::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(layer: L, filter: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [612, 1], "end": [716, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/filter/layer_filters/mod.rs:622`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps the provided [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) so that it is filtered by the given
[`Filter`].

This is equivalent to calling the [`Layer::with_filter`](../operations/tracing_subscriber.layer.Layer.md#op-7bd3ef10607f7c7b6ba70fdd) method.

See the [documentation on per-layer filtering][plf] for details.

[`Filter`]: crate::layer::Filter
[plf]: crate::layer#per-layer-filtering

<a id="op-ce0b3dc784e8dcb3c1b206a0"></a>
## on_close

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: span::Id, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:857`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-10d935a9adec413f881030b6"></a>
## on_enter

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &span::Id, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:843`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b24904239fcb04a5f97ae951"></a>
## on_event

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_event(&self, event: &Event<'_>, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:837`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7bdeae2ce7fdf6b72b81babf"></a>
## on_exit

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &span::Id, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:850`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e93a635ffda1553d9bb959d5"></a>
## on_follows_from

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_follows_from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_follows_from(&self, span: &span::Id, follows: &span::Id, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:813`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-803dadb538b82bdb6dd3a7fe"></a>
## on_id_change

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_id_change` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_id_change(&self, old: &span::Id, new: &span::Id, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:865`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f91ec91d175b943349b5aec5"></a>
## on_layer

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_layer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_layer(&mut self, subscriber: &mut S)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:728`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c8362403885ef05ddb0deb21"></a>
## on_new_span

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:793`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-89b29750a4e850d3666ff9e8"></a>
## on_record

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, span: &span::Id, values: &span::Record<'_>, cx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:806`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-df850b4a4e8f111473433e63"></a>
## on_register_dispatch

`function` · `tracing_subscriber::filter::layer_filters::Filtered::on_register_dispatch` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_register_dispatch(&self, subscriber: &Dispatch)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:724`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8804aacf4b49e1ca463b4154"></a>
## register_callsite

`function` · `tracing_subscriber::filter::layer_filters::Filtered::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "F"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::Filtered", "path": "Filtered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "registry::LookupSpan"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "layer::Filter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}]}, "is_negative": false, "span": {"begin": [718, 1], "end": [884, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/filter/layer_filters/mod.rs:741`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
