# `opentelemetry_sdk::logs::record::TraceContext`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.record.TraceContext.json).

<a id="op-594d2bea61239c1ee8091e72"></a>
## TraceContext

`struct` · `opentelemetry_sdk::logs::record::TraceContext` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TraceContext
```

Source: `src/logs/record.rs:211`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

TraceContext stores the trace context for logs that have an associated
span.

<a id="op-4483e10dccc3c1c56b7e1af2"></a>
## clone

`function` · `opentelemetry_sdk::logs::record::TraceContext::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> TraceContext
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::TraceContext", "path": "TraceContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [209, 17], "end": [209, 22], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/record.rs:209`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7eae4a3059e49b14e0d2cee2"></a>
## eq

`function` · `opentelemetry_sdk::logs::record::TraceContext::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &TraceContext) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::TraceContext", "path": "TraceContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [209, 24], "end": [209, 33], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/logs/record.rs:209`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-941d696bca1ac2983e760d3a"></a>
## fmt

`function` · `opentelemetry_sdk::logs::record::TraceContext::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::TraceContext", "path": "TraceContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [209, 10], "end": [209, 15], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/record.rs:209`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0e4e0d45605efba7e01c9290"></a>
## from

`function` · `opentelemetry_sdk::logs::record::TraceContext::from` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn from(span_context: &SpanContext) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::TraceContext", "path": "TraceContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [221, 1], "end": [229, 2], "filename": "src/logs/record.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"borrowed_ref": {"is_mutable": false, "lifetime": null, "type": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/logs/record.rs:222`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6cab79b7ee2414cb151c468b"></a>
## span_id

`struct_field` · `opentelemetry_sdk::logs::record::TraceContext::span_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
span_id: opentelemetry::SpanId
```

Source: `src/logs/record.rs:215`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Span Id

<a id="op-ea60e9ef499586838245ab59"></a>
## trace_flags

`struct_field` · `opentelemetry_sdk::logs::record::TraceContext::trace_flags` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trace_flags: Option<opentelemetry::TraceFlags>
```

Source: `src/logs/record.rs:217`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Trace flags

<a id="op-adfbaf9444c113c0757fd0e1"></a>
## trace_id

`struct_field` · `opentelemetry_sdk::logs::record::TraceContext::trace_id` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trace_id: opentelemetry::TraceId
```

Source: `src/logs/record.rs:213`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Trace id
