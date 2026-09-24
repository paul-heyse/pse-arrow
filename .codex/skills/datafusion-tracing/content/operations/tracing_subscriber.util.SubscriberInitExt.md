# `tracing_subscriber::util::SubscriberInitExt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.util.SubscriberInitExt.json).

<a id="op-7a1acd3379cd90991465958d"></a>
## SubscriberInitExt

`trait` · `tracing_subscriber::util::SubscriberInitExt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait SubscriberInitExt where Self: Into<tracing_core::dispatcher::Dispatch>
```

Source: `src/util.rs:26`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait adding utility methods for subscriber initialization.

This trait provides extension methods to make configuring and setting a
[default subscriber] more ergonomic. It is automatically implemented for all
types that can be converted into a [trace dispatcher]. Since `Dispatch`
implements `From<T>` for all `T: Subscriber`, all `Subscriber`
implementations will implement this extension trait as well. Types which
can be converted into `Subscriber`s, such as builders that construct a
`Subscriber`, may implement `Into<Dispatch>`, and will also receive an
implementation of this trait.

[default subscriber]: https://docs.rs/tracing/0.1.21/tracing/dispatcher/index.html#setting-the-default-subscriber
[trace dispatcher]: https://docs.rs/tracing/0.1.21/tracing/dispatcher/index.html

<a id="op-9358fe6ab79166d0f2a32f7d"></a>
## init

`function` · `tracing_subscriber::util::SubscriberInitExt::init` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn init(self)
```

Source: `src/util.rs:92`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Attempts to set `self` as the [global default subscriber] in the current
scope, panicking if this fails.

If the "tracing-log" feature flag is enabled, this will also attempt to
initialize a [`log`] compatibility layer. This allows the subscriber to
consume `log::Record`s as though they were `tracing` `Event`s.

This method panics if a global default subscriber has already been set,
or if a `log` logger has already been set (when the "tracing-log"
feature is enabled).

[global default subscriber]: https://docs.rs/tracing/0.1.21/tracing/dispatcher/index.html#setting-the-default-subscriber
[`log`]: https://crates.io/log

<a id="op-785efa145602da6205bcfff8"></a>
## set_default

`function` · `tracing_subscriber::util::SubscriberInitExt::set_default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn set_default(self) -> dispatcher::DefaultGuard
```

Source: `src/util.rs:41`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets `self` as the [default subscriber] in the current scope, returning a
guard that will unset it when dropped.

If the "tracing-log" feature flag is enabled, this will also initialize
a [`log`] compatibility layer. This allows the subscriber to consume
`log::Record`s as though they were `tracing` `Event`s.

[default subscriber]: https://docs.rs/tracing/0.1.21/tracing/dispatcher/index.html#setting-the-default-subscriber
[`log`]: https://crates.io/log

<a id="op-48915ac76d0003b08ded34ee"></a>
## try_init

`function` · `tracing_subscriber::util::SubscriberInitExt::try_init` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_init(self) -> Result<(), TryInitError>
```

Source: `src/util.rs:61`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Attempts to set `self` as the [global default subscriber] in the current
scope, returning an error if one is already set.

If the "tracing-log" feature flag is enabled, this will also attempt to
initialize a [`log`] compatibility layer. This allows the subscriber to
consume `log::Record`s as though they were `tracing` `Event`s.

This method returns an error if a global default subscriber has already
been set, or if a `log` logger has already been set (when the
"tracing-log" feature is enabled).

[global default subscriber]: https://docs.rs/tracing/0.1.21/tracing/dispatcher/index.html#setting-the-default-subscriber
[`log`]: https://crates.io/log
