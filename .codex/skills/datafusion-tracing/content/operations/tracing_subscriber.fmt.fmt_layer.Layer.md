# `tracing_subscriber::fmt::fmt_layer::Layer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_subscriber.fmt.fmt_layer.Layer.json).

<a id="op-fa1a59e4f05bcd8ea89adaa4"></a>
## Layer

`struct` · `tracing_subscriber::fmt::fmt_layer::Layer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
struct Layer<S, N = format::DefaultFields, E = format::Format<format::Full>, W = fn() -> io::Stdout>
```

Source: `src/fmt/fmt_layer.rs:64`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

A [`Layer`] that logs formatted representations of `tracing` events.

## Examples

Constructing a layer with the default configuration:

```rust
use tracing_subscriber::{fmt, Registry};
use tracing_subscriber::prelude::*;

let subscriber = Registry::default()
    .with(fmt::Layer::default());

tracing::subscriber::set_global_default(subscriber).unwrap();
```

Overriding the layer's behavior:

```rust
use tracing_subscriber::{fmt, Registry};
use tracing_subscriber::prelude::*;

let fmt_layer = fmt::layer()
   .with_target(false) // don't include event targets when logging
   .with_level(false); // don't include event levels when logging

let subscriber = Registry::default().with(fmt_layer);
# tracing::subscriber::set_global_default(subscriber).unwrap();
```

Setting a custom event formatter:

```rust
use tracing_subscriber::fmt::{self, format, time};
use tracing_subscriber::prelude::*;

let fmt = format().with_timer(time::Uptime::default());
let fmt_layer = fmt::layer()
    .event_format(fmt)
    .with_target(false);
# let subscriber = fmt_layer.with_subscriber(tracing_subscriber::registry::Registry::default());
# tracing::subscriber::set_global_default(subscriber).unwrap();
```

[`Layer`]: super::layer::Layer

