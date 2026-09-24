# `tracing::instrument::Instrument`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.instrument.Instrument.json).

<a id="op-f3255b14b518b17fdbec7041"></a>
## Instrument

`trait` · `tracing::instrument::Instrument` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
trait Instrument: Sized
```

Source: `src/instrument.rs:20`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Attaches spans to a [`std::future::Future`].

Extension trait allowing futures to be
instrumented with a `tracing` [span].

[span]: super::Span

Unresolved upstream links (retained, not inferred): ``std::future::Future``.

<a id="op-6c17bae75c8707a1c5840700"></a>
## in_current_span

`function` · `tracing::instrument::Instrument::in_current_span` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn in_current_span(self) -> Instrumented<Self>
```

Source: `src/instrument.rs:128`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Instruments this type with the [current] [`Span`], returning an
`Instrumented` wrapper.

The attached [`Span`] will be [entered] every time the instrumented
[`Future`] is polled or [`Drop`]ped.

This can be used to propagate the current span when spawning a new future.

# Examples

```rust
use tracing::Instrument;

# mod tokio {
#     pub(super) fn spawn(_: impl std::future::Future) {}
# }
# async fn doc() {
let span = tracing::info_span!("my_span");
let _enter = span.enter();

// ...

let future = async {
    tracing::debug!("this event will occur inside `my_span`");
    // ...
};
tokio::spawn(future.in_current_span());
# }
```

[current]: super::Span::current()
[entered]: super::Span::enter()
[`Span`]: crate::Span
[`Future`]: std::future::Future

Unresolved upstream links (retained, not inferred): ``Drop``, `std::future::Future`.

<a id="op-07b9b2ea22c2f17a5b513497"></a>
## instrument

`function` · `tracing::instrument::Instrument::instrument` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn instrument(self, span: Span) -> Instrumented<Self>
```

Source: `src/instrument.rs:86`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Instruments this type with the provided [`Span`](../operations/tracing.span.Span.md#op-1283b08586edf8ae3649f1c6), returning an
`Instrumented` wrapper.

The attached [`Span`](../operations/tracing.span.Span.md#op-1283b08586edf8ae3649f1c6) will be [entered] every time the instrumented
[`Future`] is polled or [`Drop`]ped.

# Examples

Instrumenting a future:

```rust
use tracing::Instrument;

# async fn doc() {
let my_future = async {
    // ...
};

my_future
    .instrument(tracing::info_span!("my_future"))
    .await
# }
```

The [`Span::or_current`] combinator can be used in combination with
`instrument` to ensure that the [current span] is attached to the
future if the span passed to `instrument` is [disabled]:

```
use tracing::Instrument;
# mod tokio {
#     pub(super) fn spawn(_: impl std::future::Future) {}
# }

let my_future = async {
    // ...
};

let outer_span = tracing::info_span!("outer").entered();

// If the "my_future" span is enabled, then the spawned task will
// be within both "my_future" *and* "outer", since "outer" is
// "my_future"'s parent. However, if "my_future" is disabled,
// the spawned task will *not* be in any span.
tokio::spawn(
    my_future
        .instrument(tracing::debug_span!("my_future"))
);

// Using `Span::or_current` ensures the spawned task is instrumented
// with the current span, if the new span passed to `instrument` is
// not enabled. This means that if the "my_future"  span is disabled,
// the spawned task will still be instrumented with the "outer" span:
# let my_future = async {};
tokio::spawn(
   my_future
        .instrument(tracing::debug_span!("my_future").or_current())
);
```

[entered]: super::Span::enter()
[`Span::or_current`]: super::Span::or_current()
[current span]: super::Span::current()
[disabled]: super::Span::is_disabled()
[`Future`]: std::future::Future

Unresolved upstream links (retained, not inferred): ``Drop``, `std::future::Future`.
