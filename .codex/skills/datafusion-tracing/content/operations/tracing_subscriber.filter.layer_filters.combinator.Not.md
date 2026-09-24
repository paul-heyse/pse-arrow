# `tracing_subscriber::filter::layer_filters::combinator::Not`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.layer_filters.combinator.Not.json).

<a id="op-4e51f58ef74eff3816811cef"></a>
## Not

`struct` · `tracing_subscriber::filter::layer_filters::combinator::Not` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Not<A, S>
```

Source: `src/filter/layer_filters/combinator.rs:48`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Inverts the result of a [`Filter`].

If the wrapped filter would enable a span or event, it will be disabled. If
it would disable a span or event, that span or event will be enabled.

This type is typically returned by the [`FilterExt::not`] method. See that
method's documentation for details.

[`Filter`]: crate::layer::Filter
[`FilterExt::not`]: crate::filter::FilterExt::not

<a id="op-4244129c2ad09473bdb2d678"></a>
## callsite_enabled

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::callsite_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:476`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d0776ff2b1b482f134c9ab3b"></a>
## clone

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [523, 1], "end": [533, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/layer_filters/combinator.rs:527`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e5343227ab9c4924129c0fe1"></a>
## enabled

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:472`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ec80d14ba2a05af36eeffb6f"></a>
## event_enabled

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::event_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, event: &tracing_core::Event<'_>, cx: &Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:490`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-82ae7446ede875a6aa86eed1"></a>
## fmt

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [535, 1], "end": [542, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/layer_filters/combinator.rs:539`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6d11ee5aa489906e964ff9a8"></a>
## max_level_hint

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:484`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9529bb5e2e0af820885d5012"></a>
## on_close

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:518`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2c3cb7c28a9705754be165ce"></a>
## on_enter

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:508`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-05a458bf37441b4b44131926"></a>
## on_exit

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:513`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-62af34646eb574055deeffa4"></a>
## on_new_span

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:498`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aec86f8298ee5df7fa1b02e5"></a>
## on_record

`function` · `tracing_subscriber::filter::layer_filters::combinator::Not::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::Not", "path": "Not"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}]}, "is_negative": false, "span": {"begin": [467, 1], "end": [521, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:503`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
