# `tracing_futures`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_futures.json).

<a id="op-bd2e91199977886d9a1e911e"></a>
## tracing_futures

`module` · `tracing_futures` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
mod tracing_futures
```

Source: `src/lib.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Futures compatibility for [`tracing`].

# Overview

[`tracing`] is a framework for instrumenting Rust programs to collect
structured, event-based diagnostic information. This crate provides utilities
for using `tracing` to instrument asynchronous code written using futures and
async/await.

The crate provides the following traits:

* [`Instrument`] allows a `tracing` [span] to be attached to a future, sink,
  stream, or executor.

* [`WithSubscriber`] allows a `tracing` [`Subscriber`] to be attached to a
  future, sink, stream, or executor.

*Compiler support: [requires `rustc` 1.42+][msrv]*

[msrv]: #supported-rust-versions

# Feature flags

This crate provides a number of feature flags that enable compatibility
features with other crates in the asynchronous ecosystem:

- `tokio`: Enables compatibility with the `tokio` crate, including
   [`Instrument`] and [`WithSubscriber`] implementations for
   `tokio::executor::Executor`, `tokio::runtime::Runtime`, and
   `tokio::runtime::current_thread`. Enabled by default.
- `tokio-executor`: Enables compatibility with the `tokio-executor`
   crate, including [`Instrument`] and [`WithSubscriber`]
   implementations for types implementing `tokio_executor::Executor`.
   This is intended primarily for use in crates which depend on
   `tokio-executor` rather than `tokio`; in general the `tokio` feature
   should be used instead.
- `std-future`: Enables compatibility with `std::future::Future`.
- `futures-01`: Enables compatibility with version 0.1.x of the [`futures`]
  crate.
- `futures-03`: Enables compatibility with version 0.3.x of the `futures`
  crate's `Spawn` and `LocalSpawn` traits.
- `tokio-alpha`: Enables compatibility with `tokio` 0.2's alpha releases,
  including the `tokio` 0.2 `Executor` and `TypedExecutor` traits.
- `std`: Depend on the Rust standard library.

  `no_std` users may disable this feature with `default-features = false`:

  ```toml
  [dependencies]
  tracing-futures = { version = "0.2.5", default-features = false }
  ```

The `tokio`, `std-future` and `std` features are enabled by default.

[`tracing`]: https://crates.io/crates/tracing
[span]: https://docs.rs/tracing/latest/tracing/span/index.html
[`Subscriber`]: https://docs.rs/tracing/latest/tracing/subscriber/index.html
[`Instrument`]: trait.Instrument.html
[`WithSubscriber`]: trait.WithSubscriber.html
[`futures`]: https://crates.io/crates/futures

## Supported Rust Versions

Tracing is built against the latest stable release. The minimum supported
version is 1.42. The current Tracing version is not guaranteed to build on
Rust versions earlier than the minimum supported version.

Tracing follows the same compiler support policies as the rest of the Tokio
project. The current stable Rust compiler and the three most recent minor
versions before it will always be supported. For example, if the current
stable compiler version is 1.45, the minimum supported version will not be
increased past 1.42, three minor versions prior. Increasing the minimum
supported compiler version is not considered a semver breaking change as
long as doing so complies with this policy.

