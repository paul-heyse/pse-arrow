# `tracing_subscriber::fmt::time::time_crate::LocalTime`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.time.time_crate.LocalTime.json).

<a id="op-aafaf1b0b948bafe945fd50f"></a>
## LocalTime

`struct` · `tracing_subscriber::fmt::time::time_crate::LocalTime` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct LocalTime<F>
```

Source: `src/fmt/time/time_crate.rs:32`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Formats the current [local time] using a [formatter] from the [`time` crate].

To format the current [UTC time] instead, use the [`UtcTime`](../operations/tracing_subscriber.fmt.time.time_crate.UtcTime.md#op-fd59c5fd5cb1ede2912f0bcf) type.

<div class="example-wrap" style="display:inline-block">
<pre class="compile_fail" style="white-space:normal;font:inherit;">
    <strong>Warning</strong>: The <a href = "https://docs.rs/time/0.3/time/"><code>time</code>
    crate</a> must be compiled with <code>--cfg unsound_local_offset</code> in order to use
    local timestamps. When this cfg is not enabled, local timestamps cannot be recorded, and
    events will be logged without timestamps.

   Alternatively, [`OffsetTime`](../operations/tracing_subscriber.fmt.time.time_crate.OffsetTime.md#op-4aaabac2cead751c68b88774) can log with a local offset if it is initialized early.

   See the <a href="https://docs.rs/time/0.3.4/time/#feature-flags"><code>time</code>
   documentation</a> for more details.
</pre></div>

[local time]: time::OffsetDateTime::now_local
[UTC time]:     time::OffsetDateTime::now_utc
[formatter]:    time::formatting::Formattable
[`time` crate]: time

Unresolved upstream links (retained, not inferred): `time`, `time::OffsetDateTime::now_local`, `time::formatting::Formattable`, `time::OffsetDateTime::now_utc`.

<a id="op-36287f5c48bd1ee6103cc6d5"></a>
## clone

`function` · `tracing_subscriber::fmt::time::time_crate::LocalTime::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> LocalTime<F>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::LocalTime", "path": "LocalTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 10], "end": [26, 15], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/time/time_crate.rs:26`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f4bb8a00e04ba51274097354"></a>
## default

`function` · `tracing_subscriber::fmt::time::time_crate::LocalTime::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::LocalTime", "path": "LocalTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "Formattable"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::default::Default", "path": "Default"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [190, 1], "end": [197, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/time/time_crate.rs:194`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ff7b03501404b4f64e0b7c8c"></a>
## fmt

`function` · `tracing_subscriber::fmt::time::time_crate::LocalTime::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::LocalTime", "path": "LocalTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [26, 17], "end": [26, 22], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/time/time_crate.rs:26`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a5ba45dcac1ea07ea568eb51"></a>
## format_time

`function` · `tracing_subscriber::fmt::time::time_crate::LocalTime::format_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_time(&self, w: &mut Writer<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::LocalTime", "path": "LocalTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "Formattable"}}}], "generic_params": [], "type": {"generic": "F"}}}]}, "is_negative": false, "span": {"begin": [179, 1], "end": [187, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}, "trait_path": "tracing_subscriber::fmt::time::FormatTime"}`

Source: `src/fmt/time/time_crate.rs:183`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3b4ca5d5fd8a2c16656b45a6"></a>
## new

`function` · `tracing_subscriber::fmt::time::time_crate::LocalTime::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new(format: F) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::LocalTime", "path": "LocalTime"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "time::formatting::formattable::Formattable", "path": "Formattable"}}}], "default": null, "is_synthetic": false}}, "name": "F"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [93, 1], "end": [176, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/time_crate.rs:173`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a formatter that formats the current [local time] using the
[`time` crate] with the provided provided format. The format may be any
type that implements the [`Formattable`] trait.


<div class="example-wrap" style="display:inline-block">
<pre class="compile_fail" style="white-space:normal;font:inherit;">
    <strong>Warning</strong>: The <a href = "https://docs.rs/time/0.3/time/">
    <code>time</code> crate</a> must be compiled with <code>--cfg
    unsound_local_offset</code> in order to use local timestamps. When this
    cfg is not enabled, local timestamps cannot be recorded, and
    events will be logged without timestamps.

   See the <a href="https://docs.rs/time/0.3.4/time/#feature-flags">
   <code>time</code> documentation</a> for more details.
</pre></div>

Typically, the format will be a format description string, or one of the
`time` crate's [well-known formats].

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
use tracing_subscriber::fmt::{self, time::LocalTime};
use time::macros::format_description;

let timer = LocalTime::new(format_description!("[hour]:[minute]:[second]"));
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

Using [`time::format_description::parse`]:

```
use tracing_subscriber::fmt::{self, time::LocalTime};

let time_format = time::format_description::parse("[hour]:[minute]:[second]")
    .expect("format string should be valid!");
let timer = LocalTime::new(time_format);
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

Using the [`format_description!`] macro requires enabling the `time`
crate's "macros" feature flag.

Using a [well-known format][well-known formats] (this is equivalent to
[`LocalTime::rfc_3339`](../operations/tracing_subscriber.fmt.time.time_crate.LocalTime.md#op-dcc01e7454f132b35f9d7d2a)):

```
use tracing_subscriber::fmt::{self, time::LocalTime};

let timer = LocalTime::new(time::format_description::well_known::Rfc3339);
let subscriber = tracing_subscriber::fmt()
    .with_timer(timer);
# drop(subscriber);
```

[local time]: time::OffsetDateTime::now_local()
[`time` crate]: time
[`Formattable`]: time::formatting::Formattable
[well-known formats]: time::format_description::well_known
[`format_description!`]: https://docs.rs/time/0.3/time/macros/macro.format_description.html
[`time::format_description::parse`]: time::format_description::parse()
[`time` book]: https://time-rs.github.io/book/api/format-description.html

Unresolved upstream links (retained, not inferred): `time`, `time::OffsetDateTime::now_local()`, `time::format_description::parse()`, `time::formatting::Formattable`, `time::format_description::well_known`.

<a id="op-dcc01e7454f132b35f9d7d2a"></a>
## rfc_3339

`function` · `tracing_subscriber::fmt::time::time_crate::LocalTime::rfc_3339` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn rfc_3339() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "time::format_description::well_known::rfc3339::Rfc3339", "path": "well_known::Rfc3339"}}}], "constraints": []}}, "id": "tracing_subscriber::fmt::time::time_crate::LocalTime", "path": "LocalTime"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [70, 1], "end": [90, 2], "filename": "src/fmt/time/time_crate.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/time/time_crate.rs:87`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a formatter that formats the current [local time] in the
[RFC 3339] format (a subset of the [ISO 8601] timestamp format).

# Examples

```
use tracing_subscriber::fmt::{self, time};

let subscriber = tracing_subscriber::fmt()
    .with_timer(time::LocalTime::rfc_3339());
# drop(subscriber);
```

[local time]: time::OffsetDateTime::now_local
[RFC 3339]: https://datatracker.ietf.org/doc/html/rfc3339
[ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601

Unresolved upstream links (retained, not inferred): `time::OffsetDateTime::now_local`.
