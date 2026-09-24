# `tracing::instrument::WithSubscriber`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.instrument.WithSubscriber.json).

<a id="op-4bd50093c9a272555ebecf03"></a>
## WithSubscriber

`trait` · `tracing::instrument::WithSubscriber` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
trait WithSubscriber: Sized
```

Source: `src/instrument.rs:136`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Extension trait allowing futures to be instrumented with
a `tracing` [`Subscriber`](crate::Subscriber).

<a id="op-76e6327da42dff6ad95a6736"></a>
## with_current_subscriber

`function` · `tracing::instrument::WithSubscriber::with_current_subscriber` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current_subscriber(self) -> WithDispatch<Self>
```

Source: `src/instrument.rs:228`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Attaches the current [default] [`Subscriber`] to this type, returning a
[`WithDispatch`](../operations/tracing.instrument.WithDispatch.md#op-f37fe528a1c49c9d37012bbc) wrapper.

The attached `Subscriber` will be set as the [default] when the returned
[`Future`] is polled.

This can be used to propagate the current dispatcher context when
spawning a new future that may run on a different thread.

# Examples

```
# mod tokio {
#     pub(super) fn spawn(_: impl std::future::Future) {}
# }
# use tracing::subscriber::NoSubscriber as MySubscriber;
# async fn docs() {
use tracing::instrument::WithSubscriber;

// Using `set_default` (rather than `set_global_default`) sets the
// default `Subscriber` for *this* thread only.
let _default = tracing::subscriber::set_default(MySubscriber::default());

let future = async {
    // ...
};

// If a multi-threaded async runtime is in use, this spawned task may
// run on a different thread, in a different default `Subscriber`'s context.
tokio::spawn(future);

// However, calling `with_current_subscriber` on the future before
// spawning it, ensures that the current thread's default `Subscriber` is
// propagated to the spawned task, regardless of where it executes:
# let future = async { };
tokio::spawn(future.with_current_subscriber());
# }
```
[`Subscriber`]: super::Subscriber
[default]: dispatcher#setting-the-default-subscriber
[`Future`]: std::future::Future

Unresolved upstream links (retained, not inferred): `std::future::Future`.

<a id="op-ef3821a7bc4c08d5246c199d"></a>
## with_subscriber

`function` · `tracing::instrument::WithSubscriber::with_subscriber` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn with_subscriber<S>(self, subscriber: S) -> WithDispatch<Self> where S: Into<Dispatch>
```

Source: `src/instrument.rs:176`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Attaches the provided [`Subscriber`] to this type, returning a
[`WithDispatch`](../operations/tracing.instrument.WithDispatch.md#op-f37fe528a1c49c9d37012bbc) wrapper.

The attached [`Subscriber`] will be set as the [default] when the returned
[`Future`] is polled.

# Examples

```
# use tracing::subscriber::NoSubscriber as MySubscriber;
# use tracing::subscriber::NoSubscriber as MyOtherSubscriber;
# async fn docs() {
use tracing::instrument::WithSubscriber;

// Set the default `Subscriber`
let _default = tracing::subscriber::set_default(MySubscriber::default());

tracing::info!("this event will be recorded by the default `Subscriber`");

// Create a different `Subscriber` and attach it to a future.
let other_subscriber = MyOtherSubscriber::default();
let future = async {
    tracing::info!("this event will be recorded by the other `Subscriber`");
    // ...
};

future
    // Attach the other `Subscriber` to the future before awaiting it
    .with_subscriber(other_subscriber)
    .await;

// Once the future has completed, we return to the default `Subscriber`.
tracing::info!("this event will be recorded by the default `Subscriber`");
# }
```

[`Subscriber`]: super::Subscriber
[default]: dispatcher#setting-the-default-subscriber
[`Future`]: std::future::Future

Unresolved upstream links (retained, not inferred): `std::future::Future`.
