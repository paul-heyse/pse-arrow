# `opentelemetry::trace::Event`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.Event.json).

<a id="op-007a3678b3f9b5071d006231"></a>
## Event

`struct` · `opentelemetry::trace::Event` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Event
```

Source: `src/trace/mod.rs:193`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Events record things that happened during a [`Span`](../operations/opentelemetry.trace.span.Span.md#op-7e58380e085740d966fa45c8)'s lifetime.

<a id="op-ca9a847db38ff1ff1a4a923e"></a>
## attributes

`struct_field` · `opentelemetry::trace::Event::attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
attributes: Vec<KeyValue>
```

Source: `src/trace/mod.rs:201`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Attributes that describe this event.

<a id="op-b8a133e7b4892e688a91ec20"></a>
## clone

`function` · `opentelemetry::trace::Event::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Event
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Event", "path": "Event"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [192, 10], "end": [192, 15], "filename": "src/trace/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/mod.rs:192`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1aab1ada0751c5afd3fd4f9d"></a>
## dropped_attributes_count

`struct_field` · `opentelemetry::trace::Event::dropped_attributes_count` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
dropped_attributes_count: u32
```

Source: `src/trace/mod.rs:205`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The number of attributes that were above the configured limit, and thus
dropped.

<a id="op-bd5006e89e0c54b7ab8126d2"></a>
## eq

`function` · `opentelemetry::trace::Event::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Event) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Event", "path": "Event"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [192, 24], "end": [192, 33], "filename": "src/trace/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/mod.rs:192`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b2e46e2f6a105c49fe0580ab"></a>
## fmt

`function` · `opentelemetry::trace::Event::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Event", "path": "Event"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [192, 17], "end": [192, 22], "filename": "src/trace/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/mod.rs:192`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b3d89f36892c44741863decd"></a>
## name

`struct_field` · `opentelemetry::trace::Event::name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
name: std::borrow::Cow<'static, str>
```

Source: `src/trace/mod.rs:195`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The name of this event.

<a id="op-385da900f951d0237ef55fac"></a>
## new

`function` · `opentelemetry::trace::Event::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new<T: Into<Cow<'static, str>>>(name: T, timestamp: time::SystemTime, attributes: Vec<KeyValue>, dropped_attributes_count: u32) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Event", "path": "Event"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [208, 1], "end": [233, 2], "filename": "src/trace/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/mod.rs:210`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create new `Event`

<a id="op-b603a97900e4ba66a7d6dcee"></a>
## timestamp

`struct_field` · `opentelemetry::trace::Event::timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
timestamp: time::SystemTime
```

Source: `src/trace/mod.rs:198`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The time at which this event occurred.

<a id="op-154ad47c2c6163bc1cf64112"></a>
## with_name

`function` · `opentelemetry::trace::Event::with_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_name<T: Into<Cow<'static, str>>>(name: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::Event", "path": "Event"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [208, 1], "end": [233, 2], "filename": "src/trace/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/mod.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create new `Event` with a given name.
