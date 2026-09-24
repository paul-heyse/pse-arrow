# `tracing_subscriber::registry::extensions::Extensions`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.extensions.Extensions.json).

<a id="op-3d46c3deb4031474740e0b84"></a>
## Extensions

`struct` · `tracing_subscriber::registry::extensions::Extensions` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Extensions<'a>
```

Source: `src/registry/extensions.rs:39`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

An immutable, read-only reference to a Span's extensions.

<a id="op-b60b0ac49f647d25dd50ab7b"></a>
## fmt

`function` · `tracing_subscriber::registry::extensions::Extensions::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::extensions::Extensions", "path": "Extensions"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [37, 10], "end": [37, 15], "filename": "src/registry/extensions.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/registry/extensions.rs:37`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7541e08522164f886183a918"></a>
## get

`function` · `tracing_subscriber::registry::extensions::Extensions::get` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn get<T: 'static>(&self) -> Option<&T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::extensions::Extensions", "path": "Extensions"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [43, 1], "end": [53, 2], "filename": "src/registry/extensions.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/extensions.rs:50`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Immutably borrows a type previously inserted into this `Extensions`.
