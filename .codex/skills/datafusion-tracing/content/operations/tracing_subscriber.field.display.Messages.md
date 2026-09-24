# `tracing_subscriber::field::display::Messages`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.display.Messages.json).

<a id="op-c30a8de53a8cad0fe0aba443"></a>
## Messages

`struct` · `tracing_subscriber::field::display::Messages` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Messages<V>
```

Source: `src/field/display.rs:10`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A visitor wrapper that ensures any strings named "message" are formatted
using `fmt::Display`

<a id="op-38da4cfac1e7a8a4f5ad6c41"></a>
## Visitor

`assoc_type` · `tracing_subscriber::field::display::Messages::Visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [27, 1], "end": [37, 2], "filename": "src/field/display.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/field/display.rs:31`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4b7d62b0657432ecaf3fb56c"></a>
## clone

`function` · `tracing_subscriber::field::display::Messages::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Messages<V>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 17], "end": [9, 22], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/field/display.rs:9`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3df618f0164661edaca9fd35"></a>
## finish

`function` · `tracing_subscriber::field::display::Messages::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> O
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "O"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "O"}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [83, 1], "end": [91, 2], "filename": "src/field/display.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "O"}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}, "trait_path": "tracing_subscriber::field::VisitOutput"}`

Source: `src/field/display.rs:88`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-045625d5c0bb8c41a777bd4b"></a>
## fmt

`function` · `tracing_subscriber::field::display::Messages::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 10], "end": [9, 15], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field/display.rs:9`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c2a447d807608ef5ecf25024"></a>
## make_visitor

`function` · `tracing_subscriber::field::display::Messages::make_visitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn make_visitor(&self, target: T) -> Self::Visitor
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [27, 1], "end": [37, 2], "filename": "src/field/display.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::field::MakeVisitor", "path": "MakeVisitor"}, "trait_path": "tracing_subscriber::field::MakeVisitor"}`

Source: `src/field/display.rs:34`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c29cef851a174c62f761fe80"></a>
## new

`function` · `tracing_subscriber::field::display::Messages::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(inner: V) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [25, 2], "filename": "src/field/display.rs"}, "trait": null, "trait_path": null}`

Source: `src/field/display.rs:22`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`MakeVisitor`] implementation that will wrap `inner` so
that any strings named `message` are formatted using `fmt::Display`.

[`MakeVisitor`]: super::MakeVisitor

<a id="op-c4dbc286edfa34ee725154d0"></a>
## record_bool

`function` · `tracing_subscriber::field::display::Messages::record_bool` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_bool(&mut self, field: &Field, value: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [39, 1], "end": [81, 2], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/display.rs:59`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d5deebcc808e4bb2baef09b7"></a>
## record_debug

`function` · `tracing_subscriber::field::display::Messages::record_debug` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [39, 1], "end": [81, 2], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/display.rs:78`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7354d0bed7486ed5b4305d59"></a>
## record_f64

`function` · `tracing_subscriber::field::display::Messages::record_f64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_f64(&mut self, field: &Field, value: f64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [39, 1], "end": [81, 2], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/display.rs:44`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-57bf5b60786007c459ba709d"></a>
## record_i64

`function` · `tracing_subscriber::field::display::Messages::record_i64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_i64(&mut self, field: &Field, value: i64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [39, 1], "end": [81, 2], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/display.rs:49`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-845e0b4705e77c2ab4889424"></a>
## record_str

`function` · `tracing_subscriber::field::display::Messages::record_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_str(&mut self, field: &Field, value: &str)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [39, 1], "end": [81, 2], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/display.rs:64`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Visit a string value.

<a id="op-0e8d43afaad2c1d0fb3e43b8"></a>
## record_u64

`function` · `tracing_subscriber::field::display::Messages::record_u64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_u64(&mut self, field: &Field, value: u64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [39, 1], "end": [81, 2], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/display.rs:54`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-506536ff0b57c34a3262fae2"></a>
## writer

`function` · `tracing_subscriber::field::display::Messages::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn io::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitWrite", "path": "VisitWrite"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [98, 5], "end": [106, 6], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitWrite", "path": "VisitWrite"}, "trait_path": "tracing_subscriber::field::VisitWrite"}`

Source: `src/field/display.rs:103`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a5fb8cc9081c7b0a922c1024"></a>
## writer

`function` · `tracing_subscriber::field::display::Messages::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::display::Messages", "path": "Messages"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}]}, "is_negative": false, "span": {"begin": [109, 1], "end": [117, 2], "filename": "src/field/display.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}, "trait_path": "tracing_subscriber::field::VisitFmt"}`

Source: `src/field/display.rs:114`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
