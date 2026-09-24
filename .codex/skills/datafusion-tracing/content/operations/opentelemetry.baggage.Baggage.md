# `opentelemetry::baggage::Baggage`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.baggage.Baggage.json).

<a id="op-2c88c4516fa3cd4da1db039e"></a>
## Baggage

`struct` · `opentelemetry::baggage::Baggage` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Baggage
```

Source: `src/baggage.rs:67`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A set of name/value pairs describing user-defined properties.

### Baggage Names

* ASCII strings according to the token format, defined in [RFC2616, Section 2.2]

### Baggage Values

* URL encoded UTF-8 strings.

### Baggage Value Metadata

Additional metadata can be added to values in the form of a property set,
represented as semi-colon `;` delimited list of names and/or name/value pairs,
e.g. `;k1=v1;k2;k3=v3`.

### Limits

* Maximum number of name/value pairs: `64`.
* Maximum total length of all name/value pairs: `8192`.

<https://www.w3.org/TR/baggage/#limits>

<a id="op-c6fcc8ee3d8808ba6ca2ea80"></a>
## default

`function` · `opentelemetry::baggage::Baggage::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Baggage
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 17], "end": [66, 24], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/baggage.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3ce1ddeac1ee2a85389b8dac"></a>
## fmt

`function` · `opentelemetry::baggage::Baggage::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 10], "end": [66, 15], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/baggage.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f763d4e4b6b80d6bf785584e"></a>
## fmt

`function` · `opentelemetry::baggage::Baggage::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [318, 1], "end": [333, 2], "filename": "src/baggage.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/baggage.rs:319`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dc8d8296e71870311d1f69a1"></a>
## from

`function` · `opentelemetry::baggage::Baggage::from` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(value: I) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "I"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": "IntoIterator"}}}], "generic_params": [], "type": {"generic": "I"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::KeyValueMetadata", "path": "KeyValueMetadata"}}}], "constraints": []}}, "id": "core::convert::Into", "path": "Into"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Item", "self_type": {"generic": "I"}, "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [293, 1], "end": [301, 2], "filename": "src/baggage.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "I"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/baggage.rs:298`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-79640dc19d7dbcb61ff9b51e"></a>
## from_iter

`function` · `opentelemetry::baggage::Baggage::from_iter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_iter<I: IntoIterator<Item = (Key, (StringValue, BaggageMetadata))>>(iter: I) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [263, 1], "end": [271, 2], "filename": "src/baggage.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": [{"resolved_path": {"args": null, "id": "opentelemetry::common::Key", "path": "Key"}}, {"tuple": [{"resolved_path": {"args": null, "id": "opentelemetry::common::StringValue", "path": "StringValue"}}, {"resolved_path": {"args": null, "id": "opentelemetry::baggage::BaggageMetadata", "path": "BaggageMetadata"}}]}]}}], "constraints": []}}, "id": "core::iter::traits::collect::FromIterator", "path": "FromIterator"}, "trait_path": "core::iter::traits::collect::FromIterator"}`

Source: `src/baggage.rs:264`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b99acb69572f4f010b964180"></a>
## from_iter

`function` · `opentelemetry::baggage::Baggage::from_iter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_iter<I: IntoIterator<Item = KeyValueMetadata>>(iter: I) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [283, 1], "end": [291, 2], "filename": "src/baggage.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::KeyValueMetadata", "path": "KeyValueMetadata"}}}], "constraints": []}}, "id": "core::iter::traits::collect::FromIterator", "path": "FromIterator"}, "trait_path": "core::iter::traits::collect::FromIterator"}`

Source: `src/baggage.rs:284`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ce208a0205aaff80bb2a85d1"></a>
## from_iter

`function` · `opentelemetry::baggage::Baggage::from_iter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_iter<I: IntoIterator<Item = KeyValue>>(iter: I) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [273, 1], "end": [281, 2], "filename": "src/baggage.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "opentelemetry::common::KeyValue", "path": "KeyValue"}}}], "constraints": []}}, "id": "core::iter::traits::collect::FromIterator", "path": "FromIterator"}, "trait_path": "core::iter::traits::collect::FromIterator"}`

Source: `src/baggage.rs:274`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7e7599106ddcdd6c45a0d603"></a>
## get

