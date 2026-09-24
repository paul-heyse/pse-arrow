# `opentelemetry_sdk::trace::error::TraceResult`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.error.TraceResult.json).

<a id="op-30a959b19c3f93fd72554b7a"></a>
## TraceResult

`type_alias` · `opentelemetry_sdk::trace::error::TraceResult` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
type TraceResult<T> = Result<T, TraceError>
```

Source: `src/trace/error.rs:7`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A specialized `Result` type for trace operations.
