# `opentelemetry_sdk::trace::span::Span`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span.Span.json).

<a id="op-6728bb88b006c9a3c6d6b771"></a>
## Span

`struct` · `opentelemetry_sdk::trace::span::Span` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct Span
```

Source: `src/trace/span.rs:19`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Single operation within a trace.

<a id="op-84a6925f0e30a2ccc2ee9e62"></a>
## add_event_with_timestamp

`function` · `opentelemetry_sdk::trace::span::Span::add_event_with_timestamp` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_event_with_timestamp<T>(&mut self, name: T, timestamp: SystemTime, attributes: Vec<KeyValue>) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:94`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Records events at a specific time in the context of a given `Span`.

Note that the OpenTelemetry project documents certain ["standard event names and
keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
which have prescribed semantic meanings.

<a id="op-0609260dc3cb23eecd5f2f10"></a>
## add_link

`function` · `opentelemetry_sdk::trace::span::Span::add_link` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_link(&mut self, span_context: SpanContext, attributes: Vec<KeyValue>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:175`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Add `Link` to this `Span`


<a id="op-fb1934a691ff48964023b257"></a>
## drop

`function` · `opentelemetry_sdk::trace::span::Span::drop` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [244, 1], "end": [249, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/trace/span.rs:246`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Report span on inner drop

<a id="op-ab992ebb90da211b7bd93ae1"></a>
## end_with_timestamp

`function` · `opentelemetry_sdk::trace::span::Span::end_with_timestamp` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn end_with_timestamp(&mut self, timestamp: SystemTime)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:196`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Finishes the span with given timestamp.

<a id="op-9922b017372436f66bee6509"></a>
## exported_data

`function` · `opentelemetry_sdk::trace::span::Span::exported_data` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn exported_data(&self) -> Option<trace::SpanData>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [53, 1], "end": [86, 2], "filename": "src/trace/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Convert information in this span into `exporter::trace::SpanData`.
This function copies all data from the current span, which will create a
overhead.

<a id="op-a58dedb3585fc6b67acfaf43"></a>
## fmt

`function` · `opentelemetry_sdk::trace::span::Span::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 10], "end": [18, 15], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-abefb3624382d0ced944de1f"></a>
## is_recording

`function` · `opentelemetry_sdk::trace::span::Span::is_recording` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_recording(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:130`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns true if this `Span` is recording information like events with the `add_event`
operation, attributes using `set_attributes`, status with `set_status`, etc.
Always returns false after span `end`.

<a id="op-58a6825c1456ab53d9f4de4a"></a>
## set_attribute

`function` · `opentelemetry_sdk::trace::span::Span::set_attribute` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_attribute(&mut self, attribute: KeyValue)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:139`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Sets a single `Attribute` where the attribute properties are passed as arguments.

Note that the OpenTelemetry project documents certain ["standard
attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
that have prescribed semantic meanings.

<a id="op-3bdbe3bfffc3e5aebc797b71"></a>
## set_status

`function` · `opentelemetry_sdk::trace::span::Span::set_status` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_status(&mut self, status: Status)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Sets the status of this `Span`.

If used, this will override the default span status, which is [`Status::Unset`].

Unresolved upstream links (retained, not inferred): ``Status::Unset``.

<a id="op-f8235e86a11d5b98c0b1469f"></a>
## span_context

`function` · `opentelemetry_sdk::trace::span::Span::span_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_context(&self) -> &SpanContext
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:123`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the `SpanContext` for the given `Span`.

<a id="op-b5ed70aeace2da6923399a93"></a>
## update_name

`function` · `opentelemetry_sdk::trace::span::Span::update_name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn update_name<T>(&mut self, new_name: T) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::span::Span", "path": "Span"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [88, 1], "end": [199, 2], "filename": "src/trace/span.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::span::Span", "path": "Span"}, "trait_path": "opentelemetry::trace::span::Span"}`

Source: `src/trace/span.rs:164`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Updates the `Span`'s name.
