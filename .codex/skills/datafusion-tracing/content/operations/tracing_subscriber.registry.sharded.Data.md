# `tracing_subscriber::registry::sharded::Data`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.sharded.Data.json).

<a id="op-a934e2d876b09547f78e2332"></a>
## Data

`struct` · `tracing_subscriber::registry::sharded::Data` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Data<'a>
```

Source: `src/registry/sharded.rs:111`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Span data stored in a [`Registry`](../operations/tracing_subscriber.registry.sharded.Registry.md#op-4736e765871f1c85d718ade6).

The registry stores well-known data defined by tracing: span relationships,
metadata and reference counts. Additional user-defined data provided by
[`Layer`s], such as formatted fields, metrics, or distributed traces should
be stored in the [extensions] typemap.

[`Layer`s]: crate::layer::Layer
[extensions]: Extensions

<a id="op-012cbb1f397737e36505a7f7"></a>
## extensions

`function` · `tracing_subscriber::registry::sharded::Data::extensions` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn extensions(&self) -> Extensions<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::sharded::Data", "path": "Data"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [414, 1], "end": [439, 2], "filename": "src/registry/sharded.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanData", "path": "SpanData"}, "trait_path": "tracing_subscriber::registry::SpanData"}`

Source: `src/registry/sharded.rs:427`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ac5bd7e84bca85918c5d147b"></a>
## extensions_mut

`function` · `tracing_subscriber::registry::sharded::Data::extensions_mut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn extensions_mut(&self) -> ExtensionsMut<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::sharded::Data", "path": "Data"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [414, 1], "end": [439, 2], "filename": "src/registry/sharded.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanData", "path": "SpanData"}, "trait_path": "tracing_subscriber::registry::SpanData"}`

Source: `src/registry/sharded.rs:431`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-818649889d481e94a961940c"></a>
## fmt

`function` · `tracing_subscriber::registry::sharded::Data::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::sharded::Data", "path": "Data"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 10], "end": [110, 15], "filename": "src/registry/sharded.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/registry/sharded.rs:110`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-10e0182251c7e9c55f38dd1a"></a>
## id

`function` · `tracing_subscriber::registry::sharded::Data::id` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn id(&self) -> Id
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::sharded::Data", "path": "Data"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [414, 1], "end": [439, 2], "filename": "src/registry/sharded.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanData", "path": "SpanData"}, "trait_path": "tracing_subscriber::registry::SpanData"}`

Source: `src/registry/sharded.rs:415`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-14592e8dc0601c94cb3bebf5"></a>
## is_enabled_for

`function` · `tracing_subscriber::registry::sharded::Data::is_enabled_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn is_enabled_for(&self, filter: FilterId) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::sharded::Data", "path": "Data"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [414, 1], "end": [439, 2], "filename": "src/registry/sharded.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanData", "path": "SpanData"}, "trait_path": "tracing_subscriber::registry::SpanData"}`

Source: `src/registry/sharded.rs:436`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-95fd27793ec209d6f20b3ffd"></a>
## metadata

`function` · `tracing_subscriber::registry::sharded::Data::metadata` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> &'static Metadata<'static>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::sharded::Data", "path": "Data"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [414, 1], "end": [439, 2], "filename": "src/registry/sharded.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanData", "path": "SpanData"}, "trait_path": "tracing_subscriber::registry::SpanData"}`

Source: `src/registry/sharded.rs:419`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ae49f5ba61a38d4c35756eba"></a>
## parent

`function` · `tracing_subscriber::registry::sharded::Data::parent` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn parent(&self) -> Option<&Id>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::sharded::Data", "path": "Data"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [414, 1], "end": [439, 2], "filename": "src/registry/sharded.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanData", "path": "SpanData"}, "trait_path": "tracing_subscriber::registry::SpanData"}`

Source: `src/registry/sharded.rs:423`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
