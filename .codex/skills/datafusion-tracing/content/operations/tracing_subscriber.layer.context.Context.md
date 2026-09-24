# `tracing_subscriber::layer::context::Context`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.layer.context.Context.json).

<a id="op-0acb651e530d235188552e7d"></a>
## Context

`struct` · `tracing_subscriber::layer::context::Context` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Context<'a, S>
```

Source: `src/layer/context.rs:33`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Represents information about the current context provided to [`Layer`]s by the
wrapped [`Subscriber`].

To access [stored data] keyed by a span ID, implementors of the `Layer`
trait should ensure that the `Subscriber` type parameter is *also* bound by the
[`LookupSpan`]:

```rust
use tracing::Subscriber;
use tracing_subscriber::{Layer, registry::LookupSpan};

pub struct MyLayer;

impl<S> Layer<S> for MyLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // ...
}
```

[`Layer`]: super::Layer
[`Subscriber`]: tracing_core::Subscriber
[stored data]: crate::registry::SpanRef
[`LookupSpan`]: crate::registry::LookupSpan

<a id="op-ef029797b53c3c0afaae7d34"></a>
## clone

`function` · `tracing_subscriber::layer::context::Context::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [422, 1], "end": [433, 2], "filename": "src/layer/context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/layer/context.rs:424`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-16999205c06719a6d29c8335"></a>
## current_span

`function` · `tracing_subscriber::layer::context::Context::current_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn current_span(&self) -> span::Current
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:66`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the wrapped subscriber's view of the current span.

<a id="op-be7f9f4a1610d3061e22d817"></a>
## enabled

