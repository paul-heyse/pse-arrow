# `tracing_core::field::Empty`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.Empty.json).

<a id="op-24d94f12e1960bb94e3251ba"></a>
## Empty

`struct` · `tracing_core::field::Empty` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Empty
```

Source: `src/field.rs:146`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

An empty field.

This can be used to indicate that the value of a field is not currently
present but will be recorded later.

When a field's value is `Empty`. it will not be recorded.

<a id="op-4189333c38b61b571f23bf36"></a>
## eq

`function` · `tracing_core::field::Empty::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Empty) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Empty", "path": "Empty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [145, 21], "end": [145, 30], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/field.rs:145`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c82e75e2ce66f1748823e5b5"></a>
## fmt

`function` · `tracing_core::field::Empty::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Empty", "path": "Empty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [145, 10], "end": [145, 15], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field.rs:145`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-092c23190e4f5708218f47bb"></a>
## record

`function` · `tracing_core::field::Empty::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, _: &Field, _: &mut dyn Visit)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Empty", "path": "Empty"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [784, 1], "end": [787, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "tracing_core::field::Value", "path": "Value"}, "trait_path": "tracing_core::field::Value"}`

Source: `src/field.rs:786`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
