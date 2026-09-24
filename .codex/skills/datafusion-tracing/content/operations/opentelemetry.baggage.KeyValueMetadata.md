# `opentelemetry::baggage::KeyValueMetadata`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.baggage.KeyValueMetadata.json).

<a id="op-e01123165c001216c42bef8e"></a>
## KeyValueMetadata

`struct` · `opentelemetry::baggage::KeyValueMetadata` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct KeyValueMetadata
```

Source: `src/baggage.rs:459`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

[`Baggage`](../operations/opentelemetry.baggage.Baggage.md#op-2c88c4516fa3cd4da1db039e) name/value pairs with their associated metadata.

<a id="op-21e15a762052138058867c8e"></a>
## clone

`function` · `opentelemetry::baggage::KeyValueMetadata::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> KeyValueMetadata
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::KeyValueMetadata", "path": "KeyValueMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [458, 10], "end": [458, 15], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/baggage.rs:458`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a1141df7314bfcdbff1cb160"></a>
## eq

`function` · `opentelemetry::baggage::KeyValueMetadata::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &KeyValueMetadata) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::KeyValueMetadata", "path": "KeyValueMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [458, 24], "end": [458, 33], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/baggage.rs:458`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5cb494cabd7c8791644feb88"></a>
## fmt

`function` · `opentelemetry::baggage::KeyValueMetadata::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::KeyValueMetadata", "path": "KeyValueMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [458, 17], "end": [458, 22], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/baggage.rs:458`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c5568b16442f93c4753432ae"></a>
## from

`function` · `opentelemetry::baggage::KeyValueMetadata::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(kv: KeyValue) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::KeyValueMetadata", "path": "KeyValueMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [484, 1], "end": [492, 2], "filename": "src/baggage.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::common::KeyValue", "path": "KeyValue"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/baggage.rs:485`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b295ad5cf771c0ee81071b05"></a>
## new

`function` · `opentelemetry::baggage::KeyValueMetadata::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new<K, V, S>(key: K, value: V, metadata: S) -> Self where K: Into<Key>, V: Into<StringValue>, S: Into<BaggageMetadata>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::KeyValueMetadata", "path": "KeyValueMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [468, 1], "end": [482, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:470`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new `KeyValue` pair with metadata
