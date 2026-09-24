# `tracing_subscriber::filter::layer_filters::FilterExt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.layer_filters.FilterExt.json).

<a id="op-706ea1b7b4228b64b76d4ba5"></a>
## FilterExt

`trait` · `tracing_subscriber::filter::layer_filters::FilterExt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait FilterExt<S>: layer::Filter<S>
```

Source: `src/filter/layer_filters/mod.rs:182`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Extension trait adding [combinators] for combining [`Filter`].

[combinators]: crate::filter::combinator
[`Filter`]: crate::layer::Filter

<a id="op-3c3a87367661dd9b2afacca3"></a>
## and

`function` · `tracing_subscriber::filter::layer_filters::FilterExt::and` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn and<B>(self, other: B) -> combinator::And<Self, B, S> where Self: Sized, B: layer::Filter<S>
```

Source: `src/filter/layer_filters/mod.rs:224`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines this [`Filter`] with another [`Filter`] s so that spans and
events are enabled if and only if *both* filters return `true`.

# Examples

Enabling spans or events if they have both a particular target *and* are
above a certain level:

```
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter, FilterExt},
    prelude::*,
};

// Enables spans and events with targets starting with `interesting_target`:
let target_filter = filter_fn(|meta| {
    meta.target().starts_with("interesting_target")
});

// Enables spans and events with levels `INFO` and below:
let level_filter = LevelFilter::INFO;

// Combine the two filters together, returning a filter that only enables
// spans and events that *both* filters will enable:
let filter = target_filter.and(level_filter);

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_filter(filter))
    .init();

// This event will *not* be enabled:
tracing::info!("an event with an uninteresting target");

// This event *will* be enabled:
tracing::info!(target: "interesting_target", "a very interesting event");

// This event will *not* be enabled:
tracing::debug!(target: "interesting_target", "interesting debug event...");
```

[`Filter`]: crate::layer::Filter

<a id="op-965ee52fcf3d19c61b305d92"></a>
## boxed

`function` · `tracing_subscriber::filter::layer_filters::FilterExt::boxed` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn boxed(self) -> Box<dyn layer::Filter<S> + Send + Sync + 'static> where Self: Sized + Send + Sync + 'static
```

Source: `src/filter/layer_filters/mod.rs:450`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

[Boxes] `self`, erasing its concrete type.

This is equivalent to calling [`Box::new`], but in method form, so that
it can be used when chaining combinator methods.

# Examples

When different combinations of filters are used conditionally, they may
have different types. For example, the following code won't compile,
since the `if` and `else` clause produce filters of different types:

```compile_fail
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter, FilterExt},
    prelude::*,
};

let enable_bar_target: bool = // ...
# false;

let filter = if enable_bar_target {
    filter_fn(|meta| meta.target().starts_with("foo"))
        // If `enable_bar_target` is true, add a `filter_fn` enabling
        // spans and events with the target `bar`:
        .or(filter_fn(|meta| meta.target().starts_with("bar")))
        .and(LevelFilter::INFO)
} else {
    filter_fn(|meta| meta.target().starts_with("foo"))
        .and(LevelFilter::INFO)
};

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_filter(filter))
    .init();
```

By using `boxed`, the types of the two different branches can be erased,
so the assignment to the `filter` variable is valid (as both branches
have the type `Box<dyn Filter<S> + Send + Sync + 'static>`). The
following code *does* compile:

```
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter, FilterExt},
    prelude::*,
};

let enable_bar_target: bool = // ...
# false;

let filter = if enable_bar_target {
    filter_fn(|meta| meta.target().starts_with("foo"))
        .or(filter_fn(|meta| meta.target().starts_with("bar")))
        .and(LevelFilter::INFO)
        // Boxing the filter erases its type, so both branches now
        // have the same type.
        .boxed()
} else {
    filter_fn(|meta| meta.target().starts_with("foo"))
        .and(LevelFilter::INFO)
        .boxed()
};

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_filter(filter))
    .init();
```

[Boxes]: std::boxed
[`Box::new`]: std::boxed::Box::new

Unresolved upstream links (retained, not inferred): `std::boxed::Box::new`, `std::boxed`.

<a id="op-1246ff100777c68ac1f61e6e"></a>
## not