<a id="op-afa4404612b7c572a5502fec"></a>
## compact

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::compact` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn compact(self) -> Layer<S, N, format::Format<format::Compact, T>, W> where N: for<'writer> FormatFields<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:574`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the layer being built to use a [less verbose formatter][super::format::Compact](../operations/tracing_subscriber.fmt.format.Compact.md#op-b150ae7b416c1fdd163391ea).

<a id="op-39f18a7c4766439d45756c09"></a>
## default

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::default` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [739, 1], "end": [756, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/fmt/fmt_layer.rs:740`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c3c88bff2040f2c25dcd0d0c"></a>
## event_format

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::event_format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn event_format<E2>(self, e: E2) -> Layer<S, N, E2, W> where E2: FormatEvent<S, N> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [88, 1], "end": [162, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:116`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the [event formatter][`FormatEvent`] that the layer being built will
use to format events.

The event formatter may be any type implementing the [`FormatEvent`]
trait, which is implemented for all functions taking a [`FmtContext`](../operations/tracing_subscriber.fmt.fmt_layer.FmtContext.md#op-edf8024d45f27b1a256d34ab), a
[`Writer`], and an [`Event`].

# Examples

Setting a type implementing [`FormatEvent`] as the formatter:
```rust
use tracing_subscriber::fmt::{self, format};

let layer = fmt::layer()
    .event_format(format().compact());
# // this is necessary for type inference.
# use tracing_subscriber::Layer as _;
# let _ = layer.with_subscriber(tracing_subscriber::registry::Registry::default());
```
[`FormatEvent`]: format::FormatEvent
[`Event`]: tracing::Event
[`Writer`]: format::Writer

<a id="op-d631d4d5ce3c9d1813e27c94"></a>
## flatten_event

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::flatten_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn flatten_event(self, flatten_event: bool) -> Layer<S, format::JsonFields, format::Format<format::Json, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "format::JsonFields"}}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "format::Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [641, 1], "end": [685, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:645`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the JSON layer being built to flatten event metadata.

See [`format::Json`][super::format::Json](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c)

<a id="op-6d26f9938cacd946e4f738b1"></a>
## fmt

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::fmt` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [63, 10], "end": [63, 15], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/fmt/fmt_layer.rs:63`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-48b37dfea4d2b64e0c55102e"></a>
## fmt_fields

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::fmt_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt_fields<N2>(self, fmt_fields: N2) -> Layer<S, N2, E, W> where N2: for<'writer> FormatFields<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [687, 1], "end": [737, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:690`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the field formatter that the layer being built will use to record
fields.

<a id="op-a40b04f7b0713d9e563301c2"></a>
## json

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::json` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn json(self) -> Layer<S, format::JsonFields, format::Format<format::Json, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:624`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the layer being built to use a [JSON formatter][super::format::Json](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c).

The full format includes fields from all entered spans.

# Example Output

```ignore,json
{"timestamp":"Feb 20 11:28:15.096","level":"INFO","target":"mycrate","fields":{"message":"some message", "key": "value"}}
```

# Options

- [`Layer::flatten_event`] can be used to enable flattening event fields into the root
  object.

[`Layer::flatten_event`]: Layer::flatten_event()

<a id="op-dadbbfa71276a0b1aae4016e"></a>
## log_internal_errors

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::log_internal_errors` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn log_internal_errors(self, log_internal_errors: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:372`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether to write errors from [`FormatEvent`] to the writer.
Defaults to true.

By default, `fmt::Layer` will write any `FormatEvent`-internal errors to
the writer. These errors are unlikely and will only occur if there is a
bug in the `FormatEvent` implementation or its dependencies.

If writing to the writer fails, the error message is printed to stderr
as a fallback.

[`FormatEvent`]: crate::fmt::FormatEvent

<a id="op-2f20dc0e9ea1c31796e67568"></a>
## map_event_format

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::map_event_format` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn map_event_format<E2>(self, f: impl FnOnce(E) -> E2) -> Layer<S, N, E2, W> where E2: FormatEvent<S, N> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [88, 1], "end": [162, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:147`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Updates the event formatter by applying a function to the existing event formatter.

This sets the event formatter that the layer being built will use to record fields.

# Examples

Updating an event formatter:

```rust
let layer = tracing_subscriber::fmt::layer()
    .map_event_format(|e| e.compact());
# // this is necessary for type inference.
# use tracing_subscriber::Layer as _;
# let _ = layer.with_subscriber(tracing_subscriber::registry::Registry::default());
```

<a id="op-2a8df2a4ca52470ad8ad55f4"></a>
## map_fmt_fields

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::map_fmt_fields` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn map_fmt_fields<N2>(self, f: impl FnOnce(N) -> N2) -> Layer<S, N2, E, W> where N2: for<'writer> FormatFields<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [687, 1], "end": [737, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:722`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Updates the field formatter by applying a function to the existing field formatter.

This sets the field formatter that the layer being built will use to record fields.

# Examples

Updating a field formatter:

```rust
use tracing_subscriber::field::MakeExt;
let layer = tracing_subscriber::fmt::layer()
    .map_fmt_fields(|f| f.debug_alt());
# // this is necessary for type inference.
# use tracing_subscriber::Layer as _;
# let _ = layer.with_subscriber(tracing_subscriber::registry::Registry::default());
```

<a id="op-174976f90cacad88816c2487"></a>
## map_writer

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::map_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn map_writer<W2>(self, f: impl FnOnce(W) -> W2) -> Layer<S, N, E, W2> where W2: for<'writer> MakeWriter<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:398`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Updates the [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) by applying a function to the existing [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a).

This sets the [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that the layer being built will use to write events.

# Examples

Redirect output to stderr if level is <= WARN:

```rust
use tracing::Level;
use tracing_subscriber::fmt::{self, writer::MakeWriterExt};

let stderr = std::io::stderr.with_max_level(Level::WARN);
let layer = fmt::layer()
    .map_writer(move |w| stderr.or_else(w));
# // this is necessary for type inference.
# use tracing_subscriber::Layer as _;
# let _ = layer.with_subscriber(tracing_subscriber::registry::Registry::default());
```

<a id="op-a4867b21675f4ae4c302bd67"></a>
## new

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::new` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [80, 1], "end": [85, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:82`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Returns a new [`Layer`][self::Layer](../operations/tracing_subscriber.fmt.fmt_layer.Layer.md#op-fa1a59e4f05bcd8ea89adaa4) with the default configuration.

<a id="op-b419e19e7eb004784a305394"></a>
## on_close

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::on_close` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [867, 1], "end": [1083, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/fmt/fmt_layer.rs:978`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4fa43acd8f5c859cd8a6fc5e"></a>
## on_enter

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::on_enter` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [867, 1], "end": [1083, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/fmt/fmt_layer.rs:932`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3283ada3de600d422f0a90b8"></a>
## on_event

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::on_event` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [867, 1], "end": [1083, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/fmt/fmt_layer.rs:1017`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8539dee0727a5fb79ecd7c34"></a>
## on_exit

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::on_exit` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [867, 1], "end": [1083, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/fmt/fmt_layer.rs:955`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-922c350ef5a1797b1eb492e2"></a>
## on_new_span

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::on_new_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [867, 1], "end": [1083, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/fmt/fmt_layer.rs:874`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7955104d4a2be02c1b61c7cb"></a>
## on_record

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::on_record` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'a"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'a"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatEvent", "path": "FormatEvent"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "E"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::writer::MakeWriter", "path": "MakeWriter"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "W"}}}]}, "is_negative": false, "span": {"begin": [867, 1], "end": [1083, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/fmt/fmt_layer.rs:912`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0f23f3063ceec639baf34aef"></a>
## pretty

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::pretty` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn pretty(self) -> Layer<S, format::Pretty, format::Format<format::Pretty, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:593`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the layer being built to use an [excessively pretty, human-readable formatter](crate::fmt::format::Pretty).

<a id="op-5b6add53bd5e253dd48eb850"></a>
## set_ansi

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::set_ansi` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn set_ansi(&mut self, ansi: bool)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:244`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether this layer should use ANSI terminal formatting
escape codes (such as colors).

This method is primarily expected to be used with the
[`reload::Handle::modify`](crate::reload::Handle::modify) method when changing
the writer.

<a id="op-59911c752a39a473e4e82196"></a>
## set_span_events

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::set_span_events` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn set_span_events(&mut self, kind: FmtSpan)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:262`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Modifies how synthesized events are emitted at points in the [span
lifecycle][lifecycle].

See [`Self::with_span_events`](../operations/tracing_subscriber.fmt.fmt_layer.Layer.md#op-c68b24ad5f5d471328fb0a39) for documentation on the [`FmtSpan`](../operations/tracing_subscriber.fmt.format.FmtSpan.md#op-0b00f8b7558927cae52a28c2)

This method is primarily expected to be used with the
[`reload::Handle::modify`](crate::reload::Handle::modify) method

Note that using this method modifies the span configuration instantly and does not take into
account any current spans. If the previous configuration was set to capture
`FmtSpan::ALL`, for example, using this method to change to `FmtSpan::NONE` will cause an
exit event for currently entered events not to be formatted

[lifecycle]: mod@tracing::span#the-span-lifecycle

<a id="op-e8a8b80d35333a26b71d56e2"></a>
## with_ansi

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_ansi` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_ansi(self, ansi: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:331`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the formatter emits ANSI terminal escape codes
for colors and other text formatting.

When the "ansi" crate feature flag is enabled, ANSI colors are enabled
by default unless the [`NO_COLOR`] environment variable is set to
a non-empty value.  If the [`NO_COLOR`] environment variable is set to
any non-empty value, then ANSI colors will be suppressed by default.
The [`with_ansi`] and [`set_ansi`] methods can be used to forcibly
enable ANSI colors, overriding any [`NO_COLOR`] environment variable.

[`NO_COLOR`]: https://no-color.org/

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

[`with_ansi`]: Layer::with_ansi
[`set_ansi`]: Layer::set_ansi

<a id="op-1ba32bbe794ee07ae1db0107"></a>
## with_ansi_sanitization

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_ansi_sanitization` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_ansi_sanitization(self, ansi_sanitization: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:354`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether ANSI control character sanitization is enabled.

This defaults to `true` as a protective measure against terminal
injection attacks. If this is set to `false`, ANSI sanitization is
disabled and trusted ANSI control sequences in logged values are passed
through unchanged.

<a id="op-69279943ca3655ff0d1486a9"></a>
## with_current_span

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_current_span` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_current_span(self, display_current_span: bool) -> Layer<S, format::JsonFields, format::Format<format::Json, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "format::JsonFields"}}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "format::Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [641, 1], "end": [685, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:660`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the formatter will include the current span in
formatted events.

See [`format::Json`][super::format::Json](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c)

<a id="op-b3688a2fc644462edfaf50d8"></a>
## with_file

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_file` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_file(self, display_filename: bool) -> Layer<S, N, format::Format<L, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:519`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's [source code file path][file] is
displayed.

[file]: tracing_core::Metadata::file

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::file`.

<a id="op-bfb2e13ca79a34142bf20108"></a>
## with_level

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_level` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_level(self, display_level: bool) -> Layer<S, N, format::Format<L, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:541`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's level is displayed.

<a id="op-6575f04916e3f13805537d71"></a>
## with_line_number

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_line_number` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_line_number(self, display_line_number: bool) -> Layer<S, N, format::Format<L, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:530`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's [source code line number][line] is
displayed.

[line]: tracing_core::Metadata::line

Unresolved upstream links (retained, not inferred): `tracing_core::Metadata::line`.

<a id="op-c68b24ad5f5d471328fb0a39"></a>
## with_span_events

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_span_events` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_events(self, kind: FmtSpan) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:501`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

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
- `FmtSpan::ACTIVE`: Events will be synthesized when spans are entered
  or exited.
- `FmtSpan::FULL`: Events will be synthesized whenever a span is
  created, entered, exited, or closed. If timestamps are enabled, the
  close event will contain the span's busy and idle time, as
  described above.

The options can be enabled in any combination. For instance, the following
will synthesize events whenever spans are created and closed:

```rust
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::FmtSpan;

let subscriber = fmt()
    .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
    .finish();
```

Note that the generated events will only be part of the log output by
this formatter; they will not be recorded by other `Subscriber`s or by
`Layer`s added to this subscriber.

[lifecycle]: https://docs.rs/tracing/latest/tracing/span/index.html#the-span-lifecycle
[time]: Layer::without_time()

<a id="op-d820ae82d69312164e974d1e"></a>
## with_span_list

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_span_list` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_span_list(self, display_span_list: bool) -> Layer<S, format::JsonFields, format::Format<format::Json, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::JsonFields", "path": "format::JsonFields"}}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"resolved_path": {"args": null, "id": "tracing_subscriber::fmt::format::json::Json", "path": "format::Json"}}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [641, 1], "end": [685, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:675`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the formatter will include a list (from root to leaf)
of all currently entered spans in formatted events.

See [`format::Json`][super::format::Json](../operations/tracing_subscriber.fmt.format.json.Json.md#op-8e1265e43c2726c9a4be086c)

<a id="op-78af67376472e1535c8c3ea9"></a>
## with_target

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_target` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_target(self, display_target: bool) -> Layer<S, N, format::Format<L, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:509`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not an event's target is displayed.

<a id="op-fa27966f2d5fee231dda8062"></a>
## with_test_writer

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_test_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_test_writer(self) -> Layer<S, N, E, TestWriter>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:291`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Configures the layer to support [`libtest`'s output capturing][capturing] when used in
unit tests.

See [`TestWriter`] for additional details.

# Examples

Using [`TestWriter`] to let `cargo test` capture test output:

```rust
use std::io;
use tracing_subscriber::fmt;

let layer = fmt::layer()
    .with_test_writer();
# // this is necessary for type inference.
# use tracing_subscriber::Layer as _;
# let _ = layer.with_subscriber(tracing_subscriber::registry::Registry::default());
```
[capturing]:
https://doc.rust-lang.org/book/ch11-02-running-tests.html#showing-function-output
[`TestWriter`]: super::writer::TestWriter

<a id="op-a717df783e1e38944750f8d0"></a>
## with_thread_ids

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_thread_ids` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_thread_ids(self, display_thread_ids: bool) -> Layer<S, N, format::Format<L, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:552`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the [thread ID] of the current thread is displayed
when formatting events.

[thread ID]: std::thread::ThreadId

Unresolved upstream links (retained, not inferred): `std::thread::ThreadId`.

<a id="op-a113a14c8463c97cabd5e7ee"></a>
## with_thread_names

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_thread_names` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_thread_names(self, display_thread_names: bool) -> Layer<S, N, format::Format<L, T>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:563`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets whether or not the [name] of the current thread is displayed
when formatting events.

[name]: std::thread#naming-threads

Unresolved upstream links (retained, not inferred): `std::thread#naming-threads`.

<a id="op-b43c56c870f35bfeafa89be7"></a>
## with_timer

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_timer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_timer<T2>(self, timer: T2) -> Layer<S, N, format::Format<L, T2>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:433`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Use the given [`timer`] for span and event timestamps.

See the [`time` module] for the provided timer implementations.

Note that using the `"time`"" feature flag enables the
additional time formatters [`UtcTime`] and [`LocalTime`], which use the
[`time` crate] to provide more sophisticated timestamp formatting
options.

[`timer`]: super::time::FormatTime
[`time` module]: mod@super::time
[`UtcTime`]: super::time::UtcTime
[`LocalTime`]: super::time::LocalTime
[`time` crate]: https://docs.rs/time/0.3

<a id="op-d4b2767cdfaa9b7e7a1a21fa"></a>
## with_writer

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::with_writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn with_writer<W2>(self, make_writer: W2) -> Layer<S, N, E, W2> where W2: for<'writer> MakeWriter<'writer> + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:182`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Sets the [`MakeWriter`](../operations/tracing_subscriber.fmt.writer.MakeWriter.md#op-49cad405bcbd7170e0ab390a) that the layer being built will use to write events.

# Examples

Using `stderr` rather than `stdout`:

```rust
use std::io;
use tracing_subscriber::fmt;

let layer = fmt::layer()
    .with_writer(io::stderr);
# // this is necessary for type inference.
# use tracing_subscriber::Layer as _;
# let _ = layer.with_subscriber(tracing_subscriber::registry::Registry::default());
```

<a id="op-bb66161c514c3e4201cde38d"></a>
## without_time

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::without_time` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn without_time(self) -> Layer<S, N, format::Format<L, ()>, W>
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "L"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::Format", "path": "format::Format"}}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "L"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'writer"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'writer"}], "constraints": []}}, "id": "tracing_subscriber::fmt::format::FormatFields", "path": "FormatFields"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "N"}}}]}, "is_negative": false, "span": {"begin": [415, 1], "end": [637, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:447`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Do not emit timestamps with spans and event.

<a id="op-feb4179a394dfe858c680c84"></a>
## writer

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::writer` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer(&self) -> &W
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:201`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Borrows the [writer] for this [`Layer`](../operations/tracing_subscriber.fmt.fmt_layer.Layer.md#op-fa1a59e4f05bcd8ea89adaa4).

[writer]: MakeWriter

<a id="op-37ef17eda46174789063a8f6"></a>
## writer_mut

`function` · `tracing_subscriber::fmt::fmt_layer::Layer::writer_mut` · tracing-subscriber 0.3.23
Reachability: `supported`.  Capture: hosted.

```rust
fn writer_mut(&mut self) -> &mut W
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "N"}}, {"type": {"generic": "E"}}, {"type": {"generic": "W"}}], "constraints": []}}, "id": "tracing_subscriber::fmt::fmt_layer::Layer", "path": "Layer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "N"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "E"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "W"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [165, 1], "end": [413, 2], "filename": "src/fmt/fmt_layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/fmt/fmt_layer.rs:232`. [Exact documentation build](https://docs.rs/crate/tracing-subscriber/0.3.23/json).

Mutably borrows the [writer] for this [`Layer`](../operations/tracing_subscriber.fmt.fmt_layer.Layer.md#op-fa1a59e4f05bcd8ea89adaa4).

This method is primarily expected to be used with the
[`reload::Handle::modify`](crate::reload::Handle::modify) method.

# Examples

```
# use tracing::info;
# use tracing_subscriber::{fmt,reload,Registry,prelude::*};
# fn non_blocking<T: std::io::Write>(writer: T) -> (fn() -> std::io::Stdout) {
#   std::io::stdout
# }
# fn main() {
let layer = fmt::layer().with_writer(non_blocking(std::io::stderr()));
let (layer, reload_handle) = reload::Layer::new(layer);
#
# // specifying the Registry type is required
# let _: &reload::Handle<fmt::Layer<Registry, _, _, _>, Registry> = &reload_handle;
#
info!("This will be logged to stderr");
reload_handle.modify(|layer| *layer.writer_mut() = non_blocking(std::io::stdout()));
info!("This will be logged to stdout");
# }
```

[writer]: MakeWriter
