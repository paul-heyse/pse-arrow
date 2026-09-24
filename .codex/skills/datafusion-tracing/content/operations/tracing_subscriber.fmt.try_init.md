# `tracing_subscriber::fmt::try_init`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.try_init.json).

<a id="op-cfaaaef043e2fae0ace5e5ad"></a>
## try_init

`function` · `tracing_subscriber::fmt::try_init` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_init() -> Result<(), alloc::boxed::Box<dyn Error + Send + Sync + 'static>>
```

Source: `src/fmt/mod.rs:1200`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Install a global tracing subscriber that listens for events and
filters based on the value of the [`RUST_LOG` environment variable],
if one is not already set.

If the `tracing-log` feature is enabled, this will also install
the [`LogTracer`] to convert `log` records into `tracing` `Event`s.

This is shorthand for

```rust
# fn doc() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
tracing_subscriber::fmt().try_init()
# }
```


# Errors

Returns an Error if the initialization was unsuccessful,
likely because a global subscriber was already installed by another
call to `try_init`.

[`LogTracer`]:
    https://docs.rs/tracing-log/0.1.0/tracing_log/struct.LogTracer.html
[`RUST_LOG` environment variable]: crate::filter::EnvFilter::DEFAULT_ENV
