# `tracing_subscriber::registry::extensions::ExtensionsMut`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.extensions.ExtensionsMut.json).

<a id="op-5a726ea8247ed7c5d9985c04"></a>
## ExtensionsMut

`struct` · `tracing_subscriber::registry::extensions::ExtensionsMut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct ExtensionsMut<'a>
```

Source: `src/registry/extensions.rs:58`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

An mutable reference to a Span's extensions.

<a id="op-94d7946cb343bba27cf82816"></a>
## fmt

`function` · `tracing_subscriber::registry::extensions::ExtensionsMut::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::extensions::ExtensionsMut", "path": "ExtensionsMut"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [56, 10], "end": [56, 15], "filename": "src/registry/extensions.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/registry/extensions.rs:56`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b0042e91f2308ebbdd49069d"></a>
## get_mut

`function` · `tracing_subscriber::registry::extensions::ExtensionsMut::get_mut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn get_mut<T: 'static>(&mut self) -> Option<&mut T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::extensions::ExtensionsMut", "path": "ExtensionsMut"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [62, 1], "end": [109, 2], "filename": "src/registry/extensions.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/extensions.rs:99`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Get a mutable reference to a type previously inserted on this `ExtensionsMut`.

<a id="op-cd00664c8169aa9de500e3bb"></a>
## insert

`function` · `tracing_subscriber::registry::extensions::ExtensionsMut::insert` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn insert<T: Send + Sync + 'static>(&mut self, val: T)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::extensions::ExtensionsMut", "path": "ExtensionsMut"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [62, 1], "end": [109, 2], "filename": "src/registry/extensions.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/extensions.rs:87`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Insert a type into this `Extensions`.

Note that extensions are _not_
`Layer`-specific—they are _span_-specific. This means that
other layers can access and mutate extensions that
a different Layer recorded. For example, an application might
have a layer that records execution timings, alongside a layer
that reports spans and events to a distributed
tracing system that requires timestamps for spans.
Ideally, if one layer records a timestamp _x_, the other layer
should be able to reuse timestamp _x_.

Therefore, extensions should generally be newtypes, rather than common
types like [`String`](std::string::String), to avoid accidental
cross-`Layer` clobbering.

## Panics

If `T` is already present in `Extensions`, then this method will panic.

Unresolved upstream links (retained, not inferred): `std::string::String`.

<a id="op-62e240ca7de1cc2b232d0ea7"></a>
## remove

`function` · `tracing_subscriber::registry::extensions::ExtensionsMut::remove` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::extensions::ExtensionsMut", "path": "ExtensionsMut"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [62, 1], "end": [109, 2], "filename": "src/registry/extensions.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/extensions.rs:106`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Remove a type from this `Extensions`.

If a extension of this type existed, it will be returned.

<a id="op-6132764b24ec304de8084d99"></a>
## replace

`function` · `tracing_subscriber::registry::extensions::ExtensionsMut::replace` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn replace<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::extensions::ExtensionsMut", "path": "ExtensionsMut"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [62, 1], "end": [109, 2], "filename": "src/registry/extensions.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/extensions.rs:94`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Replaces an existing `T` into this extensions.

If `T` is not present, `Option::None` will be returned.
