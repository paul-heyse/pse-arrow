# `opentelemetry::common::Array`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.common.Array.json).

<a id="op-d2fe00ffb34bd2ceb9a4fcb3"></a>
## Array

`enum` · `opentelemetry::common::Array` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Array
```

Source: `src/common.rs:157`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A [Value::Array](../operations/opentelemetry.common.Value.md#op-764d497d9432a5127b23da60) containing homogeneous values.

<a id="op-68f1283ab99357c2dbad4b1b"></a>
## Bool

`variant` · `opentelemetry::common::Array::Bool` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Bool
```

Source: `src/common.rs:159`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Array of bools

<a id="op-3ae4a1673474015a11a53460"></a>
## F64

`variant` · `opentelemetry::common::Array::F64` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
F64
```

Source: `src/common.rs:163`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Array of floats

<a id="op-09219eeebf5f94ce69bb032f"></a>
## I64

`variant` · `opentelemetry::common::Array::I64` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
I64
```

Source: `src/common.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Array of integers

<a id="op-2fe14507f154ba72893af5ea"></a>
## String

`variant` · `opentelemetry::common::Array::String` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
String
```

Source: `src/common.rs:165`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Array of strings

<a id="op-d15e59d4ed1421fd8f9d27ab"></a>
## clone

`function` · `opentelemetry::common::Array::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Array
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [156, 10], "end": [156, 15], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/common.rs:156`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-63204583a6344c34e8a3dec5"></a>
## eq

`function` · `opentelemetry::common::Array::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Array) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [156, 24], "end": [156, 33], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/common.rs:156`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-943c5fba1721de3aaa914fc9"></a>
## fmt

`function` · `opentelemetry::common::Array::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [168, 1], "end": [186, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/common.rs:169`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9f0c09292c3a4e0b8b5e69e7"></a>
## fmt

`function` · `opentelemetry::common::Array::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [156, 17], "end": [156, 22], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/common.rs:156`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-295f4b50969db9c89cc2e1f1"></a>
## from

`function` · `opentelemetry::common::Array::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: Vec<i64>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [211, 1], "end": [216, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "alloc::vec::Vec", "path": "Vec"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:211`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2b1bc778c63cfbc42913313a"></a>
## from

`function` · `opentelemetry::common::Array::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: Vec<bool>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [211, 1], "end": [216, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "bool"}}], "constraints": []}}, "id": "alloc::vec::Vec", "path": "Vec"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:211`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cd897e4a1572ff756827c286"></a>
## from

`function` · `opentelemetry::common::Array::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: Vec<f64>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [211, 1], "end": [216, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "alloc::vec::Vec", "path": "Vec"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:211`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ec16e463fb6f993ae383406c"></a>
## from

`function` · `opentelemetry::common::Array::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: Vec<StringValue>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Array", "path": "Array"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [211, 1], "end": [216, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}}], "constraints": []}}, "id": "alloc::vec::Vec", "path": "Vec"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:211`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
