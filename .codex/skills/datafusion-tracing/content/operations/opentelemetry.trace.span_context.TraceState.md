# `opentelemetry::trace::span_context::TraceState`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.span_context.TraceState.json).

<a id="op-93d50de145537665fcb28b20"></a>
## TraceState

`struct` · `opentelemetry::trace::span_context::TraceState` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TraceState
```

Source: `src/trace/span_context.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

TraceState carries system-specific configuration data, represented as a list
of key-value pairs. TraceState allows multiple tracing systems to
participate in the same trace.

Please review the [W3C specification] for details on this field.

[W3C specification]: https://www.w3.org/TR/trace-context/#tracestate-header

<a id="op-221aa27de3c52eae343982c3"></a>
## Err

`assoc_type` · `opentelemetry::trace::span_context::TraceState::Err` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Err
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [192, 1], "end": [212, 2], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/trace/span_context.rs:193`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ba92065ab018327c1e10acd9"></a>
## NONE

`assoc_const` · `opentelemetry::trace::span_context::TraceState::NONE` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
NONE
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [190, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The default `TraceState`, as a constant

<a id="op-63ad3220041d6a750a39be25"></a>
## clone

`function` · `opentelemetry::trace::span_context::TraceState::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> TraceState
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 10], "end": [14, 15], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/span_context.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2e39cf032128a6d51804c248"></a>
## default

`function` · `opentelemetry::trace::span_context::TraceState::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> TraceState
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 24], "end": [14, 31], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/span_context.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-03f1894ffc7a1f279a848460"></a>
## delete

`function` · `opentelemetry::trace::span_context::TraceState::delete` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn delete<K: Into<String>>(&self, key: K) -> Result<TraceState, TraceStateError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [190, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:152`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Removes the given key-value pair from the `TraceState`. If the key is invalid per the
[W3 Spec] an `Err` is returned. Else, a new `TraceState`
with the removed entry is returned.

If the key is not in `TraceState`. The original `TraceState` will be cloned and returned.

[W3 Spec]: https://www.w3.org/TR/trace-context/#mutating-the-tracestate-field

<a id="op-c70e68c3e70aa1f3c58d6d7a"></a>
## eq

`function` · `opentelemetry::trace::span_context::TraceState::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &TraceState) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 37], "end": [14, 46], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/span_context.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4ee189d3546f023d28e17c82"></a>
## fmt

`function` · `opentelemetry::trace::span_context::TraceState::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 17], "end": [14, 22], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_context.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-500d048d9b553d102518341d"></a>
## from_key_value

`function` · `opentelemetry::trace::span_context::TraceState::from_key_value` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_key_value<T, K, V>(trace_state: T) -> Result<Self, TraceStateError> where T: IntoIterator<Item = (K, V)>, K: ToString, V: ToString
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [190, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:77`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a new `TraceState` from the given key-value collection.

# Examples

```
use opentelemetry::trace::TraceState;

let kvs = vec![("foo", "bar"), ("apple", "banana")];
let trace_state = TraceState::from_key_value(kvs);

assert!(trace_state.is_ok());
assert_eq!(trace_state.unwrap().header(), String::from("foo=bar,apple=banana"))
```

<a id="op-7460830b05f1f0e95744a19d"></a>
## from_str

`function` · `opentelemetry::trace::span_context::TraceState::from_str` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from_str(s: &str) -> Result<Self, Self::Err>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [192, 1], "end": [212, 2], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/trace/span_context.rs:195`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2733ca0b1ef43bb7ae30398d"></a>
## get

`function` · `opentelemetry::trace::span_context::TraceState::get` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get(&self, key: &str) -> Option<&str>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [190, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:106`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Retrieves a value for a given key from the `TraceState` if it exists.

<a id="op-7e0141d92ee61bd8271dc749"></a>
## hash

`function` · `opentelemetry::trace::span_context::TraceState::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 48], "end": [14, 52], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/trace/span_context.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eeb5eb56c355025a7d2f88b3"></a>
## header

`function` · `opentelemetry::trace::span_context::TraceState::header` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn header(&self) -> String
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [190, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:174`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a new `TraceState` header string, delimiting each key and value with a `=` and each
entry with a `,`.

<a id="op-9e0ffcc6f6e6a32b29dd5c57"></a>
## header_delimited

`function` · `opentelemetry::trace::span_context::TraceState::header_delimited` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn header_delimited(&self, entry_delimiter: &str, list_delimiter: &str) -> String
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [190, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:179`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a new `TraceState` header string, with the given key/value delimiter and entry delimiter.

<a id="op-d8c5baec50f1a890c061a5fa"></a>
## insert

`function` · `opentelemetry::trace::span_context::TraceState::insert` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn insert<K, V>(&self, key: K, value: V) -> Result<TraceState, TraceStateError> where K: Into<String>, V: Into<String>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::TraceState", "path": "TraceState"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [190, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:124`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Inserts the given key-value pair into the `TraceState`. If a value already exists for the
given key, this updates the value and updates the value's position. If the key or value are
invalid per the [W3 Spec] an `Err` is returned, else a new `TraceState` with the
updated key/value is returned.

[W3 Spec]: https://www.w3.org/TR/trace-context/#mutating-the-tracestate-field