`function` · `opentelemetry::baggage::Baggage::get` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get<K: AsRef<str>>(&self, key: K) -> Option<&StringValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:93`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a reference to the value associated with a given name

# Examples

```
use opentelemetry::{baggage::Baggage, StringValue};

let mut baggage = Baggage::new();
let _ = baggage.insert("my-name", "my-value");

assert_eq!(baggage.get("my-name"), Some(&StringValue::from("my-value")))
```

<a id="op-7fcfff82e24f03cd16638295"></a>
## get_with_metadata

`function` · `opentelemetry::baggage::Baggage::get_with_metadata` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get_with_metadata<K: AsRef<str>>(&self, key: K) -> Option<&(StringValue, BaggageMetadata)>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:109`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a reference to the value and metadata associated with a given name

# Examples
```
use opentelemetry::{baggage::{Baggage, BaggageMetadata}, StringValue};

let mut baggage = Baggage::new();
let _ = baggage.insert("my-name", "my-value");

// By default, the metadata is empty
assert_eq!(baggage.get_with_metadata("my-name"), Some(&(StringValue::from("my-value"), BaggageMetadata::from(""))))
```

<a id="op-453dc255572fd1dae898b80a"></a>
## insert

`function` · `opentelemetry::baggage::Baggage::insert` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn insert<K, V>(&mut self, key: K, value: V) -> Option<StringValue> where K: Into<Key>, V: Into<StringValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:131`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Inserts a name/value pair into the baggage.

If the name was not present, [`None`] is returned. If the name was present,
the value is updated, and the old value is returned.

# Examples

```
use opentelemetry::{baggage::Baggage, StringValue};

let mut baggage = Baggage::new();
let _ = baggage.insert("my-name", "my-value");

assert_eq!(baggage.get("my-name"), Some(&StringValue::from("my-value")))
```

Unresolved upstream links (retained, not inferred): ``None``.

<a id="op-45b73a2bdeeeb1ed7ddd3607"></a>
## insert_with_metadata

`function` · `opentelemetry::baggage::Baggage::insert_with_metadata` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn insert_with_metadata<K, V, S>(&mut self, key: K, value: V, metadata: S) -> Option<(StringValue, BaggageMetadata)> where K: Into<Key>, V: Into<StringValue>, S: Into<BaggageMetadata>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:157`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Inserts a name/value(+metadata) pair into the baggage.

Same with `insert`, if the name was not present, [`None`] will be returned.
If the name is present, the old value and metadata will be returned.

Also checks for [limits](https://w3c.github.io/baggage/#limits).

# Examples

```
use opentelemetry::{baggage::{Baggage, BaggageMetadata}, StringValue};

let mut baggage = Baggage::new();
let _ = baggage.insert_with_metadata("my-name", "my-value", "test");

assert_eq!(baggage.get_with_metadata("my-name"), Some(&(StringValue::from("my-value"), BaggageMetadata::from("test"))))
```

Unresolved upstream links (retained, not inferred): ``None``.

<a id="op-d9c45f3671c06945a5da8497"></a>
## is_empty

`function` · `opentelemetry::baggage::Baggage::is_empty` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_empty(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:220`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns `true` if the baggage contains no items.

<a id="op-76c5d713de287087aaa36d80"></a>
## iter

`function` · `opentelemetry::baggage::Baggage::iter` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn iter(&self) -> Iter<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Gets an iterator over the baggage items, in any order.

<a id="op-6cc9502a30432e0444b2ed10"></a>
## len

`function` · `opentelemetry::baggage::Baggage::len` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn len(&self) -> usize
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:215`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the number of attributes for this baggage

<a id="op-abab6c606502ed0bb34a463b"></a>
## new

`function` · `opentelemetry::baggage::Baggage::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:74`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates an empty `Baggage`.

<a id="op-94be5194bf132e69eae31550"></a>
## remove

`function` · `opentelemetry::baggage::Baggage::remove` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn remove<K: AsRef<str>>(&mut self, key: K) -> Option<(StringValue, BaggageMetadata)>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::baggage::Baggage", "path": "Baggage"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 1], "end": [235, 2], "filename": "src/baggage.rs"}, "trait": null, "trait_path": null}`

Source: `src/baggage.rs:210`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Removes a name from the baggage, returning the value
corresponding to the name if the pair was previously in the map.
