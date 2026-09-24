# `tracing::subscriber::set_global_default`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.subscriber.set_global_default.json).

<a id="op-b4ef9837ba02ffd17d756917"></a>
## set_global_default

`function` · `tracing::subscriber::set_global_default` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn set_global_default<S>(subscriber: S) -> Result<(), SetGlobalDefaultError> where S: Subscriber + Send + Sync + 'static
```

Source: `src/subscriber.rs:38`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Sets this subscriber as the global default for the duration of the entire program.
Will be used as a fallback if no thread-local subscriber has been set in a thread (using `with_default`.)

Can only be set once; subsequent attempts to set the global default will fail.
Returns whether the initialization was successful.

Note: Libraries should *NOT* call `set_global_default()`! That will cause conflicts when
executables try to set them later.

[`Subscriber`]: super::subscriber::Subscriber
[`Event`]: super::event::Event
