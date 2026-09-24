# `tracing_core::dispatcher::Dispatch`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.dispatcher.Dispatch.json).

<a id="op-bdcb8c4598cc406ba313069c"></a>
## Dispatch

`struct` · `tracing_core::dispatcher::Dispatch` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Dispatch
```

Source: `src/dispatcher.rs:149`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

`Dispatch` trace data to a [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c).

<a id="op-5e5dbdadc661fed8b9bc18a3"></a>
## clone

`function` · `tracing_core::dispatcher::Dispatch::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Dispatch
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [148, 10], "end": [148, 15], "filename": "src/dispatcher.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/dispatcher.rs:148`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aedbb7574b6dc95422a134c9"></a>
## clone_span

`function` · `tracing_core::dispatcher::Dispatch::clone_span` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone_span(&self, id: &span::Id) -> span::Id
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:651`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Notifies the subscriber that a [span ID] has been cloned.

This function must only be called with span IDs that were returned by
this `Dispatch`'s [`new_span`] function. The `tracing` crate upholds
this guarantee and any other libraries implementing instrumentation APIs
must as well.

This calls the [`clone_span`] function on the `Subscriber` that this
`Dispatch` forwards to.

[span ID]: super::span::Id
[`Subscriber`]: super::subscriber::Subscriber
[`clone_span`]: super::subscriber::Subscriber::clone_span
[`new_span`]: super::subscriber::Subscriber::new_span

<a id="op-96c9a6d2832b8129d2418a01"></a>
## current_span

`function` · `tracing_core::dispatcher::Dispatch::current_span` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn current_span(&self) -> span::Current
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:710`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a type representing this subscriber's view of the current span.

This calls the [`current`] function on the `Subscriber` that this
`Dispatch` forwards to.

[`current`]: super::subscriber::Subscriber::current_span

<a id="op-ad2efd9f11d503cd00c949ca"></a>
## default

`function` · `tracing_core::dispatcher::Dispatch::default` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [729, 1], "end": [734, 2], "filename": "src/dispatcher.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/dispatcher.rs:731`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the current default dispatcher

<a id="op-890a82577d6562a4cd4f6ccf"></a>
## downcast_ref

