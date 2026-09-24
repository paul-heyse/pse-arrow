# `tracing_core::span::Current`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.span.Current.json).

<a id="op-564b71c6489a6752542300ae"></a>
## Current

`struct` · `tracing_core::span::Current` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Current
```

Source: `src/span.rs:46`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Indicates what [the `Subscriber` considers] the "current" span.

As subscribers may not track a notion of a current span, this has three
possible states:
- "unknown", indicating that the subscriber does not track a current span,
- "none", indicating that the current context is known to not be in a span,
- "some", with the current span's [`Id`](../operations/tracing_core.span.Id.md#op-a01c623911845c00e8650c4f) and [`Metadata`].

[the `Subscriber` considers]: super::subscriber::Subscriber::current_span
[`Metadata`]: super::metadata::Metadata

<a id="op-3a04b89fa55e3d10df74c842"></a>
## fmt

`function` · `tracing_core::span::Current::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::span::Current", "path": "Current"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 10], "end": [45, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:45`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-be3478e580494b5068410a76"></a>
## id

`function` · `tracing_core::span::Current::id` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn id(&self) -> Option<&Id>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::span::Current", "path": "Current"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [251, 1], "end": [316, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:302`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Borrows the `Id` of the current span, if one exists and is known.

<a id="op-aaab0d64a063d1b158f9b376"></a>
## into_inner

`function` · `tracing_core::span::Current::into_inner` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn into_inner(self) -> Option<(Id, &'static Metadata<'static>)>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::span::Current", "path": "Current"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [251, 1], "end": [316, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:294`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Consumes `self` and returns the span `Id` and `Metadata` of the current
span, if one exists and is known.

<a id="op-4df0f56f1cb220d6ad616592"></a>
## is_known

`function` · `tracing_core::span::Current::is_known` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn is_known(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::span::Current", "path": "Current"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [251, 1], "end": [316, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:288`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns `true` if the `Subscriber` that constructed this `Current` tracks a
current span.

If this returns `true` and [`id`], [`metadata`], or [`into_inner`]
return `None`, that indicates that we are currently known to *not* be
inside a span. If this returns `false`, those methods will also return
`None`, but in this case, that is because the subscriber does not keep
track of the currently-entered span.

[`id`]: Current::id()
[`metadata`]: Current::metadata()
[`into_inner`]: Current::into_inner()

<a id="op-258b1983232338cf29139cd2"></a>
## metadata

`function` · `tracing_core::span::Current::metadata` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn metadata(&self) -> Option<&'static Metadata<'static>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::span::Current", "path": "Current"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [251, 1], "end": [316, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:310`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Borrows the `Metadata` of the current span, if one exists and is known.

<a id="op-addd6cc5001052544a9c5498"></a>
## new

`function` · `tracing_core::span::Current::new` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn new(id: Id, metadata: &'static Metadata<'static>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::span::Current", "path": "Current"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [251, 1], "end": [316, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:254`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Constructs a new `Current` that indicates the current context is a span
with the given `metadata` and `metadata`.

<a id="op-628d6d645e87799d45682424"></a>
## none

`function` · `tracing_core::span::Current::none` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn none() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::span::Current", "path": "Current"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [251, 1], "end": [316, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:262`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Constructs a new `Current` that indicates the current context is *not*
in a span.
