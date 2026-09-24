# `tracing::span::Entered`

Full upstream contracts; raw type trees and source locators in [structured records](tracing.span.Entered.json).

<a id="op-5fa98f5fd356bc6e42d6afad"></a>
## Entered

`struct` · `tracing::span::Entered` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
struct Entered<'a>
```

Source: `src/span.rs:388`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

A guard representing a span which has been entered and is currently
executing.

When the guard is dropped, the span will be exited.

This is returned by the [`Span::enter`] function.

[`Span::enter`]: super::Span::enter

<a id="op-f533c54de71e385703691509"></a>
## drop

`function` · `tracing::span::Entered::drop` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn drop(&mut self)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'_"}], "constraints": []}}, "id": "tracing::span::Entered", "path": "Entered"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1565, 1], "end": [1570, 2], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::ops::drop::Drop", "path": "Drop"}, "trait_path": "core::ops::drop::Drop"}`

Source: `src/span.rs:1567`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b52beb7ba900765c78b6afef"></a>
## fmt

`function` · `tracing::span::Entered::fmt` · tracing 0.1.44
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing::span::Entered", "path": "Entered"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [386, 10], "end": [386, 15], "filename": "src/span.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/span.rs:386`. [Exact documentation build](https://docs.rs/crate/tracing/0.1.44/json).

No upstream documentation on this item; consult its owner/trait contract.
