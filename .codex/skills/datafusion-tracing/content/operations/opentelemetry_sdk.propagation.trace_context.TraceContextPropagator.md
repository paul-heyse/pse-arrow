# `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.propagation.trace_context.TraceContextPropagator.json).

<a id="op-6f9ab4009e929a66ad0904b5"></a>
## TraceContextPropagator

`struct` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TraceContextPropagator
```

Source: `src/propagation/trace_context.rs:52`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Propagates `SpanContext`s in [W3C TraceContext] format under `traceparent` and `tracestate` header.

The `traceparent` header represents the incoming request in a
tracing system in a common format, understood by all vendors.
Here’s an example of a `traceparent` header.

`traceparent: 00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01`

The `traceparent` HTTP header field identifies the incoming request in a
tracing system. It has four fields:

   - version
   - trace-id
   - parent-id
   - trace-flags

The `tracestate` header provides additional vendor-specific trace
identification information across different distributed tracing systems.
Here's an example of a `tracestate` header

`tracestate: vendorname1=opaqueValue1,vendorname2=opaqueValue2`

See the [w3c trace-context docs] for more details.

[w3c trace-context docs]: https://w3c.github.io/trace-context/
[W3C TraceContext]: https://www.w3.org/TR/trace-context/

<a id="op-32ae7c67ca445f06bf4d5a5c"></a>
## clone

`function` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> TraceContextPropagator
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::trace_context::TraceContextPropagator", "path": "TraceContextPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 10], "end": [51, 15], "filename": "src/propagation/trace_context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/propagation/trace_context.rs:51`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2e8f0fcca33d906a1d1ebabb"></a>
## default

`function` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> TraceContextPropagator
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::trace_context::TraceContextPropagator", "path": "TraceContextPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 24], "end": [51, 31], "filename": "src/propagation/trace_context.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/propagation/trace_context.rs:51`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4c21a5eefad7debb24c37cb2"></a>
## extract_with_context

`function` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator::extract_with_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn extract_with_context(&self, cx: &Context, extractor: &dyn Extractor) -> Context
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::trace_context::TraceContextPropagator", "path": "TraceContextPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [124, 1], "end": [156, 2], "filename": "src/propagation/trace_context.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/trace_context.rs:147`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Retrieves encoded `SpanContext`s using the `Extractor`. It decodes
the `SpanContext` and returns it. If no `SpanContext` was retrieved
OR if the retrieved SpanContext is invalid then an empty `SpanContext`
is returned.

<a id="op-fba00720c9189de302e3f8c3"></a>
## fields

`function` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator::fields` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> FieldIter<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::trace_context::TraceContextPropagator", "path": "TraceContextPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [124, 1], "end": [156, 2], "filename": "src/propagation/trace_context.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/trace_context.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5ff6bd8ee5f37d4e53a57e1e"></a>
## fmt

`function` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::trace_context::TraceContextPropagator", "path": "TraceContextPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [51, 17], "end": [51, 22], "filename": "src/propagation/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/propagation/trace_context.rs:51`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aaabbcb0f8f80fca675fce01"></a>
## inject_context

`function` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator::inject_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn inject_context(&self, cx: &Context, injector: &mut dyn Injector)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::trace_context::TraceContextPropagator", "path": "TraceContextPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [124, 1], "end": [156, 2], "filename": "src/propagation/trace_context.rs"}, "trait": {"args": null, "id": "opentelemetry::propagation::text_map_propagator::TextMapPropagator", "path": "TextMapPropagator"}, "trait_path": "opentelemetry::propagation::text_map_propagator::TextMapPropagator"}`

Source: `src/propagation/trace_context.rs:127`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Properly encodes the values of the `SpanContext` and injects them
into the `Injector`.

<a id="op-f0086d961ca0d228b699f320"></a>
## new

`function` · `opentelemetry_sdk::propagation::trace_context::TraceContextPropagator::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::propagation::trace_context::TraceContextPropagator", "path": "TraceContextPropagator"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [56, 1], "end": [122, 2], "filename": "src/propagation/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/propagation/trace_context.rs:58`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new `TraceContextPropagator`.
