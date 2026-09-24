# `tracing_core::field::DebugValue`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.DebugValue.json).

<a id="op-22ed177b916afdb5323e71b7"></a>
## DebugValue

`struct` · `tracing_core::field::DebugValue` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct DebugValue<T: fmt::Debug>
```

Source: `src/field.rs:364`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A `Value` which serializes as a string using `fmt::Debug`.

<a id="op-746310f3acd067dc18345067"></a>
## clone

`function` · `tracing_core::field::DebugValue::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> DebugValue<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_core::field::DebugValue", "path": "DebugValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [363, 10], "end": [363, 15], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/field.rs:363`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b17ee95ff0e4ddf0e675cc1e"></a>
## fmt

`function` · `tracing_core::field::DebugValue::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_core::field::DebugValue", "path": "DebugValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [753, 1], "end": [757, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field.rs:754`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1d423224ed1f1457c70cdbdb"></a>
## record

`function` · `tracing_core::field::DebugValue::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, key: &Field, visitor: &mut dyn Visit)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_core::field::DebugValue", "path": "DebugValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "fmt::Debug"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [744, 1], "end": [751, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "tracing_core::field::Value", "path": "Value"}, "trait_path": "tracing_core::field::Value"}`

Source: `src/field.rs:748`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
