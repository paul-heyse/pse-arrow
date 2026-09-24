# `tracing_core::field::debug`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.debug.json).

<a id="op-2046a860f36d21d051296f02"></a>
## debug

`function` · `tracing_core::field::debug` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn debug<T>(t: T) -> DebugValue<T> where T: fmt::Debug
```

Source: `src/field.rs:377`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Wraps a type implementing `fmt::Debug` as a `Value` that can be
recorded using its `Debug` implementation.
