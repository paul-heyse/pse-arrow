# `tracing_subscriber::fmt::format::debug_fn`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.debug_fn.json).

<a id="op-e26b5decdb6e101148ec916b"></a>
## debug_fn

`function` · `tracing_subscriber::fmt::format::debug_fn` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn debug_fn<F>(f: F) -> FieldFn<F> where F: Fn(&mut Writer<'_>, &tracing_core::field::Field, &dyn fmt::Debug) -> fmt::Result + Clone
```

Source: `src/fmt/format/mod.rs:287`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a [`FormatFields`](../operations/tracing_subscriber.fmt.format.FormatFields.md#op-9bc846bbdc7bd7bc3e4045c0) implementation that formats fields using the
provided function or closure.

