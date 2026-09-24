# `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.chrono_crate.ChronoUtc.json).

<a id="op-1a9e8e86151e34f219696556"></a>
## ChronoUtc

`struct` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct ChronoUtc
```

Source: `src/fmt/time/chrono_crate.rs:70`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Formats the current [UTC time] using a [formatter] from the [`chrono`] crate.

[UTC time]: chrono::Utc::now()
[formatter]: chrono::format

Unresolved upstream links (retained, not inferred): `chrono::Utc::now()`, ``chrono``, `chrono::format`.

<a id="op-45a3b60c9992c781f8bec5b1"></a>
## clone

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> ChronoUtc
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoUtc", "path": "ChronoUtc"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [69, 17], "end": [69, 22], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/time/chrono_crate.rs:69`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9b36b95a398fb7ede06f53f0"></a>
## default

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> ChronoUtc
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoUtc", "path": "ChronoUtc"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [69, 39], "end": [69, 46], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/time/chrono_crate.rs:69`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1e4eb58b0fb2fe5597ee0b6a"></a>
## eq

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc::eq` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &ChronoUtc) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoUtc", "path": "ChronoUtc"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [69, 28], "end": [69, 37], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/fmt/time/chrono_crate.rs:69`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1e50e6d05ae54c288baec08c"></a>
## fmt

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoUtc", "path": "ChronoUtc"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [69, 10], "end": [69, 15], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/time/chrono_crate.rs:69`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c0e906370320180cdd917bd1"></a>
## format_time

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc::format_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_time(&self, w: &mut Writer<'_>) -> alloc::fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoUtc", "path": "ChronoUtc"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [96, 1], "end": [104, 2], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}, "trait_path": "tracing_subscriber::fmt::time::FormatTime"}`

Source: `src/fmt/time/chrono_crate.rs:97`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9972b8d21be9479412a59c18"></a>
## new

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(format_string: String) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoUtc", "path": "ChronoUtc"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [74, 1], "end": [94, 2], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/chrono_crate.rs:89`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Format the time using the given format string.

See [`chrono::format::strftime`] for details on the supported syntax.

Unresolved upstream links (retained, not inferred): ``chrono::format::strftime``.

<a id="op-292e36f21797d8c02d0c8741"></a>
## rfc_3339

`function` · `tracing_subscriber::fmt::time::chrono_crate::ChronoUtc::rfc_3339` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn rfc_3339() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::chrono_crate::ChronoUtc", "path": "ChronoUtc"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [74, 1], "end": [94, 2], "filename": "src/fmt/time/chrono_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/chrono_crate.rs:80`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Format the time using the [`RFC 3339`] format
(a subset of [`ISO 8601`]).

[`RFC 3339`]: https://tools.ietf.org/html/rfc3339
[`ISO 8601`]: https://en.wikipedia.org/wiki/ISO_8601
