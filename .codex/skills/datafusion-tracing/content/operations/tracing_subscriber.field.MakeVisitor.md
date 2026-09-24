# `tracing_subscriber::field::MakeVisitor`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.MakeVisitor.json).

<a id="op-ee298e39fc8a5785910ade4f"></a>
## MakeVisitor

`trait` · `tracing_subscriber::field::MakeVisitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
trait MakeVisitor<T>
```

Source: `src/field/mod.rs:26`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Creates new [visitors].

A type implementing `MakeVisitor` represents a composable factory for types
implementing the [`Visit` trait][visitors]. The `MakeVisitor` trait defines
a single function, `make_visitor`, which takes in a `T`-typed `target` and
returns a type implementing `Visit` configured for that target. A target may
be a string, output stream, or data structure that the visitor will record
data to, configuration variables that determine the visitor's behavior, or
`()` when no input is required to produce a visitor.

[visitors]: tracing_core::field::Visit

<a id="op-aee9042600a2edad0e1d5755"></a>
## Visitor

`assoc_type` · `tracing_subscriber::field::MakeVisitor::Visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Visitor
```

Source: `src/field/mod.rs:28`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

The visitor type produced by this `MakeVisitor`.

<a id="op-43792274165df35e55da65c1"></a>
## make_visitor

`function` · `tracing_subscriber::field::MakeVisitor::make_visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_visitor(&self, target: T) -> Self::Visitor
```

Source: `src/field/mod.rs:31`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Make a new visitor for the provided `target`.
