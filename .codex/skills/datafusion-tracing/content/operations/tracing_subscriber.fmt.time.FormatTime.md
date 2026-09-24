# `tracing_subscriber::fmt::time::FormatTime`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.FormatTime.json).

<a id="op-3331b0854c6f665459175995"></a>
## FormatTime

`trait` · `tracing_subscriber::fmt::time::FormatTime` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait FormatTime
```

Source: `src/fmt/time/mod.rs:47`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A type that can measure and format the current time.

This trait is used by `Format` to include a timestamp with each `Event` when it is logged.

Notable default implementations of this trait are `SystemTime` and `()`. The former prints the
current time as reported by `std::time::SystemTime`, and the latter does not print the current
time at all. `FormatTime` is also automatically implemented for any function pointer with the
appropriate signature.

The full list of provided implementations can be found in [`time`].

[`time`]: self

<a id="op-ecb1770c4ea4bd0069176ea2"></a>
## format_time

`function` · `tracing_subscriber::fmt::time::FormatTime::format_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result
```

Source: `src/fmt/time/mod.rs:53`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Measure and write out the current time.

When `format_time` is called, implementors should get the current time using their desired
mechanism, and write it out to the given `fmt::Write`. Implementors must insert a trailing
space themselves if they wish to separate the time from subsequent log message text.
