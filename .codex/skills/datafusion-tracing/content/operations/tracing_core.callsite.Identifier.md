# `tracing_core::callsite::Identifier`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.callsite.Identifier.json).

<a id="op-cef1410a3f296584b90d483e"></a>
## Identifier

`struct` · `tracing_core::callsite::Identifier` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Identifier
```

Source: `src/callsite.rs:178`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Uniquely identifies a [`Callsite`]

Two `Identifier`s are equal if they both refer to the same callsite.

[`Callsite`]: super::callsite::Callsite

<a id="op-e45a4edf34713f9dc5cd1e80"></a>
## clone

`function` · `tracing_core::callsite::Identifier::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Identifier
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::Identifier", "path": "Identifier"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [177, 10], "end": [177, 15], "filename": "src/callsite.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/callsite.rs:177`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dd25f9e3df0043a20bae572c"></a>
## eq

`function` · `tracing_core::callsite::Identifier::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Identifier) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::Identifier", "path": "Identifier"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [375, 1], "end": [382, 2], "filename": "src/callsite.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/callsite.rs:376`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b423544c4d2a1660090722e2"></a>
## fmt

`function` · `tracing_core::callsite::Identifier::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::Identifier", "path": "Identifier"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [386, 1], "end": [390, 2], "filename": "src/callsite.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/callsite.rs:387`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-38c640c757bb79d0f9bb1f8b"></a>
## hash

`function` · `tracing_core::callsite::Identifier::hash` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<H>(&self, state: &mut H) where H: Hasher
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::callsite::Identifier", "path": "Identifier"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [392, 1], "end": [399, 2], "filename": "src/callsite.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/callsite.rs:393`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
