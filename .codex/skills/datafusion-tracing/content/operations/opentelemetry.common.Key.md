# `opentelemetry::common::Key`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.common.Key.json).

<a id="op-34a9700de114d39200e4e0ed"></a>
## Key

`struct` · `opentelemetry::common::Key` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Key
```

Source: `src/common.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The key part of attribute [KeyValue](../operations/opentelemetry.common.KeyValue.md#op-46d9e1217e370ef8ca578118) pairs.

See the [attribute naming] spec for guidelines.

[attribute naming]: https://github.com/open-telemetry/semantic-conventions/blob/main/docs/general/attribute-naming.md

<a id="op-73c8b5933a57194d361b9dfb"></a>
## as_ref

`function` · `opentelemetry::common::Key::as_ref` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn as_ref(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [111, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}, "trait_path": "core::convert::AsRef"}`

Source: `src/common.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6694319e22f08bf2f88065a3"></a>
## as_str

`function` · `opentelemetry::common::Key::as_str` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn as_str(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [16, 1], "end": [42, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:39`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a reference to the underlying key name

<a id="op-9025d083b4024d3d32f3e35a"></a>
## borrow

`function` · `opentelemetry::common::Key::borrow` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn borrow(&self) -> &str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [101, 1], "end": [105, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::borrow::Borrow", "path": "Borrow"}, "trait_path": "core::borrow::Borrow"}`

Source: `src/common.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-529d6c178c3f190258dafe7b"></a>
## clone

`function` · `opentelemetry::common::Key::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Key
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 10], "end": [13, 15], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/common.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fd7a7b112140ac3d09ec7f98"></a>
## cmp

`function` · `opentelemetry::common::Key::cmp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn cmp(&self, other: &Key) -> cmp::Ordering
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 50], "end": [13, 53], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::Ord", "path": "Ord"}, "trait_path": "core::cmp::Ord"}`

Source: `src/common.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e0136c69a105fd7021532d49"></a>
## eq

`function` · `opentelemetry::common::Key::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Key) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 17], "end": [13, 26], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/common.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-23f9073c5225f6f575be1797"></a>
## fmt

`function` · `opentelemetry::common::Key::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [91, 1], "end": [99, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/common.rs:92`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2cc97ada22a5f927dbda0d0b"></a>
## fmt

`function` · `opentelemetry::common::Key::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [75, 1], "end": [79, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/common.rs:76`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-01049dfb6f0eba06c42f9b6e"></a>
## from

`function` · `opentelemetry::common::Key::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(key_str: &'static str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [44, 1], "end": [49, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"primitive": "str"}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:46`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Convert a `&str` to a `Key`.

<a id="op-63758d644c90b9731d83b3ee"></a>
## from

`function` · `opentelemetry::common::Key::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(string: Arc<str>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [58, 1], "end": [63, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "alloc::sync::Arc", "path": "Arc"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:60`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Convert a `String` to a `Key`.

<a id="op-8929e0883a3d2be51a939dd1"></a>
## from

`function` · `opentelemetry::common::Key::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(string: Cow<'static, str>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [65, 1], "end": [73, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}, {"type": {"primitive": "str"}}], "constraints": []}}, "id": "alloc::borrow::Cow", "path": "Cow"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:67`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Convert a `Cow<'static, str>` to a `Key`

<a id="op-90e016789126788df0060141"></a>
## from

`function` · `opentelemetry::common::Key::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(string: String) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 1], "end": [56, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "String"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:53`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Convert a `String` to a `Key`.

<a id="op-8a0c6772dfc846dc8ae7a15e"></a>
## from_static_str

`function` · `opentelemetry::common::Key::from_static_str` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const fn from_static_str(value: &'static str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [16, 1], "end": [42, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new const `Key`.

<a id="op-35e8f87bedf394c715b7567c"></a>
## hash

`function` · `opentelemetry::common::Key::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 32], "end": [13, 36], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/common.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1121a9fecb46b03d3bcca0a4"></a>
## new

`function` · `opentelemetry::common::Key::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(value: impl Into<Key>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [16, 1], "end": [42, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new `Key`.

# Examples

```
use opentelemetry::Key;
use std::sync::Arc;

let key1 = Key::new("my_static_str");
let key2 = Key::new(String::from("my_owned_string"));
let key3 = Key::new(Arc::from("my_ref_counted_str"));
```

<a id="op-aea985fa57eff2f70a3042ab"></a>
## partial_cmp

`function` · `opentelemetry::common::Key::partial_cmp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &Key) -> option::Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [13, 38], "end": [13, 48], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/common.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
