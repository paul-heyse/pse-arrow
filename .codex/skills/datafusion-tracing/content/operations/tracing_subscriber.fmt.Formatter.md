# `tracing_subscriber::fmt::Formatter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.Formatter.json).

<a id="op-3791cf2ddae6b2c945ce0b67"></a>
## Formatter

`type_alias` · `tracing_subscriber::fmt::Formatter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
type Formatter<N = format::DefaultFields, E = format::Format<format::Full>, W = fn() -> io::Stdout> = layer::Layered<fmt_layer::Layer<registry::Registry, N, E, W>, registry::Registry>
```

Source: `src/fmt/mod.rs:240`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A `Subscriber` that logs formatted representations of `tracing` events.
This type only logs formatted events; it does not perform any filtering.
