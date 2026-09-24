# `tracing_subscriber::filter::layer_filters::combinator::And`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.layer_filters.combinator.And.json).

<a id="op-b151815b6c2dcad43162521a"></a>
## And

`struct` · `tracing_subscriber::filter::layer_filters::combinator::And` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct And<A, B, S>
```

Source: `src/filter/layer_filters/combinator.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines two [`Filter`]s so that spans and events are enabled if and only if
*both* filters return `true`.

This type is typically returned by the [`FilterExt::and`] method. See that
method's documentation for details.

[`Filter`]: crate::layer::Filter
[`FilterExt::and`]: crate::filter::FilterExt::and

<a id="op-977a5b1399fb7e9cf8762398"></a>
## callsite_enabled

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::callsite_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:120`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-493ec8b9064d3f2f37604102"></a>
## clone

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [176, 1], "end": [188, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/layer_filters/combinator.rs:181`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9a4688a93fd9110aea876ebe"></a>
## enabled

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:116`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8b613d8dc187edad600d5080"></a>
## event_enabled

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::event_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, event: &tracing_core::Event<'_>, cx: &Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:141`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1c8628dd9abdba337a980315"></a>
## fmt

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [190, 1], "end": [201, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/layer_filters/combinator.rs:195`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b0605e502f8a2d5edf34bb11"></a>
## max_level_hint

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:135`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8c0edd7c842690364b6b771d"></a>
## on_close

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:170`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5e54322857dfa993cd7ef700"></a>
## on_enter

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:158`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-58ff6acfb23cea9a2d7369d5"></a>
## on_exit

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:164`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-30e2c5c90945dbb82bc3cf8a"></a>
## on_new_span

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:146`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-92c127d24b12425dd1770243"></a>
## on_record

`function` · `tracing_subscriber::filter::layer_filters::combinator::And::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::filter::layer_filters::combinator::And", "path": "And"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [110, 1], "end": [174, 2], "filename": "src/filter/layer_filters/combinator.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Filter", "path": "Filter"}, "trait_path": "tracing_subscriber::layer::Filter"}`

Source: `src/filter/layer_filters/combinator.rs:152`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
