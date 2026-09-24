# `opentelemetry::trace::noop`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.noop.json).

<a id="op-634df68398c6beaa306559af"></a>
## noop

`module` · `opentelemetry::trace::noop` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod noop
```

Source: `src/trace/noop.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No-op trace implementation

This implementation is returned as the global tracer if no `Tracer`
has been set. It is also useful for testing purposes as it is intended
to have minimal resource utilization and runtime impact.
