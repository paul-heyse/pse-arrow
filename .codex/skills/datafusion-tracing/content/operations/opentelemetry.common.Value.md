# `opentelemetry::common::Value`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.common.Value.json).

<a id="op-ddc0e7da0503a90e74a1640a"></a>
## Value

`enum` · `opentelemetry::common::Value` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum Value
```

Source: `src/common.rs:221`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The value part of attribute [KeyValue](../operations/opentelemetry.common.KeyValue.md#op-46d9e1217e370ef8ca578118) pairs.

<a id="op-764d497d9432a5127b23da60"></a>
## Array

`variant` · `opentelemetry::common::Value::Array` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Array
```

Source: `src/common.rs:231`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Array of homogeneous values

<a id="op-0e44b86d3737090d0237e741"></a>
## Bool

`variant` · `opentelemetry::common::Value::Bool` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Bool
```

Source: `src/common.rs:223`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

bool values

<a id="op-a93b1abed46c591c19057379"></a>
## F64

`variant` · `opentelemetry::common::Value::F64` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
F64
```

Source: `src/common.rs:227`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

f64 values

<a id="op-8f86b338f3b04dc1b00012b9"></a>
## I64

`variant` · `opentelemetry::common::Value::I64` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
I64
```

Source: `src/common.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

i64 values

<a id="op-313162f7bc038a4d0d6dd6f4"></a>
## String

`variant` · `opentelemetry::common::Value::String` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
String
```

Source: `src/common.rs:229`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

String values

<a id="op-268bd55f8be5c4f8cd5e24c4"></a>
## as_str

`function` · `opentelemetry::common::Value::as_str` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn as_str(&self) -> Cow<'_, str>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [317, 1], "end": [330, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:321`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

String representation of the `Value`

This will allocate if the underlying value is not a `String`.

<a id="op-588a4c74955d9b6c5b76fd88"></a>
## clone

`function` · `opentelemetry::common::Value::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Value
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [220, 10], "end": [220, 15], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/common.rs:220`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f716eccbfb20cc7f5a4e727e"></a>
## eq

`function` · `opentelemetry::common::Value::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Value) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [220, 24], "end": [220, 33], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/common.rs:220`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c2a111fd72c2adca45b0d0aa"></a>
## fmt

`function` · `opentelemetry::common::Value::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [379, 1], "end": [389, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/common.rs:380`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d10532506adf9dc50b0f63d3"></a>
## fmt

`function` · `opentelemetry::common::Value::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [220, 17], "end": [220, 22], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/common.rs:220`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-64de06aa634980846792a219"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [348, 1], "end": [353, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "bool"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:348`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-705160ada0697dd091b3cfc1"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: Arc<str>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [367, 1], "end": [371, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "alloc::sync::Arc", "path": "Arc"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:368`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b18f116caa9467701a700033"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: String) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [361, 1], "end": [365, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "String"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:362`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d2f285f8c919dc26beef3d2c"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: &'static str) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [355, 1], "end": [359, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"primitive": "str"}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:356`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d4e5e58e4be5fcb649b50395"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: StringValue) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [348, 1], "end": [353, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:348`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d72ccf927fe72942f525ccea"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: f64) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [348, 1], "end": [353, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:348`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d7b62b743acf109971031040"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(t: i64) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [348, 1], "end": [353, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:348`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f51498228bc4438355a5cf13"></a>
## from

`function` · `opentelemetry::common::Value::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(s: Cow<'static, str>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::Value", "path": "Value"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [373, 1], "end": [377, 2], "filename": "src/common.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}, {"type": {"primitive": "str"}}], "constraints": []}}, "id": "alloc::borrow::Cow", "path": "Cow"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/common.rs:374`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
