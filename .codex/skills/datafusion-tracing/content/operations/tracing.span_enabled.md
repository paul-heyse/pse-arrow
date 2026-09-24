# `tracing::span_enabled`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.span_enabled.json).

<a id="op-aeaa6345fd504f0ab1a2dfdf"></a>
## span_enabled

`macro` · `tracing::span_enabled` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! span_enabled
```

Source: `src/macros.rs:1117`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Tests whether a span with the specified level and target would be enabled.

This is similar to [`enabled!`], but queries the current subscriber specifically for
an event, whereas [`enabled!`] queries for an event _or_ span.

See the documentation for [`enabled!]` for more details on using this macro.
See also [`span_enabled!`].

# Examples

```rust
# use tracing::{span_enabled, Level};
if span_enabled!(target: "my_crate", Level::DEBUG) {
    // some expensive work...
}
// simpler
if span_enabled!(Level::DEBUG) {
    // some expensive work...
}
// with fields
if span_enabled!(Level::DEBUG, foo_field) {
    // some expensive work...
}
```

[`enabled!`]: crate::enabled
[`span_enabled!`]: crate::span_enabled
