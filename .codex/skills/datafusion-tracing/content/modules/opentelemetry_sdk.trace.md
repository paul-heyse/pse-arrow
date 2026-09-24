# `opentelemetry_sdk::trace`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.json).

<a id="op-63d3bed91aec05650eb447c7"></a>
## trace

`module` · `opentelemetry_sdk::trace` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod trace
```

Source: `src/trace/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

# OpenTelemetry Trace SDK

The tracing SDK consist of a few main structs:

* The [`SdkTracer`](../operations/opentelemetry_sdk.trace.tracer.SdkTracer.md#op-994f3cb8ad5bc1ae17502b0e) struct which performs all tracing operations.
* The [`Span`](../operations/opentelemetry_sdk.trace.span.Span.md#op-6728bb88b006c9a3c6d6b771) struct with is a mutable object storing information about the
  current operation execution.
* The [`SdkTracerProvider`](../operations/opentelemetry_sdk.trace.provider.SdkTracerProvider.md#op-68f8d3c6d535fa6ce330056b) struct which configures and produces [`SdkTracer`](../operations/opentelemetry_sdk.trace.tracer.SdkTracer.md#op-994f3cb8ad5bc1ae17502b0e)s.
