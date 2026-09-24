# `opentelemetry::trace_context::TraceId`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace_context.TraceId.json).

<a id="op-436bb6ec32d874fe466755ee"></a>
## TraceId

`struct` · `opentelemetry::trace_context::TraceId` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TraceId
```

Source: `src/trace_context.rs:94`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A 16-byte value which identifies a given trace.

The id is valid if it contains at least one non-zero byte.

<a id="op-8ce00ee002c37f87bd559c22"></a>
## INVALID

`assoc_const` · `opentelemetry::trace_context::TraceId::INVALID` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
INVALID
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 1], "end": [125, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:98`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Invalid trace id

<a id="op-364d68ded5a7435cce5d0184"></a>
## clone

`function` · `opentelemetry::trace_context::TraceId::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> TraceId
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [93, 10], "end": [93, 15], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace_context.rs:93`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a8810a33e637ebbae8964697"></a>
## eq

`function` · `opentelemetry::trace_context::TraceId::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &TraceId) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [93, 17], "end": [93, 26], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace_context.rs:93`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d8ed41a5752a1e0bbabe857"></a>
## fmt

`function` · `opentelemetry::trace_context::TraceId::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [145, 1], "end": [149, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::LowerHex", "path": "LowerHex"}, "trait_path": "core::fmt::LowerHex"}`

Source: `src/trace_context.rs:146`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a6c8d8123ead9a62d5498798"></a>
## fmt

`function` · `opentelemetry::trace_context::TraceId::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [133, 1], "end": [137, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace_context.rs:134`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f869dc1e69cf6155fcb06fe5"></a>
## fmt

`function` · `opentelemetry::trace_context::TraceId::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [139, 1], "end": [143, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/trace_context.rs:140`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-81d09171cbff30f38a8353d4"></a>
## from

`function` · `opentelemetry::trace_context::TraceId::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: u128) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [127, 1], "end": [131, 2], "filename": "src/trace_context.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u128"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/trace_context.rs:128`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7823c06a91177d5e6b491d67"></a>
## from_bytes

`function` · `opentelemetry::trace_context::TraceId::from_bytes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const fn from_bytes(bytes: [u8; 16]) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 1], "end": [125, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a trace id from its representation as a byte array.

<a id="op-00e6a21791e1c54dbd739036"></a>
## from_hex

`function` · `opentelemetry::trace_context::TraceId::from_hex` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_hex(hex: &str) -> Result<Self, ParseIntError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 1], "end": [125, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:122`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Converts a string in base 16 to a trace id.

# Examples

```
use opentelemetry::trace::TraceId;

assert!(TraceId::from_hex("42").is_ok());
assert!(TraceId::from_hex("58406520a006649127e371903a2de979").is_ok());

assert!(TraceId::from_hex("not_hex").is_err());
```

<a id="op-17cb43c7df1312d235e10d1e"></a>
## hash

`function` · `opentelemetry::trace_context::TraceId::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [93, 38], "end": [93, 42], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/trace_context.rs:93`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b64e8e99a0b8406750486a90"></a>
## to_bytes

`function` · `opentelemetry::trace_context::TraceId::to_bytes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const fn to_bytes(self) -> [u8; 16]
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceId", "path": "TraceId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 1], "end": [125, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:106`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Return the representation of this trace id as a byte array.
