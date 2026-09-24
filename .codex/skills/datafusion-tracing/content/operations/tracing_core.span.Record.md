# `tracing_core::span::Record`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.span.Record.json).

<a id="op-13fdacc651796b5522602d2e"></a>
## Record

`struct` · `tracing_core::span::Record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Record<'a>
```

Source: `src/span.rs:31`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A set of fields recorded by a span.

<a id="op-35c90e58894cb6840fb68f0f"></a>
## contains

`function` · `tracing_core::span::Record::contains` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn contains(&self, field: &field::Field) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Record", "path": "Record"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [217, 1], "end": [247, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:239`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if this `Record` contains a value for the given `Field`.

<a id="op-56d321e9eab1caac193eeab1"></a>
## fmt

`function` · `tracing_core::span::Record::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Record", "path": "Record"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [30, 10], "end": [30, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:30`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ffef40e87c823cd48515f877"></a>
## is_empty

`function` · `tracing_core::span::Record::is_empty` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_empty(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Record", "path": "Record"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [217, 1], "end": [247, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:244`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if this `Record` contains _no_ values.

<a id="op-96196236c7bd1987f06d3246"></a>
## len

`function` · `tracing_core::span::Record::len` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn len(&self) -> usize
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Record", "path": "Record"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [217, 1], "end": [247, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:234`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the number of fields that would be visited from this `Record`
when [`Record::record()`] is called

[`Record::record()`]: Record::record()

<a id="op-222c5e14a211d567257e0655"></a>
## new

`function` · `tracing_core::span::Record::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new(values: &'a field::ValueSet<'a>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Record", "path": "Record"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [217, 1], "end": [247, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:219`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Constructs a new `Record` from a `ValueSet`.

<a id="op-b87d28df33c1901505abdb4e"></a>
## record

`function` · `tracing_core::span::Record::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, visitor: &mut dyn field::Visit)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_core::span::Record", "path": "Record"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [217, 1], "end": [247, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:226`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Records all the fields in this `Record` with the provided [Visitor].

[visitor]: super::field::Visit
