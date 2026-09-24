# `tracing::event_enabled`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.event_enabled.json).

<a id="op-5c503bd4b54f790d8060983b"></a>
## event_enabled

`macro` · `tracing::event_enabled` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! event_enabled
```

Source: `src/macros.rs:1083`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Tests whether an event with the specified level and target would be enabled.

This is similar to [`enabled!`], but queries the current subscriber specifically for
an event, whereas [`enabled!`] queries for an event _or_ span.

See the documentation for [`enabled!]` for more details on using this macro.
See also [`span_enabled!`].

# Examples

```rust
# use tracing::{event_enabled, Level};
if event_enabled!(target: "my_crate", Level::DEBUG) {
    // some expensive work...
}
// simpler
if event_enabled!(Level::DEBUG) {
    // some expensive work...
}
// with fields
if event_enabled!(Level::DEBUG, foo_field) {
    // some expensive work...
}
```

[`enabled!`]: crate::enabled
[`span_enabled!`]: crate::span_enabled
