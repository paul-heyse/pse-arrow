# `tracing_core::field::ValueSet`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.ValueSet.json).

<a id="op-81b1992dfc73d64b6f31eb3a"></a>
## ValueSet

`struct` · `tracing_core::field::ValueSet` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct ValueSet<'a>
```

Source: `src/field.rs:167`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A set of fields and values for a span.

<a id="op-72def4786593fc4851cbdc23"></a>
## callsite

`function` · `tracing_core::field::ValueSet::callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite(&self) -> callsite::Identifier
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::field::ValueSet", "path": "ValueSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1054, 1], "end": [1138, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:1061`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an [`Identifier`] that uniquely identifies the [`Callsite`]
defining the fields this `ValueSet` refers to.

[`Identifier`]: super::callsite::Identifier
[`Callsite`]: super::callsite::Callsite

<a id="op-d07ff39b83e17b584170d0cf"></a>
## fmt

`function` · `tracing_core::field::ValueSet::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::field::ValueSet", "path": "ValueSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1148, 1], "end": [1154, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/field.rs:1149`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e9fbedcf66f33ed82e7b6777"></a>
## fmt

`function` · `tracing_core::field::ValueSet::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::field::ValueSet", "path": "ValueSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1140, 1], "end": [1146, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field.rs:1141`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c9472b78a47cdfc94eb9c988"></a>
## is_empty

`function` · `tracing_core::field::ValueSet::is_empty` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_empty(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::field::ValueSet", "path": "ValueSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1054, 1], "end": [1138, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:1123`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if this `ValueSet` contains _no_ values.

<a id="op-979d55fd1a7c96009db50c3e"></a>
## len

`function` · `tracing_core::field::ValueSet::len` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn len(&self) -> usize
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::field::ValueSet", "path": "ValueSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1054, 1], "end": [1138, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:1096`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the number of fields in this `ValueSet` that would be visited
by a given [visitor] to the [`ValueSet::record()`] method.

[visitor]: Visit
[`ValueSet::record()`]: ValueSet::record()

<a id="op-da8a36633b6cb676cbe159c4"></a>
## record

`function` · `tracing_core::field::ValueSet::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, visitor: &mut dyn Visit)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing_core::field::ValueSet", "path": "ValueSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1054, 1], "end": [1138, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:1068`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visits all the fields in this `ValueSet` with the provided [visitor].

[visitor]: Visit
