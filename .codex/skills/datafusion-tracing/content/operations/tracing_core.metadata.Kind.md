# `tracing_core::metadata::Kind`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.metadata.Kind.json).

<a id="op-5b51e1e6e775e20fd1957368"></a>
## Kind

`struct` · `tracing_core::metadata::Kind` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Kind
```

Source: `src/metadata.rs:90`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Indicates whether the callsite is a span or event.

<a id="op-1c171b8957f2562ee572d2ef"></a>
## EVENT

`assoc_const` · `tracing_core::metadata::Kind::EVENT` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
EVENT
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [376, 1], "end": [414, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:382`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

`Event` callsite

<a id="op-f3c43de18ec79131d120d8a9"></a>
## HINT

`assoc_const` · `tracing_core::metadata::Kind::HINT` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
HINT
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [376, 1], "end": [414, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:390`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

`enabled!` callsite. [`Subscriber`][`crate::subscriber::Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c)s can assume
this `Kind` means they will never receive a
full event with this [`Metadata`](../operations/tracing_core.metadata.Metadata.md#op-3c5a7a9d81c273e2173bb24c).

<a id="op-93da2d51c5f1d9babceb16ad"></a>
## SPAN

`assoc_const` · `tracing_core::metadata::Kind::SPAN` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
SPAN
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [376, 1], "end": [414, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:385`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

`Span` callsite

<a id="op-59260984f4bbad5168f5043c"></a>
## clone

`function` · `tracing_core::metadata::Kind::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Kind
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [89, 10], "end": [89, 15], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metadata.rs:89`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6b79d36acb6f9ed7b875829a"></a>
## eq

`function` · `tracing_core::metadata::Kind::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Kind) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [89, 21], "end": [89, 30], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metadata.rs:89`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a3836f82b6d2e4a3e731e108"></a>
## fmt

`function` · `tracing_core::metadata::Kind::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [416, 1], "end": [449, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metadata.rs:417`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-464586741ea34724c52d32be"></a>
## hint

`function` · `tracing_core::metadata::Kind::hint` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
const fn hint(self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [376, 1], "end": [414, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:411`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Sets that this `Kind` is a [hint](Self::HINT).

This can be called on [`SPAN`](Self::SPAN) and [`EVENT`](Self::EVENT)
kinds to construct a hint callsite that also counts as a span or event.

<a id="op-b9ea9069d059d95cc07d5258"></a>
## is_event

`function` · `tracing_core::metadata::Kind::is_event` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_event(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [376, 1], "end": [414, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:398`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Return true if the callsite kind is `Event`

<a id="op-e7433e76b01a3c216eae2a3f"></a>
## is_hint

`function` · `tracing_core::metadata::Kind::is_hint` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_hint(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [376, 1], "end": [414, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:403`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Return true if the callsite kind is `Hint`

<a id="op-1669e3f8e35e195dbf5fddef"></a>
## is_span

`function` · `tracing_core::metadata::Kind::is_span` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_span(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Kind", "path": "Kind"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [376, 1], "end": [414, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:393`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Return true if the callsite kind is `Span`
