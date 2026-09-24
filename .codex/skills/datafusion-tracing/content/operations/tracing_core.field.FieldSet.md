# `tracing_core::field::FieldSet`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.FieldSet.json).

<a id="op-c3de2aec342fd60d61868736"></a>
## FieldSet

`struct` · `tracing_core::field::FieldSet` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct FieldSet
```

Source: `src/field.rs:159`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Describes the fields present on a span.

## Equality

In well-behaved applications, two `FieldSet`s [initialized] with equal
[callsite identifiers] will have identical fields. Consequently, in release
builds, [`FieldSet::eq`](../operations/tracing_core.field.FieldSet.md#op-28d62bd7ee34c886a4b502e5) *only* checks that its arguments have equal
callsites. However, the equality of field names is checked in debug builds.

[initialized]: Self::new
[callsite identifiers]: callsite::Identifier

<a id="op-91589080be2b2f0e44e93463"></a>
## contains

`function` · `tracing_core::field::FieldSet::contains` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn contains(&self, field: &Field) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [867, 1], "end": [973, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:908`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if `self` contains the given `field`.

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: If <code>field</code> shares a name with a field
in this <code>FieldSet</code>, but was created by a <code>FieldSet</code>
with a different callsite, this <code>FieldSet</code> does <em>not</em>
contain it. This is so that if two separate span callsites define a field
named "foo", the <code>Field</code> corresponding to "foo" for each
of those callsites are not equivalent.
</pre></div>

<a id="op-28d62bd7ee34c886a4b502e5"></a>
## eq

`function` · `tracing_core::field::FieldSet::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1003, 1], "end": [1033, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/field.rs:1004`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a3b4a0880964aad8a81c20cf"></a>
## field

`function` · `tracing_core::field::FieldSet::field` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn field<Q: Borrow<str> + ?Sized>(&self, name: &Q) -> Option<Field>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [867, 1], "end": [973, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:886`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the [`Field`] named `name`, or `None` if no such field exists.

[`Field`]: super::Field

<a id="op-68294a6725fe146d2d8c009d"></a>
## fmt

`function` · `tracing_core::field::FieldSet::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [984, 1], "end": [991, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field.rs:985`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d910161ddb4e33ffd65e4dd8"></a>
## fmt

`function` · `tracing_core::field::FieldSet::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [993, 1], "end": [999, 2], "filename": "src/field.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/field.rs:994`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1dd64e85464644b8989a949a"></a>
## is_empty

`function` · `tracing_core::field::FieldSet::is_empty` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_empty(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [867, 1], "end": [973, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:970`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns whether or not this `FieldSet` has fields.

<a id="op-63f2d64a6beb09c0d0313e6f"></a>
## iter

`function` · `tracing_core::field::FieldSet::iter` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn iter(&self) -> Iter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [867, 1], "end": [973, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:914`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns an iterator over the `Field`s in this `FieldSet`.

<a id="op-4f7c25a0cd4ff9fafc6c8074"></a>
## len

`function` · `tracing_core::field::FieldSet::len` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn len(&self) -> usize
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [867, 1], "end": [973, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:964`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the number of fields in this `FieldSet`.

<a id="op-b0fc3d9471f9878ca48a9b56"></a>
## new

`function` · `tracing_core::field::FieldSet::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
const fn new(names: &'static [&'static str], callsite: callsite::Identifier) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::field::FieldSet", "path": "FieldSet"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [867, 1], "end": [973, 2], "filename": "src/field.rs"}, "trait": null, "trait_path": null}`

Source: `src/field.rs:869`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Constructs a new `FieldSet` with the given array of field names and callsite.
