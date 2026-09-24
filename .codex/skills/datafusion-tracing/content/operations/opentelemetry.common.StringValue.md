# `opentelemetry::common::StringValue`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.common.StringValue.json).

<a id="op-aec6a10d8256d449d9700eec"></a>
## StringValue

`struct` · `opentelemetry::common::StringValue` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct StringValue
```

Source: `src/common.rs:237`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Wrapper for string-like values

<a id="op-e8ec0b2ab4efe6e9956ef794"></a>
## as_ref

`function` · `opentelemetry::common::StringValue::as_ref` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn as_ref(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [255, 1], "end": [259, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}, "trait_path": "core::convert::AsRef"}`

Source: `src/common.rs:256`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5c9b6b52e930715bc3c20117"></a>
## as_str

`function` · `opentelemetry::common::StringValue::as_str` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn as_str(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [261, 1], "end": [266, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:263`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a string slice to this value

<a id="op-f84bf156c91e5ad33bc39f00"></a>
## clone

`function` · `opentelemetry::common::StringValue::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> StringValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [236, 10], "end": [236, 15], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/common.rs:236`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7dc5d3c9300a8a91eaf8fdee"></a>
## eq

`function` · `opentelemetry::common::StringValue::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &StringValue) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [236, 17], "end": [236, 26], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/common.rs:236`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0ce78be95d4fab37b9fce852"></a>
## fmt

`function` · `opentelemetry::common::StringValue::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [245, 1], "end": [253, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/common.rs:246`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ad1070b692980495d04e951b"></a>
## fmt

`function` · `opentelemetry::common::StringValue::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [239, 1], "end": [243, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/common.rs:240`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2744c46b07c547a560d559ff"></a>
## from

`function` · `opentelemetry::common::StringValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: Value) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [305, 1], "end": [315, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:306`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-32f23e35407aa928465b2660"></a>
## from

`function` · `opentelemetry::common::StringValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: String) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [284, 1], "end": [288, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "String"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:285`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-49ff43af84c5cc96d5d5f53b"></a>
## from

`function` · `opentelemetry::common::StringValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: &'static str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [278, 1], "end": [282, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"primitive": "str"}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:279`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bc771467bc34c9d3d8130b1c"></a>
## from

`function` · `opentelemetry::common::StringValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: Arc<str>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [290, 1], "end": [294, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "alloc::sync::Arc", "path": "Arc"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:291`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e2ac20a0c4562a1859a73c97"></a>
## from

`function` · `opentelemetry::common::StringValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: Cow<'static, str>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [296, 1], "end": [303, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}, {"type": {"primitive": "str"}}], "constraints": []}}, "id": "alloc::borrow::Cow", "path": "Cow"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:297`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fbeb483f9ad2a20cd1715328"></a>
## hash

`function` · `opentelemetry::common::StringValue::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [236, 32], "end": [236, 36], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/common.rs:236`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
