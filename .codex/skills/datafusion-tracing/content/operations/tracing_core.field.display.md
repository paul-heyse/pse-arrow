# `tracing_core::field::display`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.display.json).

<a id="op-8b44bb2f65ec682be3b2f17f"></a>
## display

`function` · `tracing_core::field::display` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn display<T>(t: T) -> DisplayValue<T> where T: fmt::Display
```

Source: `src/field.rs:368`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Wraps a type implementing `fmt::Display` as a `Value` that can be
recorded using its `Display` implementation.
