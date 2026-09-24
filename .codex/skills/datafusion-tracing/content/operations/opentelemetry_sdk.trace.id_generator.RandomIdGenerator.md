# `opentelemetry_sdk::trace::id_generator::RandomIdGenerator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.id_generator.RandomIdGenerator.json).

<a id="op-8db8015dba1ee04ccc674825"></a>
## RandomIdGenerator

`struct` · `opentelemetry_sdk::trace::id_generator::RandomIdGenerator` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct RandomIdGenerator
```

Source: `src/trace/id_generator/mod.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Default [`IdGenerator`](../operations/opentelemetry_sdk.trace.id_generator.IdGenerator.md#op-cbc81e14419948e4abd3c6c2) implementation.

Generates Trace and Span ids using a random number generator.

<a id="op-713eed100def5d19481b5409"></a>
## clone

`function` · `opentelemetry_sdk::trace::id_generator::RandomIdGenerator::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> RandomIdGenerator
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::id_generator::RandomIdGenerator", "path": "RandomIdGenerator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 10], "end": [18, 15], "filename": "src/trace/id_generator/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/id_generator/mod.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-76d8f137b050775dd219281a"></a>
## default

`function` · `opentelemetry_sdk::trace::id_generator::RandomIdGenerator::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> RandomIdGenerator
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::id_generator::RandomIdGenerator", "path": "RandomIdGenerator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 24], "end": [18, 31], "filename": "src/trace/id_generator/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/id_generator/mod.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-90dcc335beecd1ca4973ec47"></a>
## fmt

`function` · `opentelemetry_sdk::trace::id_generator::RandomIdGenerator::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::id_generator::RandomIdGenerator", "path": "RandomIdGenerator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 17], "end": [18, 22], "filename": "src/trace/id_generator/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/id_generator/mod.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-07186a36ae6e440a44848056"></a>
## new_span_id

`function` · `opentelemetry_sdk::trace::id_generator::RandomIdGenerator::new_span_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new_span_id(&self) -> SpanId
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::id_generator::RandomIdGenerator", "path": "RandomIdGenerator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [23, 1], "end": [31, 2], "filename": "src/trace/id_generator/mod.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::id_generator::IdGenerator", "path": "IdGenerator"}, "trait_path": "opentelemetry_sdk::trace::id_generator::IdGenerator"}`

Source: `src/trace/id_generator/mod.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d74c2a11836aea7838507326"></a>
## new_trace_id

`function` · `opentelemetry_sdk::trace::id_generator::RandomIdGenerator::new_trace_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new_trace_id(&self) -> TraceId
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::id_generator::RandomIdGenerator", "path": "RandomIdGenerator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [23, 1], "end": [31, 2], "filename": "src/trace/id_generator/mod.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::id_generator::IdGenerator", "path": "IdGenerator"}, "trait_path": "opentelemetry_sdk::trace::id_generator::IdGenerator"}`

Source: `src/trace/id_generator/mod.rs:24`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
