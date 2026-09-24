# `tracing_core`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.json).

<a id="op-52dda2a9e4e737870d3f675f"></a>
## tracing_core

`module` · `tracing_core` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
mod tracing_core
```

Source: `src/lib.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Core primitives for `tracing`.

[`tracing`] is a framework for instrumenting Rust programs to collect
structured, event-based diagnostic information. This crate defines the core
primitives of `tracing`.

This crate provides:

* [`span::Id`] identifies a span within the execution of a program.

* [`Event`] represents a single event within a trace.

* [`Subscriber`], the trait implemented to collect trace data.

* [`Metadata`] and [`Callsite`] provide information describing spans and
  `Event`s.

* [`Field`], [`FieldSet`], [`Value`], and [`ValueSet`] represent the
  structured data attached to a span.

* [`Dispatch`] allows spans and events to be dispatched to `Subscriber`s.

In addition, it defines the global callsite registry and per-thread current
dispatcher which other components of the tracing system rely on.

*Compiler support: [requires `rustc` 1.65+][msrv]*

[msrv]: #supported-rust-versions

## Usage

Application authors will typically not use this crate directly. Instead,
they will use the [`tracing`] crate, which provides a much more
fully-featured API. However, this crate's API will change very infrequently,
so it may be used when dependencies must be very stable.

`Subscriber` implementations may depend on `tracing-core` rather than
`tracing`, as the additional APIs provided by `tracing` are primarily useful
for instrumenting libraries and applications, and are generally not
necessary for `Subscriber` implementations.

The [`tokio-rs/tracing`] repository contains less stable crates designed to
be used with the `tracing` ecosystem. It includes a collection of
`Subscriber` implementations, as well as utility and adapter crates.

## Crate Feature Flags

The following crate [feature flags] are available:

* `std`: Depend on the Rust standard library (enabled by default).

  `no_std` users may disable this feature with `default-features = false`:

  ```toml
  [dependencies]
  tracing-core = { version = "0.1.22", default-features = false }
  ```

  **Note**:`tracing-core`'s `no_std` support requires `liballoc`.

### Unstable Features

These feature flags enable **unstable** features. The public API may break in 0.1.x
releases. To enable these features, the `--cfg tracing_unstable` must be passed to
`rustc` when compiling.

The following unstable feature flags are currently available:

* `valuable`: Enables support for recording [field values] using the
  [`valuable`] crate.

#### Enabling Unstable Features

The easiest way to set the `tracing_unstable` cfg is to use the `RUSTFLAGS`
env variable when running `cargo` commands:

```shell
RUSTFLAGS="--cfg tracing_unstable" cargo build
```
Alternatively, the following can be added to the `.cargo/config` file in a
project to automatically enable the cfg flag for that project:

```toml
[build]
rustflags = ["--cfg", "tracing_unstable"]
```

[feature flags]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
[field values]: crate::field
[`valuable`]: https://crates.io/crates/valuable

## Supported Rust Versions

Tracing is built against the latest stable release. The minimum supported
version is 1.65. The current Tracing version is not guaranteed to build on
Rust versions earlier than the minimum supported version.

Tracing follows the same compiler support policies as the rest of the Tokio
project. The current stable Rust compiler and the three most recent minor
versions before it will always be supported. For example, if the current
stable compiler version is 1.69, the minimum supported version will not be
increased past 1.66, three minor versions prior. Increasing the minimum
supported compiler version is not considered a semver breaking change as
long as doing so complies with this policy.


[`span::Id`]: span::Id
[`Event`]: event::Event
[`Subscriber`]: subscriber::Subscriber
[`Metadata`]: metadata::Metadata
[`Callsite`]: callsite::Callsite
[`Field`]: field::Field
[`FieldSet`]: field::FieldSet
[`Value`]: field::Value
[`ValueSet`]: field::ValueSet
[`Dispatch`]: dispatcher::Dispatch
[`tokio-rs/tracing`]: https://github.com/tokio-rs/tracing
[`tracing`]: https://crates.io/crates/tracing
