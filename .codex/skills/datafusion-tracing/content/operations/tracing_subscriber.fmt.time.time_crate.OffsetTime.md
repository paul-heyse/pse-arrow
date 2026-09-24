# `tracing_subscriber::fmt::time::time_crate::OffsetTime`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.time_crate.OffsetTime.json).

<a id="op-4aaabac2cead751c68b88774"></a>
## OffsetTime

`struct` · `tracing_subscriber::fmt::time::time_crate::OffsetTime` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct OffsetTime<F>
```

Source: `src/fmt/time/time_crate.rs:62`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Formats the current time using a fixed offset and a [formatter] from the [`time` crate].

This is typically used as an alternative to [`LocalTime`](../operations/tracing_subscriber.fmt.time.time_crate.LocalTime.md#op-aafaf1b0b948bafe945fd50f). `LocalTime` determines the offset
every time it formats a message, which may be unsound or fail. With `OffsetTime`, the offset is
determined once. This makes it possible to do so while the program is still single-threaded and
handle any errors. However, this also means the offset cannot change while the program is
running (the offset will not change across DST changes).

[formatter]: time::formatting::Formattable
[`time` crate]: time

Unresolved upstream links (retained, not inferred): `time::formatting::Formattable`, `time`.

<a id="op-2a28f9a29aa8574e0d7dc22e"></a>
## clone

`function` · `tracing_subscriber::fmt::time::time_crate::OffsetTime::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> OffsetTime<F>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::OffsetTime", "path": "OffsetTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [60, 10], "end": [60, 15], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/time/time_crate.rs:60`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-671e160d536ec40ede1109e9"></a>
## fmt

`function` · `tracing_subscriber::fmt::time::time_crate::OffsetTime::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::OffsetTime", "path": "OffsetTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [60, 17], "end": [60, 22], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/time/time_crate.rs:60`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-9502646be56311311ba1f3cb"></a>
## format_time

`function` · `tracing_subscriber::fmt::time::time_crate::OffsetTime::format_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::OffsetTime", "path": "OffsetTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "time::formatting::Formattable"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [451, 1], "end": [459, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}, "trait_path": "tracing_subscriber::fmt::time::FormatTime"}`

Source: `src/fmt/time/time_crate.rs:455`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-57e77b3f984a16a533160ac3"></a>
## local_rfc_3339

`function` · `tracing_subscriber::fmt::time::time_crate::OffsetTime::local_rfc_3339` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn local_rfc_3339() -> Result<Self, time::error::IndeterminateOffset>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "time::format_description::well_known::rfc3339::Rfc3339", "path": "well_known::Rfc3339"}}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::OffsetTime", "path": "OffsetTime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [316, 1], "end": [369, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/time_crate.rs:363`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a formatter that formats the current time using the [local time offset] in the [RFC
3339] format (a subset of the [ISO 8601] timestamp format).

Returns an error if the local time offset cannot be determined. This typically occurs in
multithreaded programs. To avoid this problem, initialize `OffsetTime` before forking
threads. When using Tokio, this means initializing `OffsetTime` before the Tokio runtime.

# Examples

```
use tracing_subscriber::fmt::{self, time};

let subscriber = tracing_subscriber::fmt()
    .with_timer(time::OffsetTime::local_rfc_3339().expect("could not get local offset!"));
# drop(subscriber);
```

Using `OffsetTime` with Tokio:

```
use tracing_subscriber::fmt::time::OffsetTime;

#[tokio::main]
async fn run() {
    tracing::info!("runtime initialized");

    // At this point the Tokio runtime is initialized, and we can use both Tokio and Tracing
    // normally.
}

fn main() {
    // Because we need to get the local offset before Tokio spawns any threads, our `main`
    // function cannot use `tokio::main`.
    tracing_subscriber::fmt()
        .with_timer(OffsetTime::local_rfc_3339().expect("could not get local time offset"))
        .init();

    // Even though `run` is written as an `async fn`, because we used `tokio::main` on it
    // we can call it as a synchronous function.
    run();
}
```

[local time offset]: time::UtcOffset::current_local_offset
[RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339
[ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601

Unresolved upstream links (retained, not inferred): `time::UtcOffset::current_local_offset`.

<a id="op-2f7877d7e05117a90d9d27bc"></a>
## new

`function` · `tracing_subscriber::fmt::time::time_crate::OffsetTime::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(offset: time::UtcOffset, format: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::OffsetTime", "path": "OffsetTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "time::formatting::Formattable"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [371, 1], "end": [449, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/time_crate.rs:446`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a formatter that formats the current time using the [`time` crate] with the provided
provided format and [timezone offset]. The format may be any type that implements the
[`Formattable`] trait.


Typically, the offset will be the [local offset], and format will be a format description
string, or one of the `time` crate's [well-known formats].

If the format description is statically known, then the
[`format_description!`] macro should be used. This is identical to the
[`time::format_description::parse`] method, but runs at compile-time,
throwing an error if the format description is invalid. If the desired format
is not known statically (e.g., a user is providing a format string), then the
[`time::format_description::parse`] method should be used. Note that this
method is fallible.

See the [`time` book] for details on the format description syntax.

# Examples

Using the [`format_description!`] macro:

```
use tracing_subscriber::fmt::{self, time::OffsetTime};
use time::macros::format_description;
use time::UtcOffset;

let offset = UtcOffset::current_local_offset().expect("should get local offset!");
let timer = OffsetTime::new(offset, format_description!("[hour]:[minute]:[second]"));
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

Using [`time::format_description::parse`]:

```
use tracing_subscriber::fmt::{self, time::OffsetTime};
use time::UtcOffset;

let offset = UtcOffset::current_local_offset().expect("should get local offset!");
let time_format = time::format_description::parse("[hour]:[minute]:[second]")
    .expect("format string should be valid!");
let timer = OffsetTime::new(offset, time_format);
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

Using the [`format_description!`] macro requires enabling the `time`
crate's "macros" feature flag.

Using a [well-known format][well-known formats] (this is equivalent to
[`OffsetTime::local_rfc_3339`](../operations/tracing_subscriber.fmt.time.time_crate.OffsetTime.md#op-57e77b3f984a16a533160ac3)):

```
use tracing_subscriber::fmt::{self, time::OffsetTime};
use time::UtcOffset;

let offset = UtcOffset::current_local_offset().expect("should get local offset!");
let timer = OffsetTime::new(offset, time::format_description::well_known::Rfc3339);
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

[`time` crate]: time
[timezone offset]: time::UtcOffset
[`Formattable`]: time::formatting::Formattable
[local offset]: time::UtcOffset::current_local_offset()
[well-known formats]: time::format_description::well_known
[`format_description!`]: https://docs.rs/time/0.3/time/macros/macro.format_description.html
[`time::format_description::parse`]: time::format_description::parse
[`time` book]: https://time-rs.github.io/book/api/format-description.html

Unresolved upstream links (retained, not inferred): `time`, `time::UtcOffset`, `time::format_description::parse`, `time::formatting::Formattable`, `time::UtcOffset::current_local_offset()`, `time::format_description::well_known`.
