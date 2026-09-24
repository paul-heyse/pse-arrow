# `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.chrono_crate.ChronoLocal.json).

<a id="op-6d053f6f706f9b40e3ded0c2"></a>
## ChronoLocal

`struct` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct ChronoLocal
```

Source: `src/fmt/time/chrono_crate.rs:19`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Formats [local time]s and [UTC time]s with `FormatTime` implementations
that use the [`chrono` crate].

[local time]: [`chrono::offset::Local`]
[UTC time]: [`chrono::offset::Utc`]
[`chrono` crate]: [`chrono`]
Formats the current [local time] using a [formatter] from the [`chrono`] crate.

[local time]: chrono::Local::now()
[formatter]: chrono::format

Unresolved upstream links (retained, not inferred): ``chrono``, `chrono::format`.

<a id="op-73babfcb42a402fe6601a4a3"></a>
## clone

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ChronoLocal
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoLocal", "path": "ChronoLocal"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 17], "end": [18, 22], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/time/chrono_crate.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3e26b899dcc168ef47558b18"></a>
## default

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> ChronoLocal
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoLocal", "path": "ChronoLocal"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 39], "end": [18, 46], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/time/chrono_crate.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f44a08028cbd5ff9ca3a172a"></a>
## eq

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &ChronoLocal) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoLocal", "path": "ChronoLocal"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 28], "end": [18, 37], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/time/chrono_crate.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-054e9411219b2c3b7b580734"></a>
## fmt

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoLocal", "path": "ChronoLocal"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [18, 10], "end": [18, 15], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/time/chrono_crate.rs:18`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b1d82c220fb1f12cdfafbd5"></a>
## format_time

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal::format_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_time(&self, w: &mut Writer<'_>) -> alloc::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoLocal", "path": "ChronoLocal"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 1], "end": [62, 2], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}, "trait_path": "tracing_subscriber::fmt::time::FormatTime"}`

Source: `src/fmt/time/chrono_crate.rs:46`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-df627a9f740b9f8d3941893d"></a>
## new

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(format_string: String) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoLocal", "path": "ChronoLocal"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [23, 1], "end": [43, 2], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/chrono_crate.rs:38`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Format the time using the given format string.

See [`chrono::format::strftime`] for details on the supported syntax.

Unresolved upstream links (retained, not inferred): ``chrono::format::strftime``.

<a id="op-dce3bd735f77902a7f91067e"></a>
## rfc_3339

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoLocal::rfc_3339` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn rfc_3339() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoLocal", "path": "ChronoLocal"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [23, 1], "end": [43, 2], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/chrono_crate.rs:29`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Format the time using the [`RFC 3339`] format
(a subset of [`ISO 8601`]).

[`RFC 3339`]: https://tools.ietf.org/html/rfc3339
[`ISO 8601`]: https://en.wikipedia.org/wiki/ISO_8601
