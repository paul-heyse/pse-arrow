# `tracing_subscriber::registry::SpanRef`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.registry.SpanRef.json).

<a id="op-c46893bde415baf0f5119653"></a>
## SpanRef

`struct` · `tracing_subscriber::registry::SpanRef` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct SpanRef<'a, R: LookupSpan<'a>>
```

Source: `src/registry/mod.rs:208`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A reference to [span data] and the associated [registry].

This type implements all the same methods as [`SpanData`](../operations/tracing_subscriber.registry.SpanData.md#op-7713a4015fe2313bb70953e1), and provides
additional methods for querying the registry based on values from the span.

[registry]: LookupSpan

<a id="op-562d9f00f28758652bf9acf0"></a>
## extensions

`function` · `tracing_subscriber::registry::SpanRef::extensions` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn extensions(&self) -> Extensions<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:479`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a reference to this span's `Extensions`.

The extensions may be used by `Layer`s to store additional data
describing the span.

<a id="op-f6d170a110a0134e5c2fca52"></a>
## extensions_mut

`function` · `tracing_subscriber::registry::SpanRef::extensions_mut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn extensions_mut(&self) -> ExtensionsMut<'_>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:489`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a mutable reference to this span's `Extensions`.

The extensions may be used by `Layer`s to store additional data
describing the span.

<a id="op-2c1c25ac3b6d39975d7c0913"></a>
## fields

`function` · `tracing_subscriber::registry::SpanRef::fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fields(&self) -> &FieldSet
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:358`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a list of [fields] defined by the span.

[fields]: tracing_core::field

<a id="op-d28fffe533fce553b539c189"></a>
## fmt

`function` · `tracing_subscriber::registry::SpanRef::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Data", "self_type": {"generic": "R"}, "trait": {"args": null, "id": "tracing_subscriber::registry::LookupSpan", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [207, 10], "end": [207, 15], "filename": "src/registry/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/registry/mod.rs:207`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-30b251712d3fb2048aede071"></a>
## id

`function` · `tracing_subscriber::registry::SpanRef::id` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn id(&self) -> Id
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:341`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns this span's ID.

<a id="op-1c90b04cfa3180b82131db6e"></a>
## metadata

`function` · `tracing_subscriber::registry::SpanRef::metadata` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> &'static Metadata<'static>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:346`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a static reference to the span's metadata.

<a id="op-26c10b15a56a3ffa5b33aa1e"></a>
## name

`function` · `tracing_subscriber::registry::SpanRef::name` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn name(&self) -> &'static str
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:351`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns the span's name,

<a id="op-af50ea26eb69ccfe3ad6d46c"></a>
## parent

`function` · `tracing_subscriber::registry::SpanRef::parent` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn parent(&self) -> Option<Self>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:364`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a `SpanRef` describing this span's parent, or `None` if this
span is the root of its trace tree.

<a id="op-4eb435132dedc623cebfa2b3"></a>
## scope

`function` · `tracing_subscriber::registry::SpanRef::scope` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn scope(&self) -> Scope<'a, R>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}, {"type": {"generic": "R"}}], "constraints": []}}, "id": "tracing_subscriber::registry::SpanRef", "path": "SpanRef"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "R"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "R"}}}]}, "is_negative": false, "span": {"begin": [336, 1], "end": [513, 2], "filename": "src/registry/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/registry/mod.rs:463`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns an iterator over all parents of this span, starting with this span,
ordered from leaf to root.

The iterator will first return the span, then the span's immediate parent,
followed by that span's parent, and so on, until it reaches a root span.

```rust
use tracing::{span, Subscriber};
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
    fn on_enter(&self, id: &span::Id, ctx: Context<S>) {
        let span = ctx.span(id).unwrap();
        let scope = span.scope().map(|span| span.name()).collect::<Vec<_>>();
        println!("Entering span: {:?}", scope);
    }
}

tracing::subscriber::with_default(tracing_subscriber::registry().with(PrintingLayer), || {
    let _root = tracing::info_span!("root").entered();
    // Prints: Entering span: ["root"]
    let _child = tracing::info_span!("child").entered();
    // Prints: Entering span: ["child", "root"]
    let _leaf = tracing::info_span!("leaf").entered();
    // Prints: Entering span: ["leaf", "child", "root"]
});
```

If the opposite order (from the root to this span) is desired, calling [`Scope::from_root`](../operations/tracing_subscriber.registry.Scope.md#op-29cbc687573025f128a156cf) on
the returned iterator reverses the order.

```rust
# use tracing::{span, Subscriber};
# use tracing_subscriber::{
#     layer::{Context, Layer},
#     prelude::*,
#     registry::LookupSpan,
# };
# struct PrintingLayer;
impl<S> Layer<S> for PrintingLayer
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
{
    fn on_enter(&self, id: &span::Id, ctx: Context<S>) {
        let span = ctx.span(id).unwrap();
        let scope = span.scope().from_root().map(|span| span.name()).collect::<Vec<_>>();
        println!("Entering span: {:?}", scope);
    }
}

tracing::subscriber::with_default(tracing_subscriber::registry().with(PrintingLayer), || {
    let _root = tracing::info_span!("root").entered();
    // Prints: Entering span: ["root"]
    let _child = tracing::info_span!("child").entered();
    // Prints: Entering span: ["root", "child"]
    let _leaf = tracing::info_span!("leaf").entered();
    // Prints: Entering span: ["root", "child", "leaf"]
});
```
