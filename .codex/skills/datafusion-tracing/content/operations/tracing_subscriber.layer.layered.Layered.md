# `tracing_subscriber::layer::layered::Layered`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.layer.layered.Layered.json).

<a id="op-a34d0c9535f11404fda44764"></a>
## Layered

`struct` · `tracing_subscriber::layer::layered::Layered` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Layered<L, I, S = I>
```

Source: `src/layer/layered.rs:22`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`Subscriber`] composed of a `Subscriber` wrapped by one or more
[`Layer`]s.

[`Layer`]: crate::Layer
[`Subscriber`]: tracing_core::Subscriber

<a id="op-07091805dfd88440a4076a8b"></a>
## Data

`assoc_type` · `tracing_subscriber::layer::layered::Layered::Data` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Data
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [386, 1], "end": [400, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}, "trait_path": "tracing_subscriber::registry::LookupSpan"}`

Source: `src/layer/layered.rs:390`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d7c5ab63c342d3be97234d76"></a>
## clone

`function` · `tracing_subscriber::layer::layered::Layered::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Layered<L, I, S>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "I"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "I"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [21, 10], "end": [21, 15], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/layer/layered.rs:21`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-619789431c400dfe6562b9b0"></a>
## clone_span

`function` · `tracing_subscriber::layer::layered::Layered::clone_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone_span(&self, old: &span::Id) -> span::Id
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:171`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-78b90afa15eac36da251aeab"></a>
## current_span

`function` · `tracing_subscriber::layer::layered::Layered::current_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn current_span(&self) -> span::Current
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:209`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fe41c02e3664f9b27a4c2381"></a>
## downcast_ref

`function` · `tracing_subscriber::layer::layered::Layered::downcast_ref` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn downcast_ref<T: Any>(&self) -> Option<&T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [65, 1], "end": [87, 2], "filename": "src/layer/layered.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/layered.rs:77`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns some reference to this [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c) value if it is of type `T`,
or `None` if it isn't.

<a id="op-a0079d9eb456ce205d5f65a0"></a>
## drop_span

`function` · `tracing_subscriber::layer::layered::Layered::drop_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn drop_span(&self, id: span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:180`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d9139033a519decdac66f11"></a>
## enabled

`function` · `tracing_subscriber::layer::layered::Layered::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:105`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fc8d8161e91f645b50db4c42"></a>
## enabled

`function` · `tracing_subscriber::layer::layered::Layered::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:266`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d6fbe269d1af9d10787e4578"></a>
## enter

`function` · `tracing_subscriber::layer::layered::Layered::enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enter(&self, span: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:161`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e31e300a3d0ac96d8561e5b6"></a>
## event

`function` · `tracing_subscriber::layer::layered::Layered::event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event(&self, event: &Event<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:156`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6ebe032e65921572cc1872e9"></a>
## event_enabled

`function` · `tracing_subscriber::layer::layered::Layered::event_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, event: &Event<'_>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:146`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b72dde422cc6aa1939ea2058"></a>
## event_enabled

`function` · `tracing_subscriber::layer::layered::Layered::event_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, event: &Event<'_>, ctx: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:303`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-640f16e1179de4bc7adbed0a"></a>
## exit

`function` · `tracing_subscriber::layer::layered::Layered::exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn exit(&self, span: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:166`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-988ed61021baa13a995f523e"></a>
## fmt

`function` · `tracing_subscriber::layer::layered::Layered::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "B"}}}]}, "is_negative": false, "span": {"begin": [529, 1], "end": [555, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/layer/layered.rs:534`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3a22f8fa81fe8106856f00c8"></a>
## is

