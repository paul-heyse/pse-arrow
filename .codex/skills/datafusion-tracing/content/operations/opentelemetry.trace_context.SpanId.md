# `opentelemetry::trace_context::SpanId`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace_context.SpanId.json).

<a id="op-6e9d96bb2646bae3348bb60a"></a>
## SpanId

`struct` · `opentelemetry::trace_context::SpanId` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanId
```

Source: `src/trace_context.rs:155`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An 8-byte value which identifies a given span.

The id is valid if it contains at least one non-zero byte.

<a id="op-ab1e2609be38e8b3852c33e5"></a>
## INVALID

`assoc_const` · `opentelemetry::trace_context::SpanId::INVALID` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
INVALID
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [186, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:159`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Invalid span id

<a id="op-28948e17a600b69c4dc62d7c"></a>
## clone

`function` · `opentelemetry::trace_context::SpanId::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanId
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [154, 10], "end": [154, 15], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace_context.rs:154`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9e8b12aa8af2d136afa1a38b"></a>
## eq

`function` · `opentelemetry::trace_context::SpanId::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SpanId) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [154, 17], "end": [154, 26], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace_context.rs:154`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-85fee82458ecf9c92c180fcd"></a>
## fmt

`function` · `opentelemetry::trace_context::SpanId::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [206, 1], "end": [210, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::LowerHex", "path": "LowerHex"}, "trait_path": "core::fmt::LowerHex"}`

Source: `src/trace_context.rs:207`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9a846a5d2add40c3e01425aa"></a>
## fmt

`function` · `opentelemetry::trace_context::SpanId::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [200, 1], "end": [204, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/trace_context.rs:201`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aa9ba2bdd4913a0234e6f526"></a>
## fmt

`function` · `opentelemetry::trace_context::SpanId::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [194, 1], "end": [198, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace_context.rs:195`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3290425c7972ca1af343d508"></a>
## from

`function` · `opentelemetry::trace_context::SpanId::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: u64) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [188, 1], "end": [192, 2], "filename": "src/trace_context.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u64"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/trace_context.rs:189`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-36c3e38c58772f8223eeeab7"></a>
## from_bytes

`function` · `opentelemetry::trace_context::SpanId::from_bytes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const fn from_bytes(bytes: [u8; 8]) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [186, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:162`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a span id from its representation as a byte array.

<a id="op-022f176c9a00e4b1bacaeb42"></a>
## from_hex

`function` · `opentelemetry::trace_context::SpanId::from_hex` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_hex(hex: &str) -> Result<Self, ParseIntError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [186, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:183`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Converts a string in base 16 to a span id.

# Examples

```
use opentelemetry::trace::SpanId;

assert!(SpanId::from_hex("42").is_ok());
assert!(SpanId::from_hex("58406520a0066491").is_ok());

assert!(SpanId::from_hex("not_hex").is_err());
```

<a id="op-9672edad9b2e56f60b0bfb8d"></a>
## hash

`function` · `opentelemetry::trace_context::SpanId::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [154, 38], "end": [154, 42], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/trace_context.rs:154`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-92a5aa7c76106b8c81a0a649"></a>
## to_bytes

`function` · `opentelemetry::trace_context::SpanId::to_bytes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const fn to_bytes(self) -> [u8; 8]
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::SpanId", "path": "SpanId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 1], "end": [186, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:167`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Return the representation of this span id as a byte array.