`function` · `tracing_subscriber::filter::layer_filters::FilterExt::not` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn not(self) -> combinator::Not<Self, S> where Self: Sized
```

Source: `src/filter/layer_filters/mod.rs:373`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Inverts `self`, returning a filter that enables spans and events only if
`self` would *not* enable them.

This inverts the values returned by the [`enabled`] and [`callsite_enabled`]
methods on the wrapped filter; it does *not* invert [`event_enabled`], as
filters which do not implement filtering on event field values will return
the default `true` even for events that their [`enabled`] method disables.

Consider a normal filter defined as:

```ignore (pseudo-code)
// for spans
match callsite_enabled() {
    ALWAYS => on_span(),
    SOMETIMES => if enabled() { on_span() },
    NEVER => (),
}
// for events
match callsite_enabled() {
   ALWAYS => on_event(),
   SOMETIMES => if enabled() && event_enabled() { on_event() },
   NEVER => (),
}
```

and an inverted filter defined as:

```ignore (pseudo-code)
// for spans
match callsite_enabled() {
    ALWAYS => (),
    SOMETIMES => if !enabled() { on_span() },
    NEVER => on_span(),
}
// for events
match callsite_enabled() {
    ALWAYS => (),
    SOMETIMES => if !enabled() { on_event() },
    NEVER => on_event(),
}
```

A proper inversion would do `!(enabled() && event_enabled())` (or
`!enabled() || !event_enabled()`), but because of the implicit `&&`
relation between `enabled` and `event_enabled`, it is difficult to
short circuit and not call the wrapped `event_enabled`.

A combinator which remembers the result of `enabled` in order to call
`event_enabled` only when `enabled() == true` is possible, but requires
additional thread-local mutable state to support a very niche use case.

[`Filter`]: crate::layer::Filter
[`enabled`]: crate::layer::Filter::enabled
[`event_enabled`]: crate::layer::Filter::event_enabled
[`callsite_enabled`]: crate::layer::Filter::callsite_enabled

<a id="op-01aa9ef8b9fa3ea3145f705d"></a>
## or

`function` · `tracing_subscriber::filter::layer_filters::FilterExt::or` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn or<B>(self, other: B) -> combinator::Or<Self, B, S> where Self: Sized, B: layer::Filter<S>
```

Source: `src/filter/layer_filters/mod.rs:306`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Combines two [`Filter`]s so that spans and events are enabled if *either* filter
returns `true`.

# Examples

Enabling spans and events at the `INFO` level and above, and all spans
and events with a particular target:
```
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter, FilterExt},
    prelude::*,
};

// Enables spans and events with targets starting with `interesting_target`:
let target_filter = filter_fn(|meta| {
    meta.target().starts_with("interesting_target")
});

// Enables spans and events with levels `INFO` and below:
let level_filter = LevelFilter::INFO;

// Combine the two filters together so that a span or event is enabled
// if it is at INFO or lower, or if it has a target starting with
// `interesting_target`.
let filter = level_filter.or(target_filter);

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_filter(filter))
    .init();

// This event will *not* be enabled:
tracing::debug!("an uninteresting event");

// This event *will* be enabled:
tracing::info!("an uninteresting INFO event");

// This event *will* be enabled:
tracing::info!(target: "interesting_target", "a very interesting event");

// This event *will* be enabled:
tracing::debug!(target: "interesting_target", "interesting debug event...");
```

Enabling a higher level for a particular target by using `or` in
conjunction with the [`and`] combinator:

```
use tracing_subscriber::{
    filter::{filter_fn, LevelFilter, FilterExt},
    prelude::*,
};

// This filter will enable spans and events with targets beginning with
// `my_crate`:
let my_crate = filter_fn(|meta| {
    meta.target().starts_with("my_crate")
});

let filter = my_crate
    // Combine the `my_crate` filter with a `LevelFilter` to produce a
    // filter that will enable the `INFO` level and lower for spans and
    // events with `my_crate` targets:
    .and(LevelFilter::INFO)
    // If a span or event *doesn't* have a target beginning with
    // `my_crate`, enable it if it has the `WARN` level or lower:
    .or(LevelFilter::WARN);

tracing_subscriber::registry()
    .with(tracing_subscriber::fmt::layer().with_filter(filter))
    .init();
```

[`Filter`]: crate::layer::Filter
[`and`]: FilterExt::and
