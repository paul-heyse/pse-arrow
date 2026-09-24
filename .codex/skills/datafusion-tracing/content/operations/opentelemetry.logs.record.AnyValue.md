# `opentelemetry::logs::record::AnyValue`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.logs.record.AnyValue.json).

<a id="op-f344d1ff8e3a1a11fc46dfb1"></a>
## AnyValue

`enum` · `opentelemetry::logs::record::AnyValue` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum AnyValue
```

Source: `src/logs/record.rs:72`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Value types for representing arbitrary values in a log record.
Note: The `tracing` and `log` crates only support basic types that can be
converted to these core variants: `i64`, `f64`, `StringValue`, and `bool`.
Any complex and custom types are supported through their Debug implementation,
and converted to String. More complex types (`Bytes`, `ListAny`, and `Map`) are
included here to meet specification requirements and are available to support
custom appenders that may be implemented for other logging crates.
These types allow for handling dynamic data structures, so keep in mind the
potential performance overhead of using boxed vectors and maps in appenders.

<a id="op-3c63a817b2900f88fb1426a4"></a>
## Boolean

`variant` · `opentelemetry::logs::record::AnyValue::Boolean` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Boolean
```

Source: `src/logs/record.rs:80`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A boolean value

<a id="op-7edea7e7a9e102e3facebcfa"></a>
## Bytes

`variant` · `opentelemetry::logs::record::AnyValue::Bytes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Bytes
```

Source: `src/logs/record.rs:82`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A byte array

<a id="op-d29c81481cd10dc40ee0674e"></a>
## Double

`variant` · `opentelemetry::logs::record::AnyValue::Double` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Double
```

Source: `src/logs/record.rs:76`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A double value

<a id="op-9cd5fd482b9d2e1a78f6429f"></a>
## Int

`variant` · `opentelemetry::logs::record::AnyValue::Int` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Int
```

Source: `src/logs/record.rs:74`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An integer value

<a id="op-0c5a06d848b9665dc0347ba4"></a>
## ListAny

`variant` · `opentelemetry::logs::record::AnyValue::ListAny` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
ListAny
```

Source: `src/logs/record.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An array of `Any` values

<a id="op-cad53515f05db2c2f90f1cea"></a>
## Map

`variant` · `opentelemetry::logs::record::AnyValue::Map` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Map
```

Source: `src/logs/record.rs:86`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A map of string keys to `Any` values, arbitrarily nested.

<a id="op-2defa5f89ebd8364d008d03f"></a>
## String

`variant` · `opentelemetry::logs::record::AnyValue::String` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
String
```

Source: `src/logs/record.rs:78`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A string value

<a id="op-b5304ebb24868189e5b2ee66"></a>
## clone

`function` · `opentelemetry::logs::record::AnyValue::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 17], "end": [70, 22], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/record.rs:70`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9d9adf770a8bc721c44671e8"></a>
## eq

`function` · `opentelemetry::logs::record::AnyValue::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &AnyValue) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 24], "end": [70, 33], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/logs/record.rs:70`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2c2304c8d8fb8295acedaf23"></a>
## fmt

`function` · `opentelemetry::logs::record::AnyValue::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 10], "end": [70, 15], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/record.rs:70`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1c3ca6ff71d55be27343e5bb"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: u16) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [105, 1], "end": [105, 39], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u16"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:105`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-21feb281139848e74fa137df"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: &'static str) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [113, 1], "end": [113, 51], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": "'static", "type": {"primitive": "str"}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:113`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-24741213c749d48404b48949"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: String) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [111, 1], "end": [111, 45], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "alloc::string::String", "path": "String"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:111`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-272038999f54418e1ecd4e94"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: f64) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [108, 1], "end": [108, 42], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f64"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-273de7f144704aea1bdaa29a"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: Cow<'static, str>) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [112, 1], "end": [112, 56], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'static"}, {"type": {"primitive": "str"}}], "constraints": []}}, "id": "alloc::borrow::Cow", "path": "Cow"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:112`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2b5572127b23e5bd2e17956e"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: i64) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [102, 1], "end": [102, 39], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i64"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2d1e4ad0cee7d39a5a526bec"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: i32) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [101, 1], "end": [101, 39], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i32"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5a0941eea857ac93f89cb2a3"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: i16) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [100, 1], "end": [100, 39], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i16"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:100`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5a40d78217a88570c58fa4e5"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: u32) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [106, 1], "end": [106, 39], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u32"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:106`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-67d4a714d39ce6fa639823d4"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: &[u8]) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [118, 1], "end": [122, 2], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"slice": {"primitive": "u8"}}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:119`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7689db84239c834fd0386b2"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: u8) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [104, 1], "end": [104, 38], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "u8"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:104`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7f0c2f856a3eb04ed1f1d4f"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: bool) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [116, 1], "end": [116, 44], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "bool"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:116`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c4da74effed0d95a870dc199"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: StringValue) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [114, 1], "end": [114, 50], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:114`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d3c47814d2542bb46858bdc2"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: i8) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [99, 1], "end": [99, 38], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "i8"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:99`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f0c0542cd231e5e084169ef2"></a>
## from

`function` · `opentelemetry::logs::record::AnyValue::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(val: f32) -> AnyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [109, 1], "end": [109, 42], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "f32"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:109`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a3674e9b2425608c8ad5afb4"></a>
## from_iter

`function` · `opentelemetry::logs::record::AnyValue::from_iter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [124, 1], "end": [129, 2], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "core::iter::traits::collect::FromIterator", "path": "FromIterator"}, "trait_path": "core::iter::traits::collect::FromIterator"}`

Source: `src/logs/record.rs:126`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates an [`AnyValue::ListAny`](../operations/opentelemetry.logs.record.AnyValue.md#op-0c5a06d848b9665dc0347ba4) value from a sequence of `Into<AnyValue>` values.

<a id="op-ae4411cdb6e21c850f050b62"></a>
## from_iter

`function` · `opentelemetry::logs::record::AnyValue::from_iter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "crate::Key"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "default": null, "is_synthetic": false}}, "name": "K"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::logs::record::AnyValue", "path": "AnyValue"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [131, 1], "end": [139, 2], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": [{"generic": "K"}, {"generic": "V"}]}}], "constraints": []}}, "id": "core::iter::traits::collect::FromIterator", "path": "FromIterator"}, "trait_path": "core::iter::traits::collect::FromIterator"}`

Source: `src/logs/record.rs:134`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates an [`AnyValue::Map`](../operations/opentelemetry.logs.record.AnyValue.md#op-cad53515f05db2c2f90f1cea) value from a sequence of key-value pairs
that can be converted into a `Key` and `AnyValue` respectively.
