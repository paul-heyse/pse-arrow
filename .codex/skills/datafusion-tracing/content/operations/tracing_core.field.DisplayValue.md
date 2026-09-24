# `tracing_core::field::DisplayValue`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.DisplayValue.json).

<a id="op-666f25c381c630737fcdb303"></a>
## DisplayValue

`struct` · `tracing_core::field::DisplayValue` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct DisplayValue<T: fmt::Display>
```

Source: `src/field.rs:360`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A `Value` which serializes using `fmt::Display`.

Uses `record_debug` in the `Value` implementation to
avoid an unnecessary evaluation.

<a id="op-522e0fe94f622ae09c99d8b8"></a>
## clone

`function` · `tracing_core::field::DisplayValue::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> DisplayValue<T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_core::field::DisplayValue", "path": "DisplayValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Display", "path": "fmt::Display"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [359, 10], "end": [359, 15], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/field.rs:359`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7c0edb535fba0cca5b8bda2a"></a>
## fmt

`function` · `tracing_core::field::DisplayValue::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_core::field::DisplayValue", "path": "DisplayValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Display", "path": "fmt::Display"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [734, 1], "end": [738, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/field.rs:735`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ca383e4525e22bdd46409253"></a>
## fmt

`function` · `tracing_core::field::DisplayValue::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_core::field::DisplayValue", "path": "DisplayValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Display", "path": "fmt::Display"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [728, 1], "end": [732, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field.rs:729`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3da7d251600f5948908398be"></a>
## record

`function` · `tracing_core::field::DisplayValue::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, key: &Field, visitor: &mut dyn Visit)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_core::field::DisplayValue", "path": "DisplayValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Display", "path": "fmt::Display"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [719, 1], "end": [726, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "tracing_core::field::Value", "path": "Value"}, "trait_path": "tracing_core::field::Value"}`

Source: `src/field.rs:723`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
