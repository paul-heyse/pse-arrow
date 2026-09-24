# `tracing_subscriber::fmt::format::FmtSpan`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.FmtSpan.json).

<a id="op-0b00f8b7558927cae52a28c2"></a>
## FmtSpan

`struct` · `tracing_subscriber::fmt::format::FmtSpan` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct FmtSpan
```

Source: `src/fmt/format/mod.rs:1644`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Configures what points in the span lifecycle are logged as events.

See also [`with_span_events`].

[`with_span_events`]: super::SubscriberBuilder::with_span_events

<a id="op-ab081ae5264de710102e31bb"></a>
## ACTIVE

`assoc_const` · `tracing_subscriber::fmt::format::FmtSpan::ACTIVE` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
ACTIVE
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1646, 1], "end": [1668, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1659`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

one event per enter/exit of a span

<a id="op-6aced2b9dee7da2dadab5586"></a>
## CLOSE

`assoc_const` · `tracing_subscriber::fmt::format::FmtSpan::CLOSE` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
CLOSE
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1646, 1], "end": [1668, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1654`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

one event when the span is dropped

<a id="op-51cc60760cfcc5e0ba4ad56a"></a>
## ENTER

`assoc_const` · `tracing_subscriber::fmt::format::FmtSpan::ENTER` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
ENTER
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1646, 1], "end": [1668, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1650`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

one event per enter of a span

<a id="op-df62d5f230c7a747ca06dc9a"></a>
## EXIT

`assoc_const` · `tracing_subscriber::fmt::format::FmtSpan::EXIT` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
EXIT
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1646, 1], "end": [1668, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1652`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

one event per exit of a span

<a id="op-fb4a1e505eb773c22705e3f5"></a>
## FULL

`assoc_const` · `tracing_subscriber::fmt::format::FmtSpan::FULL` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
FULL
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1646, 1], "end": [1668, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1661`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

events at all points (new, enter, exit, drop)

<a id="op-a17cf4e5e20af2226e862aa2"></a>
## NEW

`assoc_const` · `tracing_subscriber::fmt::format::FmtSpan::NEW` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
NEW
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1646, 1], "end": [1668, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1648`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

one event when span is created

<a id="op-db0d997578101b3a04744e90"></a>
## NONE

`assoc_const` · `tracing_subscriber::fmt::format::FmtSpan::NONE` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
NONE
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1646, 1], "end": [1668, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:1657`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

spans are ignored (this is the default)

<a id="op-85d04b8e4377959dc9606418"></a>
## Output

`assoc_type` · `tracing_subscriber::fmt::format::FmtSpan::Output` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1693, 1], "end": [1693, 39], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitOr", "path": "BitOr"}, "trait_path": "core::ops::bit::BitOr"}`

Source: `src/fmt/format/mod.rs:1693`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-88dc2617e04aac1a74d30516"></a>
## Output

`assoc_type` · `tracing_subscriber::fmt::format::FmtSpan::Output` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1692, 1], "end": [1692, 41], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitAnd", "path": "BitAnd"}, "trait_path": "core::ops::bit::BitAnd"}`

Source: `src/fmt/format/mod.rs:1692`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a9e578cc40a9a5a904d89c39"></a>
## Output

`assoc_type` · `tracing_subscriber::fmt::format::FmtSpan::Output` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1694, 1], "end": [1694, 41], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitXor", "path": "BitXor"}, "trait_path": "core::ops::bit::BitXor"}`

Source: `src/fmt/format/mod.rs:1694`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-15fca450aa44a86582e3ad45"></a>
## bitand

`function` · `tracing_subscriber::fmt::format::FmtSpan::bitand` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn bitand(self, rhs: Self) -> Self::Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1692, 1], "end": [1692, 41], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitAnd", "path": "BitAnd"}, "trait_path": "core::ops::bit::BitAnd"}`

Source: `src/fmt/format/mod.rs:1692`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cf3fc595c816d15b1ebac3a2"></a>
## bitand_assign

`function` · `tracing_subscriber::fmt::format::FmtSpan::bitand_assign` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn bitand_assign(&mut self, rhs: Self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1696, 1], "end": [1696, 61], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitAndAssign", "path": "BitAndAssign"}, "trait_path": "core::ops::bit::BitAndAssign"}`

Source: `src/fmt/format/mod.rs:1696`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c8938b640a4f4ea8fff7fe51"></a>
## bitor

`function` · `tracing_subscriber::fmt::format::FmtSpan::bitor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn bitor(self, rhs: Self) -> Self::Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1693, 1], "end": [1693, 39], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitOr", "path": "BitOr"}, "trait_path": "core::ops::bit::BitOr"}`

Source: `src/fmt/format/mod.rs:1693`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-79ff3e3f379b27a0b9956f88"></a>
## bitor_assign

`function` · `tracing_subscriber::fmt::format::FmtSpan::bitor_assign` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn bitor_assign(&mut self, rhs: Self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1697, 1], "end": [1697, 59], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitOrAssign", "path": "BitOrAssign"}, "trait_path": "core::ops::bit::BitOrAssign"}`

Source: `src/fmt/format/mod.rs:1697`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a2ccdab209a329f634b7480c"></a>
## bitxor

`function` · `tracing_subscriber::fmt::format::FmtSpan::bitxor` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn bitxor(self, rhs: Self) -> Self::Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1694, 1], "end": [1694, 41], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitXor", "path": "BitXor"}, "trait_path": "core::ops::bit::BitXor"}`

Source: `src/fmt/format/mod.rs:1694`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7d1b9839991a93affe9ecd5"></a>
## bitxor_assign

`function` · `tracing_subscriber::fmt::format::FmtSpan::bitxor_assign` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn bitxor_assign(&mut self, rhs: Self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1698, 1], "end": [1698, 61], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitXorAssign", "path": "BitXorAssign"}, "trait_path": "core::ops::bit::BitXorAssign"}`

Source: `src/fmt/format/mod.rs:1698`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d368b681d9208ff0f02b2aae"></a>
## clone

`function` · `tracing_subscriber::fmt::format::FmtSpan::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> FmtSpan
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1643, 10], "end": [1643, 15], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/format/mod.rs:1643`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1378b894c12f5318ab6842e8"></a>
## cmp

`function` · `tracing_subscriber::fmt::format::FmtSpan::cmp` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn cmp(&self, other: &FmtSpan) -> cmp::Ordering
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1643, 32], "end": [1643, 35], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::cmp::Ord", "path": "Ord"}, "trait_path": "core::cmp::Ord"}`

Source: `src/fmt/format/mod.rs:1643`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-790e99f585c342888c0e37e9"></a>
## eq

`function` · `tracing_subscriber::fmt::format::FmtSpan::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &FmtSpan) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1643, 21], "end": [1643, 30], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/format/mod.rs:1643`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-fddc301aa91a8d8331292f47"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::FmtSpan::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1700, 1], "end": [1727, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:1701`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ba9a0b3fd4c5ee52983b9b65"></a>
## partial_cmp

`function` · `tracing_subscriber::fmt::format::FmtSpan::partial_cmp` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &FmtSpan) -> option::Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::FmtSpan", "path": "FmtSpan"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1643, 37], "end": [1643, 47], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/fmt/format/mod.rs:1643`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
