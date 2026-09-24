# `tracing_subscriber::field::debug::Alt`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.debug.Alt.json).

<a id="op-238ca35cc10cc31e4231375f"></a>
## Alt

`struct` · `tracing_subscriber::field::debug::Alt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Alt<V>
```

Source: `src/field/debug.rs:10`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A visitor wrapper that ensures any `fmt::Debug` fields are formatted using
the alternate (`:#`) formatter.

<a id="op-d8515288bbff7c58f9f853f1"></a>
## Visitor

`assoc_type` · `tracing_subscriber::field::debug::Alt::Visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [25, 1], "end": [35, 2], "filename": "src/field/debug.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/field/debug.rs:29`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-69c51c191e79e5b161c35f4f"></a>
## clone

`function` · `tracing_subscriber::field::debug::Alt::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Alt<V>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 17], "end": [9, 22], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/field/debug.rs:9`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d26bbea0cdd6cb0f668d0646"></a>
## finish

`function` · `tracing_subscriber::field::debug::Alt::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> O
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "O"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "O"}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [77, 1], "end": [85, 2], "filename": "src/field/debug.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "O"}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}, "trait_path": "tracing_subscriber::field::VisitOutput"}`

Source: `src/field/debug.rs:82`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d878688c7d73ad93d9e0263b"></a>
## fmt

`function` · `tracing_subscriber::field::debug::Alt::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 10], "end": [9, 15], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field/debug.rs:9`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-baffa352a0db34d78faf9dc6"></a>
## make_visitor

`function` · `tracing_subscriber::field::debug::Alt::make_visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_visitor(&self, target: T) -> Self::Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [25, 1], "end": [35, 2], "filename": "src/field/debug.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/field/debug.rs:32`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9e8c2beb507af691d431793a"></a>
## new

`function` · `tracing_subscriber::field::debug::Alt::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(inner: V) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [23, 2], "filename": "src/field/debug.rs"}, "trait": null, "trait_path": null}`

Source: `src/field/debug.rs:20`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Wraps the provided visitor so that any `fmt::Debug` fields are formatted
using the alternative (`:#`) formatter.

<a id="op-db69d80b9284805cfbc60650"></a>
## record_bool

`function` · `tracing_subscriber::field::debug::Alt::record_bool` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_bool(&mut self, field: &Field, value: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [37, 1], "end": [75, 2], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/debug.rs:57`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c67619a4cecb1a443ccabdca"></a>
## record_debug

`function` · `tracing_subscriber::field::debug::Alt::record_debug` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [37, 1], "end": [75, 2], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/debug.rs:72`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7e0bda882a5d01d6e6d2f802"></a>
## record_f64

`function` · `tracing_subscriber::field::debug::Alt::record_f64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_f64(&mut self, field: &Field, value: f64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [37, 1], "end": [75, 2], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/debug.rs:42`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f0a98bbc97dbbf3835c95931"></a>
## record_i64

`function` · `tracing_subscriber::field::debug::Alt::record_i64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_i64(&mut self, field: &Field, value: i64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [37, 1], "end": [75, 2], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/debug.rs:47`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3e82982a30704258d9499151"></a>
## record_str

`function` · `tracing_subscriber::field::debug::Alt::record_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_str(&mut self, field: &Field, value: &str)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [37, 1], "end": [75, 2], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/debug.rs:62`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit a string value.

<a id="op-6c1b3eb56b916ae33236d499"></a>
## record_u64

`function` · `tracing_subscriber::field::debug::Alt::record_u64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_u64(&mut self, field: &Field, value: u64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [37, 1], "end": [75, 2], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/debug.rs:52`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-30eff234ec6bd3ea1b08a2df"></a>
## writer

`function` · `tracing_subscriber::field::debug::Alt::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [103, 1], "end": [111, 2], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}, "trait_path": "tracing_subscriber::field::VisitFmt"}`

Source: `src/field/debug.rs:108`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5304766f34204bb2e2b4bae9"></a>
## writer

`function` · `tracing_subscriber::field::debug::Alt::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn io::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::debug::Alt", "path": "Alt"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitWrite", "path": "VisitWrite"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [92, 5], "end": [100, 6], "filename": "src/field/debug.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitWrite", "path": "VisitWrite"}, "trait_path": "tracing_subscriber::field::VisitWrite"}`

Source: `src/field/debug.rs:97`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
