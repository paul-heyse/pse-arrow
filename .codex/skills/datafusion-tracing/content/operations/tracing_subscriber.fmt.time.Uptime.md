# `tracing_subscriber::fmt::time::Uptime`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.Uptime.json).

<a id="op-4aa1c932df4f3599bb886867"></a>
## Uptime

`struct` · `tracing_subscriber::fmt::time::Uptime` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Uptime
```

Source: `src/fmt/time/mod.rs:118`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Retrieve and print the relative elapsed wall-clock time since an epoch.

The `Default` implementation for `Uptime` makes the epoch the current time.

<a id="op-878a4bda77d8d8a7912eed9d"></a>
## clone

`function` · `tracing_subscriber::fmt::time::Uptime::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Uptime
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::Uptime", "path": "Uptime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [117, 17], "end": [117, 22], "filename": "src/fmt/time/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/time/mod.rs:117`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-277785c797eabc46b90b6ed0"></a>
## default

`function` · `tracing_subscriber::fmt::time::Uptime::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::Uptime", "path": "Uptime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [122, 1], "end": [128, 2], "filename": "src/fmt/time/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/time/mod.rs:123`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2b51ae0b28235d8ba121d1c0"></a>
## eq

`function` · `tracing_subscriber::fmt::time::Uptime::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Uptime) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::Uptime", "path": "Uptime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [117, 34], "end": [117, 43], "filename": "src/fmt/time/mod.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/time/mod.rs:117`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-048692841b8526fa016f72e8"></a>
## fmt

`function` · `tracing_subscriber::fmt::time::Uptime::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::Uptime", "path": "Uptime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [117, 10], "end": [117, 15], "filename": "src/fmt/time/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/time/mod.rs:117`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d9ffb7fde7e5ae43d22ad53"></a>
## format_time

`function` · `tracing_subscriber::fmt::time::Uptime::format_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::Uptime", "path": "Uptime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [146, 1], "end": [151, 2], "filename": "src/fmt/time/mod.rs"}, "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}, "trait_path": "tracing_subscriber::fmt::time::FormatTime"}`

Source: `src/fmt/time/mod.rs:147`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-893f126c9bdd03eb1015006d"></a>
## from

`function` · `tracing_subscriber::fmt::time::Uptime::from` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn from(epoch: Instant) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::Uptime", "path": "Uptime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [130, 1], "end": [134, 2], "filename": "src/fmt/time/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "std::time::Instant", "path": "Instant"}}}], "constraints": []}}, "id": "core::convert::From", "path": "From"}, "trait_path": "core::convert::From"}`

Source: `src/fmt/time/mod.rs:131`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.
