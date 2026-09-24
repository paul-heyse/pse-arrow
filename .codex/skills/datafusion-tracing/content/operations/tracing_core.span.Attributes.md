# `tracing_core::span::Attributes`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.span.Attributes.json).

<a id="op-c26f3e8618922ff48422c6fb"></a>
## Attributes

`struct` · `tracing_core::span::Attributes` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Attributes<'a>
```

Source: `src/span.rs:23`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Attributes provided to a `Subscriber` describing a new span when it is
created.

<a id="op-499a1d6226c4f434143882c1"></a>
## child_of

`function` · `tracing_core::span::Attributes::child_of` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn child_of(parent: Id, metadata: &'static Metadata<'static>, values: &'a field::ValueSet<'a>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:130`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `Attributes` describing a new child span of the specified
parent span, with the provided metadata and values.

<a id="op-bc092d28bf35dbaf106e632d"></a>
## contains

`function` · `tracing_core::span::Attributes::contains` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn contains(&self, field: &field::Field) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:190`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if this set of `Attributes` contains a value for the
given `Field`.

<a id="op-8af30409b464177e5cef70bd"></a>
## fields

`function` · `tracing_core::span::Attributes::fields` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> &FieldSet
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:210`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the set of all [fields] defined by this span's [`Metadata`].

Note that the [`FieldSet`] returned by this method includes *all* the
fields declared by this span, not just those with values that are recorded
as part of this set of `Attributes`. Other fields with values not present in
this `Attributes`' value set may [record] values later.

[fields]: crate::field
[record]: Attributes::record()
[`Metadata`]: crate::metadata::Metadata
[`FieldSet`]: crate::field::FieldSet

<a id="op-363fa6e2afd7281403425c3e"></a>
## fmt

`function` · `tracing_core::span::Attributes::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [22, 10], "end": [22, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:22`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e10e8d9776769b4b1c1458c9"></a>
## is_contextual

`function` · `tracing_core::span::Attributes::is_contextual` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_contextual(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:165`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if the new span's parent should be determined based on the
current context.

If this is true and the current thread is currently inside a span, then
that span should be the new span's parent. Otherwise, if the current
thread is _not_ inside a span, then the new span will be the root of its
own trace tree.

<a id="op-d58553dba1ce658ff582c120"></a>
## is_empty

`function` · `tracing_core::span::Attributes::is_empty` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_empty(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:195`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if this set of `Attributes` contains _no_ values.

<a id="op-4dd114436de903816c000326"></a>
## is_root

`function` · `tracing_core::span::Attributes::is_root` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_root(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:154`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if the new span should be a root.

<a id="op-1519c9c96a54614773618230"></a>
## metadata

`function` · `tracing_core::span::Attributes::metadata` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> &'static Metadata<'static>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:143`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a reference to the new span's metadata.

<a id="op-788ac926e4f6dcb6a3ba53c6"></a>
## new

`function` · `tracing_core::span::Attributes::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new(metadata: &'static Metadata<'static>, values: &'a field::ValueSet<'a>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:110`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `Attributes` describing a new child span of the current span,
with the provided metadata and values.

<a id="op-8d14b45283071d4120fe44e9"></a>
## new_root

`function` · `tracing_core::span::Attributes::new_root` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new_root(metadata: &'static Metadata<'static>, values: &'a field::ValueSet<'a>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:120`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `Attributes` describing a new span at the root of its own trace
tree, with the provided metadata and values.

<a id="op-9b87f810acbb1ca9e91b50fa"></a>
## parent

`function` · `tracing_core::span::Attributes::parent` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn parent(&self) -> Option<&Id>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:173`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the new span's explicitly-specified parent, if there is one.

Otherwise (if the new span is a root or is a child of the current span),
returns `None`.

<a id="op-f3ceed5a33b84b4dda86e7f9"></a>
## record

`function` · `tracing_core::span::Attributes::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, visitor: &mut dyn field::Visit)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:184`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Records all the fields in this set of `Attributes` with the provided
[Visitor].

[visitor]: super::field::Visit

<a id="op-9ffe08e0f6b0c3695b09704b"></a>
## values

`function` · `tracing_core::span::Attributes::values` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn values(&self) -> &field::ValueSet<'a>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Attributes", "path": "Attributes"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [213, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:149`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a reference to a `ValueSet` containing any values the new span
was created with.
