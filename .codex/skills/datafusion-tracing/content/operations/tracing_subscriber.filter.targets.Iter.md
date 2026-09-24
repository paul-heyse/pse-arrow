# `tracing_subscriber::filter::targets::Iter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.filter.targets.Iter.json).

<a id="op-117661306127b14106d9f3ac"></a>
## Iter

`struct` · `tracing_subscriber::filter::targets::Iter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Iter<'a>
```

Source: `src/filter/targets.rs:565`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A borrowing iterator over the [target]-[level] pairs of a `Targets` filter.

This struct is created by [`iter`] method of [`Targets`](../operations/tracing_subscriber.filter.targets.Targets.md#op-641e1a5b3033eaed5a14d773), or from the `IntoIterator`
implementation for `&Targets`.

[target]: tracing_core::Metadata::target
[level]: tracing_core::Level
[`iter`]: Targets::iter

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::target`.

<a id="op-21eb00eba951a11a6e174e5a"></a>
## Item

`assoc_type` · `tracing_subscriber::filter::targets::Iter::Item` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Item
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::filter::targets::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [583, 1], "end": [593, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/filter/targets.rs:584`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d23cc25204278b8002884638"></a>
## fmt

`function` · `tracing_subscriber::filter::targets::Iter::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::filter::targets::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [564, 10], "end": [564, 15], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/filter/targets.rs:564`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f05e42721d1afc88f4ed7ba3"></a>
## next

`function` · `tracing_subscriber::filter::targets::Iter::next` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn next(&mut self) -> Option<Self::Item>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::filter::targets::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [583, 1], "end": [593, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/filter/targets.rs:586`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d4b3b56656696c25132eaaf5"></a>
## size_hint

`function` · `tracing_subscriber::filter::targets::Iter::size_hint` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn size_hint(&self) -> (usize, Option<usize>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::filter::targets::Iter", "path": "Iter"}}, "generics": {"params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [583, 1], "end": [593, 2], "filename": "src/filter/targets.rs"}, "trait": {"args": null, "id": "core::iter::traits::iterator::Iterator", "path": "Iterator"}, "trait_path": "core::iter::traits::iterator::Iterator"}`

Source: `src/filter/targets.rs:590`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
