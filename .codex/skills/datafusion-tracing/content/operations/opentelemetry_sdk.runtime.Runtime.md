# `opentelemetry_sdk::runtime::Runtime`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.runtime.Runtime.json).

<a id="op-9c7c0607dc3adcadbbf65e89"></a>
## Runtime

`trait` · `opentelemetry_sdk::runtime::Runtime` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait Runtime: Clone + Send + Sync + 'static
```

Source: `src/runtime.rs:23`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A runtime is an abstraction of an async runtime like [Tokio]. It allows
OpenTelemetry to work with any current and hopefully future runtime implementations.

[Tokio]: https://crates.io/crates/tokio

# Note

OpenTelemetry expects a *multithreaded* runtime because its types can move across threads.
For this reason, this trait requires the `Send` and `Sync` bounds. Single-threaded runtimes
can implement this trait in a way that spawns the tasks on the same thread as the calling code.

<a id="op-1531642c4b7fd2d75ef46c95"></a>
## delay

`function` · `opentelemetry_sdk::runtime::Runtime::delay` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn delay(&self, duration: Duration) -> impl Future<Output = ()> + Send + 'static
```

Source: `src/runtime.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Return a future that resolves after the specified [Duration].

Unresolved upstream links (retained, not inferred): `Duration`.

<a id="op-ad4d64d2e2f75b37201908f6"></a>
## spawn

`function` · `opentelemetry_sdk::runtime::Runtime::spawn` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn spawn<F>(&self, future: F) where F: Future<Output = ()> + Send + 'static
```

Source: `src/runtime.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Spawn a new task or thread, which executes the given future.

# Note

This is mainly used to run batch span processing in the background. Note, that the function
does not return a handle. OpenTelemetry will use a different way to wait for the future to
finish when the caller shuts down.

At the moment, the shutdown happens by blocking the
current thread. This means runtime implementations need to make sure they can still execute
the given future even if the main thread is blocked.
