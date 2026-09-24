# `tracing_subscriber::registry::LookupSpan`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.LookupSpan.json).

<a id="op-d0bf19fd7852e1b6f08511d9"></a>
## LookupSpan

`trait` · `tracing_subscriber::registry::LookupSpan` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait LookupSpan<'a>
```

Source: `src/registry/mod.rs:92`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Provides access to stored span data.

Subscribers which store span data and associate it with span IDs should
implement this trait; if they do, any [`Layer`]s wrapping them can look up
metadata via the [`Context`] type's [`span()`] method.

[`Layer`]: super::layer::Layer
[`Context`]: super::layer::Context
[`span()`]: super::layer::Context::span

<a id="op-b790df6599dac5a936f1a05d"></a>
## Data

`assoc_type` · `tracing_subscriber::registry::LookupSpan::Data` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Data
```

Source: `src/registry/mod.rs:94`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The type of span data stored in this registry.

<a id="op-f7d578d2c654c0b0c7053fe4"></a>
## register_filter

`function` · `tracing_subscriber::registry::LookupSpan::register_filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn register_filter(&mut self) -> FilterId
```

Source: `src/registry/mod.rs:148`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Registers a [`Filter`] for [per-layer filtering] with this
[`Subscriber`].

The [`Filter`] can then use the returned [`FilterId`] to
[check if it previously enabled a span][check].

# Panics

If this `Subscriber` does not support [per-layer filtering].

[`Filter`]: crate::layer::Filter
[per-layer filtering]: crate::layer::Layer#per-layer-filtering
[`Subscriber`]: tracing_core::Subscriber
[`FilterId`]: crate::filter::FilterId
[check]: SpanData::is_enabled_for

<a id="op-f849d7c991e9f3a0c5296f81"></a>
## span

`function` · `tracing_subscriber::registry::LookupSpan::span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn span(&'a self, id: &Id) -> Option<SpanRef<'a, Self>> where Self: Sized
```

Source: `src/registry/mod.rs:118`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a [`SpanRef`](../operations/tracing_subscriber.registry.SpanRef.md#op-c46893bde415baf0f5119653) for the span with the given `Id`, if it exists.

A `SpanRef` is similar to [`SpanData`](../operations/tracing_subscriber.registry.SpanData.md#op-7713a4015fe2313bb70953e1), but it allows performing
additional lookups against the registryr that stores the wrapped data.

In general, _users_ of the `LookupSpan` trait should use this method
rather than the [`span_data`] method; while _implementors_ of this trait
should only implement `span_data`.

[`span_data`]: LookupSpan::span_data()

<a id="op-e0781b8901806cf03d76dbd8"></a>
## span_data

`function` · `tracing_subscriber::registry::LookupSpan::span_data` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn span_data(&'a self, id: &Id) -> Option<Self::Data>
```

Source: `src/registry/mod.rs:106`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the [`SpanData`](../operations/tracing_subscriber.registry.SpanData.md#op-7713a4015fe2313bb70953e1) for a given `Id`, if it exists.

<pre class="ignore" style="white-space:normal;font:inherit;">
<strong>Note</strong>: users of the <code>LookupSpan</code> trait should
typically call the <a href="#method.span"><code>span</code></a> method rather
than this method. The <code>span</code> method is implemented by
<em>calling</em> <code>span_data</code>, but returns a reference which is
capable of performing more sophisiticated queries.
</pre>

