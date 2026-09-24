# `tracing::enabled`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.enabled.json).

<a id="op-3383abc1e97aee1544e7701c"></a>
## enabled

`macro` · `tracing::enabled` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
macro_rules! enabled
```

Source: `src/macros.rs:1214`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Checks whether a span or event is [enabled] based on the provided [metadata].

[enabled]: crate::Subscriber::enabled
[metadata]: crate::Metadata

This macro is a specialized tool: it is intended to be used prior
to an expensive computation required *just* for that event, but
*cannot* be done as part of an argument to that event, such as
when multiple events are emitted (e.g., iterating over a collection
and emitting an event for each item).

# Usage

[Subscribers] can make filtering decisions based all the data included in a
span or event's [`Metadata`]. This means that it is possible for `enabled!`
to return a _false positive_ (indicating that something would be enabled
when it actually would not be) or a _false negative_ (indicating that
something would be disabled when it would actually be enabled).

[Subscribers]: crate::subscriber::Subscriber
[`Metadata`]: crate::metadata::Metadata

This occurs when a subscriber is using a _more specific_ filter than the
metadata provided to the `enabled!` macro. Some situations that can result
in false positives or false negatives include:

- If a subscriber is using a filter which may enable a span or event based
  on field names, but `enabled!` is invoked without listing field names,
  `enabled!` may return a false negative if a specific field name would
  cause the subscriber to enable something that would otherwise be disabled.
- If a subscriber is using a filter which enables or disables specific events by
  file path and line number,  a particular event may be enabled/disabled
  even if an `enabled!` invocation with the same level, target, and fields
  indicated otherwise.
- The subscriber can choose to enable _only_ spans or _only_ events, which `enabled`
  will not reflect.

`enabled!()` requires a [level](crate::Level) argument, an optional `target:`
argument, and an optional set of field names. If the fields are not provided,
they are considered to be unknown. `enabled!` attempts to match the
syntax of `event!()` as closely as possible, which can be seen in the
examples below.

# Examples

If the current subscriber is interested in recording `DEBUG`-level spans and
events in the current file and module path, this will evaluate to true:
```rust
use tracing::{enabled, Level};

if enabled!(Level::DEBUG) {
    // some expensive work...
}
```

If the current subscriber is interested in recording spans and events
in the current file and module path, with the target "my_crate", and at the
level  `DEBUG`, this will evaluate to true:
```rust
# use tracing::{enabled, Level};
if enabled!(target: "my_crate", Level::DEBUG) {
    // some expensive work...
}
```

If the current subscriber is interested in recording spans and events
in the current file and module path, with the target "my_crate", at
the level `DEBUG`, and with a field named "hello", this will evaluate
to true:

```rust
# use tracing::{enabled, Level};
if enabled!(target: "my_crate", Level::DEBUG, hello) {
    // some expensive work...
}
```

# Alternatives

`enabled!` queries subscribers with [`Metadata`] where
[`is_event`] and [`is_span`] both return `false`. Alternatively,
use [`event_enabled!`](../operations/tracing.event_enabled.md#op-5c503bd4b54f790d8060983b) or [`span_enabled!`] to ensure one of these
returns true.


[`Metadata`]: crate::Metadata
[`is_event`]: crate::Metadata::is_event
[`is_span`]: crate::Metadata::is_span
[`enabled!`]: crate::enabled
[`span_enabled!`]: crate::span_enabled

Unresolved upstream links (retained, not inferred): `crate::Metadata::is_span`, `crate::Metadata::is_event`.
