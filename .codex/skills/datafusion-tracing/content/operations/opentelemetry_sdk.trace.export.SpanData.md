# `opentelemetry_sdk::trace::export::SpanData`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.export.SpanData.json).

<a id="op-481ea1f068c0b153049e9a92"></a>
## SpanData

`struct` · `opentelemetry_sdk::trace::export::SpanData` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanData
```

Source: `src/trace/export.rs:80`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`SpanData` contains all the information collected by a `Span` and can be used
by exporters as a standard input.

<a id="op-9be60330ca6c6c5d14ee1e37"></a>
## attributes

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::attributes` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
attributes: Vec<opentelemetry::KeyValue>
```

Source: `src/trace/export.rs:96`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span attributes

<a id="op-b799957fb051429dd4f63f51"></a>
## clone

`function` · `opentelemetry_sdk::trace::export::SpanData::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanData
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanData", "path": "SpanData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [79, 10], "end": [79, 15], "filename": "src/trace/export.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/export.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f99ab006c1393013c889bbad"></a>
## dropped_attributes_count

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::dropped_attributes_count` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
dropped_attributes_count: u32
```

Source: `src/trace/export.rs:99`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The number of attributes that were above the configured limit, and thus
dropped.

<a id="op-fd279976ea7b1d69a928de48"></a>
## end_time

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::end_time` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
end_time: std::time::SystemTime
```

Source: `src/trace/export.rs:94`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span end time

<a id="op-95a7d53a9e938581d00cdce8"></a>
## eq

`function` · `opentelemetry_sdk::trace::export::SpanData::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SpanData) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanData", "path": "SpanData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [79, 24], "end": [79, 33], "filename": "src/trace/export.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/export.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e13161727f99f9001a51a3f4"></a>
## events

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::events` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
events: trace::SpanEvents
```

Source: `src/trace/export.rs:101`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span events

<a id="op-e9af2b363c07d72286e5cb72"></a>
## fmt

`function` · `opentelemetry_sdk::trace::export::SpanData::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanData", "path": "SpanData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [79, 17], "end": [79, 22], "filename": "src/trace/export.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/export.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-154dd5874c2da2ecb36aa366"></a>
## instrumentation_scope

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::instrumentation_scope` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
instrumentation_scope: opentelemetry::InstrumentationScope
```

Source: `src/trace/export.rs:107`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Instrumentation scope that produced this span

<a id="op-7867171ce2fb0d51d04b0ad7"></a>
## links

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::links` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
links: trace::SpanLinks
```

Source: `src/trace/export.rs:103`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span Links

<a id="op-af5d60db8b7b29addf95938a"></a>
## name

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
name: std::borrow::Cow<'static, str>
```

Source: `src/trace/export.rs:90`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span name

<a id="op-fc230f2bdca0490e24703e62"></a>
## parent_span_id

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::parent_span_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
parent_span_id: opentelemetry::trace::SpanId
```

Source: `src/trace/export.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span parent id

<a id="op-d01134f2de0689c776b3e6a0"></a>
## parent_span_is_remote

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::parent_span_is_remote` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
parent_span_is_remote: bool
```

Source: `src/trace/export.rs:86`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Parent span is remote flag (for span flags)

<a id="op-68edda179aa3083259533f8a"></a>
## span_context

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::span_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
span_context: opentelemetry::trace::SpanContext
```

Source: `src/trace/export.rs:82`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Exportable `SpanContext`

<a id="op-fb69eb15c9c15e0cc14d698f"></a>
## span_kind

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::span_kind` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
span_kind: opentelemetry::trace::SpanKind
```

Source: `src/trace/export.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span kind

<a id="op-40bcfd90cc9b25b301533ad6"></a>
## start_time

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::start_time` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
start_time: std::time::SystemTime
```

Source: `src/trace/export.rs:92`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span start time

<a id="op-75bbdf8b9d7be6cfcc452d21"></a>
## status

`struct_field` · `opentelemetry_sdk::trace::export::SpanData::status` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
status: opentelemetry::trace::Status
```

Source: `src/trace/export.rs:105`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span status
