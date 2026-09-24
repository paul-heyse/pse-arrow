# `tracing_futures::WithSubscriber`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_futures.WithSubscriber.json).

<a id="op-c39af7ca2f96032e131f7149"></a>
## WithSubscriber

`trait` · `tracing_futures::WithSubscriber` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
trait WithSubscriber: Sized
```

Source: `src/lib.rs:202`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Extension trait allowing futures, streams, and sinks to be instrumented with
a `tracing` [`Subscriber`].

[`Subscriber`]: https://docs.rs/tracing/latest/tracing/subscriber/trait.Subscriber.html

<a id="op-0fe7f4c226944648678efbbb"></a>
## with_current_subscriber

`function` · `tracing_futures::WithSubscriber::with_current_subscriber` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current_subscriber(self) -> WithDispatch<Self>
```

Source: `src/lib.rs:237`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Attaches the current [default] [`Subscriber`] to this type, returning a
`WithDispatch` wrapper.

When the wrapped type is a future, stream, or sink, the attached
subscriber will be set as the [default] while it is being polled.
When the wrapped type is an executor, the subscriber will be set as the
default for any futures spawned on that executor.

This can be used to propagate the current dispatcher context when
spawning a new future.

[`Subscriber`]: https://docs.rs/tracing/latest/tracing/subscriber/trait.Subscriber.html
[default]: https://docs.rs/tracing/latest/tracing/dispatcher/index.html#setting-the-default-subscriber

<a id="op-831e9c1760f19f51923835c0"></a>
## with_subscriber

`function` · `tracing_futures::WithSubscriber::with_subscriber` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn with_subscriber<S>(self, subscriber: S) -> WithDispatch<Self> where S: Into<Dispatch>
```

Source: `src/lib.rs:213`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Attaches the provided [`Subscriber`] to this type, returning a
`WithDispatch` wrapper.

When the wrapped type is a future, stream, or sink, the attached
subscriber will be set as the [default] while it is being polled.
When the wrapped type is an executor, the subscriber will be set as the
default for any futures spawned on that executor.

[`Subscriber`]: https://docs.rs/tracing/latest/tracing/subscriber/trait.Subscriber.html
[default]: https://docs.rs/tracing/latest/tracing/dispatcher/index.html#setting-the-default-subscriber
