# `tracing_subscriber::fmt::time::uptime`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.uptime.json).

<a id="op-65e31ff929783ce5a4b9841d"></a>
## uptime

`function` · `tracing_subscriber::fmt::time::uptime` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn uptime() -> Uptime
```

Source: `src/fmt/time/mod.rs:85`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new `Uptime` timestamp provider.

With this timer, timestamps will be formatted with the amount of time
elapsed since the timestamp provider was constructed.

This can then be configured further to determine how timestamps should be
configured.

This is equivalent to calling
```rust
# fn timer() -> tracing_subscriber::fmt::time::Uptime {
tracing_subscriber::fmt::time::Uptime::default()
# }
```
