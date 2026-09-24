# `opentelemetry_sdk::runtime`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.runtime.json).

<a id="op-06d6ccc282566471cfbb1dca"></a>
## runtime

`module` · `opentelemetry_sdk::runtime` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod runtime
```

Source: `src/runtime.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Provides an abstraction of several async runtimes

This  allows OpenTelemetry to work with any current or future runtime. There is currently
built-in implementation for [Tokio].

[Tokio]: https://crates.io/crates/tokio
