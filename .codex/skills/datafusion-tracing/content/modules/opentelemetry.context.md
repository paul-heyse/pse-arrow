# `opentelemetry::context`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.context.json).

<a id="op-0caec9305feab8e344172450"></a>
## context

`module` · `opentelemetry::context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod context
```

Source: `src/context.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Execution-scoped context propagation.

The `context` module provides mechanisms for propagating values across API boundaries and between
logically associated execution units. It enables cross-cutting concerns to access their data in-process
using a shared context object.

# Main Types

- [`Context`](../operations/opentelemetry.context.Context.md#op-ca59114006a0744de26919a1): An immutable, execution-scoped collection of values.

