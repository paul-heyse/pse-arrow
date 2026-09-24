# `opentelemetry::trace::span_context::TraceStateError`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.span_context.TraceStateError.json).

<a id="op-9f1b78f92e3737b10979713b"></a>
## TraceStateError

`enum` · `opentelemetry::trace::span_context::TraceStateError` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
enum TraceStateError
```

Source: `src/trace/span_context.rs:220`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Error returned by `TraceState` operations.

<a id="op-01b5ac12b1baf1e203e799d7"></a>
## Key

`variant` · `opentelemetry::trace::span_context::TraceStateError::Key` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Key
```

Source: `src/trace/span_context.rs:225`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The key is invalid.

See <https://www.w3.org/TR/trace-context/#key> for requirement for keys.

<a id="op-0d9b1504b2ece41ffe7642c2"></a>
## List

`variant` · `opentelemetry::trace::span_context::TraceStateError::List` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
List
```

Source: `src/trace/span_context.rs:237`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The list is invalid.

See <https://www.w3.org/TR/trace-context/#list> for requirement for list members.

<a id="op-e59c16ca97da205c94844465"></a>
## Value

`variant` · `opentelemetry::trace::span_context::TraceStateError::Value` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Value
```

Source: `src/trace/span_context.rs:231`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The value is invalid.

See <https://www.w3.org/TR/trace-context/#value> for requirement for values.