`function` · `tracing_subscriber::layer::layered::Layered::is` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn is<T: Any>(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [65, 1], "end": [87, 2], "filename": "src/layer/layered.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/layered.rs:71`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if this [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c) is the same type as `T`.

<a id="op-0284f57cf1a485aa61be11f0"></a>
## max_level_hint

`function` · `tracing_subscriber::layer::layered::Layered::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:122`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a299781ae5417cd655711277"></a>
## new_span

`function` · `tracing_subscriber::layer::layered::Layered::new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new_span(&self, span: &span::Attributes<'_>) -> span::Id
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:130`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4fac81722425fdd0d5e71d48"></a>
## on_close

`function` · `tracing_subscriber::layer::layered::Layered::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:332`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-069f7fd42f3f8ed5bd70ce04"></a>
## on_enter

`function` · `tracing_subscriber::layer::layered::Layered::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:320`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-239c9fe21fdb485c9da79db7"></a>
## on_event

`function` · `tracing_subscriber::layer::layered::Layered::on_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:314`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-17c14178ea10c760ab862986"></a>
## on_exit

`function` · `tracing_subscriber::layer::layered::Layered::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:326`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-37643f6999c497bc6ac0da35"></a>
## on_follows_from

`function` · `tracing_subscriber::layer::layered::Layered::on_follows_from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_follows_from(&self, span: &span::Id, follows: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:297`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-676df70a8cfb8a2b8ec6ecc4"></a>
## on_id_change

`function` · `tracing_subscriber::layer::layered::Layered::on_id_change` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_id_change(&self, old: &span::Id, new: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:338`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-63734c8923146753b127ec0d"></a>
## on_layer

`function` · `tracing_subscriber::layer::layered::Layered::on_layer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_layer(&mut self, subscriber: &mut S)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:255`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-45c545c7f8e4278bfde00415"></a>
## on_new_span

`function` · `tracing_subscriber::layer::layered::Layered::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:285`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6cf51e77c8341365e3fdaf8e"></a>
## on_record

`function` · `tracing_subscriber::layer::layered::Layered::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:291`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4023e739f0b6f74dd02bb3e8"></a>
## on_register_dispatch

`function` · `tracing_subscriber::layer::layered::Layered::on_register_dispatch` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_register_dispatch(&self, subscriber: &Dispatch)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:250`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f75e2f72edc29854d5f67110"></a>
## on_register_dispatch

`function` · `tracing_subscriber::layer::layered::Layered::on_register_dispatch` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_register_dispatch(&self, subscriber: &Dispatch)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:94`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-764b78d2f379f0f1c066b14a"></a>
## record

`function` · `tracing_subscriber::layer::layered::Layered::record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, span: &span::Id, values: &span::Record<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:136`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-46085f47c1d06d26a11a3517"></a>
## record_follows_from

`function` · `tracing_subscriber::layer::layered::Layered::record_follows_from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_follows_from(&self, span: &span::Id, follows: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:141`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-189f27c748ae24a7878fcc4c"></a>
## register_callsite

`function` · `tracing_subscriber::layer::layered::Layered::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:99`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a909034f8830410e2443b6a6"></a>
## register_callsite

`function` · `tracing_subscriber::layer::layered::Layered::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "A"}}, {"type": {"generic": "B"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "A"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "B"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "A"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "B"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [244, 1], "end": [384, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer/layered.rs:260`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cc3075e949451cc7b9143c36"></a>
## register_filter

`function` · `tracing_subscriber::layer::layered::Layered::register_filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_filter(&mut self) -> FilterId
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [386, 1], "end": [400, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}, "trait_path": "tracing_subscriber::registry::LookupSpan"}`

Source: `src/layer/layered.rs:397`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-250fc12ac2e3705b9d119c30"></a>
## span_data

`function` · `tracing_subscriber::layer::layered::Layered::span_data` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn span_data(&'a self, id: &span::Id) -> Option<Self::Data>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [386, 1], "end": [400, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}, "trait_path": "tracing_subscriber::registry::LookupSpan"}`

Source: `src/layer/layered.rs:392`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5b9fcfa35de32e193edb71eb"></a>
## try_close

`function` · `tracing_subscriber::layer::layered::Layered::try_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_close(&self, id: span::Id) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::layered::Layered", "path": "Layered"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}}}], "generic_params": [], "type": {"generic": "L"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [89, 1], "end": [242, 2], "filename": "src/layer/layered.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/layer/layered.rs:184`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
