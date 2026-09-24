# `tracing_core::metadata::LevelFilter`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.metadata.LevelFilter.json).

<a id="op-6d789785cc5ceea1e8c62c98"></a>
## LevelFilter

`struct` · `tracing_core::metadata::LevelFilter` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct LevelFilter
```

Source: `src/metadata.rs:239`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

A filter comparable to a verbosity [`Level`](../operations/tracing_core.metadata.Level.md#op-f8804717be954252aadd54ff).

If a [`Level`](../operations/tracing_core.metadata.Level.md#op-f8804717be954252aadd54ff) is considered less than or equal to a `LevelFilter`, it
should be considered enabled; if greater than the `LevelFilter`, that level
is disabled. See [`LevelFilter::current`](../operations/tracing_core.metadata.LevelFilter.md#op-5068429d655fd17dc510f0af) for more details.

Note that this is essentially identical to the `Level` type, but with the
addition of an [`OFF`] level that completely disables all trace
instrumentation.

See the documentation for the [`Level`](../operations/tracing_core.metadata.Level.md#op-f8804717be954252aadd54ff) type to see how `Level`s
and `LevelFilter`s interact.

[`OFF`]: LevelFilter::OFF

<a id="op-b96abfdba3bc4ffcb67fe713"></a>
## DEBUG

`assoc_const` · `tracing_core::metadata::LevelFilter::DEBUG` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
DEBUG
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:653`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "debug" level.

Designates lower priority information.

<a id="op-5e4f604e2234396922f3be88"></a>
## ERROR

`assoc_const` · `tracing_core::metadata::LevelFilter::ERROR` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
ERROR
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:641`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "error" level.

Designates very serious errors.

<a id="op-877619941576351940aeb3d7"></a>
## Err

`assoc_type` · `tracing_core::metadata::LevelFilter::Err` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
Err
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [783, 1], "end": [809, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/metadata.rs:784`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-797f81a2570fbf2d02054dea"></a>
## INFO

`assoc_const` · `tracing_core::metadata::LevelFilter::INFO` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
INFO
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:649`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "info" level.

Designates useful information.

<a id="op-5db5fa767073360a607dce55"></a>
## OFF

`assoc_const` · `tracing_core::metadata::LevelFilter::OFF` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
OFF
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:637`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "off" level.

Designates that trace instrumentation should be completely disabled.

<a id="op-63137a8875cf0eb2e27470ab"></a>
## TRACE

`assoc_const` · `tracing_core::metadata::LevelFilter::TRACE` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
TRACE
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:657`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "trace" level.

Designates very low priority, often extremely verbose, information.

<a id="op-02dd331c034b07c76e18e600"></a>
## WARN

`assoc_const` · `tracing_core::metadata::LevelFilter::WARN` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
WARN
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:645`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "warn" level.

Designates hazardous situations.

<a id="op-6a8e7f1ef3e53acd3003efcb"></a>
## clone

`function` · `tracing_core::metadata::LevelFilter::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> LevelFilter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 16], "end": [238, 21], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metadata.rs:238`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f403c2179ea3c8faad1ee053"></a>
## cmp

