# `tracing_core::field::valuable`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.valuable.json).

<a id="op-87eb250a15d063ef506bcd78"></a>
## valuable

`function` · `tracing_core::field::valuable` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn valuable<T>(t: &T) -> valuable::Value<'_> where T: valuable::Valuable
```

Source: `src/field.rs:390`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Wraps a type implementing [`Valuable`] as a `Value` that
can be recorded using its `Valuable` implementation.

[`Valuable`]: https://docs.rs/valuable/latest/valuable/trait.Valuable.html
