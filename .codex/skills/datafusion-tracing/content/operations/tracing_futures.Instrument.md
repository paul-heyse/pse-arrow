# `tracing_futures::Instrument`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_futures.Instrument.json).

<a id="op-0e287983f5ef2664ae58a1ef"></a>
## Instrument

`trait` · `tracing_futures::Instrument` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
trait Instrument: Sized
```

Source: `src/lib.rs:125`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Extension trait allowing futures, streams, sinks, and executors to be
instrumented with a `tracing` [span].

[span]: https://docs.rs/tracing/latest/tracing/span/index.html

<a id="op-a27c328cf35388b63281f4ff"></a>
## in_current_span

`function` · `tracing_futures::Instrument::in_current_span` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn in_current_span(self) -> Instrumented<Self>
```

Source: `src/lib.rs:191`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Instruments this type with the [current] `Span`, returning an
`Instrumented` wrapper.

If the instrumented type is a future, stream, or sink, the attached `Span`
will be [entered] every time it is polled. If the instrumented type
is a future executor, every future spawned on that executor will be
instrumented by the attached `Span`.

This can be used to propagate the current span when spawning a new future.

# Examples

```rust,ignore
use tracing_futures::Instrument;

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

[current]: https://docs.rs/tracing/latest/tracing/span/struct.Span.html#method.current
[entered]: https://docs.rs/tracing/latest/tracing/span/struct.Span.html#method.enter

<a id="op-020485ba53fd6a5efaf57584"></a>
## instrument

`function` · `tracing_futures::Instrument::instrument` · tracing-futures 0.2.5
Reachability: `supported`.  Capture: hosted.

```rust
fn instrument(self, span: Span) -> Instrumented<Self>
```

Source: `src/lib.rs:154`. [Exact documentation build](https://docs.rs/crate/tracing-futures/0.2.5/json).

Instruments this type with the provided `Span`, returning an
`Instrumented` wrapper.

If the instrumented type is a future, stream, or sink, the attached `Span`
will be [entered] every time it is polled. If the instrumented type
is a future executor, every future spawned on that executor will be
instrumented by the attached `Span`.

# Examples

Instrumenting a future:

```rust,ignore
use tracing_futures::Instrument;

# async fn doc() {
let my_future = async {
    // ...
};

my_future
    .instrument(tracing::info_span!("my_future"))
    .await
# }
```

[entered]: https://docs.rs/tracing/latest/tracing/span/struct.Span.html#method.enter
