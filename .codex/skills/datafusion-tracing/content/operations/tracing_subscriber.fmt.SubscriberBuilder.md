# `tracing_subscriber::fmt::SubscriberBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.SubscriberBuilder.json).

<a id="op-38fcd8fc52ca33acc34d54e9"></a>
## SubscriberBuilder

`struct` · `tracing_subscriber::fmt::SubscriberBuilder` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct SubscriberBuilder<N = format::DefaultFields, E = format::Format<format::Full>, F = filter::LevelFilter, W = fn() -> io::Stdout>
```

Source: `src/fmt/mod.rs:250`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Configures and constructs `Subscriber`s.

<a id="op-db9ad2e2fa06304d5a4cf40a"></a>
## compact

`function` · `tracing_subscriber::fmt::SubscriberBuilder::compact` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn compact(self) -> SubscriberBuilder<N, format::Format<format::Compact, T>, F, W> where N: for<'writer> FormatFields<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:758`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the subscriber being built to use a less verbose formatter.

See [`format::Compact`](../operations/tracing_subscriber.fmt.format.Compact.md#op-b150ae7b416c1fdd163391ea).

<a id="op-65632a068ed876d528245a9a"></a>
## default

`function` · `tracing_subscriber::fmt::SubscriberBuilder::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [467, 1], "end": [475, 2], "filename": "src/fmt/mod.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/mod.rs:468`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bf88b18bb6fd60016b0199eb"></a>
## event_format

`function` · `tracing_subscriber::fmt::SubscriberBuilder::event_format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_format<E2>(self, fmt_event: E2) -> SubscriberBuilder<N, E2, F, W> where E2: FormatEvent<Registry, N> + 'static, N: for<'writer> FormatFields<'writer> + 'static, W: for<'writer> MakeWriter<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:1031`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the [event formatter][`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) that the subscriber being built
will use to format events that occur.

The event formatter may be any type implementing the [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb)
trait, which is implemented for all functions taking a [`FmtContext`](../operations/tracing_subscriber.fmt.fmt_layer.FmtContext.md#op-edf8024d45f27b1a256d34ab), a
[`Writer`], and an [`Event`](../operations/tracing_core.event.Event.md#op-7ee85389e31294d1a098f039).

# Examples

Setting a type implementing [`FormatEvent`](../operations/tracing_subscriber.fmt.format.FormatEvent.md#op-f71c6d57702e5d6dfe4a8abb) as the formatter:

```rust
use tracing_subscriber::fmt::format;

let subscriber = tracing_subscriber::fmt()
    .event_format(format().compact())
    .finish();
```

[`Writer`]: struct@self::format::Writer

<a id="op-d6a30c4168513721f22f0e7b"></a>
## finish

`function` · `tracing_subscriber::fmt::SubscriberBuilder::finish` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn finish(self) -> Subscriber<N, E, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::Formatter", "path": "Formatter"}}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "layer::Layer"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "layer::Layer"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "fmt_layer::Layer"}}}}]}, "is_negative": false, "span": {"begin": [477, 1], "end": [522, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:486`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Finish the builder, returning a new `FmtSubscriber`.

<a id="op-394fab8e82f7b1e327d855b1"></a>
## flatten_event

`function` · `tracing_subscriber::fmt::SubscriberBuilder::flatten_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flatten_event(self, flatten_event: bool) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "format::JsonFields"}}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "format::Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [800, 1], "end": [841, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:804`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the json subscriber being built to flatten event metadata.

See [`format::Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c) for details.

<a id="op-9895b93ed8a5426b043e1378"></a>
## fmt

`function` · `tracing_subscriber::fmt::SubscriberBuilder::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [248, 10], "end": [248, 15], "filename": "src/fmt/mod.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/mod.rs:248`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-13d396907ce5c1072f4454bf"></a>
## fmt_fields

`function` · `tracing_subscriber::fmt::SubscriberBuilder::fmt_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt_fields<N2>(self, fmt_fields: N2) -> SubscriberBuilder<N2, E, F, W> where N2: for<'writer> FormatFields<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:897`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the field formatter that the subscriber being built will use to record
fields.

For example:
```rust
use tracing_subscriber::fmt::format;
use tracing_subscriber::prelude::*;

let formatter =
    // Construct a custom formatter for `Debug` fields
    format::debug_fn(|writer, field, value| write!(writer, "{}: {:?}", field, value))
        // Use the `tracing_subscriber::MakeFmtExt` trait to wrap the
        // formatter so that a delimiter is added between fields.
        .delimited(", ");

let subscriber = tracing_subscriber::fmt()
    .fmt_fields(formatter)
    .finish();
# drop(subscriber)
```

<a id="op-96e89557f97ec50851a8f3f9"></a>
## init

`function` · `tracing_subscriber::fmt::SubscriberBuilder::init` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn init(self)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::Formatter", "path": "Formatter"}}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "layer::Layer"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "layer::Layer"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "fmt_layer::Layer"}}}}]}, "is_negative": false, "span": {"begin": [477, 1], "end": [522, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:518`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Install this Subscriber as the global default.

If the `tracing-log` feature is enabled, this will also install
the LogTracer to convert `Log` records into `tracing` `Event`s.

# Panics
Panics if the initialization was unsuccessful, likely because a
global subscriber was already installed by another call to `try_init`.

<a id="op-8bbeb04e1820a94e73df6516"></a>
## json

`function` · `tracing_subscriber::fmt::SubscriberBuilder::json` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn json(self) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W> where N: for<'writer> FormatFields<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:785`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the subscriber being built to use a JSON formatter.

See [`format::Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c) for details.

<a id="op-11583fd0fa57836deb97eb86"></a>
## log_internal_errors

`function` · `tracing_subscriber::fmt::SubscriberBuilder::log_internal_errors` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn log_internal_errors(self, log_internal_errors: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:667`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether to write errors from [`FormatEvent`] to the writer.
Defaults to true.

By default, `fmt::Layer` will write any `FormatEvent`-internal errors to
the writer. These errors are unlikely and will only occur if there is a
bug in the `FormatEvent` implementation or its dependencies.

If writing to the writer fails, the error message is printed to stderr
as a fallback.

[`FormatEvent`]: crate::fmt::FormatEvent

<a id="op-3726ca3b24c16b828352944c"></a>
## map_event_format

`function` · `tracing_subscriber::fmt::SubscriberBuilder::map_event_format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn map_event_format<E2>(self, f: impl FnOnce(E) -> E2) -> SubscriberBuilder<N, E2, F, W> where E2: FormatEvent<Registry, N> + 'static, N: for<'writer> FormatFields<'writer> + 'static, W: for<'writer> MakeWriter<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:1111`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Updates the event formatter by applying a function to the existing event formatter.

This sets the event formatter that the subscriber being built will use to record fields.

# Examples

Updating an event formatter:

```rust
let subscriber = tracing_subscriber::fmt()
    .map_event_format(|e| e.compact())
    .finish();
```

<a id="op-4d790b8bc24a39f64be41403"></a>
## map_fmt_fields

`function` · `tracing_subscriber::fmt::SubscriberBuilder::map_fmt_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn map_fmt_fields<N2>(self, f: impl FnOnce(N) -> N2) -> SubscriberBuilder<N2, E, F, W> where N2: for<'writer> FormatFields<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:1137`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Updates the field formatter by applying a function to the existing field formatter.

This sets the field formatter that the subscriber being built will use to record fields.

# Examples

Updating a field formatter:

```rust
use tracing_subscriber::field::MakeExt;
let subscriber = tracing_subscriber::fmt()
    .map_fmt_fields(|f| f.debug_alt())
    .finish();
```

<a id="op-f30fb6a659e53e1b72618df6"></a>
## map_writer

`function` · `tracing_subscriber::fmt::SubscriberBuilder::map_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn map_writer<W2>(self, f: impl FnOnce(W) -> W2) -> SubscriberBuilder<N, E, F, W2> where W2: for<'writer> MakeWriter<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:1164`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Updates the [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) by applying a function to the existing [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a).

This sets the [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that the subscriber being built will use to write events.

# Examples

Redirect output to stderr if level is <= WARN:

```rust
use tracing::Level;
use tracing_subscriber::fmt::{self, writer::MakeWriterExt};

let stderr = std::io::stderr.with_max_level(Level::WARN);
let layer = tracing_subscriber::fmt()
    .map_writer(move |w| stderr.or_else(w))
    .finish();
```

<a id="op-372b077603bbf15419289456"></a>
## pretty

`function` · `tracing_subscriber::fmt::SubscriberBuilder::pretty` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn pretty(self) -> SubscriberBuilder<format::Pretty, format::Format<format::Pretty, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:771`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the subscriber being built to use an [excessively pretty, human-readable formatter](crate::fmt::format::Pretty).

<a id="op-3918e8ba4f86f63144a87a8a"></a>
## reload_handle

`function` · `tracing_subscriber::fmt::SubscriberBuilder::reload_handle` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn reload_handle(&self) -> reload::Handle<EnvFilter, Formatter<N, E, W>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "crate::EnvFilter"}}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::Formatter", "path": "Formatter"}}}], "constraints": []}}, "id": "tracing_subscriber::reload::Layer", "path": "crate::reload::Layer"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "tracing_core::Subscriber"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::Formatter", "path": "Formatter"}}}}]}, "is_negative": false, "span": {"begin": [865, 1], "end": [874, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:871`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a `Handle` that may be used to reload the constructed subscriber's
filter.

<a id="op-e3984f34c88588a3a1f380f4"></a>
## try_init

`function` · `tracing_subscriber::fmt::SubscriberBuilder::try_init` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn try_init(self) -> Result<(), Box<dyn Error + Send + Sync + 'static>>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::Formatter", "path": "Formatter"}}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "layer::Layer"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "F"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "layer::Layer"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::registry::sharded::Registry", "path": "crate::registry::Registry"}}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "fmt_layer::Layer"}}}}]}, "is_negative": false, "span": {"begin": [477, 1], "end": [522, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:503`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Install this Subscriber as the global default if one is
not already set.

If the `tracing-log` feature is enabled, this will also install
the LogTracer to convert `Log` records into `tracing` `Event`s.

# Errors
Returns an Error if the initialization was unsuccessful, likely
because a global subscriber was already installed by another
call to `try_init`.

<a id="op-24ec46ca4ef59c2622927f83"></a>
## with_ansi

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_ansi` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_ansi(self, ansi: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:633`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the formatter emits ANSI terminal escape codes
for colors and other text formatting.

Enabling ANSI escapes (calling `with_ansi(true)`) requires the "ansi"
crate feature flag. Calling `with_ansi(true)` without the "ansi"
feature flag enabled will panic if debug assertions are enabled, or
print a warning otherwise.

This method itself is still available without the feature flag. This
is to allow ANSI escape codes to be explicitly *disabled* without
having to opt-in to the dependencies required to emit ANSI formatting.
This way, code which constructs a formatter that should never emit
ANSI escape codes can ensure that they are not used, regardless of
whether or not other crates in the dependency graph enable the "ansi"
feature flag.

<a id="op-c7937bf4693a71b57896422c"></a>
## with_ansi_sanitization

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_ansi_sanitization` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_ansi_sanitization(self, ansi_sanitization: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:646`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether ANSI control character sanitization is enabled.

This defaults to `true` as a protective measure against terminal
injection attacks. If this is set to `false`, ANSI sanitization is
disabled and trusted ANSI control sequences in logged values are passed
through unchanged.

<a id="op-6b4af9eb115e9f91ebb6ed39"></a>
## with_current_span

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_current_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current_span(self, display_current_span: bool) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "format::JsonFields"}}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "format::Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [800, 1], "end": [841, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:818`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the JSON subscriber being built will include the current span
in formatted events.

See [`format::Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c) for details.

<a id="op-a50bd4335c07d0cd1a6fc30d"></a>
## with_env_filter

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_env_filter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_env_filter(self, filter: impl Into<EnvFilter>) -> SubscriberBuilder<N, E, EnvFilter, W> where Formatter<N, E, W>: tracing_core::Subscriber + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:957`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the [`EnvFilter`] that the subscriber will use to determine if
a span or event is enabled.

Note that this method requires the "env-filter" feature flag to be enabled.

If a filter was previously set, or a maximum level was set by the
[`with_max_level`] method, that value is replaced by the new filter.

# Examples

Setting a filter based on the value of the `RUST_LOG` environment
variable:
```rust
use tracing_subscriber::{fmt, EnvFilter};

fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

Setting a filter based on a pre-set filter directive string:
```rust
use tracing_subscriber::fmt;

fmt()
    .with_env_filter("my_crate=info,my_crate::my_mod=debug,[my_span]=trace")
    .init();
```

Adding additional directives to a filter constructed from an env var:
```rust
use tracing_subscriber::{fmt, filter::{EnvFilter, LevelFilter}};

# fn filter() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
let filter = EnvFilter::try_from_env("MY_CUSTOM_FILTER_ENV_VAR")?
    // Set the base level when not matched by other directives to WARN.
    .add_directive(LevelFilter::WARN.into())
    // Set the max level for `my_crate::my_mod` to DEBUG, overriding
    // any directives parsed from the env variable.
    .add_directive("my_crate::my_mod=debug".parse()?);

fmt()
    .with_env_filter(filter)
    .try_init()?;
# Ok(())}
```
[`EnvFilter`]: super::filter::EnvFilter
[`with_max_level`]: SubscriberBuilder::with_max_level()

<a id="op-8b85a50ef7b8ba3365b799f4"></a>
## with_file

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_file` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_file(self, display_filename: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:692`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's [source code file path][file] is
displayed.

[file]: tracing_core::Metadata::file

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::file`.

<a id="op-0390c2f3ee3d617e8502a0e2"></a>
## with_filter_reloading

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_filter_reloading` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_filter_reloading(self) -> SubscriberBuilder<N, E, reload::Layer<EnvFilter, Formatter<N, E, W>>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::filter::env::EnvFilter", "path": "crate::EnvFilter"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "tracing_core::Subscriber"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::Formatter", "path": "Formatter"}}}}]}, "is_negative": false, "span": {"begin": [845, 1], "end": [861, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:851`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Configures the subscriber being built to allow filter reloading at
runtime.

<a id="op-0a879ddde7c7583c45166300"></a>
## with_level

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_level` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_level(self, display_level: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:717`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's level is displayed.

<a id="op-8295bdd9decf62a863e39338"></a>
## with_line_number

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_line_number` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_line_number(self, display_line_number: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:706`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's [source code line number][line] is
displayed.

[line]: tracing_core::Metadata::line

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::line`.

<a id="op-aaa93e25c0cc204eb1ed1198"></a>
## with_max_level

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_max_level` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_level(self, filter: impl Into<LevelFilter>) -> SubscriberBuilder<N, E, LevelFilter, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:1000`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the maximum [verbosity level] that will be enabled by the
subscriber.

If the max level has already been set, or a [`EnvFilter`] was added by
[`with_env_filter`], this replaces that configuration with the new
maximum level.

# Examples

Enable up to the `DEBUG` verbosity level:
```rust
use tracing_subscriber::fmt;
use tracing::Level;

fmt()
    .with_max_level(Level::DEBUG)
    .init();
```
This subscriber won't record any spans or events!
```rust
use tracing_subscriber::{fmt, filter::LevelFilter};

let subscriber = fmt()
    .with_max_level(LevelFilter::OFF)
    .finish();
```
[verbosity level]: tracing_core::Level
[`EnvFilter`]: struct@crate::filter::EnvFilter
[`with_env_filter`]: fn@Self::with_env_filter

<a id="op-384b2f67166a07c59dc687cb"></a>
## with_span_events

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_span_events` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_events(self, kind: format::FmtSpan) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:611`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Configures how synthesized events are emitted at points in the [span
lifecycle][lifecycle].

The following options are available:

- `FmtSpan::NONE`: No events will be synthesized when spans are
  created, entered, exited, or closed. Data from spans will still be
  included as the context for formatted events. This is the default.
- `FmtSpan::NEW`: An event will be synthesized when spans are created.
- `FmtSpan::ENTER`: An event will be synthesized when spans are entered.
- `FmtSpan::EXIT`: An event will be synthesized when spans are exited.
- `FmtSpan::CLOSE`: An event will be synthesized when a span closes. If
  [timestamps are enabled][time] for this formatter, the generated
  event will contain fields with the span's _busy time_ (the total
  time for which it was entered) and _idle time_ (the total time that
  the span existed but was not entered).
- `FmtSpan::ACTIVE`: An event will be synthesized when spans are entered
  or exited.
- `FmtSpan::FULL`: Events will be synthesized whenever a span is
  created, entered, exited, or closed. If timestamps are enabled, the
  close event will contain the span's busy and idle time, as
  described above.

The options can be enabled in any combination. For instance, the following
will synthesize events whenever spans are created and closed:

```rust
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt;

let subscriber = fmt()
    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    .finish();
```

Note that the generated events will only be part of the log output by
this formatter; they will not be recorded by other `Subscriber`s or by
`Layer`s added to this subscriber.

[lifecycle]: https://docs.rs/tracing/latest/tracing/span/index.html#the-span-lifecycle
[time]: SubscriberBuilder::without_time()

<a id="op-0364af82b5648b1cbc7d76e2"></a>
## with_span_list

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_span_list` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_list(self, display_span_list: bool) -> SubscriberBuilder<format::JsonFields, format::Format<format::Json, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "format::JsonFields"}}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "format::Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [800, 1], "end": [841, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:832`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the JSON subscriber being built will include a list (from
root to leaf) of all currently entered spans in formatted events.

See [`format::Json`](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c) for details.

<a id="op-c71478d7d31026006d057761"></a>
## with_target

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_target` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_target(self, display_target: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:678`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's target is displayed.

<a id="op-2eb32dbf00416a2cb055018b"></a>
## with_test_writer

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_test_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_test_writer(self) -> SubscriberBuilder<N, E, F, TestWriter>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:1091`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Configures the subscriber to support [`libtest`'s output capturing][capturing] when used in
unit tests.

See [`TestWriter`] for additional details.

# Examples

Using [`TestWriter`] to let `cargo test` capture test output. Note that we do not install it
globally as it may cause conflicts.

```rust
use tracing_subscriber::fmt;
use tracing::subscriber;

subscriber::set_default(
    fmt()
        .with_test_writer()
        .finish()
);
```

[capturing]:
https://doc.rust-lang.org/book/ch11-02-running-tests.html#showing-function-output
[`TestWriter`]: writer::TestWriter

<a id="op-49794169753c4eda44b0dcca"></a>
## with_thread_ids

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_thread_ids` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_thread_ids(self, display_thread_ids: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:745`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the [thread ID] of the current thread is displayed
when formatting events.

[thread ID]: std::thread::ThreadId

Unresolved upstream links (retained, not inferred): `std::thread::ThreadId`.

<a id="op-b930034fe5f0edf008cfc8b0"></a>
## with_thread_names

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_thread_names` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_thread_names(self, display_thread_names: bool) -> SubscriberBuilder<N, format::Format<L, T>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:731`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the [name] of the current thread is displayed
when formatting events.

[name]: std::thread#naming-threads

Unresolved upstream links (retained, not inferred): `std::thread#naming-threads`.

<a id="op-1450d283658e3a6a5e74b83c"></a>
## with_timer

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_timer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_timer<T2>(self, timer: T2) -> SubscriberBuilder<N, format::Format<L, T2>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:555`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Use the given [`timer`] for log message timestamps.

See the [`time` module] for the provided timer implementations.

Note that using the `"time`"" feature flag enables the
additional time formatters [`UtcTime`] and [`LocalTime`], which use the
[`time` crate] to provide more sophisticated timestamp formatting
options.

[`timer`]: time::FormatTime
[`time` module]: mod@time
[`UtcTime`]: time::UtcTime
[`LocalTime`]: time::LocalTime
[`time` crate]: https://docs.rs/time/0.3

<a id="op-bca255eed4c6df3a002bb22d"></a>
## with_writer

`function` · `tracing_subscriber::fmt::SubscriberBuilder::with_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_writer<W2>(self, make_writer: W2) -> SubscriberBuilder<N, E, F, W2> where W2: for<'writer> MakeWriter<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [876, 1], "end": [1173, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:1057`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that the subscriber being built will use to write events.

# Examples

Using `stderr` rather than `stdout`:

```rust
use tracing_subscriber::fmt;
use std::io;

fmt()
    .with_writer(io::stderr)
    .init();
```

<a id="op-6a575b9ae953ffb877c70bf3"></a>
## without_time

`function` · `tracing_subscriber::fmt::SubscriberBuilder::without_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn without_time(self) -> SubscriberBuilder<N, format::Format<L, ()>, F, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "F"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::SubscriberBuilder", "path": "SubscriberBuilder"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "F"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [537, 1], "end": [796, 2], "filename": "src/fmt/mod.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/mod.rs:563`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Do not emit timestamps with log messages.
