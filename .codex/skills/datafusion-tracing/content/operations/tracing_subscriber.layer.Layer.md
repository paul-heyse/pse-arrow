# `tracing_subscriber::layer::Layer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.layer.Layer.json).

<a id="op-4c1ba1a6be909c9a1b91bff8"></a>
## Layer

`trait` · `tracing_subscriber::layer::Layer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait Layer<S> where S: Subscriber, Self: 'static
```

Source: `src/layer/mod.rs:728`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A composable handler for `tracing` events.

A `Layer` implements a behavior for recording or collecting traces that can
be composed together with other `Layer`s to build a [`Subscriber`]. See the
[module-level documentation](crate::layer) for details.

[`Subscriber`]: tracing_core::Subscriber

<a id="op-0cb46f508bf4be2b307fc2e2"></a>
## and_then

`function` · `tracing_subscriber::layer::Layer::and_then` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn and_then<L>(self, layer: L) -> Layered<L, Self, S> where L: Layer<S>, Self: Sized
```

Source: `src/layer/mod.rs:1040`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Composes this layer around the given `Layer`, returning a `Layered`
struct implementing `Layer`.

The returned `Layer` will call the methods on this `Layer` and then
those of the new `Layer`, before calling the methods on the subscriber
it wraps. For example:

```rust
# use tracing_subscriber::layer::Layer;
# use tracing_core::Subscriber;
pub struct FooLayer {
    // ...
}

pub struct BarLayer {
    // ...
}

pub struct MySubscriber {
    // ...
}

impl<S: Subscriber> Layer<S> for FooLayer {
    // ...
}

impl<S: Subscriber> Layer<S> for BarLayer {
    // ...
}

# impl FooLayer {
# fn new() -> Self { Self {} }
# }
# impl BarLayer {
# fn new() -> Self { Self { }}
# }
# impl MySubscriber {
# fn new() -> Self { Self { }}
# }
# use tracing_core::{span::{Id, Attributes, Record}, Metadata, Event};
# impl tracing_core::Subscriber for MySubscriber {
#   fn new_span(&self, _: &Attributes) -> Id { Id::from_u64(1) }
#   fn record(&self, _: &Id, _: &Record) {}
#   fn event(&self, _: &Event) {}
#   fn record_follows_from(&self, _: &Id, _: &Id) {}
#   fn enabled(&self, _: &Metadata) -> bool { false }
#   fn enter(&self, _: &Id) {}
#   fn exit(&self, _: &Id) {}
# }
let subscriber = FooLayer::new()
    .and_then(BarLayer::new())
    .with_subscriber(MySubscriber::new());
```

Multiple layers may be composed in this manner:

```rust
# use tracing_subscriber::layer::Layer;
# use tracing_core::Subscriber;
# pub struct FooLayer {}
# pub struct BarLayer {}
# pub struct MySubscriber {}
# impl<S: Subscriber> Layer<S> for FooLayer {}
# impl<S: Subscriber> Layer<S> for BarLayer {}
# impl FooLayer {
# fn new() -> Self { Self {} }
# }
# impl BarLayer {
# fn new() -> Self { Self { }}
# }
# impl MySubscriber {
# fn new() -> Self { Self { }}
# }
# use tracing_core::{span::{Id, Attributes, Record}, Metadata, Event};
# impl tracing_core::Subscriber for MySubscriber {
#   fn new_span(&self, _: &Attributes) -> Id { Id::from_u64(1) }
#   fn record(&self, _: &Id, _: &Record) {}
#   fn event(&self, _: &Event) {}
#   fn record_follows_from(&self, _: &Id, _: &Id) {}
#   fn enabled(&self, _: &Metadata) -> bool { false }
#   fn enter(&self, _: &Id) {}
#   fn exit(&self, _: &Id) {}
# }
pub struct BazLayer {
    // ...
}

impl<S: Subscriber> Layer<S> for BazLayer {
    // ...
}
# impl BazLayer { fn new() -> Self { BazLayer {} } }

let subscriber = FooLayer::new()
    .and_then(BarLayer::new())
    .and_then(BazLayer::new())
    .with_subscriber(MySubscriber::new());
