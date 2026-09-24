# `tracing::instrument::Instrumented`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.instrument.Instrumented.json).

<a id="op-0ed511920bac15e24130ac7d"></a>
## Instrumented

`struct` · `tracing::instrument::Instrumented` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
struct Instrumented<T>
```

Source: `src/instrument.rs:254`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

A [`Future`] that has been instrumented with a `tracing` [`Span`].

This type is returned by the [`Instrument`](../operations/tracing.instrument.Instrument.md#op-f3255b14b518b17fdbec7041) extension trait. See that
trait's documentation for details.

[`Future`]: std::future::Future
[`Span`]: crate::Span

Unresolved upstream links (retained, not inferred): `std::future::Future`.

<a id="op-3a48f5a759e89a5ab3a0b9bb"></a>
## Output

`assoc_type` · `tracing::instrument::Instrumented::Output` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [315, 1], "end": [323, 2], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/instrument.rs:316`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fd111053808cf49ff196aff4"></a>
## clone

`function` · `tracing::instrument::Instrumented::clone` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Instrumented<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [264, 21], "end": [264, 26], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/instrument.rs:264`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d069dfcd316c8b0a9228adf7"></a>
## drop

`function` · `tracing::instrument::Instrumented::drop` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [254, 1], "end": [288, 2], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/instrument.rs:254`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8352aeac7c1d908de9d21f0a"></a>
## fmt

`function` · `tracing::instrument::Instrumented::fmt` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [264, 14], "end": [264, 19], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/instrument.rs:264`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-96aa6728c2a993c04e3d50e9"></a>
## inner

`function` · `tracing::instrument::Instrumented::inner` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner(&self) -> &T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [327, 1], "end": [374, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:339`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Borrows the wrapped type.

<a id="op-d91fdb71a91fc34ac76af2e9"></a>
## inner_mut

`function` · `tracing::instrument::Instrumented::inner_mut` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_mut(&mut self) -> &mut T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [327, 1], "end": [374, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:344`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Mutably borrows the wrapped type.

<a id="op-50df3115165eedb9e635b409"></a>
## inner_pin_mut

`function` · `tracing::instrument::Instrumented::inner_pin_mut` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_pin_mut(Pin<&mut self>) -> Pin<&mut T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [327, 1], "end": [374, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:354`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Get a pinned mutable reference to the wrapped type.

<a id="op-f72e01cbad8e01d87b17beb4"></a>
## inner_pin_ref

`function` · `tracing::instrument::Instrumented::inner_pin_ref` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn inner_pin_ref(Pin<&self>) -> Pin<&T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [327, 1], "end": [374, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:349`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Get a pinned reference to the wrapped type.

<a id="op-eced57a2291f959d7ae14049"></a>
## into_inner

`function` · `tracing::instrument::Instrumented::into_inner` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn into_inner(self) -> T
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [327, 1], "end": [374, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:361`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Consumes the `Instrumented`, returning the wrapped type.

Note that this drops the span.

<a id="op-9101db1e0e344a56236b0379"></a>
## poll

`function` · `tracing::instrument::Instrumented::poll` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn poll(Pin<&mut self>, cx: &mut Context<'_>) -> Poll<Self::Output>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [315, 1], "end": [323, 2], "filename": "src/instrument.rs"}, "trait": {"args": null, "id": "core::future::future::Future", "path": "Future"}, "trait_path": "core::future::future::Future"}`

Source: `src/instrument.rs:318`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7d238fbd1dc65ccc7fe1ecb2"></a>
## span

`function` · `tracing::instrument::Instrumented::span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn span(&self) -> &Span
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [327, 1], "end": [374, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:329`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Borrows the `Span` that this type is instrumented by.

<a id="op-62a54c43f3139af6802631e6"></a>
## span_mut

`function` · `tracing::instrument::Instrumented::span_mut` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn span_mut(&mut self) -> &mut Span
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing::instrument::Instrumented", "path": "Instrumented"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [327, 1], "end": [374, 2], "filename": "src/instrument.rs"}, "trait": null, "trait_path": null}`

Source: `src/instrument.rs:334`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Mutably borrows the `Span` that this type is instrumented by.
