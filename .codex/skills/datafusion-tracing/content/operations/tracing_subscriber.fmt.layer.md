# `tracing_subscriber::fmt::layer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.layer.json).

<a id="op-f3636f2044ea0dc03f1ed8e4"></a>
## layer

`function` · `tracing_subscriber::fmt::layer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn layer<S>() -> Layer<S>
```

Source: `src/fmt/mod.rs:337`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [formatting layer] that can be [composed] with other layers to
construct a [`Subscriber`](../operations/tracing_subscriber.fmt.Subscriber.md#op-7360e26773e1081257bf258e).

This is a shorthand for the equivalent [`Layer::default()`] function.

[formatting layer]: Layer
[composed]: crate::layer
[`Layer::default()`]: Layer::default
