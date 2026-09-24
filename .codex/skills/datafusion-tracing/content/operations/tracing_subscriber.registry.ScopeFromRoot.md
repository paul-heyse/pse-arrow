# `tracing_subscriber::registry::ScopeFromRoot`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.ScopeFromRoot.json).

<a id="op-93d46ac134cee7c6233c23cc"></a>
## ScopeFromRoot

`struct` · `tracing_subscriber::registry::ScopeFromRoot` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct ScopeFromRoot<'a, R> where R: LookupSpan<'a>
```

Source: `src/registry/mod.rs:239`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

An iterator over the parents of a span, ordered from root to leaf.

This is returned by the [`Scope::from_root`](../operations/tracing_subscriber.registry.Scope.md#op-29cbc687573025f128a156cf) method.

<a id="op-1af312f04258d7d715e3e138"></a>
## Item

`assoc_type` · `tracing_subscriber::registry::ScopeFromRoot::Item` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::ScopeFromRoot", "path": "ScopeFromRoot"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [278, 5], "end": [293, 6], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/registry/mod.rs:282`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cd5d1d092ef5af3675789ae9"></a>
## fmt

`function` · `tracing_subscriber::registry::ScopeFromRoot::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::ScopeFromRoot", "path": "ScopeFromRoot"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [295, 5], "end": [302, 6], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/registry/mod.rs:299`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1cc24fd28631d9139e182273"></a>
## next

`function` · `tracing_subscriber::registry::ScopeFromRoot::next` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn next(&mut self) -> Option<Self::Item>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::ScopeFromRoot", "path": "ScopeFromRoot"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [278, 5], "end": [293, 6], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/registry/mod.rs:285`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-104a33bc3198fdf03d8503f0"></a>
## size_hint

`function` · `tracing_subscriber::registry::ScopeFromRoot::size_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn size_hint(&self) -> (usize, Option<usize>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::ScopeFromRoot", "path": "ScopeFromRoot"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [278, 5], "end": [293, 6], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/registry/mod.rs:290`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
