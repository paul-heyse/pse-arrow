# `tracing_subscriber::filter::layer_filters::FilterId`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.layer_filters.FilterId.json).

<a id="op-ca4e7220569d32cbf2e6ed55"></a>
## FilterId

`struct` · `tracing_subscriber::filter::layer_filters::FilterId` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct FilterId
```

Source: `src/filter/layer_filters/mod.rs:90`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Uniquely identifies an individual [`Filter`] instance in the context of
a [`Subscriber`].

When adding a [`Filtered`](../operations/tracing_subscriber.filter.layer_filters.Filtered.md#op-54bb3c78606034b0e2ea655e) [`Layer`] to a [`Subscriber`], the [`Subscriber`]
generates a `FilterId` for that [`Filtered`](../operations/tracing_subscriber.filter.layer_filters.Filtered.md#op-54bb3c78606034b0e2ea655e) layer. The [`Filtered`](../operations/tracing_subscriber.filter.layer_filters.Filtered.md#op-54bb3c78606034b0e2ea655e) layer
will then use the generated ID to query whether a particular span was
previously enabled by that layer's [`Filter`].

**Note**: Currently, the [`Registry`] type provided by this crate is the
**only** [`Subscriber`] implementation capable of participating in per-layer
filtering. Therefore, the `FilterId` type cannot currently be constructed by
code outside of `tracing-subscriber`. In the future, new APIs will be added to `tracing-subscriber` to
allow non-Registry [`Subscriber`]s to also participate in per-layer
filtering. When those APIs are added, subscribers will be responsible
for generating and assigning `FilterId`s.

[`Filter`]: crate::layer::Filter
[`Subscriber`]: tracing_core::Subscriber
[`Layer`]: crate::layer::Layer
[`Registry`]: crate::registry::Registry

<a id="op-42405a7becfd0b7516859e29"></a>
## clone

`function` · `tracing_subscriber::filter::layer_filters::FilterId::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> FilterId
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::layer_filters::FilterId", "path": "FilterId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [89, 16], "end": [89, 21], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/filter/layer_filters/mod.rs:89`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ae8d39683c9bb8e348f189d7"></a>
## fmt

`function` · `tracing_subscriber::filter::layer_filters::FilterId::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::layer_filters::FilterId", "path": "FilterId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1031, 1], "end": [1037, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Binary", "path": "Binary"}, "trait_path": "core::fmt::Binary"}`

Source: `src/filter/layer_filters/mod.rs:1032`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e684ca65fabdc73c9f3e1867"></a>
## fmt

`function` · `tracing_subscriber::filter::layer_filters::FilterId::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::layer_filters::FilterId", "path": "FilterId"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1010, 1], "end": [1029, 2], "filename": "src/filter/layer_filters/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/layer_filters/mod.rs:1011`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
