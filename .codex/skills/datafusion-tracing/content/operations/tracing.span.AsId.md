# `tracing::span::AsId`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.span.AsId.json).

<a id="op-d3026856b0e9429394e836be"></a>
## AsId

`trait` · `tracing::span::AsId` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
trait AsId: sealed::Sealed
```

Source: `src/span.rs:336`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Trait implemented by types which have a span `Id`.

<a id="op-d9004ce80958635288f061bf"></a>
## as_id

`function` · `tracing::span::AsId::as_id` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn as_id(&self) -> Option<&Id>
```

Source: `src/span.rs:339`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns the `Id` of the span that `self` corresponds to, or `None` if
this corresponds to a disabled span.
