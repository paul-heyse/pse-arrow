# `tracing::span::EnteredSpan`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.span.EnteredSpan.json).

<a id="op-563d7a0c7d3b4e05b06837d6"></a>
## EnteredSpan

`struct` · `tracing::span::EnteredSpan` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
struct EnteredSpan
```

Source: `src/span.rs:402`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

An owned version of [`Entered`](../operations/tracing.span.Entered.md#op-5fa98f5fd356bc6e42d6afad), a guard representing a span which has been
entered and is currently executing.

When the guard is dropped, the span will be exited.

This is returned by the [`Span::entered`] function.

[`Span::entered`]: super::Span::entered()

<a id="op-6c7ccb5f72460efcdb07378a"></a>
## Target

`assoc_type` · `tracing::span::EnteredSpan::Target` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
Target
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::EnteredSpan", "path": "EnteredSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1556, 1], "end": [1563, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::ops::deref::Deref", "path": "Deref"}, "trait_path": "core::ops::deref::Deref"}`

Source: `src/span.rs:1557`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c004cec1d61feaeb75be7770"></a>
## deref

`function` · `tracing::span::EnteredSpan::deref` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn deref(&self) -> &Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::EnteredSpan", "path": "EnteredSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1556, 1], "end": [1563, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::ops::deref::Deref", "path": "Deref"}, "trait_path": "core::ops::deref::Deref"}`

Source: `src/span.rs:1560`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-054bf95a4b9add45afa0419c"></a>
## drop

`function` · `tracing::span::EnteredSpan::drop` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::EnteredSpan", "path": "EnteredSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1572, 1], "end": [1577, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/span.rs:1574`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5f56e68c59421619aa82395f"></a>
## exit

`function` · `tracing::span::EnteredSpan::exit` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn exit(self) -> Span
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::EnteredSpan", "path": "EnteredSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1540, 1], "end": [1554, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1548`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Exits this span, returning the underlying [`Span`](../operations/tracing.span.Span.md#op-1283b08586edf8ae3649f1c6).

<a id="op-9ccf6c79ef96dc1b7c46387a"></a>
## fmt

`function` · `tracing::span::EnteredSpan::fmt` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::EnteredSpan", "path": "EnteredSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [400, 10], "end": [400, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:400`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-67cfb320f3fc561f1c4f75a4"></a>
## id

`function` · `tracing::span::EnteredSpan::id` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn id(&self) -> Option<Id>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing::span::EnteredSpan", "path": "EnteredSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1540, 1], "end": [1554, 2], "filename": "src/span.rs"}, "trait": null, "trait_path": null}`

Source: `src/span.rs:1542`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

Returns this span's `Id`, if it is enabled.
