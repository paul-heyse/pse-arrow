# `tracing_subscriber::registry::Scope`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.Scope.json).

<a id="op-c213fa89ae0d389dfbd5d880"></a>
## Scope

`struct` · `tracing_subscriber::registry::Scope` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Scope<'a, R>
```

Source: `src/registry/mod.rs:220`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

An iterator over the parents of a span, ordered from leaf to root.

This is returned by the [`SpanRef::scope`](../operations/tracing_subscriber.registry.SpanRef.md#op-4eb435132dedc623cebfa2b3) method.

<a id="op-eefa8c4845ed1f3bbb46d655"></a>
## Item

`assoc_type` · `tracing_subscriber::registry::Scope::Item` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::Scope", "path": "Scope"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [305, 1], "end": [334, 2], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/registry/mod.rs:309`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eac5e774e13ab82cbc376bb1"></a>
## fmt

`function` · `tracing_subscriber::registry::Scope::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::Scope", "path": "Scope"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [219, 10], "end": [219, 15], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/registry/mod.rs:219`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-29cbc687573025f128a156cf"></a>
## from_root

`function` · `tracing_subscriber::registry::Scope::from_root` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from_root(self) -> ScopeFromRoot<'a, R>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::Scope", "path": "Scope"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [252, 5], "end": [276, 6], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:267`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Flips the order of the iterator, so that it is ordered from root to leaf.

The iterator will first return the root span, then that span's immediate child,
and so on until it finally returns the span that [`SpanRef::scope`](../operations/tracing_subscriber.registry.SpanRef.md#op-4eb435132dedc623cebfa2b3) was called on.

If any items were consumed from the [`Scope`](../operations/tracing_subscriber.registry.Scope.md#op-c213fa89ae0d389dfbd5d880) before calling this method then they
will *not* be returned from the [`ScopeFromRoot`](../operations/tracing_subscriber.registry.ScopeFromRoot.md#op-93d46ac134cee7c6233c23cc).

**Note**: this will allocate if there are many spans remaining, or if the
"smallvec" feature flag is not enabled.

<a id="op-1b655828128ade76b4e6a1d5"></a>
## next

`function` · `tracing_subscriber::registry::Scope::next` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn next(&mut self) -> Option<Self::Item>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::Scope", "path": "Scope"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [305, 1], "end": [334, 2], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/registry/mod.rs:311`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