```

<a id="op-795099c45322e075d31d1b80"></a>
## boxed

`function` · `tracing_subscriber::layer::Layer::boxed` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn boxed(self) -> Box<dyn Layer<S> + Send + Sync + 'static> where Self: Sized + Layer<S> + Send + Sync + 'static, S: Subscriber
```

Source: `src/layer/mod.rs:1235`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Erases the type of this [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8), returning a [`Box`]ed `dyn
Layer` trait object.

This can be used when a function returns a `Layer` which may be of
one of several types, or when a `Layer` subscriber has a very long type
signature.

# Examples

The following example will *not* compile, because the value assigned to
`log_layer` may have one of several different types:

```compile_fail
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use tracing_subscriber::{Layer, filter::LevelFilter, prelude::*};
use std::{path::PathBuf, fs::File, io};

/// Configures whether logs are emitted to a file, to stdout, or to stderr.
pub enum LogConfig {
    File(PathBuf),
    Stdout,
    Stderr,
}

let config = // ...
    # LogConfig::Stdout;

// Depending on the config, construct a layer of one of several types.
let log_layer = match config {
    // If logging to a file, use a maximally-verbose configuration.
    LogConfig::File(path) => {
        let file = File::create(path)?;
        tracing_subscriber::fmt::layer()
            .with_thread_ids(true)
            .with_thread_names(true)
            // Selecting the JSON logging format changes the layer's
            // type.
            .json()
            .with_span_list(true)
            // Setting the writer to use our log file changes the
            // layer's type again.
            .with_writer(file)
    },

    // If logging to stdout, use a pretty, human-readable configuration.
    LogConfig::Stdout => tracing_subscriber::fmt::layer()
        // Selecting the "pretty" logging format changes the
        // layer's type!
        .pretty()
        .with_writer(io::stdout)
        // Add a filter based on the RUST_LOG environment variable;
        // this changes the type too!
        .and_then(tracing_subscriber::EnvFilter::from_default_env()),

    // If logging to stdout, only log errors and warnings.
    LogConfig::Stderr => tracing_subscriber::fmt::layer()
        // Changing the writer changes the layer's type
        .with_writer(io::stderr)
        // Only log the `WARN` and `ERROR` levels. Adding a filter
        // changes the layer's type to `Filtered<LevelFilter, ...>`.
        .with_filter(LevelFilter::WARN),
};

tracing_subscriber::registry()
    .with(log_layer)
    .init();
# Ok(()) }
```

However, adding a call to `.boxed()` after each match arm erases the
layer's type, so this code *does* compile:

```
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use tracing_subscriber::{Layer, filter::LevelFilter, prelude::*};
# use std::{path::PathBuf, fs::File, io};
# pub enum LogConfig {
#    File(PathBuf),
#    Stdout,
#    Stderr,
# }
# let config = LogConfig::Stdout;
let log_layer = match config {
    LogConfig::File(path) => {
        let file = File::create(path)?;
        tracing_subscriber::fmt::layer()
            .with_thread_ids(true)
            .with_thread_names(true)
            .json()
            .with_span_list(true)
            .with_writer(file)
            // Erase the type by boxing the layer
            .boxed()
    },

    LogConfig::Stdout => tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(io::stdout)
        .and_then(tracing_subscriber::EnvFilter::from_default_env())
        // Erase the type by boxing the layer
        .boxed(),

    LogConfig::Stderr => tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .with_filter(LevelFilter::WARN)
        // Erase the type by boxing the layer
        .boxed(),
};

tracing_subscriber::registry()
    .with(log_layer)
    .init();
# Ok(()) }
```

Unresolved upstream links (retained, not inferred): ``Box``.

<a id="op-bf52d490c84266759988e51e"></a>
## enabled

`function` · `tracing_subscriber::layer::Layer::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool
```