`function` · `tracing_subscriber::layer::context::Context::enabled` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:76`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns whether the wrapped subscriber would enable the current span.

<a id="op-c274004bdaeeac1a992cb490"></a>
## event

`function` · `tracing_subscriber::layer::context::Context::event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event(&self, event: &Event<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:106`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Records the provided `event` with the wrapped subscriber.

# Notes

- The subscriber is free to expect that the event's callsite has been
  [registered][register], and may panic or fail to observe the event if this is
  not the case. The `tracing` crate's macros ensure that all events are
  registered, but if the event is constructed through other means, the
  user is responsible for ensuring that [`register_callsite`][register]
  has been called prior to calling this method.
- This does _not_ call [`enabled`] on the inner subscriber. If the
  caller wishes to apply the wrapped subscriber's filter before choosing
  whether to record the event, it may first call [`Context::enabled`] to
  check whether the event would be enabled. This allows `Layer`s to
  elide constructing the event if it would not be recorded.

[register]: tracing_core::subscriber::Subscriber::register_callsite()
[`enabled`]: tracing_core::subscriber::Subscriber::enabled()
[`Context::enabled`]: Context::enabled()

Unresolved upstream links (retained, not inferred): `tracing_core::subscriber::Subscriber::enabled()`, `tracing_core::subscriber::Subscriber::register_callsite()`.

<a id="op-f08e7c9a674253fafb02ad26"></a>
## event_scope

`function` · `tracing_subscriber::layer::context::Context::event_scope` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_scope(&self, event: &Event<'_>) -> Option<registry::Scope<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:363`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an iterator over the [stored data] for all the spans in the
current context, starting with the parent span of the specified event,
and ending with the root of the trace tree and ending with the current span.

<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: Compared to <a href="#method.scope"><code>scope</code></a> this
returns the spans in reverse order (from leaf to root). Use
<a href="../registry/struct.Scope.html#method.from_root"><code>Scope::from_root</code></a>
in case root-to-leaf ordering is desired.
</pre>

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This requires the wrapped subscriber to
    implement the <a href="../registry/trait.LookupSpan.html"><code>
    LookupSpan</code></a> trait. See the documentation on
    <a href="./struct.Context.html"><code>Context</code>'s
    declaration</a> for details.
</pre>

[stored data]: crate::registry::SpanRef

<a id="op-4f2ca7c8c5afeda79eab4db7"></a>
## event_span

`function` · `tracing_subscriber::layer::context::Context::event_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_span(&self, event: &Event<'_>) -> Option<SpanRef<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:169`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a [`SpanRef`](../operations/tracing_subscriber.registry.SpanRef.md#op-c46893bde415baf0f5119653) for the parent span of the given [`Event`](../operations/tracing_core.event.Event.md#op-7ee85389e31294d1a098f039), if
it has a parent.

If the event has an explicitly overridden parent, this method returns
a reference to that span. If the event's parent is the current span,
this returns a reference to the current span, if there is one. If this
returns `None`, then either the event's parent was explicitly set to
`None`, or the event's parent was defined contextually, but no span
is currently entered.

Compared to [`Context::current_span`](../operations/tracing_subscriber.layer.context.Context.md#op-16999205c06719a6d29c8335) and [`Context::lookup_current`](../operations/tracing_subscriber.layer.context.Context.md#op-279b784874fc32c9c94c32fc),
this respects overrides provided by the [`Event`](../operations/tracing_core.event.Event.md#op-7ee85389e31294d1a098f039).

Compared to [`Event::parent`], this automatically falls back to the contextual
span, if required.

```rust
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    layer::{Context, Layer},
    prelude::*,
    registry::LookupSpan,
};

struct PrintingLayer;
impl<S> Layer<S> for PrintingLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_event(&self, event: &Event, ctx: Context<S>) {
        let span = ctx.event_span(event);
        println!("Event in span: {:?}", span.map(|s| s.name()));
    }
}

tracing::subscriber::with_default(tracing_subscriber::registry().with(PrintingLayer), || {
    tracing::info!("no span");
    // Prints: Event in span: None

    let span = tracing::info_span!("span");
    tracing::info!(parent: &span, "explicitly specified");
    // Prints: Event in span: Some("span")

    let _guard = span.enter();
    tracing::info!("contextual span");
    // Prints: Event in span: Some("span")
});
```

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This requires the wrapped subscriber to
    implement the <a href="../registry/trait.LookupSpan.html"><code>
    LookupSpan</code></a> trait. See the documentation on
    <a href="./struct.Context.html"><code>Context</code>'s
    declaration</a> for details.
</pre>

Unresolved upstream links (retained, not inferred): ``Event::parent``.

<a id="op-a0c31dc230cbb7691c7a0851"></a>
## exists

`function` · `tracing_subscriber::layer::context::Context::exists` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn exists(&self, id: &span::Id) -> bool where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:234`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if an active span exists for the given `Id`.

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This requires the wrapped subscriber to
    implement the <a href="../registry/trait.LookupSpan.html"><code>
    LookupSpan</code></a> trait. See the documentation on
    <a href="./struct.Context.html"><code>Context</code>'s
    declaration</a> for details.
</pre>

<a id="op-904f7708110bfc3519260748"></a>
## fmt

`function` · `tracing_subscriber::layer::context::Context::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [32, 10], "end": [32, 15], "filename": "src/layer/context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/layer/context.rs:32`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-279b784874fc32c9c94c32fc"></a>
## lookup_current

`function` · `tracing_subscriber::layer::context::Context::lookup_current` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn lookup_current(&self) -> Option<registry::SpanRef<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:256`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns [stored data] for the span that the wrapped subscriber considers
to be the current.

If this returns `None`, then we are not currently within a span.

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This requires the wrapped subscriber to
    implement the <a href="../registry/trait.LookupSpan.html"><code>
    LookupSpan</code></a> trait. See the documentation on
    <a href="./struct.Context.html"><code>Context</code>'s
    declaration</a> for details.
</pre>

[stored data]: crate::registry::SpanRef

<a id="op-4421ec50bcb2dfe38e513821"></a>
## metadata

`function` · `tracing_subscriber::layer::context::Context::metadata` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self, id: &span::Id) -> Option<&'static Metadata<'static>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:188`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns metadata for the span with the given `id`, if it exists.

If this returns `None`, then no span exists for that ID (either it has
closed or the ID is invalid).

<a id="op-c1d9008b2af47e4fcc66e76e"></a>
## span

`function` · `tracing_subscriber::layer::context::Context::span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn span(&self, id: &span::Id) -> Option<registry::SpanRef<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:211`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns [stored data] for the span with the given `id`, if it exists.

If this returns `None`, then no span exists for that ID (either it has
closed or the ID is invalid).

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This requires the wrapped subscriber to
    implement the <a href="../registry/trait.LookupSpan.html"><code>
    LookupSpan</code></a> trait. See the documentation on
    <a href="./struct.Context.html"><code>Context</code>'s
    declaration</a> for details.
</pre>

[stored data]: crate::registry::SpanRef

<a id="op-c6d804051c33f80e2c60a97a"></a>
## span_scope

`function` · `tracing_subscriber::layer::context::Context::span_scope` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn span_scope(&self, id: &span::Id) -> Option<registry::Scope<'_, S>> where S: for<'lookup> LookupSpan<'lookup>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::context::Context", "path": "Context"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [51, 1], "end": [409, 2], "filename": "src/layer/context.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer/context.rs:336`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an iterator over the [stored data] for all the spans in the
current context, starting with the specified span and ending with the
root of the trace tree.

<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: This returns the spans in reverse order (from leaf to root). Use
<a href="../registry/struct.Scope.html#method.from_root"><code>Scope::from_root</code></a>
in case root-to-leaf ordering is desired.
</pre>

<pre class="ignore" style="white-space:normal;font:inherit;">
    <strong>Note</strong>: This requires the wrapped subscriber to
    implement the <a href="../registry/trait.LookupSpan.html"><code>
    LookupSpan</code></a> trait. See the documentation on
    <a href="./struct.Context.html"><code>Context</code>'s
    declaration</a> for details.
</pre>

[stored data]: crate::registry::SpanRef
