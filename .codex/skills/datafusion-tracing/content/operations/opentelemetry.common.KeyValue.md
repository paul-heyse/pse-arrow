# `opentelemetry::common::KeyValue`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.common.KeyValue.json).

<a id="op-46d9e1217e370ef8ca578118"></a>
## KeyValue

`struct` · `opentelemetry::common::KeyValue` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct KeyValue
```

Source: `src/common.rs:394`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A key-value pair describing an attribute.

<a id="op-e7e0f731029b0f86c30a4c83"></a>
## clone

`function` · `opentelemetry::common::KeyValue::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> KeyValue
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::KeyValue", "path": "KeyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [392, 10], "end": [392, 15], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/common.rs:392`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-99893bda5f7035215ed3a58f"></a>
## eq

`function` · `opentelemetry::common::KeyValue::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &KeyValue) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::KeyValue", "path": "KeyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [392, 24], "end": [392, 33], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/common.rs:392`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-91fe0bfa9d93de06578034bf"></a>
## fmt

`function` · `opentelemetry::common::KeyValue::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::KeyValue", "path": "KeyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [392, 17], "end": [392, 22], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/common.rs:392`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9e9e54f32c20126273a32d29"></a>
## hash

`function` · `opentelemetry::common::KeyValue::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<H: Hasher>(&self, state: &mut H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::KeyValue", "path": "KeyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [432, 1], "end": [448, 2], "filename": "src/common.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/common.rs:433`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d74af911bb2b74871890f786"></a>
## key

`struct_field` · `opentelemetry::common::KeyValue::key` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
key: Key
```

Source: `src/common.rs:396`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The attribute name

<a id="op-9bef34b19992f1af08c9de9c"></a>
## new

`function` · `opentelemetry::common::KeyValue::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new<K, V>(key: K, value: V) -> Self where K: Into<Key>, V: Into<Value>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::common::KeyValue", "path": "KeyValue"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [402, 1], "end": [414, 2], "filename": "src/common.rs"}, "trait": null, "trait_path": null}`

Source: `src/common.rs:404`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new `KeyValue` pair.

<a id="op-871adf893f976828f2c02256"></a>
## value

`struct_field` · `opentelemetry::common::KeyValue::value` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
value: Value
```

Source: `src/common.rs:399`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The attribute value
