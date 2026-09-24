# `tracing_subscriber::layer::Filter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.layer.Filter.json).

<a id="op-fd31ac1a2cd1ed4f447625f3"></a>
## Filter

`trait` · `tracing_subscriber::layer::Filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait Filter<S>
```

Source: `src/layer/mod.rs:1264`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A per-[`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) filter that determines whether a span or event is enabled
for an individual layer.

See [the module-level documentation][plf] for details on using [`Filter`](../operations/tracing_subscriber.layer.Filter.md#op-fd31ac1a2cd1ed4f447625f3)s.

[plf]: crate::layer#per-layer-filtering

<a id="op-49af4e3414ecfc52a0cb5887"></a>
## callsite_enabled

`function` · `tracing_subscriber::layer::Filter::callsite_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn callsite_enabled(&self, meta: &'static Metadata<'static>) -> Interest
```

Source: `src/layer/mod.rs:1394`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an [`Interest`] indicating whether this layer will [always],
[sometimes], or [never] be interested in the given [`Metadata`].

When a given callsite will [always] or [never] be enabled, the results
of evaluating the filter may be cached for improved performance.
Therefore, if a filter is capable of determining that it will always or
never enable a particular callsite, providing an implementation of this
function is recommended.

<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: If a <code>Filter</code> will perform
<em>dynamic filtering</em> that depends on the current context in which
a span or event was observed (e.g. only enabling an event when it
occurs within a particular span), it <strong>must</strong> return
<code>Interest::sometimes()</code> from this method. If it returns
<code>Interest::always()</code> or <code>Interest::never()</code>, the
<code>enabled</code> method may not be called when a particular instance
of that span or event is recorded.
</pre>

This method is broadly similar to [`Subscriber::register_callsite`];
however, since the returned value represents only the interest of
*this* layer, the resulting behavior is somewhat different.

If a [`Subscriber`] returns [`Interest::always()`][always] or
[`Interest::never()`][never] for a given [`Metadata`], its [`enabled`]
method is then *guaranteed* to never be called for that callsite. On the
other hand, when a `Filter` returns [`Interest::always()`][always] or
[`Interest::never()`][never] for a callsite, _other_ [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8)s may have
differing interests in that callsite. If this is the case, the callsite
will receive [`Interest::sometimes()`][sometimes], and the [`enabled`]
method will still be called for that callsite when it records a span or
event.

Returning [`Interest::always()`][always] or [`Interest::never()`][never] from
`Filter::callsite_enabled` will permanently enable or disable a
callsite (without requiring subsequent calls to [`enabled`]) if and only
if the following is true:

- all [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8)s that comprise the subscriber include `Filter`s
  (this includes a tree of [`Layered`](../operations/tracing_subscriber.layer.layered.Layered.md#op-a34d0c9535f11404fda44764) layers that share the same
  `Filter`)
- all those `Filter`s return the same [`Interest`].

For example, if a [`Subscriber`] consists of two [`Filtered`] layers,
and both of those layers return [`Interest::never()`][never], that
callsite *will* never be enabled, and the [`enabled`] methods of those
[`Filter`](../operations/tracing_subscriber.layer.Filter.md#op-fd31ac1a2cd1ed4f447625f3)s will not be called.

## Default Implementation

The default implementation of this method assumes that the
`Filter`'s [`enabled`] method _may_ perform dynamic filtering, and
returns [`Interest::sometimes()`][sometimes], to ensure that [`enabled`]
is called to determine whether a particular _instance_ of the callsite
is enabled in the current context. If this is *not* the case, and the
`Filter`'s [`enabled`] method will always return the same result
for a particular [`Metadata`], this method can be overridden as
follows:

```
use tracing_subscriber::layer;
use tracing_core::{Metadata, subscriber::Interest};

struct MyFilter {
    // ...
}

impl MyFilter {
    // The actual logic for determining whether a `Metadata` is enabled
    // must be factored out from the `enabled` method, so that it can be
    // called without a `Context` (which is not provided to the
    // `callsite_enabled` method).
    fn is_enabled(&self, metadata: &Metadata<'_>) -> bool {
        // ...
        # drop(metadata); true
    }
}

impl<S> layer::Filter<S> for MyFilter {
    fn enabled(&self, metadata: &Metadata<'_>, _: &layer::Context<'_, S>) -> bool {
        // Even though we are implementing `callsite_enabled`, we must still provide a
        // working implementation of `enabled`, as returning `Interest::always()` or
        // `Interest::never()` will *allow* caching, but will not *guarantee* it.
        // Other filters may still return `Interest::sometimes()`, so we may be
        // asked again in `enabled`.
        self.is_enabled(metadata)
    }

    fn callsite_enabled(&self, metadata: &'static Metadata<'static>) -> Interest {
        // The result of `self.enabled(metadata, ...)` will always be
        // the same for any given `Metadata`, so we can convert it into
        // an `Interest`:
        if self.is_enabled(metadata) {
            Interest::always()
        } else {
            Interest::never()
        }
    }
}
```

[`Metadata`]: tracing_core::Metadata
[`Interest`]: tracing_core::Interest
[always]: tracing_core::Interest::always
[sometimes]: tracing_core::Interest::sometimes
[never]: tracing_core::Interest::never
[`Subscriber::register_callsite`]: tracing_core::Subscriber::register_callsite
[`Subscriber`]: tracing_core::Subscriber
[`enabled`]: Filter::enabled
[`Filtered`]: crate::filter::Filtered

Unresolved upstream links (retained, not inferred): `tracing_core::Interest::always`, `tracing_core::Interest::never`, `tracing_core::Interest::sometimes`, `tracing_core::Subscriber::register_callsite`.

<a id="op-104b1d844775b41b079882e6"></a>
## enabled

`function` · `tracing_subscriber::layer::Filter::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, meta: &Metadata<'_>, cx: &Context<'_, S>) -> bool
```

Source: `src/layer/mod.rs:1281`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if this layer is interested in a span or event with the
given [`Metadata`] in the current [`Context`](../operations/tracing_subscriber.layer.context.Context.md#op-0acb651e530d235188552e7d), similarly to
[`Subscriber::enabled`].

If this returns `false`, the span or event will be disabled _for the
wrapped [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8)_. Unlike [`Layer::enabled`](../operations/tracing_subscriber.layer.Layer.md#op-bf52d490c84266759988e51e), the span or event will
still be recorded if any _other_ layers choose to enable it. However,
the layer [filtered] by this filter will skip recording that span or
event.

If all layers indicate that they do not wish to see this span or event,
it will be disabled.

[`metadata`]: tracing_core::Metadata
[`Subscriber::enabled`]: tracing_core::Subscriber::enabled
[filtered]: crate::filter::Filtered

Unresolved upstream links (retained, not inferred): `tracing_core::Subscriber::enabled`.

<a id="op-5acd83c66b6bfc670a28b845"></a>
## event_enabled

`function` · `tracing_subscriber::layer::Filter::event_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, event: &Event<'_>, cx: &Context<'_, S>) -> bool
```

Source: `src/layer/mod.rs:1414`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Called before the filtered [`Layer]'s [`on_event`], to determine if
`on_event` should be called.

