# `tracing_subscriber::reload::Handle`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.reload.Handle.json).

<a id="op-d5dad50b907abab55bc696da"></a>
## Handle

`struct` · `tracing_subscriber::reload::Handle` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Handle<L, S>
```

Source: `src/reload.rs:95`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Allows reloading the state of an associated [`Layer`](crate::layer::Layer).

<a id="op-251a1a94cd43e90856fa3ee5"></a>
## clone

`function` · `tracing_subscriber::reload::Handle::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::reload::Handle", "path": "Handle"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [353, 1], "end": [360, 2], "filename": "src/reload.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/reload.rs:354`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f1d5df7230e2034824fa627c"></a>
## clone_current

`function` · `tracing_subscriber::reload::Handle::clone_current` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone_current(&self) -> Option<L> where L: Clone
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::reload::Handle", "path": "Handle"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [286, 1], "end": [351, 2], "filename": "src/reload.rs"}, "trait": null, "trait_path": null}`

Source: `src/reload.rs:335`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a clone of the layer or filter's current value if it still exists.
Otherwise, if the subscriber has been dropped, returns `None`.

<a id="op-61258bba6552c6782da5c33b"></a>
## fmt

`function` · `tracing_subscriber::reload::Handle::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::reload::Handle", "path": "Handle"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [94, 10], "end": [94, 15], "filename": "src/reload.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/reload.rs:94`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f7b8809efd23341df562a5b2"></a>
## modify

`function` · `tracing_subscriber::reload::Handle::modify` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn modify(&self, f: impl FnOnce(&mut L)) -> Result<(), Error>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::reload::Handle", "path": "Handle"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [286, 1], "end": [351, 2], "filename": "src/reload.rs"}, "trait": null, "trait_path": null}`

Source: `src/reload.rs:308`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Invokes a closure with a mutable reference to the current layer or filter,
allowing it to be modified in place.

<a id="op-f5691bb6eaf55eb5d8aeca1b"></a>
## reload

`function` · `tracing_subscriber::reload::Handle::reload` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn reload(&self, new_value: impl Into<L>) -> Result<(), Error>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::reload::Handle", "path": "Handle"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [286, 1], "end": [351, 2], "filename": "src/reload.rs"}, "trait": null, "trait_path": null}`

Source: `src/reload.rs:300`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Replace the current [`Layer`] or [`Filter`] with the provided `new_value`.

[`Handle::reload`](../operations/tracing_subscriber.reload.Handle.md#op-f5691bb6eaf55eb5d8aeca1b) cannot be used with the [`Filtered`] layer; use
[`Handle::modify`](../operations/tracing_subscriber.reload.Handle.md#op-f7b8809efd23341df562a5b2) instead (see [this issue] for additional details).

However, if the _only_ the [`Filter`]  needs to be modified, use
`reload::Layer` to wrap the `Filter` directly.

[`Layer`]: crate::layer::Layer
[`Filter`]: crate::layer::Filter
[`Filtered`]: crate::filter::Filtered

[this issue]: https://github.com/tokio-rs/tracing/issues/1629

<a id="op-741272216db413e0dd181592"></a>
## with_current

`function` · `tracing_subscriber::reload::Handle::with_current` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current<T>(&self, f: impl FnOnce(&L) -> T) -> Result<T, Error>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::reload::Handle", "path": "Handle"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [286, 1], "end": [351, 2], "filename": "src/reload.rs"}, "trait": null, "trait_path": null}`

Source: `src/reload.rs:344`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Invokes a closure with a borrowed reference to the current layer or filter,
returning the result (or an error if the subscriber no longer exists).