Source: `src/layer/mod.rs:869`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if this layer is interested in a span or event with the
given `metadata` in the current [`Context`](../operations/tracing_subscriber.layer.context.Context.md#op-0acb651e530d235188552e7d), similarly to
[`Subscriber::enabled`].

By default, this always returns `true`, allowing the wrapped subscriber
to choose to disable the span.

<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: This method (and <a href="#method.register_callsite">
<code>Layer::register_callsite</code></a>) determine whether a span or event is
globally enabled, <em>not</em> whether the individual layer will be
notified about that span or event. This is intended to be used
by layers that implement filtering for the entire stack. Layers which do
not wish to be notified about certain spans or events but do not wish to
globally disable them should ignore those spans or events in their
<a href="#method.on_event"><code>on_event</code></a>,
<a href="#method.on_enter"><code>on_enter</code></a>,
<a href="#method.on_exit"><code>on_exit</code></a>, and other notification
methods.
</pre>


See [the trait-level documentation] for more information on filtering
with `Layer`s.

[`Interest`]: tracing_core::Interest
[`Subscriber::enabled`]: tracing_core::Subscriber::enabled()
[`Layer::register_callsite`]: Layer::register_callsite()
[`on_event`]: Layer::on_event()
[`on_enter`]: Layer::on_enter()
[`on_exit`]: Layer::on_exit()
[the trait-level documentation]: #filtering-with-layers

Unresolved upstream links (retained, not inferred): `tracing_core::Subscriber::enabled()`.

<a id="op-ddaabed96bb016dd783e422f"></a>
## event_enabled

`function` · `tracing_subscriber::layer::Layer::event_enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, _event: &Event<'_>, _ctx: Context<'_, S>) -> bool
```

Source: `src/layer/mod.rs:923`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Called before [`on_event`], to determine if `on_event` should be called.

<div class="example-wrap" style="display:inline-block">
<pre class="ignore" style="white-space:normal;font:inherit;">

**Note**: This method determines whether an event is globally enabled,
*not* whether the individual `Layer` will be notified about the
event. This is intended to be used by `Layer`s that implement
filtering for the entire stack. `Layer`s which do not wish to be
notified about certain events but do not wish to globally disable them
should ignore those events in their [on_event][Self::on_event](../operations/tracing_subscriber.layer.Layer.md#op-15bf6d0857502ca2d97e4bd1).

</pre></div>

See [the trait-level documentation] for more information on filtering
with `Layer`s.

[`on_event`]: Self::on_event
[`Interest`]: tracing_core::Interest
[the trait-level documentation]: #filtering-with-layers

<a id="op-0ec2d3e1932d45768843f8f4"></a>
## on_close

`function` · `tracing_subscriber::layer::Layer::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, _id: span::Id, _ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:937`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that the span with the given ID has been closed.

<a id="op-a351326be900db143e10528e"></a>
## on_enter

`function` · `tracing_subscriber::layer::Layer::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, _id: &span::Id, _ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:931`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that a span with the given ID was entered.

<a id="op-15bf6d0857502ca2d97e4bd1"></a>
## on_event

`function` · `tracing_subscriber::layer::Layer::on_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:928`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that an event has occurred.

<a id="op-5895c9c1e516a18c3eefe5ad"></a>
## on_exit

`function` · `tracing_subscriber::layer::Layer::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, _id: &span::Id, _ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:934`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that the span with the given ID was exited.

<a id="op-446a3c717521220b96dba380"></a>
## on_follows_from

`function` · `tracing_subscriber::layer::Layer::on_follows_from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_follows_from(&self, _span: &span::Id, _follows: &span::Id, _ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:900`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that a span with the ID `span` recorded that it
follows from the span with the ID `follows`.

<a id="op-36e393b9f5ce0e9954e8c6b0"></a>
## on_id_change

`function` · `tracing_subscriber::layer::Layer::on_id_change` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_id_change(&self, _old: &span::Id, _new: &span::Id, _ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:941`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that a span ID has been cloned, and that the
subscriber returned a different ID.

<a id="op-cacfa9f7e5bf66f4017b2a44"></a>
## on_layer

`function` · `tracing_subscriber::layer::Layer::on_layer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_layer(&mut self, subscriber: &mut S)
```

Source: `src/layer/mod.rs:785`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Performs late initialization when attaching a `Layer` to a
[`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c).

This is a callback that is called when the `Layer` is added to a
[`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c) (e.g. in [`Layer::with_subscriber`](../operations/tracing_subscriber.layer.Layer.md#op-2f33f8d942a646eb3d8e5f41) and
[`SubscriberExt::with`](../operations/tracing_subscriber.layer.SubscriberExt.md#op-0268c7766658f40cc07d8c16)). Since this can only occur before the
[`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c) has been set as the default, both the `Layer` and
[`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c) are passed to this method _mutably_. This gives the
`Layer` the opportunity to set any of its own fields with values
received by method calls on the [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c).

For example, [`Filtered`] layers implement `on_layer` to call the
[`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c)'s [`register_filter`] method, and store the returned
[`FilterId`] as a field.

**Note** In most cases, `Layer` implementations will not need to
implement this method. However, in cases where a type implementing
`Layer` wraps one or more other types that implement `Layer`, like the
[`Layered`](../operations/tracing_subscriber.layer.layered.Layered.md#op-a34d0c9535f11404fda44764) and [`Filtered`] types in this crate, that type MUST ensure
that the inner `Layer`s' `on_layer` methods are called. Otherwise,
functionality that relies on `on_layer`, such as [per-layer filtering],
may not work correctly.

[`Filtered`]: crate::filter::Filtered
[`register_filter`]: crate::registry::LookupSpan::register_filter
[per-layer filtering]: #per-layer-filtering
[`FilterId`]: crate::filter::FilterId

<a id="op-501bbdafad438573cae702ef"></a>
## on_new_span

`function` · `tracing_subscriber::layer::Layer::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:876`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that a new span was constructed with the given
`Attributes` and `Id`.

<a id="op-de23a21f9d492196ee6efda9"></a>
## on_record

`function` · `tracing_subscriber::layer::Layer::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, _span: &span::Id, _values: &span::Record<'_>, _ctx: Context<'_, S>)
```

Source: `src/layer/mod.rs:893`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Notifies this layer that a span with the given `Id` recorded the given
`values`.

<a id="op-37a0cc162283aa68420a0e50"></a>
## on_register_dispatch

`function` · `tracing_subscriber::layer::Layer::on_register_dispatch` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_register_dispatch(&self, subscriber: &Dispatch)
```

Source: `src/layer/mod.rs:754`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Performs late initialization when installing this layer as a
[`Subscriber`].

## Avoiding Memory Leaks

`Layer`s should not store the [`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c) pointing to the [`Subscriber`]
that they are a part of. Because the `Dispatch` owns the `Subscriber`,
storing the `Dispatch` within the `Subscriber` will create a reference
count cycle, preventing the `Dispatch` from ever being dropped.

Instead, when it is necessary to store a cyclical reference to the
`Dispatch` within a `Layer`, use [`Dispatch::downgrade`] to convert a
`Dispatch` into a [`WeakDispatch`]. This type is analogous to
[`std::sync::Weak`], and does not create a reference count cycle. A
[`WeakDispatch`] can be stored within a subscriber without causing a
memory leak, and can be [upgraded] into a `Dispatch` temporarily when
the `Dispatch` must be accessed by the subscriber.

[`WeakDispatch`]: tracing_core::dispatcher::WeakDispatch
[upgraded]: tracing_core::dispatcher::WeakDispatch::upgrade
[`Subscriber`]: tracing_core::Subscriber

Unresolved upstream links (retained, not inferred): ``std::sync::Weak``, `tracing_core::dispatcher::WeakDispatch::upgrade`, ``Dispatch::downgrade``.

<a id="op-a3dca9d32377d9e88d8c1899"></a>
## register_callsite

`function` · `tracing_subscriber::layer::Layer::register_callsite` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Source: `src/layer/mod.rs:829`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Registers a new callsite with this layer, returning whether or not
the layer is interested in being notified about the callsite, similarly
to [`Subscriber::register_callsite`].

By default, this returns [`Interest::always()`] if [`self.enabled`] returns
true, or [`Interest::never()`] if it returns false.

<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: This method (and <a href="#method.enabled">
<code>Layer::enabled</code></a>) determine whether a span or event is
globally enabled, <em>not</em> whether the individual layer will be
notified about that span or event. This is intended to be used
by layers that implement filtering for the entire stack. Layers which do
not wish to be notified about certain spans or events but do not wish to
globally disable them should ignore those spans or events in their
<a href="#method.on_event"><code>on_event</code></a>,
<a href="#method.on_enter"><code>on_enter</code></a>,
<a href="#method.on_exit"><code>on_exit</code></a>, and other notification
methods.
</pre>

See [the trait-level documentation] for more information on filtering
with `Layer`s.

Layers may also implement this method to perform any behaviour that
should be run once per callsite. If the layer wishes to use
`register_callsite` for per-callsite behaviour, but does not want to
globally enable or disable those callsites, it should always return
[`Interest::always()`].

[`Interest`]: tracing_core::Interest
[`Subscriber::register_callsite`]: tracing_core::Subscriber::register_callsite()
[`Interest::never()`]: tracing_core::subscriber::Interest::never()
[`Interest::always()`]: tracing_core::subscriber::Interest::always()
[`self.enabled`]: Layer::enabled()
[`Layer::enabled`]: Layer::enabled()
[`on_event`]: Layer::on_event()
[`on_enter`]: Layer::on_enter()
[`on_exit`]: Layer::on_exit()
[the trait-level documentation]: #filtering-with-layers

Unresolved upstream links (retained, not inferred): `tracing_core::subscriber::Interest::never()`, `tracing_core::subscriber::Interest::always()`, `tracing_core::Subscriber::register_callsite()`.

<a id="op-7bd3ef10607f7c7b6ba70fdd"></a>
## with_filter

`function` · `tracing_subscriber::layer::Layer::with_filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_filter<F>(self, filter: F) -> filter::Filtered<Self, F, S> where Self: Sized, F: Filter<S>
```

Source: `src/layer/mod.rs:1111`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines `self` with a [`Filter`](../operations/tracing_subscriber.layer.Filter.md#op-fd31ac1a2cd1ed4f447625f3), returning a [`Filtered`] layer.

The [`Filter`](../operations/tracing_subscriber.layer.Filter.md#op-fd31ac1a2cd1ed4f447625f3) will control which spans and events are enabled for
this layer. See [the trait-level documentation][plf] for details on
per-layer filtering.

[`Filtered`]: crate::filter::Filtered
[plf]: crate::layer#per-layer-filtering

<a id="op-2f33f8d942a646eb3d8e5f41"></a>
## with_subscriber

`function` · `tracing_subscriber::layer::Layer::with_subscriber` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_subscriber(self, inner: S) -> Layered<Self, S> where Self: Sized
```

Source: `src/layer/mod.rs:1092`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

 Composes this `Layer` with the given [`Subscriber`], returning a
 `Layered` struct that implements [`Subscriber`].

 The returned `Layered` subscriber will call the methods on this `Layer`
 and then those of the wrapped subscriber.

 For example:
 ```rust
 # use tracing_subscriber::layer::Layer;
 # use tracing_core::Subscriber;
 pub struct FooLayer {
     // ...
 }

 pub struct MySubscriber {
     // ...
 }

 impl<S: Subscriber> Layer<S> for FooLayer {
     // ...
 }

 # impl FooLayer {
 # fn new() -> Self { Self {} }
 # }
 # impl MySubscriber {
 # fn new() -> Self { Self { }}
 # }
 # use tracing_core::{span::{Id, Attributes, Record}, Metadata};
 # impl tracing_core::Subscriber for MySubscriber {
 #   fn new_span(&self, _: &Attributes) -> Id { Id::from_u64(0) }
 #   fn record(&self, _: &Id, _: &Record) {}
 #   fn event(&self, _: &tracing_core::Event) {}
 #   fn record_follows_from(&self, _: &Id, _: &Id) {}
 #   fn enabled(&self, _: &Metadata) -> bool { false }
 #   fn enter(&self, _: &Id) {}
 #   fn exit(&self, _: &Id) {}
 # }
 let subscriber = FooLayer::new()
     .with_subscriber(MySubscriber::new());
```

 [`Subscriber`]: tracing_core::Subscriber
