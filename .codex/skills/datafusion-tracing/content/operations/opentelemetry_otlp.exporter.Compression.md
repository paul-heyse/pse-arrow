# `opentelemetry_otlp::exporter::Compression`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.exporter.Compression.json).

<a id="op-dfdc34334d6db40f477b7a32"></a>
## Compression

`enum` · `opentelemetry_otlp::exporter::Compression` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Compression
```

Source: `src/exporter/mod.rs:142`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

The compression algorithm to use when sending data.

<a id="op-58f0ddd547a749e0a48f3af7"></a>
## Err

`assoc_type` · `opentelemetry_otlp::exporter::Compression::Err` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Err
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [158, 1], "end": [170, 2], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/exporter/mod.rs:159`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ca27c9f61d9f0eb1ff6be32e"></a>
## Gzip

`variant` · `opentelemetry_otlp::exporter::Compression::Gzip` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Gzip
```

Source: `src/exporter/mod.rs:144`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Compresses data using gzip.

<a id="op-f2c93aa0dfeaf268f2faf6b0"></a>
## Zstd

`variant` · `opentelemetry_otlp::exporter::Compression::Zstd` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Zstd
```

Source: `src/exporter/mod.rs:146`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

Compresses data using zstd.

<a id="op-3bfefbe9208bd40c3cc49e93"></a>
## clone

`function` · `opentelemetry_otlp::exporter::Compression::clone` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Compression
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [141, 10], "end": [141, 15], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/exporter/mod.rs:141`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7501fbbb034662ad4a4c378f"></a>
## deserialize

`function` · `opentelemetry_otlp::exporter::Compression::deserialize` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn deserialize<__D>(__deserializer: __D) -> _serde::__private226::Result<Self, __D::Error> where __D: _serde::Deserializer<'de>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'de"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [140, 42], "end": [140, 53], "filename": "src/exporter/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'de"}], "constraints": []}}, "id": "serde_core::de::Deserialize", "path": "Deserialize"}, "trait_path": "serde_core::de::Deserialize"}`

Source: `src/exporter/mod.rs:140`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9a4ef79f08e6730dce538229"></a>
## eq

`function` · `opentelemetry_otlp::exporter::Compression::eq` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Compression) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [141, 34], "end": [141, 43], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/exporter/mod.rs:141`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7e656f5b8c5d9180b0faff19"></a>
## fmt

`function` · `opentelemetry_otlp::exporter::Compression::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [149, 1], "end": [156, 2], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/exporter/mod.rs:150`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cdc1999be9ff780ab0ff63de"></a>
## fmt

`function` · `opentelemetry_otlp::exporter::Compression::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [141, 23], "end": [141, 28], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/exporter/mod.rs:141`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e648dd202aa9429382fa6879"></a>
## from_str

`function` · `opentelemetry_otlp::exporter::Compression::from_str` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_str(s: &str) -> Result<Self, Self::Err>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [158, 1], "end": [170, 2], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/exporter/mod.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f5bdb87cfd68dd651a27af39"></a>
## serialize

`function` · `opentelemetry_otlp::exporter::Compression::serialize` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn serialize<__S>(&self, __serializer: __S) -> _serde::__private226::Result<__S::Ok, __S::Error> where __S: _serde::Serializer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::exporter::Compression", "path": "Compression"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [140, 55], "end": [140, 64], "filename": "src/exporter/mod.rs"}, "trait": {"args": null, "id": "serde_core::ser::Serialize", "path": "Serialize"}, "trait_path": "serde_core::ser::Serialize"}`

Source: `src/exporter/mod.rs:140`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
