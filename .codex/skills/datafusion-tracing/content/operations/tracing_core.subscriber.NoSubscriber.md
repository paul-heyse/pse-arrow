# `tracing_core::subscriber::NoSubscriber`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.subscriber.NoSubscriber.json).

<a id="op-3f81448a48246e30f41614f5"></a>
## NoSubscriber

`struct` · `tracing_core::subscriber::NoSubscriber` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct NoSubscriber
```

Source: `src/subscriber.rs:672`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A no-op [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c).

[`NoSubscriber`](../operations/tracing_core.subscriber.NoSubscriber.md#op-3f81448a48246e30f41614f5) implements the [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c) trait by never being enabled,
never being interested in any callsite, and dropping all spans and events.

<a id="op-b03929b1089a1e87aa873ed5"></a>
## clone

`function` · `tracing_core::subscriber::NoSubscriber::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> NoSubscriber
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [671, 16], "end": [671, 21], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/subscriber.rs:671`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ec864759fa61277a4b1097ad"></a>
## default

`function` · `tracing_core::subscriber::NoSubscriber::default` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> NoSubscriber
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [671, 30], "end": [671, 37], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/subscriber.rs:671`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a644874c2dea430434edb234"></a>
## enabled

`function` · `tracing_core::subscriber::NoSubscriber::enabled` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, _metadata: &Metadata<'_>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:691`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-39155b8973d857a41bc7205e"></a>
## enter

`function` · `tracing_core::subscriber::NoSubscriber::enter` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn enter(&self, _span: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:695`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a189e6328234336b565276d0"></a>
## event

`function` · `tracing_core::subscriber::NoSubscriber::event` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn event(&self, _event: &Event<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:684`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-89a98a0b4358bafb33fbef28"></a>
## exit

`function` · `tracing_core::subscriber::NoSubscriber::exit` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn exit(&self, _span: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:696`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ea15bd13f8621c9d4dfb70fa"></a>
## fmt

`function` · `tracing_core::subscriber::NoSubscriber::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [671, 23], "end": [671, 28], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/subscriber.rs:671`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-06b4ac3a728f9644f260fc9b"></a>
## new

`function` · `tracing_core::subscriber::NoSubscriber::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
const fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [699, 1], "end": [705, 2], "filename": "src/subscriber.rs"}, "trait": null, "trait_path": null}`

Source: `src/subscriber.rs:702`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a new `NoSubscriber`.

<a id="op-987fe114f30d2f501c363bfd"></a>
## new_span

`function` · `tracing_core::subscriber::NoSubscriber::new_span` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new_span(&self, _: &span::Attributes<'_>) -> span::Id
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:680`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1c0ea241a2251374f6a40643"></a>
## record

`function` · `tracing_core::subscriber::NoSubscriber::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, _span: &span::Id, _values: &span::Record<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:686`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3c10a9d56aa8f4f2dcd02d3b"></a>
## record_follows_from

`function` · `tracing_core::subscriber::NoSubscriber::record_follows_from` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_follows_from(&self, _span: &span::Id, _follows: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:688`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ae78ebb89f0e1fe21585d4f9"></a>
## register_callsite

`function` · `tracing_core::subscriber::NoSubscriber::register_callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, _: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::subscriber::NoSubscriber", "path": "NoSubscriber"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [674, 1], "end": [697, 2], "filename": "src/subscriber.rs"}, "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}, "trait_path": "tracing_core::subscriber::Subscriber"}`

Source: `src/subscriber.rs:676`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
