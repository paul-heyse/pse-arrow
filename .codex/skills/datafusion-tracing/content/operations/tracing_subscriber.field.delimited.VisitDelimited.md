# `tracing_subscriber::field::delimited::VisitDelimited`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.field.delimited.VisitDelimited.json).

<a id="op-d995d7e2ac529a7122bfd456"></a>
## VisitDelimited

`struct` · `tracing_subscriber::field::delimited::VisitDelimited` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct VisitDelimited<D, V>
```

Source: `src/field/delimited.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A visitor wrapper that inserts a delimiter after the wrapped visitor formats
a field value.

<a id="op-c992af06a3b2ec28b6a00028"></a>
## finish

`function` · `tracing_subscriber::field::delimited::VisitDelimited::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "D"}}}]}, "is_negative": false, "span": {"begin": [114, 1], "end": [123, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"tuple": []}}, {"type": {"resolved_path": {"args": null, "id": "core::fmt::Error", "path": "Error"}}}], "constraints": []}}, "id": "core::result::Result", "path": "Result"}}}], "constraints": []}}, "id": "tracing_subscriber::field::VisitOutput", "path": "VisitOutput"}, "trait_path": "tracing_subscriber::field::VisitOutput"}`

Source: `src/field/delimited.rs:119`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ca60b89577ec03ac368b0fb7"></a>
## fmt

`function` · `tracing_subscriber::field::delimited::VisitDelimited::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 10], "end": [17, 15], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/field/delimited.rs:17`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-27f63fb9e46144f67b19d2ba"></a>
## new

`function` · `tracing_subscriber::field::delimited::VisitDelimited::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(delimiter: D, inner: V) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [52, 1], "end": [81, 2], "filename": "src/field/delimited.rs"}, "trait": null, "trait_path": null}`

Source: `src/field/delimited.rs:57`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`Visit`] implementation that wraps `inner` so that
each formatted field is separated by the provided `delimiter`.

[`Visit`]: tracing_core::field::Visit

<a id="op-8ca2e1c306a91b941a64c441"></a>
## record_bool

`function` · `tracing_subscriber::field::delimited::VisitDelimited::record_bool` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_bool(&mut self, field: &Field, value: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "D"}}}]}, "is_negative": false, "span": {"begin": [83, 1], "end": [112, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/delimited.rs:98`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-42e74590a0e3866e74bae1e1"></a>
## record_debug

`function` · `tracing_subscriber::field::delimited::VisitDelimited::record_debug` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "D"}}}]}, "is_negative": false, "span": {"begin": [83, 1], "end": [112, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/delimited.rs:108`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-094847cdd8a193050e992cee"></a>
## record_i64

`function` · `tracing_subscriber::field::delimited::VisitDelimited::record_i64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_i64(&mut self, field: &Field, value: i64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "D"}}}]}, "is_negative": false, "span": {"begin": [83, 1], "end": [112, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/delimited.rs:88`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0908fd998ade20ffa9c58044"></a>
## record_str

`function` · `tracing_subscriber::field::delimited::VisitDelimited::record_str` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_str(&mut self, field: &Field, value: &str)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "D"}}}]}, "is_negative": false, "span": {"begin": [83, 1], "end": [112, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/delimited.rs:103`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d4d09d0ff148ce0e76fdb36"></a>
## record_u64

`function` · `tracing_subscriber::field::delimited::VisitDelimited::record_u64` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn record_u64(&mut self, field: &Field, value: u64)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "D"}}}]}, "is_negative": false, "span": {"begin": [83, 1], "end": [112, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "tracing_core::field::Visit", "path": "Visit"}, "trait_path": "tracing_core::field::Visit"}`

Source: `src/field/delimited.rs:93`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-878ffa21674ae4d9e06f02f8"></a>
## writer

`function` · `tracing_subscriber::field::delimited::VisitDelimited::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&mut self) -> &mut dyn fmt::Write
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "D"}}, {"type": {"generic": "V"}}], "constraints": []}}, "id": "tracing_subscriber::field::delimited::VisitDelimited", "path": "VisitDelimited"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "D"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "V"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}}}], "generic_params": [], "type": {"generic": "V"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"primitive": "str"}}], "constraints": []}}, "id": "core::convert::AsRef", "path": "AsRef"}}}], "generic_params": [], "type": {"generic": "D"}}}]}, "is_negative": false, "span": {"begin": [125, 1], "end": [133, 2], "filename": "src/field/delimited.rs"}, "trait": {"args": null, "id": "tracing_subscriber::field::VisitFmt", "path": "VisitFmt"}, "trait_path": "tracing_subscriber::field::VisitFmt"}`

Source: `src/field/delimited.rs:130`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
