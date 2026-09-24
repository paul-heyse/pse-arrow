# `opentelemetry::trace::span_context::SpanContext`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.span_context.SpanContext.json).

<a id="op-f21a0f1d9f45c44092215518"></a>
## SpanContext

`struct` · `opentelemetry::trace::span_context::SpanContext` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanContext
```

Source: `src/trace/span_context.rs:250`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Immutable portion of a [`Span`] which can be serialized and propagated.

This representation conforms to the [W3C TraceContext specification].

Spans that do not have the `sampled` flag set in their [`TraceFlags`](../operations/opentelemetry.trace_context.TraceFlags.md#op-47031c18e731037a4e7c9356) will
be ignored by most tracing tools.

[`Span`]: crate::trace::Span
[W3C TraceContext specification]: https://www.w3.org/TR/trace-context

<a id="op-3d9730a9b49131b24c33e0b6"></a>
## NONE

`assoc_const` · `opentelemetry::trace::span_context::SpanContext::NONE` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
NONE
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:260`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

An invalid span context

<a id="op-6c7b06b6c38595253f076db8"></a>
## clone

`function` · `opentelemetry::trace::span_context::SpanContext::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SpanContext
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 10], "end": [249, 15], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/span_context.rs:249`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d0b055c3b5e79b535c71bd05"></a>
## empty_context

`function` · `opentelemetry::trace::span_context::SpanContext::empty_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn empty_context() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:269`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create an invalid empty span context

<a id="op-bce22c763de80e0570091734"></a>
## eq

`function` · `opentelemetry::trace::span_context::SpanContext::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SpanContext) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 24], "end": [249, 33], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace/span_context.rs:249`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fbe15c2e15ccbce87535f187"></a>
## fmt

`function` · `opentelemetry::trace::span_context::SpanContext::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 17], "end": [249, 22], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/span_context.rs:249`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-860c96fca45b020731ab874e"></a>
## hash

`function` · `opentelemetry::trace::span_context::SpanContext::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [249, 35], "end": [249, 39], "filename": "src/trace/span_context.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/trace/span_context.rs:249`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bc109101d5722501bd75d897"></a>
## is_remote

`function` · `opentelemetry::trace::span_context::SpanContext::is_remote` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_remote(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:315`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns `true` if the span context was propagated from a remote parent.

<a id="op-368bcb3c9cd71d33d61b194a"></a>
## is_sampled

`function` · `opentelemetry::trace::span_context::SpanContext::is_sampled` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_sampled(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:322`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns `true` if the `sampled` trace flag is set.

Spans that are not sampled will be ignored by most tracing tools.

<a id="op-abd8caaeb2c5e399b8fbdf0b"></a>
## is_valid

`function` · `opentelemetry::trace::span_context::SpanContext::is_valid` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_valid(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:310`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns `true` if the span context has a valid (non-zero) `trace_id` and a
valid (non-zero) `span_id`.

<a id="op-7b01575a48c20a84fde16a55"></a>
## new

`function` · `opentelemetry::trace::span_context::SpanContext::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(trace_id: TraceId, span_id: SpanId, trace_flags: TraceFlags, is_remote: bool, trace_state: TraceState) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:274`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Construct a new `SpanContext`

<a id="op-d47c379e2c28d76912043b50"></a>
## span_id

`function` · `opentelemetry::trace::span_context::SpanContext::span_id` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn span_id(&self) -> SpanId
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:296`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The [`SpanId`](../operations/opentelemetry.trace_context.SpanId.md#op-6e9d96bb2646bae3348bb60a) for this span context.

<a id="op-e4320ec7ff8330bf5a4aabdc"></a>
## trace_flags

`function` · `opentelemetry::trace::span_context::SpanContext::trace_flags` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn trace_flags(&self) -> TraceFlags
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:304`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns details about the trace.

Unlike `TraceState` values, these are present in all traces. The current
version of the specification only supports a single flag [`TraceFlags::SAMPLED`](../operations/opentelemetry.trace_context.TraceFlags.md#op-58f140bfab75c1c5c1497c8d).

<a id="op-a9447b64abe72d03fb9a45bc"></a>
## trace_id

`function` · `opentelemetry::trace::span_context::SpanContext::trace_id` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn trace_id(&self) -> TraceId
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:291`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The [`TraceId`](../operations/opentelemetry.trace_context.TraceId.md#op-436bb6ec32d874fe466755ee) for this span context.

<a id="op-f1ff1b4797d7d40454b52032"></a>
## trace_state

`function` · `opentelemetry::trace::span_context::SpanContext::trace_state` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn trace_state(&self) -> &TraceState
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace::span_context::SpanContext", "path": "SpanContext"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [258, 1], "end": [330, 2], "filename": "src/trace/span_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/span_context.rs:327`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A reference to the span context's [`TraceState`](../operations/opentelemetry.trace.span_context.TraceState.md#op-93d50de145537665fcb28b20).
