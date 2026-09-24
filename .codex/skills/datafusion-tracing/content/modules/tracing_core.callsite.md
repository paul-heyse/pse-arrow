# `tracing_core::callsite`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.callsite.json).

<a id="op-1740ba2098821878828cbb38"></a>
## callsite

`module` · `tracing_core::callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
mod callsite
```

Source: `src/callsite.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Callsites represent the source locations from which spans or events
originate.

# What Are Callsites?

Every span or event in `tracing` is associated with a [`Callsite`](../operations/tracing_core.callsite.Callsite.md#op-f2f04985f5653f1f8f822082). A
callsite is a small `static` value that is responsible for the following:

* Storing the span or event's [`Metadata`],
* Uniquely [identifying](Identifier) the span or event definition,
* Caching the subscriber's [`Interest`][^1] in that span or event, to avoid
  re-evaluating filters.

# Registering Callsites

When a span or event is recorded for the first time, its callsite
[`register`](../operations/tracing_core.callsite.register.md#op-e372a6d4be6dbbb70cf0c6ee)s itself with the global callsite registry. Registering a
callsite calls the [`Subscriber::register_callsite`][`register_callsite`]
method with that callsite's [`Metadata`] on every currently active
subscriber. This serves two primary purposes: informing subscribers of the
callsite's existence, and performing static filtering.

## Callsite Existence

If a [`Subscriber`] implementation wishes to allocate storage for each
unique span/event location in the program, or pre-compute some value
that will be used to record that span or event in the future, it can
do so in its [`register_callsite`] method.

## Performing Static Filtering

The [`register_callsite`] method returns an [`Interest`] value,
which indicates that the subscriber either [always] wishes to record
that span or event, [sometimes] wishes to record it based on a
dynamic filter evaluation, or [never] wishes to record it.

When registering a new callsite, the [`Interest`]s returned by every
currently active subscriber are combined, and the result is stored at
each callsite. This way, when the span or event occurs in the
future, the cached [`Interest`] value can be checked efficiently
to determine if the span or event should be recorded, without
needing to perform expensive filtering (i.e. calling the
[`Subscriber::enabled`] method every time a span or event occurs).

### Rebuilding Cached Interest

When a new [`Dispatch`] is created (i.e. a new subscriber becomes
active), any previously cached [`Interest`] values are re-evaluated
for all callsites in the program. This way, if the new subscriber
will enable a callsite that was not previously enabled, the
[`Interest`] in that callsite is updated. Similarly, when a
subscriber is dropped, the interest cache is also re-evaluated, so
that any callsites enabled only by that subscriber are disabled.

In addition, the [`rebuild_interest_cache`](../operations/tracing_core.callsite.rebuild_interest_cache.md#op-514604dfb2bc1df5b1a3e933) function in this module can be
used to manually invalidate all cached interest and re-register those
callsites. This function is useful in situations where a subscriber's
interest can change, but it does so relatively infrequently. The subscriber
may wish for its interest to be cached most of the time, and return
[`Interest::always`][always] or [`Interest::never`][never] in its
[`register_callsite`] method, so that its [`Subscriber::enabled`] method
doesn't need to be evaluated every time a span or event is recorded.
However, when the configuration changes, the subscriber can call
[`rebuild_interest_cache`](../operations/tracing_core.callsite.rebuild_interest_cache.md#op-514604dfb2bc1df5b1a3e933) to re-evaluate the entire interest cache with its
new configuration. This is a relatively costly operation, but if the
configuration changes infrequently, it may be more efficient than calling
[`Subscriber::enabled`] frequently.

# Implementing Callsites

In most cases, instrumenting code using `tracing` should *not* require
implementing the [`Callsite`](../operations/tracing_core.callsite.Callsite.md#op-f2f04985f5653f1f8f822082) trait directly. When using the [`tracing`
crate's macros][macros] or the [`#[instrument]` attribute][instrument], a
`Callsite` is automatically generated.

However, code which provides alternative forms of `tracing` instrumentation
may need to interact with the callsite system directly. If
instrumentation-side code needs to produce a `Callsite` to emit spans or
events, the [`DefaultCallsite`](../operations/tracing_core.callsite.DefaultCallsite.md#op-58b6356d7f4118a36fb5badb) struct provided in this module is a
ready-made `Callsite` implementation that is suitable for most uses. When
possible, the use of `DefaultCallsite` should be preferred over implementing
[`Callsite`](../operations/tracing_core.callsite.Callsite.md#op-f2f04985f5653f1f8f822082) for user types, as `DefaultCallsite` may benefit from
additional performance optimizations.

[^1]: Returned by the [`Subscriber::register_callsite`][`register_callsite`]
    method.

[`Metadata`]: crate::metadata::Metadata
[`Interest`]: crate::subscriber::Interest
[`Subscriber`]: crate::subscriber::Subscriber
[`register_callsite`]: crate::subscriber::Subscriber::register_callsite
[`Subscriber::enabled`]: crate::subscriber::Subscriber::enabled
[always]: crate::subscriber::Interest::always
[sometimes]: crate::subscriber::Interest::sometimes
[never]: crate::subscriber::Interest::never
[`Dispatch`]: crate::dispatcher::Dispatch
[macros]: https://docs.rs/tracing/latest/tracing/#macros
[instrument]: https://docs.rs/tracing/latest/tracing/attr.instrument.html
