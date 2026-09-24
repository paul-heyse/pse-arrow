# `tracing_subscriber::registry::SpanData`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.SpanData.json).

<a id="op-7713a4015fe2313bb70953e1"></a>
## SpanData

`trait` · `tracing_subscriber::registry::SpanData` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait SpanData<'a>
```

Source: `src/registry/mod.rs:157`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A stored representation of data associated with a span.

<a id="op-2862fcdd8b558cd07a17983e"></a>
## extensions

`function` · `tracing_subscriber::registry::SpanData::extensions` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn extensions(&self) -> Extensions<'_>
```

Source: `src/registry/mod.rs:173`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a reference to this span's `Extensions`.

The extensions may be used by `Layer`s to store additional data
describing the span.

<a id="op-4c89eee0b26582209458beef"></a>
## extensions_mut

`function` · `tracing_subscriber::registry::SpanData::extensions_mut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn extensions_mut(&self) -> ExtensionsMut<'_>
```

Source: `src/registry/mod.rs:181`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a mutable reference to this span's `Extensions`.

The extensions may be used by `Layer`s to store additional data
describing the span.

<a id="op-6e145e2abdc91249b7110aa4"></a>
## id

`function` · `tracing_subscriber::registry::SpanData::id` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn id(&self) -> Id
```

Source: `src/registry/mod.rs:159`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns this span's ID.

<a id="op-b502a5dffc3339bb1011f9f1"></a>
## is_enabled_for

`function` · `tracing_subscriber::registry::SpanData::is_enabled_for` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn is_enabled_for(&self, filter: FilterId) -> bool
```

Source: `src/registry/mod.rs:195`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns `true` if this span is enabled for the [per-layer filter][plf]
corresponding to the provided [`FilterId`].

## Default Implementation

By default, this method assumes that the [`LookupSpan`](../operations/tracing_subscriber.registry.LookupSpan.md#op-d0bf19fd7852e1b6f08511d9) implementation
does not support [per-layer filtering][plf], and always returns `true`.

[plf]: crate::layer::Layer#per-layer-filtering
[`FilterId`]: crate::filter::FilterId

<a id="op-237f539fb0764cfd4ecf32f7"></a>
## metadata

`function` · `tracing_subscriber::registry::SpanData::metadata` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> &'static Metadata<'static>
```

Source: `src/registry/mod.rs:162`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a reference to the span's `Metadata`.

<a id="op-c297161eb4ddb01474724bfb"></a>
## parent

`function` · `tracing_subscriber::registry::SpanData::parent` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn parent(&self) -> Option<&Id>
```

Source: `src/registry/mod.rs:165`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a reference to the ID
