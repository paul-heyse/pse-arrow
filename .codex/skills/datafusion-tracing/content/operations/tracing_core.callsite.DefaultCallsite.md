# `tracing_core::callsite::DefaultCallsite`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.callsite.DefaultCallsite.json).

<a id="op-58b6356d7f4118a36fb5badb"></a>
## DefaultCallsite

`struct` · `tracing_core::callsite::DefaultCallsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct DefaultCallsite
```

Source: `src/callsite.rs:192`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A default [`Callsite`](../operations/tracing_core.callsite.Callsite.md#op-f2f04985f5653f1f8f822082) implementation.

<a id="op-902f7146ff4ba01bd448051f"></a>
## fmt

`function` · `tracing_core::callsite::DefaultCallsite::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::DefaultCallsite", "path": "DefaultCallsite"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [191, 10], "end": [191, 15], "filename": "src/callsite.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/callsite.rs:191`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2d1838407598a6ea9f0ae7f7"></a>
## interest

`function` · `tracing_core::callsite::DefaultCallsite::interest` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn interest(&'static self) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::DefaultCallsite", "path": "DefaultCallsite"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [271, 1], "end": [355, 2], "filename": "src/callsite.rs"}, "trait": null, "trait_path": null}`

Source: `src/callsite.rs:347`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the callsite's cached `Interest`, or registers it for the
first time if it has not yet been registered.

<a id="op-78ed2233eb231ba6c50d4f56"></a>
## metadata

`function` · `tracing_core::callsite::DefaultCallsite::metadata` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> &Metadata<'static>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::DefaultCallsite", "path": "DefaultCallsite"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [357, 1], "end": [371, 2], "filename": "src/callsite.rs"}, "trait": {"args": null, "id": "tracing_core::callsite::Callsite", "path": "Callsite"}, "trait_path": "tracing_core::callsite::Callsite"}`

Source: `src/callsite.rs:368`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3709ea7f7720e521d93a843b"></a>
## new

`function` · `tracing_core::callsite::DefaultCallsite::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
const fn new(meta: &'static Metadata<'static>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::DefaultCallsite", "path": "DefaultCallsite"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [271, 1], "end": [355, 2], "filename": "src/callsite.rs"}, "trait": null, "trait_path": null}`

Source: `src/callsite.rs:281`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a new `DefaultCallsite` with the specified `Metadata`.

<a id="op-509e01db724fe831141c81d4"></a>
## register

`function` · `tracing_core::callsite::DefaultCallsite::register` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn register(&'static self) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::DefaultCallsite", "path": "DefaultCallsite"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [271, 1], "end": [355, 2], "filename": "src/callsite.rs"}, "trait": null, "trait_path": null}`

Source: `src/callsite.rs:308`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Registers this callsite with the global callsite registry.

If the callsite is already registered, this does nothing. When using
[`DefaultCallsite`](../operations/tracing_core.callsite.DefaultCallsite.md#op-58b6356d7f4118a36fb5badb), this method should be preferred over
[`tracing_core::callsite::register`], as it ensures that the callsite is
only registered a single time.

Other callsite implementations will generally ensure that
callsites are not re-registered through another mechanism.

See the [documentation on callsite registration][reg-docs] for details
on the global callsite registry.

[`tracing_core::callsite::register`]: crate::callsite::register
[reg-docs]: crate::callsite#registering-callsites

<a id="op-14da289c7bdd365adaa0c931"></a>
## set_interest

`function` · `tracing_core::callsite::DefaultCallsite::set_interest` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn set_interest(&self, interest: Interest)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::DefaultCallsite", "path": "DefaultCallsite"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [357, 1], "end": [371, 2], "filename": "src/callsite.rs"}, "trait": {"args": null, "id": "tracing_core::callsite::Callsite", "path": "Callsite"}, "trait_path": "tracing_core::callsite::Callsite"}`

Source: `src/callsite.rs:358`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
