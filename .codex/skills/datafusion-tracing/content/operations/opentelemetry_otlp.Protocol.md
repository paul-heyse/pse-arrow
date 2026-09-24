# `opentelemetry_otlp::Protocol`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.Protocol.json).

<a id="op-4887053d893406b1deb7b89d"></a>
## Protocol

`enum` · `opentelemetry_otlp::Protocol` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Protocol
```

Source: `src/lib.rs:441`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

The communication protocol to use when exporting data.

<a id="op-bd5f34407cfd33490f086a2e"></a>
## Grpc

`variant` · `opentelemetry_otlp::Protocol::Grpc` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Grpc
```

Source: `src/lib.rs:443`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

GRPC protocol

<a id="op-b33e3f8c28d5de297101b6f3"></a>
## HttpBinary

`variant` · `opentelemetry_otlp::Protocol::HttpBinary` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
HttpBinary
```

Source: `src/lib.rs:445`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

HTTP protocol with binary protobuf

<a id="op-b6ea27b6aa12a36a65537390"></a>
## HttpJson

`variant` · `opentelemetry_otlp::Protocol::HttpJson` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
HttpJson
```

Source: `src/lib.rs:447`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

HTTP protocol with JSON payload

<a id="op-a47077179265239f16fa7007"></a>
## clone

`function` · `opentelemetry_otlp::Protocol::clone` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Protocol
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::Protocol", "path": "Protocol"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [440, 10], "end": [440, 15], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/lib.rs:440`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-90cd517e757eed0b5cb7a6e6"></a>
## deserialize

`function` · `opentelemetry_otlp::Protocol::deserialize` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn deserialize<__D>(__deserializer: __D) -> _serde::__private226::Result<Self, __D::Error> where __D: _serde::Deserializer<'de>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::Protocol", "path": "Protocol"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'de"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [439, 42], "end": [439, 53], "filename": "src/lib.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'de"}], "constraints": []}}, "id": "serde_core::de::Deserialize", "path": "Deserialize"}, "trait_path": "serde_core::de::Deserialize"}`

Source: `src/lib.rs:439`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-15902a622171017ec1ff83ba"></a>
## eq

`function` · `opentelemetry_otlp::Protocol::eq` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Protocol) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::Protocol", "path": "Protocol"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [440, 34], "end": [440, 43], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/lib.rs:440`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-196f9c88aac4af7976875484"></a>
## fmt

`function` · `opentelemetry_otlp::Protocol::fmt` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::Protocol", "path": "Protocol"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [440, 23], "end": [440, 28], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/lib.rs:440`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-97ae18512d0dec3b23c49123"></a>
## serialize

`function` · `opentelemetry_otlp::Protocol::serialize` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn serialize<__S>(&self, __serializer: __S) -> _serde::__private226::Result<__S::Ok, __S::Error> where __S: _serde::Serializer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_otlp::Protocol", "path": "Protocol"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [439, 55], "end": [439, 64], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "serde_core::ser::Serialize", "path": "Serialize"}, "trait_path": "serde_core::ser::Serialize"}`

Source: `src/lib.rs:439`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