This gives a chance to filter events based on their fields. Note,
however, that this *does not* override [`enabled`], and is not even
called if [`enabled`] returns `false`.

## Default Implementation

By default, this method returns `true`, indicating that no events are
filtered out based on their fields.

[`enabled`]: crate::layer::Filter::enabled
[`on_event`]: crate::layer::Layer::on_event

<a id="op-2e42e1baab0662273a9f7d60"></a>
## max_level_hint

`function` · `tracing_subscriber::layer::Filter::max_level_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn max_level_hint(&self) -> Option<LevelFilter>
```

Source: `src/layer/mod.rs:1448`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an optional hint of the highest [verbosity level][level] that
this `Filter` will enable.

If this method returns a [`LevelFilter`], it will be used as a hint to
determine the most verbose level that will be enabled. This will allow
spans and events which are more verbose than that level to be skipped
more efficiently. An implementation of this method is optional, but
strongly encouraged.

If the maximum level the `Filter` will enable can change over the
course of its lifetime, it is free to return a different value from
multiple invocations of this method. However, note that changes in the
maximum level will **only** be reflected after the callsite [`Interest`]
cache is rebuilt, by calling the
[`tracing_core::callsite::rebuild_interest_cache`][rebuild] function.
Therefore, if the `Filter will change the value returned by this
method, it is responsible for ensuring that
[`rebuild_interest_cache`][rebuild] is called after the value of the max
level changes.

## Default Implementation

By default, this method returns `None`, indicating that the maximum
level is unknown.

[level]: tracing_core::metadata::Level
[`LevelFilter`]: crate::filter::LevelFilter
[`Interest`]: tracing_core::subscriber::Interest
[rebuild]: tracing_core::callsite::rebuild_interest_cache

<a id="op-04a5709a0659b887ca6e9d0b"></a>
## on_close

`function` · `tracing_subscriber::layer::Filter::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: span::Id, ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:1493`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this filter that a span with the given ID has been closed.

By default, this method does nothing. `Filter` implementations that
need to be notified when a span is closed can override this method.

<a id="op-2d18e530aa5ae59d5aeaebf7"></a>
## on_enter

`function` · `tracing_subscriber::layer::Filter::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:1477`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this filter that a span with the given ID was entered.

By default, this method does nothing. `Filter` implementations that
need to be notified when a span is entered can override this method.

<a id="op-1d19fba59da48e45193b7aef"></a>
## on_exit

`function` · `tracing_subscriber::layer::Filter::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:1485`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this filter that a span with the given ID was exited.

By default, this method does nothing. `Filter` implementations that
need to be notified when a span is exited can override this method.

<a id="op-da649213da0e43a5b12dee9c"></a>
## on_new_span

`function` · `tracing_subscriber::layer::Filter::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:1458`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this filter that a new span was constructed with the given
`Attributes` and `Id`.

By default, this method does nothing. `Filter` implementations that
need to be notified when new spans are created can override this
method.

<a id="op-99b67e470fd5268737377d5b"></a>
## on_record

`function` · `tracing_subscriber::layer::Filter::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, id: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:1469`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this filter that a span with the given `Id` recorded the given
`values`.

By default, this method does nothing. `Filter` implementations that
need to be notified when new spans are created can override this
method.
