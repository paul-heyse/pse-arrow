# `opentelemetry_sdk::trace::events::SpanEvents`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.events.SpanEvents.json).

<a id="op-7b1a56304ba00616e752bb7e"></a>
## SpanEvents

`struct` · `opentelemetry_sdk::trace::events::SpanEvents` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanEvents
```

Source: `src/trace/events.rs:9`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Stores span events along with dropped count.

<a id="op-b440749d6e9a140ebe78f4f8"></a>
## IntoIter

`assoc_type` · `opentelemetry_sdk::trace::events::SpanEvents::IntoIter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
IntoIter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [24, 1], "end": [31, 2], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": "IntoIterator"}, "trait_path": "core::iter::traits::collect::IntoIterator"}`

Source: `src/trace/events.rs:26`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5ddac53898218515cfcb7900"></a>
## Item

`assoc_type` · `opentelemetry_sdk::trace::events::SpanEvents::Item` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [24, 1], "end": [31, 2], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": "IntoIterator"}, "trait_path": "core::iter::traits::collect::IntoIterator"}`

Source: `src/trace/events.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4f653bc31382bf011a263cff"></a>
## Target

`assoc_type` · `opentelemetry_sdk::trace::events::SpanEvents::Target` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Target
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [16, 1], "end": [22, 2], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::ops::deref::Deref", "path": "Deref"}, "trait_path": "core::ops::deref::Deref"}`

Source: `src/trace/events.rs:17`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d7ed22b8006a6e388b74e24d"></a>
## clone

`function` · `opentelemetry_sdk::trace::events::SpanEvents::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanEvents
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [7, 10], "end": [7, 15], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/events.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cf6862e326cc1a200a04209a"></a>
## default

`function` · `opentelemetry_sdk::trace::events::SpanEvents::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> SpanEvents
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [7, 24], "end": [7, 31], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/events.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1d3e2254f45dd126381593c6"></a>
## deref

`function` · `opentelemetry_sdk::trace::events::SpanEvents::deref` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn deref(&self) -> &Self::Target
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [16, 1], "end": [22, 2], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::ops::deref::Deref", "path": "Deref"}, "trait_path": "core::ops::deref::Deref"}`

Source: `src/trace/events.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a1ccdd8c58109e6482e00e1b"></a>
## dropped_count

`struct_field` · `opentelemetry_sdk::trace::events::SpanEvents::dropped_count` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
dropped_count: u32
```

Source: `src/trace/events.rs:13`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The number of Events dropped from the span.

<a id="op-e4e691ff78c18d1b74fa948c"></a>
## eq

`function` · `opentelemetry_sdk::trace::events::SpanEvents::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SpanEvents) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [7, 33], "end": [7, 42], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/events.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dd3f2bdc3c0008420375525b"></a>
## events

`struct_field` · `opentelemetry_sdk::trace::events::SpanEvents::events` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
events: Vec<opentelemetry::trace::Event>
```

Source: `src/trace/events.rs:11`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The events stored as a vector. Could be empty if there are no events.

<a id="op-723357ce10f1b50ea513374e"></a>
## fmt

`function` · `opentelemetry_sdk::trace::events::SpanEvents::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [7, 17], "end": [7, 22], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/events.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a8677e61fe6894d46e817dc1"></a>
## into_iter

`function` · `opentelemetry_sdk::trace::events::SpanEvents::into_iter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn into_iter(self) -> Self::IntoIter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::events::SpanEvents", "path": "SpanEvents"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [24, 1], "end": [31, 2], "filename": "src/trace/events.rs"}, "trait": {"args": null, "id": "core::iter::traits::collect::IntoIterator", "path": "IntoIterator"}, "trait_path": "core::iter::traits::collect::IntoIterator"}`

Source: `src/trace/events.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
