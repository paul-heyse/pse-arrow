# `tracing_core::field::Field`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.Field.json).

<a id="op-d16f69bd65cdad5606214c7c"></a>
## Field

`struct` · `tracing_core::field::Field` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Field
```

Source: `src/field.rs:134`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

An opaque key allowing _O_(1) access to a field in a `Span`'s key-value
data.

As keys are defined by the _metadata_ of a span, rather than by an
individual instance of a span, a key may be used to access the same field
across all instances of a given span with the same metadata. Thus, when a
subscriber observes a new span, it need only access a field by name _once_,
and use the key for that name for all other accesses.

<a id="op-60f1eb70e9ad770d87d68a87"></a>
## as_ref

`function` · `tracing_core::field::Field::as_ref` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn as_ref(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [829, 1], "end": [833, 2], "filename": "src/field.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}, "trait_path": "core::convert::AsRef"}`

Source: `src/field.rs:830`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7796da44d1365eb1b14d969e"></a>
## callsite

`function` · `tracing_core::field::Field::callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite(&self) -> callsite::Identifier
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [821, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:808`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an [`Identifier`] that uniquely identifies the [`Callsite`]
which defines this field.

[`Identifier`]: super::callsite::Identifier
[`Callsite`]: super::callsite::Callsite

<a id="op-33b37c836a1fcebdaed8352d"></a>
## clone

`function` · `tracing_core::field::Field::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [853, 1], "end": [863, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/field.rs:854`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b2403717a3586fbe077b772d"></a>
## eq

`function` · `tracing_core::field::Field::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [835, 1], "end": [839, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/field.rs:836`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8ddac0c27585a496c5af3b28"></a>
## fmt

`function` · `tracing_core::field::Field::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [133, 10], "end": [133, 15], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field.rs:133`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-95c95f0742f843780750647b"></a>
## fmt

`function` · `tracing_core::field::Field::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [823, 1], "end": [827, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/field.rs:824`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-73293d7cb729c89be60d67f2"></a>
## hash

`function` · `tracing_core::field::Field::hash` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<H>(&self, state: &mut H) where H: Hasher
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [843, 1], "end": [851, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/field.rs:844`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d844fcf169ffe0bf484e766c"></a>
## index

`function` · `tracing_core::field::Field::index` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn index(&self) -> usize
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [821, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:818`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the index of this field in its [`FieldSet`](../operations/tracing_core.field.FieldSet.md#op-c3de2aec342fd60d61868736).

<a id="op-e5cceba5da764df896094967"></a>
## name

`function` · `tracing_core::field::Field::name` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn name(&self) -> &'static str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::Field", "path": "Field"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [801, 1], "end": [821, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:813`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a string representing the name of the field.
