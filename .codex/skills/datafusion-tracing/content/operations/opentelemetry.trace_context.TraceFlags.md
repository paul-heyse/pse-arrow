# `opentelemetry::trace_context::TraceFlags`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace_context.TraceFlags.json).

<a id="op-47031c18e731037a4e7c9356"></a>
## TraceFlags

`struct` · `opentelemetry::trace_context::TraceFlags` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TraceFlags
```

Source: `src/trace_context.rs:16`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Flags that can be set on a `SpanContext`.

The current version of the specification only supports a single flag
[`TraceFlags::SAMPLED`](../operations/opentelemetry.trace_context.TraceFlags.md#op-58f140bfab75c1c5c1497c8d).

See the W3C TraceContext specification's [trace-flags] section for more
details.

[trace-flags]: https://www.w3.org/TR/trace-context/#trace-flags

<a id="op-431670c429720a52718b9658"></a>
## NOT_SAMPLED

`assoc_const` · `opentelemetry::trace_context::TraceFlags::NOT_SAMPLED` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
NOT_SAMPLED
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 1], "end": [58, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:25`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Trace flags with the `sampled` flag set to `0`.

Spans that are not sampled will be ignored by most tracing tools.
See the `sampled` section of the [W3C TraceContext specification] for details.

[W3C TraceContext specification]: https://www.w3.org/TR/trace-context/#sampled-flag

<a id="op-5ed0a0cf5b678b38ac0e08d1"></a>
## Output

`assoc_type` · `opentelemetry::trace_context::TraceFlags::Output` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [68, 1], "end": [74, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitOr", "path": "BitOr"}, "trait_path": "core::ops::bit::BitOr"}`

Source: `src/trace_context.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8dff7a270806035b9929b4ba"></a>
## Output

`assoc_type` · `opentelemetry::trace_context::TraceFlags::Output` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [60, 1], "end": [66, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitAnd", "path": "BitAnd"}, "trait_path": "core::ops::bit::BitAnd"}`

Source: `src/trace_context.rs:61`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dfc7ed963a0388613cc1ec57"></a>
## Output

`assoc_type` · `opentelemetry::trace_context::TraceFlags::Output` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [76, 1], "end": [82, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::ops::bit::Not", "path": "Not"}, "trait_path": "core::ops::bit::Not"}`

Source: `src/trace_context.rs:77`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-58f140bfab75c1c5c1497c8d"></a>
## SAMPLED

`assoc_const` · `opentelemetry::trace_context::TraceFlags::SAMPLED` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
SAMPLED
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 1], "end": [58, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:33`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Trace flags with the `sampled` flag set to `1`.

Spans that are not sampled will be ignored by most tracing tools.
See the `sampled` section of the [W3C TraceContext specification] for details.

[W3C TraceContext specification]: https://www.w3.org/TR/trace-context/#sampled-flag

<a id="op-9614c70fcdac5928e8a539dc"></a>
## bitand

`function` · `opentelemetry::trace_context::TraceFlags::bitand` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn bitand(self, rhs: Self) -> Self::Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [60, 1], "end": [66, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitAnd", "path": "BitAnd"}, "trait_path": "core::ops::bit::BitAnd"}`

Source: `src/trace_context.rs:63`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ca2f72282983a279ce5734b2"></a>
## bitor

`function` · `opentelemetry::trace_context::TraceFlags::bitor` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn bitor(self, rhs: Self) -> Self::Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [68, 1], "end": [74, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::ops::bit::BitOr", "path": "BitOr"}, "trait_path": "core::ops::bit::BitOr"}`

Source: `src/trace_context.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f92446f74e338fb663de1b23"></a>
## clone

`function` · `opentelemetry::trace_context::TraceFlags::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> TraceFlags
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [15, 10], "end": [15, 15], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace_context.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c129e7927ac27da915c22e57"></a>
## default

`function` · `opentelemetry::trace_context::TraceFlags::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> TraceFlags
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [15, 24], "end": [15, 31], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace_context.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8e03870d5808ac1b45db6e68"></a>
## eq

`function` · `opentelemetry::trace_context::TraceFlags::eq` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &TraceFlags) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [15, 33], "end": [15, 42], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/trace_context.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-114f6c4f3990cbb905893ad0"></a>
## fmt

`function` · `opentelemetry::trace_context::TraceFlags::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [84, 1], "end": [88, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::LowerHex", "path": "LowerHex"}, "trait_path": "core::fmt::LowerHex"}`

Source: `src/trace_context.rs:85`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-882ed92b08bbc7e53ce2d3aa"></a>
## fmt

`function` · `opentelemetry::trace_context::TraceFlags::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [15, 17], "end": [15, 22], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace_context.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7d781b425c7cf1ff6babf1d9"></a>
## hash

`function` · `opentelemetry::trace_context::TraceFlags::hash` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [15, 54], "end": [15, 58], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/trace_context.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-79ce4099db00cb89cfff2ed3"></a>
## is_sampled

`function` · `opentelemetry::trace_context::TraceFlags::is_sampled` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_sampled(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 1], "end": [58, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns `true` if the `sampled` flag is set

<a id="op-866e9a50f07e77e1d366870f"></a>
## new

`function` · `opentelemetry::trace_context::TraceFlags::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
const fn new(flags: u8) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 1], "end": [58, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:36`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Construct new trace flags

<a id="op-832aa52f155b8a944da7f441"></a>
## not

`function` · `opentelemetry::trace_context::TraceFlags::not` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn not(self) -> Self::Output
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [76, 1], "end": [82, 2], "filename": "src/trace_context.rs"}, "trait": {"args": null, "id": "core::ops::bit::Not", "path": "Not"}, "trait_path": "core::ops::bit::Not"}`

Source: `src/trace_context.rs:79`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4ba11934e45aa2972df14d12"></a>
## to_u8

`function` · `opentelemetry::trace_context::TraceFlags::to_u8` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn to_u8(self) -> u8
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 1], "end": [58, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:55`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns the flags as a `u8`

<a id="op-a0d3846edeb461db75a7a112"></a>
## with_sampled

`function` · `opentelemetry::trace_context::TraceFlags::with_sampled` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_sampled(&self, sampled: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::trace_context::TraceFlags", "path": "TraceFlags"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 1], "end": [58, 2], "filename": "src/trace_context.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace_context.rs:46`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns copy of the current flags with the `sampled` flag set.
