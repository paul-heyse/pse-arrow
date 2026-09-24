# `tracing_subscriber::fmt::format::Format`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.format.Format.json).

<a id="op-758341dc152c0b1f07c0c605"></a>
## Format

`struct` · `tracing_subscriber::fmt::format::Format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Format<F = Full, T = super::time::SystemTime>
```

Source: `src/fmt/format/mod.rs:407`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A pre-configured event formatter.

You will usually want to use this as the [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) for a [`FmtSubscriber`].

The default logging format, [`Full`](../operations/tracing_subscriber.fmt.format.Full.md#op-b6feb895c3a5317d8f8397fd) includes all fields in each event and its containing
spans. The [`Compact`](../operations/tracing_subscriber.fmt.format.Compact.md#op-b150ae7b416c1fdd163391ea) logging format is intended to produce shorter log
lines; it displays each event's fields, along with fields from the current
span context, but other information is abbreviated. The [`Pretty`](../operations/tracing_subscriber.fmt.format.pretty.Pretty.md#op-9ae7afec101d61be119e29f4) logging
format is an extra-verbose, multi-line human-readable logging format
intended for use in development.

[`FmtSubscriber`]: super::Subscriber

<a id="op-e6f8de0db8c3d74566c73efe"></a>
## clone

`function` · `tracing_subscriber::fmt::format::Format::clone` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::clone::Clone", "path": "$crate::clone::Clone"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [406, 17], "end": [406, 22], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/fmt/format/mod.rs:406`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-40bf9ca3ea1346cc960e4f20"></a>
## compact

`function` · `tracing_subscriber::fmt::format::Format::compact` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn compact(self) -> Format<Compact, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:634`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Use a less verbose output format.

See [`Compact`](../operations/tracing_subscriber.fmt.format.Compact.md#op-b150ae7b416c1fdd163391ea).

<a id="op-f75b1eda52703b64d643663e"></a>
## default

`function` · `tracing_subscriber::fmt::format::Format::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Full", "path": "Full"}}}, {"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::time::SystemTime", "path": "super::time::SystemTime"}}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [613, 1], "end": [628, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/format/mod.rs:614`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3d8f7853d7c5a5865db13dd2"></a>
## flatten_event

`function` · `tracing_subscriber::fmt::format::Format::flatten_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flatten_event(self, flatten_event: bool) -> Format<Json, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [883, 1], "end": [920, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:894`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Use the full JSON format with the event's event fields flattened.

# Example Output

```ignore,json
{"timestamp":"Feb 20 11:28:15.096","level":"INFO","target":"mycrate", "message":"some message", "key": "value"}
```
See [`Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c).

<a id="op-65c70f1b00b1a3a9177fef1d"></a>
## fmt

`function` · `tracing_subscriber::fmt::format::Format::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [406, 10], "end": [406, 15], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/format/mod.rs:406`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7cd9b74323870acc985eeba4"></a>
## format_event

`function` · `tracing_subscriber::fmt::format::Format::format_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_event(&self, ctx: &FmtContext<'_, S, N>, writer: Writer<'_>, event: &Event<'_>) -> fmt::Result where S: Subscriber + for<'a> LookupSpan<'a>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "super::Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'lookup"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'lookup"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [215, 1], "end": [332, 2], "filename": "src/fmt/format/json.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}, "trait_path": "tracing_subscriber::fmt::format::FormatEvent"}`

Source: `src/fmt/format/json.rs:221`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b05b84c95f93299199bb4abf"></a>
## format_event

`function` · `tracing_subscriber::fmt::format::Format::format_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_event(&self, ctx: &FmtContext<'_, S, N>, writer: Writer<'_>, event: &Event<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Full", "path": "Full"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [922, 1], "end": [1048, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}, "trait_path": "tracing_subscriber::fmt::format::FormatEvent"}`

Source: `src/fmt/format/mod.rs:928`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-dd4ecf5e7b9e08984707b029"></a>
## format_event

`function` · `tracing_subscriber::fmt::format::Format::format_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_event(&self, ctx: &FmtContext<'_, C, N>, writer: Writer<'_>, event: &Event<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::pretty::Pretty", "path": "Pretty"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "C"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "C"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [169, 1], "end": [335, 2], "filename": "src/fmt/format/pretty.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "C"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}, "trait_path": "tracing_subscriber::fmt::format::FormatEvent"}`

Source: `src/fmt/format/pretty.rs:175`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e3f624059fa7e913d474cb2b"></a>
## format_event

`function` · `tracing_subscriber::fmt::format::Format::format_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn format_event(&self, ctx: &FmtContext<'_, S, N>, writer: Writer<'_>, event: &Event<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::Compact", "path": "Compact"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_subscriber::fmt::time::FormatTime", "path": "FormatTime"}}}], "generic_params": [], "type": {"generic": "T"}}}]}, "is_negative": false, "span": {"begin": [1050, 1], "end": [1180, 2], "filename": "src/fmt/format/mod.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}, "trait_path": "tracing_subscriber::fmt::format::FormatEvent"}`

Source: `src/fmt/format/mod.rs:1056`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c8a808ae7c6eb679402c62e2"></a>
## json

`function` · `tracing_subscriber::fmt::format::Format::json` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn json(self) -> Format<Json, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:704`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Use the full JSON format.

The full format includes fields from all entered spans.

# Example Output

```ignore,json
{"timestamp":"Feb 20 11:28:15.096","level":"INFO","target":"mycrate","fields":{"message":"some message", "key": "value"}}
```

# Options

- [`Format::flatten_event`](../operations/tracing_subscriber.fmt.format.Format.md#op-3d8f7853d7c5a5865db13dd2) can be used to enable flattening event fields into the root
  object.

<a id="op-e6a3dcc67ff74a46968fd54f"></a>
## pretty

`function` · `tracing_subscriber::fmt::format::Format::pretty` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn pretty(self) -> Format<Pretty, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:673`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Use an excessively pretty, human-readable output format.

See [`Pretty`](../operations/tracing_subscriber.fmt.format.pretty.Pretty.md#op-9ae7afec101d61be119e29f4).

Note that this requires the `"ansi"` feature to be enabled.

# Options

[`Format::with_ansi`](../operations/tracing_subscriber.fmt.format.Format.md#op-8ef0c9e7906381bf9f64f3da) can be used to disable ANSI terminal escape codes (which enable
formatting such as colors, bold, italic, etc) in event formatting. However, a field
formatter must be manually provided to avoid ANSI in the formatting of parent spans, like
so:

```
# use tracing_subscriber::fmt::format;
tracing_subscriber::fmt()
   .pretty()
   .with_ansi(false)
   .fmt_fields(format::PrettyFields::new().with_ansi(false))
   // ... other settings ...
   .init();
```

<a id="op-8ef0c9e7906381bf9f64f3da"></a>
## with_ansi

`function` · `tracing_subscriber::fmt::format::Format::with_ansi` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_ansi(self, ansi: bool) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:765`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Enable ANSI terminal colors for formatted output.

<a id="op-8164f71411d14a11d6dedac1"></a>
## with_current_span

`function` · `tracing_subscriber::fmt::format::Format::with_current_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current_span(self, display_current_span: bool) -> Format<Json, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [883, 1], "end": [920, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:905`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the formatter will include the current span in
formatted events.

See [`format::Json`][Json](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c)

<a id="op-aca173cb26cad44ca8546155"></a>
## with_file

`function` · `tracing_subscriber::fmt::format::Format::with_file` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_file(self, display_filename: bool) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:814`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's [source code file path][file] is
displayed.

[file]: tracing_core::Metadata::file

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::file`.

<a id="op-756a8b81b4d1f660c2aade14"></a>
## with_level

`function` · `tracing_subscriber::fmt::format::Format::with_level` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_level(self, display_level: bool) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:781`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's level is displayed.

<a id="op-0fb793da8b61eed27be6690d"></a>
## with_line_number

`function` · `tracing_subscriber::fmt::format::Format::with_line_number` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_line_number(self, display_line_number: bool) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:825`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's [source code line number][line] is
displayed.

[line]: tracing_core::Metadata::line

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::line`.

<a id="op-2d60fa86bc65d414a9b5bc38"></a>
## with_source_location

`function` · `tracing_subscriber::fmt::format::Format::with_source_location` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_source_location(self, display_location: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:837`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the source code location from which an event
originated is displayed.

This is equivalent to calling [`Format::with_file`](../operations/tracing_subscriber.fmt.format.Format.md#op-aca173cb26cad44ca8546155) and
[`Format::with_line_number`](../operations/tracing_subscriber.fmt.format.Format.md#op-0fb793da8b61eed27be6690d) with the same value.

<a id="op-d953df2b67297125081d626e"></a>
## with_span_list

`function` · `tracing_subscriber::fmt::format::Format::with_span_list` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_list(self, display_span_list: bool) -> Format<Json, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [883, 1], "end": [920, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:916`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the formatter will include a list (from root to
leaf) of all currently entered spans in formatted events.

See [`format::Json`][Json](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c)

<a id="op-19641fbf22b435e1c2d5a237"></a>
## with_target

`function` · `tracing_subscriber::fmt::format::Format::with_target` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_target(self, display_target: bool) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:773`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's target is displayed.

<a id="op-954b1d1774917a7ddfe02e5d"></a>
## with_thread_ids

`function` · `tracing_subscriber::fmt::format::Format::with_thread_ids` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_thread_ids(self, display_thread_id: bool) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:792`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the [thread ID] of the current thread is displayed
when formatting events.

[thread ID]: std::thread::ThreadId

Unresolved upstream links (retained, not inferred): `std::thread::ThreadId`.

<a id="op-2468db3e28fbbb1ff099a86d"></a>
## with_thread_names

`function` · `tracing_subscriber::fmt::format::Format::with_thread_names` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_thread_names(self, display_thread_name: bool) -> Format<F, T>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:803`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the [name] of the current thread is displayed
when formatting events.

[name]: std::thread#naming-threads

Unresolved upstream links (retained, not inferred): `std::thread#naming-threads`.

<a id="op-c7437b73527d679b1f477f1b"></a>
## with_timer

`function` · `tracing_subscriber::fmt::format::Format::with_timer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_timer<T2>(self, timer: T2) -> Format<F, T2>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:733`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Use the given [`timer`] for log message timestamps.

See [`time` module] for the provided timer implementations.

Note that using the `"time"` feature flag enables the
additional time formatters [`UtcTime`] and [`LocalTime`], which use the
[`time` crate] to provide more sophisticated timestamp formatting
options.

[`timer`]: super::time::FormatTime
[`time` module]: mod@super::time
[`UtcTime`]: super::time::UtcTime
[`LocalTime`]: super::time::LocalTime
[`time` crate]: https://docs.rs/time/0.3

<a id="op-9baf46c7256de0ca81a482b8"></a>
## without_time

`function` · `tracing_subscriber::fmt::format::Format::without_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn without_time(self) -> Format<F, ()>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "F"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "Format"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [630, 1], "end": [879, 2], "filename": "src/fmt/format/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/format/mod.rs:749`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Do not emit timestamps with log messages.