`function` · `tracing_core::metadata::LevelFilter::cmp` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn cmp(&self, other: &Self) -> cmp::Ordering
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1021, 1], "end": [1026, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::Ord", "path": "Ord"}, "trait_path": "core::cmp::Ord"}`

Source: `src/metadata.rs:1023`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5068429d655fd17dc510f0af"></a>
## current

`function` · `tracing_core::metadata::LevelFilter::current` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn current() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:704`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a `LevelFilter` that matches the most verbose [`Level`] that any
currently active [`Subscriber`] will enable.

User code should treat this as a *hint*. If a given span or event has a
level *higher* than the returned `LevelFilter`, it will not be enabled.
However, if the level is less than or equal to this value, the span or
event is *not* guaranteed to be enabled; the subscriber will still
filter each callsite individually.

Therefore, comparing a given span or event's level to the returned
`LevelFilter` **can** be used for determining if something is
*disabled*, but **should not** be used for determining if something is
*enabled*.

[`Level`]: super::Level
[`Subscriber`]: super::Subscriber

<a id="op-9eb4af271c95076f0f2a948e"></a>
## eq

`function` · `tracing_core::metadata::LevelFilter::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [987, 1], "end": [992, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metadata.rs:989`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f245c6307edfde5f619b52e0"></a>
## eq

`function` · `tracing_core::metadata::LevelFilter::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 27], "end": [238, 36], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metadata.rs:238`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2469c2ca5822b39459cdf83e"></a>
## fmt

`function` · `tracing_core::metadata::LevelFilter::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [757, 1], "end": [768, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/metadata.rs:758`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c22ffa36effdc2cc3a455fb1"></a>
## fmt

`function` · `tracing_core::metadata::LevelFilter::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [770, 1], "end": [781, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metadata.rs:771`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4b286ce00a04f3f793623104"></a>
## from

`function` · `tracing_core::metadata::LevelFilter::from` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn from(level: Level) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [612, 1], "end": [617, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metadata.rs:614`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-feab64de6ebfb9d126d341c3"></a>
## from

`function` · `tracing_core::metadata::LevelFilter::from` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn from(level: Option<Level>) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [619, 1], "end": [624, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::option::Option", "path": "Option"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/metadata.rs:621`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f2707d4f0f1ebb55799a3527"></a>
## from_level

`function` · `tracing_core::metadata::LevelFilter::from_level` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
const fn from_level(level: Level) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:661`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns a `LevelFilter` that enables spans and events with verbosity up
to and including `level`.

<a id="op-9e9c17997a145f6e093d0eea"></a>
## from_str

`function` · `tracing_core::metadata::LevelFilter::from_str` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn from_str(from: &str) -> Result<Self, Self::Err>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [783, 1], "end": [809, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/metadata.rs:785`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-23213af3228cc5f4ec041289"></a>
## ge

`function` · `tracing_core::metadata::LevelFilter::ge` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn ge(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [994, 1], "end": [1019, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1016`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-46677443108c796a968545dc"></a>
## ge

`function` · `tracing_core::metadata::LevelFilter::ge` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn ge(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1028, 1], "end": [1053, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1050`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-484d74a09f5ba6b0515e8d28"></a>
## gt

`function` · `tracing_core::metadata::LevelFilter::gt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn gt(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [994, 1], "end": [1019, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1011`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a83997f9e1b55bba9a06ca43"></a>
## gt

`function` · `tracing_core::metadata::LevelFilter::gt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn gt(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1028, 1], "end": [1053, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1045`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-21eb9926ee9ad3b51698ff33"></a>
## hash

`function` · `tracing_core::metadata::LevelFilter::hash` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 38], "end": [238, 42], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/metadata.rs:238`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0cbbd6b9511cd74f6a4e3f1d"></a>
## into_level

`function` · `tracing_core::metadata::LevelFilter::into_level` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
const fn into_level(self) -> Option<Level>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [633, 1], "end": [755, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:669`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the most verbose [`Level`](../operations/tracing_core.metadata.Level.md#op-f8804717be954252aadd54ff) that this filter accepts, or `None`
if it is [`OFF`].

[`OFF`]: LevelFilter::OFF

<a id="op-2bf6585a638d42641f265a07"></a>
## le

`function` · `tracing_core::metadata::LevelFilter::le` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn le(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1028, 1], "end": [1053, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1040`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3040ae12151a1490aca546bc"></a>
## le

`function` · `tracing_core::metadata::LevelFilter::le` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn le(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [994, 1], "end": [1019, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1006`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-10a7ef669aa0df837b15b671"></a>
## lt

`function` · `tracing_core::metadata::LevelFilter::lt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn lt(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1028, 1], "end": [1053, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1035`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8cd2a87c659d8a81004b978e"></a>
## lt

`function` · `tracing_core::metadata::LevelFilter::lt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn lt(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [994, 1], "end": [1019, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1001`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-18ec04bd50c47b0eb05a2c2b"></a>
## partial_cmp

`function` · `tracing_core::metadata::LevelFilter::partial_cmp` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [1028, 1], "end": [1053, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:1030`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8ef4e272672cc966e47ee7e9"></a>
## partial_cmp

`function` · `tracing_core::metadata::LevelFilter::partial_cmp` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [994, 1], "end": [1019, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:996`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
