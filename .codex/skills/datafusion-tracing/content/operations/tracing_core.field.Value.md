# `tracing_core::field::Value`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.field.Value.json).

<a id="op-3bdffa798a494864d07fca1b"></a>
## Value

`trait` · `tracing_core::field::Value` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
trait Value: sealed::Sealed
```

Source: `src/field.rs:350`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A field value of an erased type.

Implementors of `Value` may call the appropriate typed recording methods on
the [visitor] passed to their `record` method in order to indicate how
their data should be recorded.

[visitor]: Visit

<a id="op-95b4d7e923013aac166173fc"></a>
## record

`function` · `tracing_core::field::Value::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, key: &Field, visitor: &mut dyn Visit)
```

Source: `src/field.rs:352`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Visits this value with the given `Visitor`.