`function` · `tracing_core::dispatcher::Dispatch::downcast_ref` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn downcast_ref<T: Any>(&self) -> Option<&T>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:724`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns some reference to the `Subscriber` this `Dispatch` forwards to
if it is of type `T`, or `None` if it isn't.

<a id="op-3d911653bdde5f29cad4eafd"></a>
## downgrade

`function` · `tracing_core::dispatcher::Dispatch::downgrade` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn downgrade(&self) -> WeakDispatch
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:502`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Creates a [`WeakDispatch`](../operations/tracing_core.dispatcher.WeakDispatch.md#op-0beab352f71fbbf38e94b386) from this `Dispatch`.

A [`WeakDispatch`](../operations/tracing_core.dispatcher.WeakDispatch.md#op-0beab352f71fbbf38e94b386) is similar to a [`Dispatch`](../operations/tracing_core.dispatcher.Dispatch.md#op-bdcb8c4598cc406ba313069c), but it does not prevent
the underlying [`Subscriber`](../operations/tracing_core.subscriber.Subscriber.md#op-d03861aa726092c9d924061c) from being dropped. Instead, it only permits
access while other references to the `Subscriber` exist. This is equivalent
to the standard library's [`Arc::downgrade`] method, but for `Dispatch`
rather than `Arc`.

The primary use for creating a [`WeakDispatch`](../operations/tracing_core.dispatcher.WeakDispatch.md#op-0beab352f71fbbf38e94b386) is to allow a `Subscriber`
to hold a cyclical reference to itself without creating a memory leak.
See [here] for details.

[`Arc::downgrade`]: std::sync::Arc::downgrade
[here]: Subscriber#avoiding-memory-leaks

Unresolved upstream links (retained, not inferred): `std::sync::Arc::downgrade`.

<a id="op-46686383443d79747d3a330e"></a>
## drop_span

`function` · `tracing_core::dispatcher::Dispatch::drop_span` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn drop_span(&self, id: span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:679`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Notifies the subscriber that a [span ID] has been dropped.

This function must only be called with span IDs that were returned by
this `Dispatch`'s [`new_span`] function. The `tracing` crate upholds
this guarantee and any other libraries implementing instrumentation APIs
must as well.

This calls the [`drop_span`] function on the [`Subscriber`] that this
`Dispatch` forwards to.

<pre class="compile_fail" style="white-space:normal;font:inherit;">
    <strong>Deprecated</strong>: The <a href="#method.try_close"><code>
    try_close</code></a> method is functionally identical, but returns
    <code>true</code> if the span is now closed. It should be used
    instead of this method.
</pre>

[span ID]: super::span::Id
[`Subscriber`]: super::subscriber::Subscriber
[`drop_span`]: super::subscriber::Subscriber::drop_span
[`new_span`]: super::subscriber::Subscriber::new_span
[`try_close`]: Self::try_close()

<a id="op-59cc1becf3a5c0e11ed4fc0c"></a>
## enabled

`function` · `tracing_core::dispatcher::Dispatch::enabled` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:594`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns true if a span with the specified [metadata] would be
recorded.

This calls the [`enabled`] function on the [`Subscriber`] that this
`Dispatch` forwards to.

[metadata]: super::metadata::Metadata
[`Subscriber`]: super::subscriber::Subscriber
[`enabled`]: super::subscriber::Subscriber::enabled

<a id="op-2ebbe3198bcfd3842199b1e1"></a>
## enter

`function` · `tracing_core::dispatcher::Dispatch::enter` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn enter(&self, span: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:621`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Records that a span has been can_enter.

This calls the [`enter`] function on the [`Subscriber`] that this
`Dispatch` forwards to.

[`Subscriber`]: super::subscriber::Subscriber
[`enter`]: super::subscriber::Subscriber::enter

<a id="op-7c17283aaed18117f8dac913"></a>
## event

`function` · `tracing_core::dispatcher::Dispatch::event` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn event(&self, event: &Event<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:607`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Records that an [`Event`] has occurred.

This calls the [`event`] function on the [`Subscriber`] that this
`Dispatch` forwards to.

[`Event`]: super::event::Event
[`Subscriber`]: super::subscriber::Subscriber
[`event`]: super::subscriber::Subscriber::event

<a id="op-59f7ef13722e3e2e02a03244"></a>
## exit

`function` · `tracing_core::dispatcher::Dispatch::exit` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn exit(&self, span: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:632`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Records that a span has been exited.

This calls the [`exit`] function on the [`Subscriber`] that this
`Dispatch` forwards to.

[`Subscriber`]: super::subscriber::Subscriber
[`exit`]: super::subscriber::Subscriber::exit

<a id="op-9e9eb2062c3fcb1dd44a413a"></a>
## fmt

`function` · `tracing_core::dispatcher::Dispatch::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [736, 1], "end": [749, 2], "filename": "src/dispatcher.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/dispatcher.rs:737`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7675f939fc207f1484da2186"></a>
## from

`function` · `tracing_core::dispatcher::Dispatch::from` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn from(subscriber: S) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [751, 1], "end": [759, 2], "filename": "src/dispatcher.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/dispatcher.rs:756`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c77c6b1b94d1951ab1f892c5"></a>
## is

`function` · `tracing_core::dispatcher::Dispatch::is` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is<T: Any>(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:717`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if this `Dispatch` forwards to a `Subscriber` of type
`T`.

<a id="op-f05d362d48014c7fcd740d82"></a>
## new

`function` · `tracing_core::dispatcher::Dispatch::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new<S>(subscriber: S) -> Self where S: Subscriber + Send + Sync + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:472`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a `Dispatch` that forwards to the given [`Subscriber`].

[`Subscriber`]: super::subscriber::Subscriber

<a id="op-182ced619ada82e85e23e4d4"></a>
## new_span

`function` · `tracing_core::dispatcher::Dispatch::new_span` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new_span(&self, span: &span::Attributes<'_>) -> span::Id
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:555`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Record the construction of a new span, returning a new [ID] for the
span being constructed.

This calls the [`new_span`] function on the [`Subscriber`] that this
`Dispatch` forwards to.

[ID]: super::span::Id
[`Subscriber`]: super::subscriber::Subscriber
[`new_span`]: super::subscriber::Subscriber::new_span

<a id="op-32d91e13873d14cc5ba6b3e0"></a>
## none

`function` · `tracing_core::dispatcher::Dispatch::none` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn none() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:463`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a new `Dispatch` that discards events and spans.

<a id="op-7f467d377959335c1f2d3c71"></a>
## record

`function` · `tracing_core::dispatcher::Dispatch::record` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record(&self, span: &span::Id, values: &span::Record<'_>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:567`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Record a set of values on a span.

This calls the [`record`] function on the [`Subscriber`] that this
`Dispatch` forwards to.

[`Subscriber`]: super::subscriber::Subscriber
[`record`]: super::subscriber::Subscriber::record

<a id="op-45f811dcf08f771ac4ccfaf8"></a>
## record_follows_from

`function` · `tracing_core::dispatcher::Dispatch::record_follows_from` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn record_follows_from(&self, span: &span::Id, follows: &span::Id)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:580`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Adds an indication that `span` follows from the span with the id
`follows`.

This calls the [`record_follows_from`] function on the [`Subscriber`]
that this `Dispatch` forwards to.

[`Subscriber`]: super::subscriber::Subscriber
[`record_follows_from`]: super::subscriber::Subscriber::record_follows_from

<a id="op-75d71ac96910299a244e6ddb"></a>
## register_callsite

`function` · `tracing_core::dispatcher::Dispatch::register_callsite` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> subscriber::Interest
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:525`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Registers a new callsite with this subscriber, returning whether or not
the subscriber is interested in being notified about the callsite.

This calls the [`register_callsite`] function on the [`Subscriber`]
that this `Dispatch` forwards to.

[`Subscriber`]: super::subscriber::Subscriber
[`register_callsite`]: super::subscriber::Subscriber::register_callsite

<a id="op-e45eeed45149055271d4be8a"></a>
## try_close

`function` · `tracing_core::dispatcher::Dispatch::try_close` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn try_close(&self, id: span::Id) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::dispatcher::Dispatch", "path": "Dispatch"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [460, 1], "end": [727, 2], "filename": "src/dispatcher.rs"}, "trait": null, "trait_path": null}`

Source: `src/dispatcher.rs:699`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Notifies the subscriber that a [span ID] has been dropped, and returns
`true` if there are now 0 IDs referring to that span.

This function must only be called with span IDs that were returned by
this `Dispatch`'s [`new_span`] function. The `tracing` crate upholds
this guarantee and any other libraries implementing instrumentation APIs
must as well.

This calls the [`try_close`] function on the [`Subscriber`] that this
 `Dispatch` forwards to.

[span ID]: super::span::Id
[`Subscriber`]: super::subscriber::Subscriber
[`try_close`]: super::subscriber::Subscriber::try_close
[`new_span`]: super::subscriber::Subscriber::new_span
