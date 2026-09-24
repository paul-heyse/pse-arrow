# `opentelemetry::baggage::BaggageMetadata`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.baggage.BaggageMetadata.json).

<a id="op-9b7a73c17917c955c26935bd"></a>
## BaggageMetadata

`struct` · `opentelemetry::baggage::BaggageMetadata` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BaggageMetadata
```

Source: `src/baggage.rs:430`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An optional property set that can be added to [`Baggage`](../operations/opentelemetry.baggage.Baggage.md#op-2c88c4516fa3cd4da1db039e) values.

`BaggageMetadata` can be added to values in the form of a property set,
represented as semi-colon `;` delimited list of names and/or name/value
pairs, e.g. `;k1=v1;k2;k3=v3`.

<a id="op-65d5053ff6a3c7fea6c36453"></a>
## as_str

`function` · `opentelemetry::baggage::BaggageMetadata::as_str` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn as_str(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [432, 1], "end": [437, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:434`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Return underlying string

<a id="op-8ffdb3ff30725268c51e115c"></a>
## clone

`function` · `opentelemetry::baggage::BaggageMetadata::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> BaggageMetadata
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [429, 10], "end": [429, 15], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/baggage.rs:429`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d1b1ccfb07c6546cf05519e0"></a>
## default

`function` · `opentelemetry::baggage::BaggageMetadata::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> BaggageMetadata
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [429, 51], "end": [429, 58], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/baggage.rs:429`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ef2061455424b28dfeaf68ce"></a>
## eq

`function` · `opentelemetry::baggage::BaggageMetadata::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &BaggageMetadata) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [429, 36], "end": [429, 45], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/baggage.rs:429`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6491bed24fa5eca867788507"></a>
## fmt

`function` · `opentelemetry::baggage::BaggageMetadata::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [429, 17], "end": [429, 22], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/baggage.rs:429`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f6d5d4442e3988b3c39e4b27"></a>
## fmt

`function` · `opentelemetry::baggage::BaggageMetadata::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [451, 1], "end": [455, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/baggage.rs:452`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1452dfe835be4ecd89da580a"></a>
## from

`function` · `opentelemetry::baggage::BaggageMetadata::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: String) -> BaggageMetadata
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [439, 1], "end": [443, 2], "filename": "src/baggage.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "String"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/baggage.rs:440`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ff65bda53eb29479f637bba1"></a>
## from

`function` · `opentelemetry::baggage::BaggageMetadata::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: &str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [445, 1], "end": [449, 2], "filename": "src/baggage.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"primitive": "str"}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/baggage.rs:446`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d37f4d604098feac3dd8da13"></a>
## partial_cmp

`function` · `opentelemetry::baggage::BaggageMetadata::partial_cmp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &BaggageMetadata) -> option::Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [429, 24], "end": [429, 34], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/baggage.rs:429`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
