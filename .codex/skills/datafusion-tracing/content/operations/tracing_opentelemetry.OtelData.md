# `tracing_opentelemetry::OtelData`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_opentelemetry.OtelData.json).

<a id="op-d366a05e21c71f9adac7b414"></a>
## OtelData

`struct` · `tracing_opentelemetry::OtelData` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
struct OtelData
```

Source: `src/lib.rs:134`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Per-span OpenTelemetry data tracked by this crate.

<a id="op-9d649f42bfe53e5f9cd1814d"></a>
## fmt

`function` · `tracing_opentelemetry::OtelData::fmt` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_opentelemetry::OtelData", "path": "OtelData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [133, 10], "end": [133, 15], "filename": "src/lib.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/lib.rs:133`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-357a120aff3833ddd0a07cbd"></a>
## span_id

`function` · `tracing_opentelemetry::OtelData::span_id` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_id(&self) -> Option<opentelemetry::SpanId>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_opentelemetry::OtelData", "path": "OtelData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [141, 1], "end": [173, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:166`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Gets the span ID of the span.

Returns `None` if the context has not been built yet. This can be forced e.g. by calling
[`context`] on the span (not on `OtelData`) or if [context activation] was not explicitly
opted-out of, simply entering the span for the first time.

[`context`]: OpenTelemetrySpanExt::context
[context activation]: OpenTelemetryLayer::with_context_activation

<a id="op-ed7ec2a1b421399101f0c603"></a>
## trace_id

`function` · `tracing_opentelemetry::OtelData::trace_id` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn trace_id(&self) -> Option<opentelemetry::TraceId>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_opentelemetry::OtelData", "path": "OtelData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [141, 1], "end": [173, 2], "filename": "src/lib.rs"}, "trait": null, "trait_path": null}`

Source: `src/lib.rs:150`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Gets the trace ID of the span.

Returns `None` if the context has not been built yet. This can be forced e.g. by calling
[`context`] on the span (not on `OtelData`) or if [context activation] was not explicitly
opted-out of, simply entering the span for the first time.

[`context`]: OpenTelemetrySpanExt::context
[context activation]: OpenTelemetryLayer::with_context_activation
