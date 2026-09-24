# `tracing_subscriber::fmt::fmt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.fmt.json).

<a id="op-63ef70baea72667f29b16b18"></a>
## fmt

`function` · `tracing_subscriber::fmt::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt() -> SubscriberBuilder
```

Source: `src/fmt/mod.rs:324`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`SubscriberBuilder`](../operations/tracing_subscriber.fmt.SubscriberBuilder.md#op-38fcd8fc52ca33acc34d54e9) for configuring a [formatting subscriber].

This is essentially shorthand for [`SubscriberBuilder::default()]`.

# Examples

Using [`init`] to set the default subscriber:

```rust
tracing_subscriber::fmt().init();
```

Configuring the output format:

```rust

tracing_subscriber::fmt()
    // Configure formatting settings.
    .with_target(false)
    .with_timer(tracing_subscriber::fmt::time::uptime())
    .with_level(true)
    // Set the subscriber as the default.
    .init();
```

[`try_init`] returns an error if the default subscriber could not be set:

```rust
use std::error::Error;

fn init_subscriber() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    tracing_subscriber::fmt()
        // Configure the subscriber to emit logs in JSON format.
        .json()
        // Configure the subscriber to flatten event fields in the output JSON objects.
        .flatten_event(true)
        // Set the subscriber as the default, returning an error if this fails.
        .try_init()?;

    Ok(())
}
```

Rather than setting the subscriber as the default, [`finish`] _returns_ the
constructed subscriber, which may then be passed to other functions:

```rust
let subscriber = tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .compact()
    .finish();

tracing::subscriber::with_default(subscriber, || {
    // the subscriber will only be set as the default
    // inside this closure...
})
```

[formatting subscriber]: Subscriber
[`SubscriberBuilder::default()`]: SubscriberBuilder::default
[`init`]: SubscriberBuilder::init()
[`try_init`]: SubscriberBuilder::try_init()
[`finish`]: SubscriberBuilder::finish()
