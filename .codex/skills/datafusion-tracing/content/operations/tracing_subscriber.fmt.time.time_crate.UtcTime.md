# `tracing_subscriber::fmt::time::time_crate::UtcTime`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.time_crate.UtcTime.json).

<a id="op-fd59c5fd5cb1ede2912f0bcf"></a>
## UtcTime

`struct` · `tracing_subscriber::fmt::time::time_crate::UtcTime` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct UtcTime<F>
```

Source: `src/fmt/time/time_crate.rs:46`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Formats the current [UTC time] using a [formatter] from the [`time` crate].

To format the current [local time] instead, use the [`LocalTime`](../operations/tracing_subscriber.fmt.time.time_crate.LocalTime.md#op-aafaf1b0b948bafe945fd50f) type.

[local time]: time::OffsetDateTime::now_local
[UTC time]:     time::OffsetDateTime::now_utc
[formatter]:    time::formatting::Formattable
[`time` crate]: time

Unresolved upstream links (retained, not inferred): `time`, `time::formatting::Formattable`, `time::OffsetDateTime::now_local`, `time::OffsetDateTime::now_utc`.

<a id="op-ed247de099be19dc06e9cc4a"></a>
## clone

`function` · `tracing_subscriber::fmt::time::time_crate::UtcTime::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> UtcTime<F>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::UtcTime", "path": "UtcTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 10], "end": [45, 15], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/time/time_crate.rs:45`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8c534205ed3387532a603813"></a>
## default

`function` · `tracing_subscriber::fmt::time::time_crate::UtcTime::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::UtcTime", "path": "UtcTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "Formattable"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::default::Default", "path": "Default"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [304, 1], "end": [311, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/time/time_crate.rs:308`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9432af3925de5056aa4ee1fc"></a>
## fmt

`function` · `tracing_subscriber::fmt::time::time_crate::UtcTime::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::UtcTime", "path": "UtcTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [45, 17], "end": [45, 22], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/time/time_crate.rs:45`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7b88a902106981199cc92e56"></a>
## format_time

`function` · `tracing_subscriber::fmt::time::time_crate::UtcTime::format_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::UtcTime", "path": "UtcTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "Formattable"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [295, 1], "end": [302, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}, "trait_path": "tracing_subscriber::fmt::time::FormatTime"}`

Source: `src/fmt/time/time_crate.rs:299`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c63d5ce5195ce274dc47ab8f"></a>
## new

`function` · `tracing_subscriber::fmt::time::time_crate::UtcTime::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(format: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::UtcTime", "path": "UtcTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "Formattable"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [223, 1], "end": [293, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/time_crate.rs:290`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a formatter that formats the current [UTC time] using the
[`time` crate], with the provided provided format. The format may be any
type that implements the [`Formattable`] trait.

Typically, the format will be a format description string, or one of the
`time` crate's [well-known formats].

If the format description is statically known, then the
[`format_description!`] macro should be used. This is identical to the
[`time::format_description::parse`] method, but runs at compile-time,
failing  an error if the format description is invalid. If the desired format
is not known statically (e.g., a user is providing a format string), then the
[`time::format_description::parse`] method should be used. Note that this
method is fallible.

See the [`time` book] for details on the format description syntax.

# Examples

Using the [`format_description!`] macro:

```
use tracing_subscriber::fmt::{self, time::UtcTime};
use time::macros::format_description;

let timer = UtcTime::new(format_description!("[hour]:[minute]:[second]"));
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

Using the [`format_description!`] macro requires enabling the `time`
crate's "macros" feature flag.

Using [`time::format_description::parse`]:

```
use tracing_subscriber::fmt::{self, time::UtcTime};

let time_format = time::format_description::parse("[hour]:[minute]:[second]")
    .expect("format string should be valid!");
let timer = UtcTime::new(time_format);
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

Using a [well-known format][well-known formats] (this is equivalent to
[`UtcTime::rfc_3339`](../operations/tracing_subscriber.fmt.time.time_crate.UtcTime.md#op-97fdfa9160a061440ce17f3f)):

```
use tracing_subscriber::fmt::{self, time::UtcTime};

let timer = UtcTime::new(time::format_description::well_known::Rfc3339);
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

[UTC time]: time::OffsetDateTime::now_utc()
[`time` crate]: time
[`Formattable`]: time::formatting::Formattable
[well-known formats]: time::format_description::well_known
[`format_description!`]: https://docs.rs/time/0.3/time/macros/macro.format_description.html
[`time::format_description::parse`]: time::format_description::parse
[`time` book]: https://time-rs.github.io/book/api/format-description.html

Unresolved upstream links (retained, not inferred): `time`, `time::format_description::parse`, `time::OffsetDateTime::now_utc()`, `time::formatting::Formattable`, `time::format_description::well_known`.

<a id="op-97fdfa9160a061440ce17f3f"></a>
## rfc_3339

`function` · `tracing_subscriber::fmt::time::time_crate::UtcTime::rfc_3339` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn rfc_3339() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "time::format_description::well_known::rfc3339::Rfc3339", "path": "well_known::Rfc3339"}}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::UtcTime", "path": "UtcTime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [201, 1], "end": [221, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/time_crate.rs:218`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a formatter that formats the current [UTC time] in the
[RFC 3339] format, which is a subset of the [ISO 8601] timestamp format.

# Examples

```
use tracing_subscriber::fmt::{self, time};

let subscriber = tracing_subscriber::fmt()
    .with_timer(time::UtcTime::rfc_3339());
# drop(subscriber);
```

[local time]: time::OffsetDateTime::now_utc
[RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339
[ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601

Unresolved upstream links (retained, not inferred): `time::OffsetDateTime::now_utc`.
