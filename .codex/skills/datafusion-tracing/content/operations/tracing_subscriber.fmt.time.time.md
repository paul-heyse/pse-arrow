# `tracing_subscriber::fmt::time::time`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.time.json).

<a id="op-083925ee6e24a81a2c9609be"></a>
## time

`function` · `tracing_subscriber::fmt::time::time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn time() -> SystemTime
```

Source: `src/fmt/time/mod.rs:67`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `SystemTime` timestamp provider.

This can then be configured further to determine how timestamps should be
configured.

This is equivalent to calling
```rust
# fn timer() -> tracing_subscriber::fmt::time::SystemTime {
tracing_subscriber::fmt::time::SystemTime::default()
# }
```
