# `tracing_subscriber::fmt::format::format`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.format.json).

<a id="op-710c591eb3765324b50d52d1"></a>
## format

`function` · `tracing_subscriber::fmt::format::format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format() -> Format
```

Source: `src/fmt/format/mod.rs:273`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the default configuration for an event formatter.

Methods on the returned event formatter can be used for further
configuration. For example:

```rust
let format = tracing_subscriber::fmt::format()
    .without_time()         // Don't include timestamps
    .with_target(false)     // Don't include event targets.
    .with_level(false)      // Don't include event levels.
    .compact();             // Use a more compact, abbreviated format.

// Use the configured formatter when building a new subscriber.
tracing_subscriber::fmt()
    .event_format(format)
    .init();
```
