# `opentelemetry_sdk::resource::Iter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.Iter.json).

<a id="op-501bbe5e755fe09c99062753"></a>
## Iter

`struct` · `opentelemetry_sdk::resource::Iter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Iter<'a>
```

Source: `src/resource/mod.rs:235`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

An iterator over the entries of a `Resource`.

<a id="op-512bc2bc8b46c085316677a3"></a>
## Item

`assoc_type` · `opentelemetry_sdk::resource::Iter::Item` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry_sdk::resource::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [237, 1], "end": [243, 2], "filename": "src/resource/mod.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/resource/mod.rs:238`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d704a76893da8a87fb6d90dd"></a>
## fmt

`function` · `opentelemetry_sdk::resource::Iter::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry_sdk::resource::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [234, 10], "end": [234, 15], "filename": "src/resource/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/resource/mod.rs:234`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-361f39633abae3319e8ae2bf"></a>
## next

`function` · `opentelemetry_sdk::resource::Iter::next` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn next(&mut self) -> Option<Self::Item>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "opentelemetry_sdk::resource::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [237, 1], "end": [243, 2], "filename": "src/resource/mod.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/resource/mod.rs:240`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
