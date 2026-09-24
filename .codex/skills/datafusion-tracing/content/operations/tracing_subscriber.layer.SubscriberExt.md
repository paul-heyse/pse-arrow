# `tracing_subscriber::layer::SubscriberExt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.layer.SubscriberExt.json).

<a id="op-c90d1e00a4750dbeed1d3670"></a>
## SubscriberExt

`trait` · `tracing_subscriber::layer::SubscriberExt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait SubscriberExt: Subscriber + sealed::Sealed
```

Source: `src/layer/mod.rs:1500`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait adding a `with(Layer)` combinator to `Subscriber`s.

<a id="op-0268c7766658f40cc07d8c16"></a>
## with

`function` · `tracing_subscriber::layer::SubscriberExt::with` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with<L>(self, layer: L) -> Layered<L, Self> where L: Layer<Self>, Self: Sized
```

Source: `src/layer/mod.rs:1502`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps `self` with the provided `layer`.
