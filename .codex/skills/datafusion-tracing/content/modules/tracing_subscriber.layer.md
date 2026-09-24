# `tracing_subscriber::layer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.layer.json).

<a id="op-2ad2c88abbc1580a0091cf9d"></a>
## layer

`module` · `tracing_subscriber::layer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
mod layer
```

Source: `src/layer/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

 The [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) trait, a composable abstraction for building [`Subscriber`]s.

 The [`Subscriber`] trait in `tracing-core` represents the _complete_ set of
 functionality required to consume `tracing` instrumentation. This means that
 a single `Subscriber` instance is a self-contained implementation of a
 complete strategy for collecting traces; but it _also_ means that the
 `Subscriber` trait cannot easily be composed with other `Subscriber`s.

 In particular, [`Subscriber`]s are responsible for generating [span IDs] and
 assigning them to spans. Since these IDs must uniquely identify a span
 within the context of the current trace, this means that there may only be
 a single `Subscriber` for a given thread at any point in time &mdash;
 otherwise, there would be no authoritative source of span IDs.

 On the other hand, the majority of the [`Subscriber`] trait's functionality
 is composable: any number of subscribers may _observe_ events, span entry
 and exit, and so on, provided that there is a single authoritative source of
 span IDs. The [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) trait represents this composable subset of the
 [`Subscriber`] behavior; it can _observe_ events and spans, but does not
 assign IDs.

 # Composing Layers

 Since a [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) does not implement a complete strategy for collecting
 traces, it must be composed with a `Subscriber` in order to be used. The
 [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) trait is generic over a type parameter (called `S` in the trait
 definition), representing the types of `Subscriber` they can be composed
 with. Thus, a [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) may be implemented that will only compose with a
 particular `Subscriber` implementation, or additional trait bounds may be
 added to constrain what types implementing `Subscriber` a `Layer` can wrap.

 `Layer`s may be added to a `Subscriber` by using the [`SubscriberExt::with`](../operations/tracing_subscriber.layer.SubscriberExt.md#op-0268c7766658f40cc07d8c16)
 method, which is provided by `tracing-subscriber`'s [prelude]. This method
 returns a [`Layered`](../operations/tracing_subscriber.layer.layered.Layered.md#op-a34d0c9535f11404fda44764) struct that implements `Subscriber` by composing the
 `Layer` with the `Subscriber`.

 For example:
 ```rust
 use tracing_subscriber::Layer;
 use tracing_subscriber::prelude::*;
 use tracing::Subscriber;

 pub struct MyLayer {
     // ...
 }

 impl<S: Subscriber> Layer<S> for MyLayer {
     // ...
 }

 pub struct MySubscriber {
     // ...
 }

 # use tracing_core::{span::{Id, Attributes, Record}, Metadata, Event};
 impl Subscriber for MySubscriber {
     // ...
 #   fn new_span(&self, _: &Attributes) -> Id { Id::from_u64(1) }
 #   fn record(&self, _: &Id, _: &Record) {}
 #   fn event(&self, _: &Event) {}
 #   fn record_follows_from(&self, _: &Id, _: &Id) {}
 #   fn enabled(&self, _: &Metadata) -> bool { false }
 #   fn enter(&self, _: &Id) {}
 #   fn exit(&self, _: &Id) {}
 }
 # impl MyLayer {
 # fn new() -> Self { Self {} }
 # }
 # impl MySubscriber {
 # fn new() -> Self { Self {} }
 # }

 let subscriber = MySubscriber::new()
     .with(MyLayer::new());

 tracing::subscriber::set_global_default(subscriber);
 ```

 Multiple `Layer`s may be composed in the same manner:
 ```rust
 # use tracing_subscriber::{Layer, layer::SubscriberExt};
 # use tracing::Subscriber;
 pub struct MyOtherLayer {
     // ...
 }

 impl<S: Subscriber> Layer<S> for MyOtherLayer {
     // ...
 }

 pub struct MyThirdLayer {
     // ...
 }

 impl<S: Subscriber> Layer<S> for MyThirdLayer {
     // ...
 }
 # pub struct MyLayer {}
 # impl<S: Subscriber> Layer<S> for MyLayer {}
 # pub struct MySubscriber { }
 # use tracing_core::{span::{Id, Attributes, Record}, Metadata, Event};
 # impl Subscriber for MySubscriber {
 #   fn new_span(&self, _: &Attributes) -> Id { Id::from_u64(1) }
 #   fn record(&self, _: &Id, _: &Record) {}
 #   fn event(&self, _: &Event) {}
 #   fn record_follows_from(&self, _: &Id, _: &Id) {}
 #   fn enabled(&self, _: &Metadata) -> bool { false }
 #   fn enter(&self, _: &Id) {}
 #   fn exit(&self, _: &Id) {}
 # }
 # impl MyLayer {
 # fn new() -> Self { Self {} }
 # }
 # impl MyOtherLayer {
 # fn new() -> Self { Self {} }
 # }
 # impl MyThirdLayer {
 # fn new() -> Self { Self {} }
 # }
 # impl MySubscriber {
 # fn new() -> Self { Self {} }
 # }

 let subscriber = MySubscriber::new()
     .with(MyLayer::new())
     .with(MyOtherLayer::new())
     .with(MyThirdLayer::new());

 tracing::subscriber::set_global_default(subscriber);
 ```

 The [`Layer::with_subscriber`](../operations/tracing_subscriber.layer.Layer.md#op-2f33f8d942a646eb3d8e5f41) constructs the [`Layered`](../operations/tracing_subscriber.layer.layered.Layered.md#op-a34d0c9535f11404fda44764) type from a
 [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) and [`Subscriber`], and is called by [`SubscriberExt::with`](../operations/tracing_subscriber.layer.SubscriberExt.md#op-0268c7766658f40cc07d8c16). In
 general, it is more idiomatic to use [`SubscriberExt::with`](../operations/tracing_subscriber.layer.SubscriberExt.md#op-0268c7766658f40cc07d8c16), and treat
 [`Layer::with_subscriber`](../operations/tracing_subscriber.layer.Layer.md#op-2f33f8d942a646eb3d8e5f41) as an implementation detail, as `with_subscriber`
 calls must be nested, leading to less clear code for the reader.

 ## Runtime Configuration With `Layer`s

 In some cases, a particular [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) may be enabled or disabled based on
 runtime configuration. This can introduce challenges, because the type of a
 layered [`Subscriber`] depends on which layers are added to it: if an `if`
 or `match` expression adds some [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) implementation in one branch,
 and other layers in another, the [`Subscriber`] values returned by those
 branches will have different types. For example, the following _will not_
 work:

 ```compile_fail
 # fn docs() -> Result<(), Box<dyn std::error::Error + 'static>> {
 # struct Config {
 #    is_prod: bool,
 #    path: &'static str,
 # }
 # let cfg = Config { is_prod: false, path: "debug.log" };
 use std::fs::File;
 use tracing_subscriber::{Registry, prelude::*};

 let stdout_log = tracing_subscriber::fmt::layer().pretty();
 let subscriber = Registry::default().with(stdout_log);

 // The compile error will occur here because the if and else
 // branches have different (and therefore incompatible) types.
 let subscriber = if cfg.is_prod {
     let file = File::create(cfg.path)?;
     let layer = tracing_subscriber::fmt::layer()
         .json()
         .with_writer(Arc::new(file));
     layer.with(subscriber)
 } else {
     layer
 };

 tracing::subscriber::set_global_default(subscriber)
     .expect("Unable to set global subscriber");
 # Ok(()) }
 ```

 However, a [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) wrapped in an [`Option`] [also implements the `Layer`
 trait][option-impl]. This allows individual layers to be enabled or disabled at
 runtime while always producing a [`Subscriber`] of the same type. For
 example:

 ```
 # fn docs() -> Result<(), Box<dyn std::error::Error + 'static>> {
 # struct Config {
 #    is_prod: bool,
 #    path: &'static str,
 # }
 # let cfg = Config { is_prod: false, path: "debug.log" };
 use std::fs::File;
 use tracing_subscriber::{Registry, prelude::*};

 let stdout_log = tracing_subscriber::fmt::layer().pretty();
 let subscriber = Registry::default().with(stdout_log);

 // if `cfg.is_prod` is true, also log JSON-formatted logs to a file.
 let json_log = if cfg.is_prod {
     let file = File::create(cfg.path)?;
     let json_log = tracing_subscriber::fmt::layer()
         .json()
         .with_writer(file);
     Some(json_log)
 } else {
     None
 };

 // If `cfg.is_prod` is false, then `json` will be `None`, and this layer
 // will do nothing. However, the subscriber will still have the same type
 // regardless of whether the `Option`'s value is `None` or `Some`.
 let subscriber = subscriber.with(json_log);

 tracing::subscriber::set_global_default(subscriber)
    .expect("Unable to set global subscriber");
 # Ok(()) }
 ```

 If a [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) may be one of several different types, note that [`Box<dyn
 Layer<S> + Send + Sync>` implements `Layer`][box-impl].
 This may be used to erase the type of a [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8).

 For example, a function that configures a [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) to log to one of
 several outputs might return a `Box<dyn Layer<S> + Send + Sync + 'static>`:
 ```
 use tracing_subscriber::{
     Layer,
     registry::LookupSpan,
     prelude::*,
 };
 use std::{path::PathBuf, fs::File, io};

 /// Configures whether logs are emitted to a file, to stdout, or to stderr.
 pub enum LogConfig {
     File(PathBuf),
     Stdout,
     Stderr,
 }

 impl LogConfig {
     pub fn layer<S>(self) -> Box<dyn Layer<S> + Send + Sync + 'static>
     where
         S: tracing_core::Subscriber,
         for<'a> S: LookupSpan<'a>,
     {
         // Shared configuration regardless of where logs are output to.
         let fmt = tracing_subscriber::fmt::layer()
             .with_target(true)
             .with_thread_names(true);

         // Configure the writer based on the desired log target:
         match self {
             LogConfig::File(path) => {
                 let file = File::create(path).expect("failed to create log file");
                 Box::new(fmt.with_writer(file))
             },
             LogConfig::Stdout => Box::new(fmt.with_writer(io::stdout)),
             LogConfig::Stderr => Box::new(fmt.with_writer(io::stderr)),
         }
     }
 }

 let config = LogConfig::Stdout;
 tracing_subscriber::registry()
     .with(config.layer())
     .init();
 ```

 The [`Layer::boxed`](../operations/tracing_subscriber.layer.Layer.md#op-795099c45322e075d31d1b80) method is provided to make boxing a `Layer`
 more convenient, but [`Box::new`] may be used as well.

 When the number of `Layer`s varies at runtime, note that a
 [`Vec<L> where L: Layer` also implements `Layer`][vec-impl]. This
 can be used to add a variable number of `Layer`s to a `Subscriber`:

 ```
 use tracing_subscriber::{Layer, prelude::*};
 struct MyLayer {
     // ...
 }
 # impl MyLayer { fn new() -> Self { Self {} }}

 impl<S: tracing_core::Subscriber> Layer<S> for MyLayer {
     // ...
 }

 /// Returns how many layers we need
 fn how_many_layers() -> usize {
     // ...
     # 3
 }

 // Create a variable-length `Vec` of layers
 let mut layers = Vec::new();
 for _ in 0..how_many_layers() {
     layers.push(MyLayer::new());
 }

 tracing_subscriber::registry()
     .with(layers)
     .init();
 ```

 If a variable number of `Layer` is needed and those `Layer`s have
 different types, a `Vec` of [boxed `Layer` trait objects][box-impl] may
 be used. For example:

 ```
 use tracing_subscriber::{filter::LevelFilter, Layer, prelude::*};
 use std::fs::File;
 # fn main() -> Result<(), Box<dyn std::error::Error>> {
 struct Config {
     enable_log_file: bool,
     enable_stdout: bool,
     enable_stderr: bool,
     // ...
 }
 # impl Config {
 #    fn from_config_file()-> Result<Self, Box<dyn std::error::Error>> {
 #         // don't enable the log file so that the example doesn't actually create it
 #         Ok(Self { enable_log_file: false, enable_stdout: true, enable_stderr: true })
 #    }
 # }

 let cfg = Config::from_config_file()?;

 // Based on our dynamically loaded config file, create any number of layers:
 let mut layers = Vec::new();

 if cfg.enable_log_file {
     let file = File::create("myapp.log")?;
     let layer = tracing_subscriber::fmt::layer()
         .with_thread_names(true)
         .with_target(true)
         .json()
         .with_writer(file)
         // Box the layer as a type-erased trait object, so that it can
         // be pushed to the `Vec`.
         .boxed();
     layers.push(layer);
 }

 if cfg.enable_stdout {
     let layer = tracing_subscriber::fmt::layer()
         .pretty()
         .with_filter(LevelFilter::INFO)
         // Box the layer as a type-erased trait object, so that it can
         // be pushed to the `Vec`.
         .boxed();
     layers.push(layer);
 }

 if cfg.enable_stdout {
     let layer = tracing_subscriber::fmt::layer()
         .with_target(false)
         .with_filter(LevelFilter::WARN)
         // Box the layer as a type-erased trait object, so that it can
         // be pushed to the `Vec`.
         .boxed();
     layers.push(layer);
 }

 tracing_subscriber::registry()
     .with(layers)
     .init();
# Ok(()) }
 ```

 Finally, if the number of layers _changes_ at runtime, a `Vec` of
 subscribers can be used alongside the [`reload`](crate::reload) module to
 add or remove subscribers dynamically at runtime.

 [option-impl]: Layer#impl-Layer<S>-for-Option<L>
 [box-impl]: Layer#impl-Layer%3CS%3E-for-Box%3Cdyn%20Layer%3CS%3E%20+%20Send%20+%20Sync%3E
 [vec-impl]: Layer#impl-Layer<S>-for-Vec<L>
 [prelude]: crate::prelude

 # Recording Traces

 The [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) trait defines a set of methods for consuming notifications from
 tracing instrumentation, which are generally equivalent to the similarly
 named methods on [`Subscriber`]. Unlike [`Subscriber`], the methods on
 `Layer` are additionally passed a [`Context`](../operations/tracing_subscriber.layer.context.Context.md#op-0acb651e530d235188552e7d) type, which exposes additional
 information provided by the wrapped subscriber (such as [the current span])
 to the layer.

 # Filtering with `Layer`s

 As well as strategies for handling trace events, the `Layer` trait may also
 be used to represent composable _filters_. This allows the determination of
 what spans and events should be recorded to be decoupled from _how_ they are
 recorded: a filtering layer can be applied to other layers or
 subscribers. `Layer`s can be used to implement _global filtering_, where a
 `Layer` provides a filtering strategy for the entire subscriber.
 Additionally, individual recording `Layer`s or sets of `Layer`s may be
 combined with _per-layer filters_ that control what spans and events are
 recorded by those layers.

 ## Global Filtering

 A `Layer` that implements a filtering strategy should override the
 [`register_callsite`] and/or [`enabled`] methods. It may also choose to implement
 methods such as [`on_enter`], if it wishes to filter trace events based on
 the current span context.

 Note that the [`Layer::register_callsite`] and [`Layer::enabled`] methods
 determine whether a span or event is enabled *globally*. Thus, they should
 **not** be used to indicate whether an individual layer wishes to record a
 particular span or event. Instead, if a layer is only interested in a subset
 of trace data, but does *not* wish to disable other spans and events for the
 rest of the layer stack should ignore those spans and events in its
 notification methods.

 The filtering methods on a stack of `Layer`s are evaluated in a top-down
 order, starting with the outermost `Layer` and ending with the wrapped
 [`Subscriber`]. If any layer returns `false` from its [`enabled`] method, or
 [`Interest::never()`] from its [`register_callsite`] method, filter
 evaluation will short-circuit and the span or event will be disabled.

 ### Enabling Interest

 Whenever an tracing event (or span) is emitted, it goes through a number of
 steps to determine how and how much it should be processed. The earlier an
 event is disabled, the less work has to be done to process the event, so
 `Layer`s that implement filtering should attempt to disable unwanted
 events as early as possible. In order, each event checks:

 - [`register_callsite`], once per callsite (roughly: once per time that
   `event!` or `span!` is written in the source code; this is cached at the
   callsite). See [`Subscriber::register_callsite`] and
   [`tracing_core::callsite`](../modules/tracing_core.callsite.md#op-1740ba2098821878828cbb38) for a summary of how this behaves.
 - [`enabled`], once per emitted event (roughly: once per time that `event!`
   or `span!` is *executed*), and only if `register_callsite` registers an
   [`Interest::sometimes`]. This is the main customization point to globally
   filter events based on their [`Metadata`](../operations/tracing_core.metadata.Metadata.md#op-3c5a7a9d81c273e2173bb24c). If an event can be disabled
   based only on [`Metadata`](../operations/tracing_core.metadata.Metadata.md#op-3c5a7a9d81c273e2173bb24c), it should be, as this allows the construction
   of the actual `Event`/`Span` to be skipped.
 - For events only (and not spans), [`event_enabled`] is called just before
   processing the event. This gives layers one last chance to say that
   an event should be filtered out, now that the event's fields are known.

 ## Per-Layer Filtering

 **Note**: per-layer filtering APIs currently require the [`"registry"` crate
 feature flag][feat] to be enabled.

 Sometimes, it may be desirable for one `Layer` to record a particular subset
 of spans and events, while a different subset of spans and events are
 recorded by other `Layer`s. For example:

 - A layer that records metrics may wish to observe only events including
   particular tracked values, while a logging layer ignores those events.
 - If recording a distributed trace is expensive, it might be desirable to
   only send spans with `INFO` and lower verbosity to the distributed tracing
   system, while logging more verbose spans to a file.
 - Spans and events with a particular target might be recorded differently
   from others, such as by generating an HTTP access log from a span that
   tracks the lifetime of an HTTP request.

 The [`Filter`] trait is used to control what spans and events are
 observed by an individual `Layer`, while still allowing other `Layer`s to
 potentially record them. The [`Layer::with_filter`](../operations/tracing_subscriber.layer.Layer.md#op-7bd3ef10607f7c7b6ba70fdd) method combines a
 `Layer` with a [`Filter`], returning a [`Filtered`] layer.

 This crate's [`filter`] module provides a number of types which implement
 the [`Filter`] trait, such as [`LevelFilter`], [`Targets`], and
 [`FilterFn`]. These [`Filter`]s provide ready-made implementations of
 common forms of filtering. For custom filtering policies, the [`FilterFn`]
 and [`DynFilterFn`] types allow implementing a [`Filter`] with a closure or
 function pointer. In addition, when more control is required, the [`Filter`]
 trait may also be implemented for user-defined types.

 [`Option<Filter>`] also implements [`Filter`], which allows for an optional
 filter. [`None`] filters out _nothing_ (that is, allows everything through). For
 example:

 ```rust
 # use tracing_subscriber::{filter::filter_fn, Layer};
 # use tracing_core::{Metadata, subscriber::Subscriber};
 # struct MyLayer<S>(std::marker::PhantomData<S>);
 # impl<S> MyLayer<S> { fn new() -> Self { Self(std::marker::PhantomData)} }
 # impl<S: Subscriber> Layer<S> for MyLayer<S> {}
 # fn my_filter(_: &str) -> impl Fn(&Metadata) -> bool { |_| true  }
 fn setup_tracing<S: Subscriber>(filter_config: Option<&str>) {
     let layer = MyLayer::<S>::new()
         .with_filter(filter_config.map(|config| filter_fn(my_filter(config))));
 //...
 }
 ```

 <pre class="compile_fail" style="white-space:normal;font:inherit;">
     <strong>Warning</strong>: Currently, the <a href="../struct.Registry.html">
     <code>Registry</code></a> type defined in this crate is the only root
     <code>Subscriber</code> capable of supporting <code>Layer</code>s with
     per-layer filters. In the future, new APIs will be added to allow other
     root <code>Subscriber</code>s to support per-layer filters.
 </pre>

 For example, to generate an HTTP access log based on spans with
 the `http_access` target, while logging other spans and events to
 standard out, a [`Filter`] can be added to the access log layer:

 ```
 use tracing_subscriber::{filter, prelude::*};

 // Generates an HTTP access log.
 let access_log = // ...
     # filter::LevelFilter::INFO;

 // Add a filter to the access log layer so that it only observes
 // spans and events with the `http_access` target.
 let access_log = access_log.with_filter(filter::filter_fn(|metadata| {
     // Returns `true` if and only if the span or event's target is
     // "http_access".
     metadata.target() == "http_access"
 }));

 // A general-purpose logging layer.
 let fmt_layer = tracing_subscriber::fmt::layer();

 // Build a subscriber that combines the access log and stdout log
 // layers.
 tracing_subscriber::registry()
     .with(fmt_layer)
     .with(access_log)
     .init();
 ```

 Multiple layers can have their own, separate per-layer filters. A span or
 event will be recorded if it is enabled by _any_ per-layer filter, but it
 will be skipped by the layers whose filters did not enable it. Building on
 the previous example:

 ```
 use tracing_subscriber::{filter::{filter_fn, LevelFilter}, prelude::*};

 let access_log = // ...
     # LevelFilter::INFO;
 let fmt_layer = tracing_subscriber::fmt::layer();

 tracing_subscriber::registry()
     // Add the filter for the "http_access" target to the access
     // log layer, like before.
     .with(access_log.with_filter(filter_fn(|metadata| {
         metadata.target() == "http_access"
     })))
     // Add a filter for spans and events with the INFO level
     // and below to the logging layer.
     .with(fmt_layer.with_filter(LevelFilter::INFO))
     .init();

 // Neither layer will observe this event
 tracing::debug!(does_anyone_care = false, "a tree fell in the forest");

 // This event will be observed by the logging layer, but not
 // by the access log layer.
 tracing::warn!(dose_roentgen = %3.8, "not great, but not terrible");

 // This event will be observed only by the access log layer.
 tracing::trace!(target: "http_access", "HTTP request started");

 // Both layers will observe this event.
 tracing::error!(target: "http_access", "HTTP request failed with a very bad error!");
 ```

 A per-layer filter can be applied to multiple [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8)s at a time, by
 combining them into a [`Layered`](../operations/tracing_subscriber.layer.layered.Layered.md#op-a34d0c9535f11404fda44764) layer using [`Layer::and_then`](../operations/tracing_subscriber.layer.Layer.md#op-0cb46f508bf4be2b307fc2e2), and then
 calling [`Layer::with_filter`](../operations/tracing_subscriber.layer.Layer.md#op-7bd3ef10607f7c7b6ba70fdd) on the resulting [`Layered`](../operations/tracing_subscriber.layer.layered.Layered.md#op-a34d0c9535f11404fda44764) layer.

 Consider the following:
 - `layer_a` and `layer_b`, which should only receive spans and events at
   the [`INFO`] [level] and above.
 - A third layer, `layer_c`, which should receive spans and events at
   the [`DEBUG`] [level] as well.

 The layers and filters would be composed thusly:

 ```
 use tracing_subscriber::{filter::LevelFilter, prelude::*};

 let layer_a = // ...
 # LevelFilter::INFO;
 let layer_b =  // ...
 # LevelFilter::INFO;
 let layer_c =  // ...
 # LevelFilter::INFO;

 let info_layers = layer_a
     // Combine `layer_a` and `layer_b` into a `Layered` layer:
     .and_then(layer_b)
     // ...and then add an `INFO` `LevelFilter` to that layer:
     .with_filter(LevelFilter::INFO);

 tracing_subscriber::registry()
     // Add `layer_c` with a `DEBUG` filter.
     .with(layer_c.with_filter(LevelFilter::DEBUG))
     .with(info_layers)
     .init();
```

 If a [`Filtered`] [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) is combined with another [`Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8)
 [`Layer::and_then`](../operations/tracing_subscriber.layer.Layer.md#op-0cb46f508bf4be2b307fc2e2), and a filter is added to the [`Layered`](../operations/tracing_subscriber.layer.layered.Layered.md#op-a34d0c9535f11404fda44764) layer, that
 layer will be filtered by *both* the inner filter and the outer filter.
 Only spans and events that are enabled by *both* filters will be
 observed by that layer. This can be used to implement complex filtering
 trees.

 As an example, consider the following constraints:
 - Suppose that a particular [target] is used to indicate events that
   should be counted as part of a metrics system, which should be only
   observed by a layer that collects metrics.
 - A log of high-priority events ([`INFO`] and above) should be logged
   to stdout, while more verbose events should be logged to a debugging log file.
 - Metrics-focused events should *not* be included in either log output.

 In that case, it is possible to apply a filter to both logging layers to
 exclude the metrics events, while additionally adding a [`LevelFilter`]
 to the stdout log:

 ```
 # // wrap this in a function so we don't actually create `debug.log` when
 # // running the doctests..
 # fn docs() -> Result<(), Box<dyn std::error::Error + 'static>> {
 use tracing_subscriber::{filter, prelude::*};
 use std::{fs::File, sync::Arc};

 // A layer that logs events to stdout using the human-readable "pretty"
 // format.
 let stdout_log = tracing_subscriber::fmt::layer()
     .pretty();

 // A layer that logs events to a file.
 let file = File::create("debug.log")?;
 let debug_log = tracing_subscriber::fmt::layer()
     .with_writer(Arc::new(file));

 // A layer that collects metrics using specific events.
 let metrics_layer = /* ... */ filter::LevelFilter::INFO;

 tracing_subscriber::registry()
     .with(
         stdout_log
             // Add an `INFO` filter to the stdout logging layer
             .with_filter(filter::LevelFilter::INFO)
             // Combine the filtered `stdout_log` layer with the
             // `debug_log` layer, producing a new `Layered` layer.
             .and_then(debug_log)
             // Add a filter to *both* layers that rejects spans and
             // events whose targets start with `metrics`.
             .with_filter(filter::filter_fn(|metadata| {
                 !metadata.target().starts_with("metrics")
             }))
     )
     .with(
         // Add a filter to the metrics label that *only* enables
         // events whose targets start with `metrics`.
         metrics_layer.with_filter(filter::filter_fn(|metadata| {
             metadata.target().starts_with("metrics")
         }))
     )
     .init();

 // This event will *only* be recorded by the metrics layer.
 tracing::info!(target: "metrics::cool_stuff_count", value = 42);

 // This event will only be seen by the debug log file layer:
 tracing::debug!("this is a message, and part of a system of messages");

 // This event will be seen by both the stdout log layer *and*
 // the debug log file layer, but not by the metrics layer.
 tracing::warn!("the message is a warning about danger!");
 # Ok(()) }
 ```

 [`Subscriber`]: tracing_core::subscriber::Subscriber
 [span IDs]: tracing_core::span::Id
 [the current span]: Context::current_span
 [`register_callsite`]: Layer::register_callsite
 [`enabled`]: Layer::enabled
 [`event_enabled`]: Layer::event_enabled
 [`on_enter`]: Layer::on_enter
 [`Layer::register_callsite`]: Layer::register_callsite
 [`Layer::enabled`]: Layer::enabled
 [`Interest::never()`]: tracing_core::subscriber::Interest::never()
 [`Filtered`]: crate::filter::Filtered
 [`filter`]: crate::filter
 [`Targets`]: crate::filter::Targets
 [`FilterFn`]: crate::filter::FilterFn
 [`DynFilterFn`]: crate::filter::DynFilterFn
 [level]: tracing_core::Level
 [`INFO`]: tracing_core::Level::INFO
 [`DEBUG`]: tracing_core::Level::DEBUG
 [target]: tracing_core::Metadata::target
 [`LevelFilter`]: crate::filter::LevelFilter
 [feat]: crate#feature-flags

Unresolved upstream links (retained, not inferred): `tracing_core::Level::DEBUG`, ``Box::new``, `tracing_core::Level::INFO`, ``Interest::sometimes``, ``Option<Filter>``, `tracing_core::subscriber::Interest::never()`, `tracing_core::Metadata::target`, ``Option``, ``Subscriber::register_callsite``, ``None``.
