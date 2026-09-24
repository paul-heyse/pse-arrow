# `tracing::subscriber::with_default`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.subscriber.with_default.json).

<a id="op-4dcdd0d53fb0523d2b659e19"></a>
## with_default

`function` · `tracing::subscriber::with_default` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn with_default<T, S>(subscriber: S, f: impl FnOnce() -> T) -> T where S: Subscriber + Send + Sync + 'static
```

Source: `src/subscriber.rs:20`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Sets this [`Subscriber`] as the default for the current thread for the
duration of a closure.

The default subscriber is used when creating a new [`Span`] or
[`Event`].


[`Span`]: super::span::Span
[`Subscriber`]: super::subscriber::Subscriber
[`Event`]: super::event::Event
