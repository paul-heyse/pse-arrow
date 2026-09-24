# `tracing::subscriber::set_default`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.subscriber.set_default.json).

<a id="op-ae9a4ce1097daf5406ee790d"></a>
## set_default

`function` · `tracing::subscriber::set_default` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn set_default<S>(subscriber: S) -> DefaultGuard where S: Subscriber + Send + Sync + 'static
```

Source: `src/subscriber.rs:57`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Sets the [`Subscriber`] as the default for the current thread for the
duration of the lifetime of the returned [`DefaultGuard`].

The default subscriber is used when creating a new [`Span`] or [`Event`].

[`Span`]: super::span::Span
[`Subscriber`]: super::subscriber::Subscriber
[`Event`]: super::event::Event
[`DefaultGuard`]: super::dispatcher::DefaultGuard
