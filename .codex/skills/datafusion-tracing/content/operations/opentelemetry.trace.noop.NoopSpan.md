# `opentelemetry::trace::noop::NoopSpan`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.noop.NoopSpan.json).

<a id="op-d9642dbe14fbf48c83ad2d6b"></a>
## NoopSpan

`struct` · `opentelemetry::trace::noop::NoopSpan` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct NoopSpan
```

Source: `src/trace/noop.rs:37`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A no-op instance of a `Span`.

<a id="op-999203dbd08918988b176044"></a>
## DEFAULT

`assoc_const` · `opentelemetry::trace::noop::NoopSpan::DEFAULT` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
DEFAULT
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 1], "end": [46, 2], "filename": "src/trace/noop.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/noop.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The default `NoopSpan`, as a constant

<a id="op-2d6728c8b67e54a5a85f3305"></a>
## add_event

`function` · `opentelemetry::trace::noop::NoopSpan::add_event` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event<T>(&mut self, _name: T, _attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Ignores all events

<a id="op-b732cfb5b4af50f344642206"></a>
## add_event_with_timestamp

`function` · `opentelemetry::trace::noop::NoopSpan::add_event_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event_with_timestamp<T>(&mut self, _name: T, _timestamp: SystemTime, _attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:58`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Ignores all events with timestamps

<a id="op-a1d0038f1b769eb285ebdf1f"></a>
## add_link

`function` · `opentelemetry::trace::noop::NoopSpan::add_link` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link(&mut self, _span_context: trace::SpanContext, _attributes: Vec<KeyValue>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:97`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9545ab3947f276c1bffcd555"></a>
## clone

`function` · `opentelemetry::trace::noop::NoopSpan::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> NoopSpan
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [36, 10], "end": [36, 15], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/noop.rs:36`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ddaa23fb1e612620066fc238"></a>
## end_with_timestamp

`function` · `opentelemetry::trace::noop::NoopSpan::end_with_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end_with_timestamp(&mut self, _timestamp: SystemTime)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Ignores `Span` endings

<a id="op-72fdc117bfc0f4fb130f66e4"></a>
## fmt

`function` · `opentelemetry::trace::noop::NoopSpan::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [36, 17], "end": [36, 22], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/noop.rs:36`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-30898fbf579c888bca4d0f41"></a>
## is_recording

`function` · `opentelemetry::trace::noop::NoopSpan::is_recording` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_recording(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:75`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns false, signifying that this span is never recording.

<a id="op-5f8e8204671933d516b68954"></a>
## set_attribute

`function` · `opentelemetry::trace::noop::NoopSpan::set_attribute` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attribute(&mut self, _attribute: KeyValue)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:80`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Ignores all attributes

<a id="op-9f7f6502bae2224c37df7594"></a>
## set_status

`function` · `opentelemetry::trace::noop::NoopSpan::set_status` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_status(&mut self, _status: trace::Status)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:85`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Ignores status

<a id="op-2a52980fccb4cfc30c37b79c"></a>
## span_context

`function` · `opentelemetry::trace::noop::NoopSpan::span_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_context(&self) -> &trace::SpanContext
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:70`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns an invalid `SpanContext`.

<a id="op-22b79d3f760daf7ddace7f75"></a>
## update_name

`function` · `opentelemetry::trace::noop::NoopSpan::update_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn update_name<T>(&mut self, _new_name: T) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopSpan", "path": "NoopSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [48, 1], "end": [105, 2], "filename": "src/trace/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/noop.rs:90`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Ignores name updates
