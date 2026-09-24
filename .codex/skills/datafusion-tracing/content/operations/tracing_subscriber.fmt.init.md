# `tracing_subscriber::fmt::init`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.init.json).

<a id="op-a407aed5352eebedf333f2a1"></a>
## init

`function` · `tracing_subscriber::fmt::init` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn init()
```

Source: `src/fmt/mod.rs:1262`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Install a global tracing subscriber that listens for events and
filters based on the value of the [`RUST_LOG` environment variable].

The configuration of the subscriber initialized by this function
depends on what [feature flags](crate#feature-flags) are enabled.

If the `tracing-log` feature is enabled, this will also install
the LogTracer to convert `Log` records into `tracing` `Event`s.

If the `env-filter` feature is enabled, this is shorthand for

```rust
# use tracing_subscriber::EnvFilter;
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

# Panics
Panics if the initialization was unsuccessful, likely because a
global subscriber was already installed by another call to `try_init`.

[`RUST_LOG` environment variable]: crate::filter::EnvFilter::DEFAULT_ENV
