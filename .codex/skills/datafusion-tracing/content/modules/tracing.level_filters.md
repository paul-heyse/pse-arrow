# `tracing::level_filters`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.level_filters.json).

<a id="op-86d380268e337fa07a8343e4"></a>
## level_filters

`module` · `tracing::level_filters` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
mod level_filters
```

Source: `src/level_filters.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Trace verbosity level filtering.

# Compile time filters

Trace verbosity levels can be statically disabled at compile time via Cargo
features, similar to the [`log` crate]. Trace instrumentation at disabled
levels will be skipped and will not even be present in the resulting binary
unless the verbosity level is specified dynamically. This level is
configured separately for release and debug builds. The features are:

* `max_level_off`
* `max_level_error`
* `max_level_warn`
* `max_level_info`
* `max_level_debug`
* `max_level_trace`
* `release_max_level_off`
* `release_max_level_error`
* `release_max_level_warn`
* `release_max_level_info`
* `release_max_level_debug`
* `release_max_level_trace`

These features control the value of the `STATIC_MAX_LEVEL` constant. The
instrumentation macros macros check this value before recording an event or
constructing a span. By default, no levels are disabled.

For example, a crate can disable trace level instrumentation in debug builds
and trace, debug, and info level instrumentation in release builds with the
following configuration:

```toml
[dependencies]
tracing = { version = "0.1", features = ["max_level_debug", "release_max_level_warn"] }
```
## Notes

Please note that `tracing`'s static max level features do *not* control the
[`log`] records that may be emitted when [`tracing`'s "log" feature flag][f] is
enabled. This is to allow `tracing` to be disabled entirely at compile time
while still emitting `log` records --- such as when a library using
`tracing` is used by an application using `log` that doesn't want to
generate any `tracing`-related code, but does want to collect `log` records.

This means that if the "log" feature is in use, some code may be generated
for `log` records emitted by disabled `tracing` events. If this is not
desirable, `log` records may be disabled separately using [`log`'s static
max level features][`log` crate].

[`log`]: https://docs.rs/log/
[`log` crate]: https://docs.rs/log/latest/log/#compile-time-filters
[f]: https://docs.rs/tracing/latest/tracing/#emitting-log-records
