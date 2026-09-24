# `tracing::instrument::WithDispatch`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.instrument.WithDispatch.json).

<a id="op-f37fe528a1c49c9d37012bbc"></a>
## WithDispatch

`struct` · `tracing::instrument::WithDispatch` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
struct WithDispatch<T>
```

Source: `src/instrument.rs:236`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

A [`Future`] that has been instrumented with a `tracing` [`Subscriber`].

This type is returned by the [`WithSubscriber`](../operations/tracing.instrument.WithSubscriber.md#op-4bd50093c9a272555ebecf03) extension trait. See that
trait's documentation for details.

[`Future`]: std::future::Future
[`Subscriber`]: crate::Subscriber

Unresolved upstream links (retained, not inferred): `std::future::Future`.

<a id="op-4074d10f482452a39ddba013"></a>
## Output

`assoc_type` · `tracing::instrument::WithDispatch::Output` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [380, 1], "end": [390, 2], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/instrument.rs:381`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2c14f4c4fb87d791347fb528"></a>
## clone

`function` · `tracing::instrument::WithDispatch::clone` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> WithDispatch<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [244, 14], "end": [244, 19], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/instrument.rs:244`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-46083a3b16b4a74c21e04056"></a>
## dispatcher

`function` · `tracing::instrument::WithDispatch::dispatcher` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn dispatcher(&self) -> &Dispatch
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [397, 1], "end": [429, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:399`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Borrows the [`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c) that is entered when this type is polled.

<a id="op-86ea87a427dce53f3bd2035b"></a>
## fmt

`function` · `tracing::instrument::WithDispatch::fmt` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [244, 21], "end": [244, 26], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/instrument.rs:244`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c94a4f4c61114d770d6ca3b9"></a>
## inner

`function` · `tracing::instrument::WithDispatch::inner` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner(&self) -> &T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [397, 1], "end": [429, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:404`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Borrows the wrapped type.

<a id="op-65ee1814c90a38e9ebeac329"></a>
## inner_mut

`function` · `tracing::instrument::WithDispatch::inner_mut` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_mut(&mut self) -> &mut T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [397, 1], "end": [429, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:409`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Mutably borrows the wrapped type.

<a id="op-3a5ed69fe92278a302af7dc1"></a>
## inner_pin_mut

`function` · `tracing::instrument::WithDispatch::inner_pin_mut` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_pin_mut(Pin<&mut self>) -> Pin<&mut T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [397, 1], "end": [429, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:419`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Get a pinned mutable reference to the wrapped type.

<a id="op-573a74eec0c669100ace77e0"></a>
## inner_pin_ref

`function` · `tracing::instrument::WithDispatch::inner_pin_ref` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_pin_ref(Pin<&self>) -> Pin<&T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [397, 1], "end": [429, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:414`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Get a pinned reference to the wrapped type.

<a id="op-6d3e385aaecdaa17dcd55015"></a>
## into_inner

`function` · `tracing::instrument::WithDispatch::into_inner` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn into_inner(self) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [397, 1], "end": [429, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:426`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Consumes the `Instrumented`, returning the wrapped type.

Note that this drops the span.

<a id="op-afa9a5fc9aec0c34f12f75db"></a>
## poll

`function` · `tracing::instrument::WithDispatch::poll` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn poll(Pin<&mut self>, cx: &mut Context<'_>) -> Poll<Self::Output>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::WithDispatch", "path": "WithDispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [380, 1], "end": [390, 2], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/instrument.rs:383`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.
