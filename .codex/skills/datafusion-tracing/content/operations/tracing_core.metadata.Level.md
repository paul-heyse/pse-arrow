# `tracing_core::metadata::Level`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_core.metadata.Level.json).

<a id="op-f8804717be954252aadd54ff"></a>
## Level

`struct` · `tracing_core::metadata::Level` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
struct Level
```

Source: `src/metadata.rs:221`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Describes the level of verbosity of a span or event.

# Comparing Levels

`Level` implements the [`PartialOrd`] and [`Ord`] traits, allowing two
`Level`s to be compared to determine which is considered more or less
verbose. Levels which are more verbose are considered "greater than" levels
which are less verbose, with [`Level::ERROR`](../operations/tracing_core.metadata.Level.md#op-ffc4ae1325d1d396681bf6c9) considered the lowest, and
[`Level::TRACE`](../operations/tracing_core.metadata.Level.md#op-588e15fc4e6ffb3b405caa87) considered the highest.

For example:
```
use tracing_core::Level;

assert!(Level::TRACE > Level::DEBUG);
assert!(Level::ERROR < Level::WARN);
assert!(Level::INFO <= Level::DEBUG);
assert_eq!(Level::TRACE, Level::TRACE);
```

# Filtering

`Level`s are typically used to implement filtering that determines which
spans and events are enabled. Depending on the use case, more or less
verbose diagnostics may be desired. For example, when running in
development, [`DEBUG`]-level traces may be enabled by default. When running in
production, only [`INFO`]-level and lower traces might be enabled. Libraries
may include very verbose diagnostics at the [`DEBUG`] and/or [`TRACE`] levels.
Applications using those libraries typically chose to ignore those traces. However, when
debugging an issue involving said libraries, it may be useful to temporarily
enable the more verbose traces.

The [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98) type is provided to enable filtering traces by
verbosity. `Level`s can be compared against [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98)s, and
[`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98) has a variant for each `Level`, which compares analogously
to that level. In addition, [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98) adds a [`LevelFilter::OFF`](../operations/tracing_core.metadata.LevelFilter.md#op-5db5fa767073360a607dce55)
variant, which is considered "less verbose" than every other `Level`. This is
intended to allow filters to completely disable tracing in a particular context.

For example:
```
use tracing_core::{Level, LevelFilter};

assert!(LevelFilter::OFF < Level::TRACE);
assert!(LevelFilter::TRACE > Level::DEBUG);
assert!(LevelFilter::ERROR < Level::WARN);
assert!(LevelFilter::INFO <= Level::DEBUG);
assert!(LevelFilter::INFO >= Level::INFO);
```

## Examples

Below is a simple example of how a [`Subscriber`] could implement filtering through
a [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98). When a span or event is recorded, the [`Subscriber::enabled`] method
compares the span or event's `Level` against the configured [`LevelFilter`](../operations/tracing_core.metadata.LevelFilter.md#op-6d789785cc5ceea1e8c62c98).
The optional [`Subscriber::max_level_hint`] method can also be implemented to allow spans
and events above a maximum verbosity level to be skipped more efficiently,
often improving performance in short-lived programs.

```
use tracing_core::{span, Event, Level, LevelFilter, Subscriber, Metadata};
# use tracing_core::span::{Id, Record, Current};

#[derive(Debug)]
pub struct MySubscriber {
    /// The most verbose level that this subscriber will enable.
    max_level: LevelFilter,

    // ...
}

impl MySubscriber {
    /// Returns a new `MySubscriber` which will record spans and events up to
    /// `max_level`.
    pub fn with_max_level(max_level: LevelFilter) -> Self {
        Self {
            max_level,
            // ...
        }
    }
}
impl Subscriber for MySubscriber {
    fn enabled(&self, meta: &Metadata<'_>) -> bool {
        // A span or event is enabled if it is at or below the configured
        // maximum level.
        meta.level() <= &self.max_level
    }

    // This optional method returns the most verbose level that this
    // subscriber will enable. Although implementing this method is not
    // *required*, it permits additional optimizations when it is provided,
    // allowing spans and events above the max level to be skipped
    // more efficiently.
    fn max_level_hint(&self) -> Option<LevelFilter> {
        Some(self.max_level)
    }

    // Implement the rest of the subscriber...
    fn new_span(&self, span: &span::Attributes<'_>) -> span::Id {
        // ...
        # drop(span); Id::from_u64(1)
    }

    fn event(&self, event: &Event<'_>) {
        // ...
        # drop(event);
    }

    // ...
    # fn enter(&self, _: &Id) {}
    # fn exit(&self, _: &Id) {}
    # fn record(&self, _: &Id, _: &Record<'_>) {}
    # fn record_follows_from(&self, _: &Id, _: &Id) {}
}
```

It is worth noting that the `tracing-subscriber` crate provides [additional
APIs][envfilter] for performing more sophisticated filtering, such as
enabling different levels based on which module or crate a span or event is
recorded in.

[`DEBUG`]: Level::DEBUG
[`INFO`]: Level::INFO
[`TRACE`]: Level::TRACE
[`Subscriber::enabled`]: crate::subscriber::Subscriber::enabled
[`Subscriber::max_level_hint`]: crate::subscriber::Subscriber::max_level_hint
[`Subscriber`]: crate::subscriber::Subscriber
[envfilter]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html

Unresolved upstream links (retained, not inferred): ``PartialOrd``, ``Ord``.

<a id="op-f283497330efd0bc67767fd8"></a>
## DEBUG

`assoc_const` · `tracing_core::metadata::Level::DEBUG` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
DEBUG
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [509, 1], "end": [543, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:525`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "debug" level.

Designates lower priority information.

<a id="op-ffc4ae1325d1d396681bf6c9"></a>
## ERROR

`assoc_const` · `tracing_core::metadata::Level::ERROR` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
ERROR
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [509, 1], "end": [543, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:513`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "error" level.

Designates very serious errors.

<a id="op-08212f80b2f980595c730489"></a>
## Err

`assoc_type` · `tracing_core::metadata::Level::Err` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
Err
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [561, 1], "end": [583, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/metadata.rs:562`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c1c996cb5c1d8242fb1c2b62"></a>
## INFO

`assoc_const` · `tracing_core::metadata::Level::INFO` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
INFO
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [509, 1], "end": [543, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:521`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "info" level.

Designates useful information.

<a id="op-588e15fc4e6ffb3b405caa87"></a>
## TRACE

`assoc_const` · `tracing_core::metadata::Level::TRACE` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
TRACE
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [509, 1], "end": [543, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:529`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "trace" level.

Designates very low priority, often extremely verbose, information.

<a id="op-71b850c1234ee19d8a90a7ae"></a>
## WARN

`assoc_const` · `tracing_core::metadata::Level::WARN` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
WARN
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [509, 1], "end": [543, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:517`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

The "warn" level.

Designates hazardous situations.

<a id="op-e1f5f11f1535f4344fcdabf6"></a>
## as_str

`function` · `tracing_core::metadata::Level::as_str` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn as_str(&self) -> &'static str
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [509, 1], "end": [543, 2], "filename": "src/metadata.rs"}, "trait": null, "trait_path": null}`

Source: `src/metadata.rs:534`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

Returns the string representation of the `Level`.

This returns the same string as the `fmt::Display` implementation.

<a id="op-1b822c56ae9993f10ca07597"></a>
## clone

`function` · `tracing_core::metadata::Level::clone` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Level
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [220, 16], "end": [220, 21], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metadata.rs:220`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-32cb1f9dd55e6b5eae52d178"></a>
## cmp

`function` · `tracing_core::metadata::Level::cmp` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn cmp(&self, other: &Self) -> cmp::Ordering
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [945, 1], "end": [950, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::Ord", "path": "Ord"}, "trait_path": "core::cmp::Ord"}`

Source: `src/metadata.rs:947`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c592149ec30becdfaeb878e1"></a>
## eq

`function` · `tracing_core::metadata::Level::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [220, 30], "end": [220, 39], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metadata.rs:220`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cb4219b4291c33a859ba91de"></a>
## eq

`function` · `tracing_core::metadata::Level::eq` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [911, 1], "end": [916, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}}], "constraints": []}}, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/metadata.rs:913`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c6d79893d33e29023b79f5d3"></a>
## fmt

`function` · `tracing_core::metadata::Level::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [545, 1], "end": [555, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::fmt::Display", "path": "Display"}, "trait_path": "core::fmt::Display"}`

Source: `src/metadata.rs:546`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c738dd20122f78015a9549b2"></a>
## fmt

`function` · `tracing_core::metadata::Level::fmt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [220, 23], "end": [220, 28], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metadata.rs:220`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-45e2c182bb421e790d1f4c34"></a>
## from_str

`function` · `tracing_core::metadata::Level::from_str` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn from_str(s: &str) -> Result<Self, ParseLevelError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [561, 1], "end": [583, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::str::traits::FromStr", "path": "FromStr"}, "trait_path": "core::str::traits::FromStr"}`

Source: `src/metadata.rs:563`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-073eefccc2b001abca59964d"></a>
## ge

`function` · `tracing_core::metadata::Level::ge` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn ge(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [952, 1], "end": [977, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:974`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d2c4c0cddd2e4a5739679003"></a>
## ge

`function` · `tracing_core::metadata::Level::ge` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn ge(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [918, 1], "end": [943, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:940`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1375a734e3a2b154c9d4d2cd"></a>
## gt

`function` · `tracing_core::metadata::Level::gt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn gt(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [918, 1], "end": [943, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:935`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-95fe96e369ecf31b328ee526"></a>
## gt

`function` · `tracing_core::metadata::Level::gt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn gt(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [952, 1], "end": [977, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:969`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a7d35b669cf3e0d7c664e605"></a>
## hash

`function` · `tracing_core::metadata::Level::hash` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn hash<__H: hash::Hasher>(&self, state: &mut __H)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [220, 45], "end": [220, 49], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::hash::Hash", "path": "Hash"}, "trait_path": "core::hash::Hash"}`

Source: `src/metadata.rs:220`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ac4fced17108ab94b822e516"></a>
## le

`function` · `tracing_core::metadata::Level::le` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn le(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [918, 1], "end": [943, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:930`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b95eb13ace775e34d3e61c45"></a>
## le

`function` · `tracing_core::metadata::Level::le` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn le(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [952, 1], "end": [977, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:964`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a2716cb968e2ee2132ccff67"></a>
## lt

`function` · `tracing_core::metadata::Level::lt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn lt(&self, other: &LevelFilter) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [952, 1], "end": [977, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:959`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a495979b5af78335942ed289"></a>
## lt

`function` · `tracing_core::metadata::Level::lt` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn lt(&self, other: &Level) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [918, 1], "end": [943, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:925`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-414c47d49a807ec12bb5cfc5"></a>
## partial_cmp

`function` · `tracing_core::metadata::Level::partial_cmp` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &LevelFilter) -> Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [952, 1], "end": [977, 2], "filename": "src/metadata.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_core::metadata::LevelFilter", "path": "LevelFilter"}}}], "constraints": []}}, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:954`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a04c39f8fb87c35b4741ef07"></a>
## partial_cmp

`function` · `tracing_core::metadata::Level::partial_cmp` · tracing-core 0.1.36
Reachability: `supported`.  Capture: hosted.

```rust
fn partial_cmp(&self, other: &Level) -> Option<cmp::Ordering>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_core::metadata::Level", "path": "Level"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [918, 1], "end": [943, 2], "filename": "src/metadata.rs"}, "trait": {"args": null, "id": "core::cmp::PartialOrd", "path": "PartialOrd"}, "trait_path": "core::cmp::PartialOrd"}`

Source: `src/metadata.rs:920`. [Exact documentation build](https://docs.rs/crate/tracing-core/0.1.36/json).

No upstream documentation on this item; consult its owner/trait contract.
