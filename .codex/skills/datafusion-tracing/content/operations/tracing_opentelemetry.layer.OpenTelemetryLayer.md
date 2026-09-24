# `tracing_opentelemetry::layer::OpenTelemetryLayer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_opentelemetry.layer.OpenTelemetryLayer.json).

<a id="op-62abbc216c2acb69ef65794a"></a>
## OpenTelemetryLayer

`struct` · `tracing_opentelemetry::layer::OpenTelemetryLayer` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
struct OpenTelemetryLayer<S, T>
```

Source: `src/layer.rs:39`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

An [OpenTelemetry] propagation layer for use in a project that uses
[tracing].

[OpenTelemetry]: https://opentelemetry.io
[tracing]: https://github.com/tokio-rs/tracing

<a id="op-da344d315e4ff5ea149551a3"></a>
## default

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::default` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"resolved_path": {"args": null, "id": "opentelemetry::trace::noop::NoopTracer", "path": "noop::NoopTracer"}}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [52, 1], "end": [59, 2], "filename": "src/layer.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/layer.rs:56`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5bc18c54f5d26a31b2e964c1"></a>
## new

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::new` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(tracer: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:610`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Set the [`Tracer`] that this layer will use to produce and track
OpenTelemetry [`Span`]s.

[`Tracer`]: opentelemetry::trace::Tracer
[`Span`]: opentelemetry::trace::Span

# Examples

```no_run
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use opentelemetry::trace::TracerProvider as _;
use tracing_subscriber::Registry;

// Create an OTLP pipeline exporter for a `trace_demo` service.

let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .build()
    .unwrap();

let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    .with_simple_exporter(otlp_exporter)
    .build()
    .tracer("trace_demo");

// Create a layer with the configured tracer
let otel_layer = OpenTelemetryLayer::new(tracer);

// Use the tracing subscriber `Registry`, or any other subscriber
// that impls `LookupSpan`
let subscriber = Registry::default().with(otel_layer);
# drop(subscriber);
```

<a id="op-0575a2c31c3a70f17aa7364e"></a>
## on_close

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::on_close` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [1011, 1], "end": [1395, 2], "filename": "src/layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer.rs:1334`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Exports an OpenTelemetry [`Span`] on close.

[`Span`]: opentelemetry::trace::Span

<a id="op-47fa0d9381b995e031378a1a"></a>
## on_enter

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::on_enter` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [1011, 1], "end": [1395, 2], "filename": "src/layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer.rs:1086`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b2d4d13813d110125bd8a282"></a>
## on_event

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::on_event` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [1011, 1], "end": [1395, 2], "filename": "src/layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer.rs:1207`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Records OpenTelemetry [`Event`] data on event.

Note: an [`ERROR`]-level event will also set the OpenTelemetry span status code to
[`Error`], signaling that an error has occurred.

[`Event`]: opentelemetry::trace::Event
[`ERROR`]: tracing::Level::ERROR
[`Error`]: opentelemetry::trace::StatusCode::Error

Unresolved upstream links (retained, not inferred): `tracing::Level::ERROR`.

<a id="op-b5aa82fb36783ba0c4bbbf3b"></a>
## on_exit

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::on_exit` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [1011, 1], "end": [1395, 2], "filename": "src/layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer.rs:1117`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1dcd2eb16be2c9fd90003339"></a>
## on_follows_from

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::on_follows_from` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_follows_from(&self, id: &Id, follows: &Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [1011, 1], "end": [1395, 2], "filename": "src/layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer.rs:1167`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0b4fd4255b595b24bd6ce00a"></a>
## on_new_span

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::on_new_span` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &Attributes<'_>, id: &span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [1011, 1], "end": [1395, 2], "filename": "src/layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer.rs:1021`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Creates an [OpenTelemetry `Span`] for the corresponding [tracing `Span`].

[OpenTelemetry `Span`]: opentelemetry::trace::Span
[tracing `Span`]: tracing::Span

<a id="op-258799a24358c6bc8652e675"></a>
## on_record

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::on_record` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, id: &Id, values: &Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [1011, 1], "end": [1395, 2], "filename": "src/layer.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/layer.rs:1145`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Record OpenTelemetry [`attributes`] for the given values.

[`attributes`]: opentelemetry::trace::SpanBuilder::attributes

Unresolved upstream links (retained, not inferred): `opentelemetry::trace::SpanBuilder::attributes`.

<a id="op-8a3b6a7907f3e785aaddc2a5"></a>
## with_context_activation

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_context_activation` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_context_activation(self, context_activation: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:842`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not an OpenTelemetry Context should be activated on span entry.

When enabled, entering a span will activate its OpenTelemetry context, making it
available to other OpenTelemetry instrumentation. This allows for proper context
propagation across different instrumentation libraries.

By default, context activation is enabled.

<a id="op-0c2ba26c3c6fd6467cd0ef1b"></a>
## with_error_events_to_exceptions

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_error_events_to_exceptions` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_error_events_to_exceptions(self, error_events_to_exceptions: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:738`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not a subset of events following the described schema are mapped to
events following the [OpenTelemetry semantic conventions for
exceptions][conv].

* Only events without a message field (unnamed events) and at least one field with the name error
  are considered for mapping.

By default, these events are mapped.

[conv]: https://github.com/open-telemetry/semantic-conventions/tree/main/docs/exceptions/

<a id="op-fbc04dfa06c8c51beae50b95"></a>
## with_error_events_to_status

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_error_events_to_status` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_error_events_to_status(self, error_events_to_status: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:718`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not an event considered for exception mapping (see [`OpenTelemetryLayer::with_error_records_to_exceptions`](../operations/tracing_opentelemetry.layer.OpenTelemetryLayer.md#op-ca6c5370c59519820ae47282))
should be propagated to the span status error description.

By default, these events do set the span status error description.

<a id="op-615e2df403965b6b79ca1c68"></a>
## with_error_fields_to_exceptions

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_error_fields_to_exceptions` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_error_fields_to_exceptions(self, error_fields_to_exceptions: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:704`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not span and event metadata should include OpenTelemetry
exception fields such as `exception.message` and `exception.backtrace`
when an `Error` value is recorded. If multiple error values are recorded
on the same span/event, only the most recently recorded error value will
show up under these fields.

These attributes follow the [OpenTelemetry semantic conventions for
exceptions][conv].

By default, these attributes are recorded.
Note that this only works for `(dyn Error + 'static)`.
See [Implementations on Foreign Types of tracing::Value][impls] or [`OpenTelemetryLayer::with_error_events_to_exceptions`](../operations/tracing_opentelemetry.layer.OpenTelemetryLayer.md#op-0c2ba26c3c6fd6467cd0ef1b)

[conv]: https://github.com/open-telemetry/semantic-conventions/tree/main/docs/exceptions/
[impls]: https://docs.rs/tracing/0.1.37/tracing/trait.Value.html#foreign-impls

<a id="op-ca6c5370c59519820ae47282"></a>
## with_error_records_to_exceptions

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_error_records_to_exceptions` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_error_records_to_exceptions(self, error_records_to_exceptions: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:763`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not reporting an `Error` value on an event will
propagate the OpenTelemetry exception fields such as `exception.message`
and `exception.backtrace` to the corresponding span. You do not need to
enable `with_exception_fields` in order to enable this. If multiple
error values are recorded on the same span/event, only the most recently
recorded error value will show up under these fields.

These attributes follow the [OpenTelemetry semantic conventions for
exceptions][conv].

By default, these attributes are propagated to the span. Note that this only works for `(dyn Error + 'static)`.
See [Implementations on Foreign Types of tracing::Value][impls] or [`OpenTelemetryLayer::with_error_events_to_exceptions`](../operations/tracing_opentelemetry.layer.OpenTelemetryLayer.md#op-0c2ba26c3c6fd6467cd0ef1b)

[conv]: https://github.com/open-telemetry/semantic-conventions/tree/main/docs/exceptions/
[impls]: https://docs.rs/tracing/0.1.37/tracing/trait.Value.html#foreign-impls

<a id="op-238d234aa76553dfc8254309"></a>
## with_level

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_level` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_level(self, level: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:818`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not span metadata should include the `tracing` verbosity level information as a `level` field.

The level is always added to events, and based on [`OpenTelemetryLayer::with_error_events_to_status`](../operations/tracing_opentelemetry.layer.OpenTelemetryLayer.md#op-fbc04dfa06c8c51beae50b95)
error-level events will mark the span status as an error.

By default, level information is disabled.

<a id="op-84906cb881af06a6334b01ce"></a>
## with_location

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_location` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_location(self, location: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:782`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not span and event metadata should include OpenTelemetry
attributes with location information, such as the file, module and line number.

These attributes follow the [OpenTelemetry semantic conventions for
source locations][conv].

By default, locations are enabled.

[conv]: https://github.com/open-telemetry/semantic-conventions/blob/main/docs/general/attributes.md#source-code-attributes/

<a id="op-5b33505acf106f606b7d5668"></a>
## with_target

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_target` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_target(self, target: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:828`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not span metadata should include an attribute with `target` from `tracing` spans.

By default, the target attribute is enabled..

<a id="op-b4a2cb55b89cc5378fed1296"></a>
## with_threads

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_threads` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_threads(self, threads: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:805`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not spans record additional attributes for the thread
name and thread ID of the thread they were created on, following the
[OpenTelemetry semantic conventions for threads][conv].

By default, thread attributes are enabled.

[conv]: https://github.com/open-telemetry/semantic-conventions/blob/main/docs/general/attributes.md#general-thread-attributes/

<a id="op-f5aea571cac813a197e53b4b"></a>
## with_tracer

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_tracer` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_tracer<Tracer>(self, tracer: Tracer) -> OpenTelemetryLayer<S, Tracer> where Tracer: otel::Tracer + 'static, Tracer::Span: Send + Sync
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:666`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Set the [`Tracer`] that this layer will use to produce and track
OpenTelemetry [`Span`]s.

[`Tracer`]: opentelemetry::trace::Tracer
[`Span`]: opentelemetry::trace::Span

# Examples

```no_run
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
use opentelemetry::trace::TracerProvider;

// Create an OTLP pipeline exporter for a `trace_demo` service.

let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
    .with_tonic()
    .build()
    .unwrap();

let tracer = opentelemetry_sdk::trace::SdkTracerProvider::builder()
    .with_simple_exporter(otlp_exporter)
    .build()
    .tracer("trace_demo");

// Create a layer with the configured tracer
let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

// Use the tracing subscriber `Registry`, or any other subscriber
// that impls `LookupSpan`
let subscriber = Registry::default().with(otel_layer);
# drop(subscriber);
```

<a id="op-b98e4bc4a8f42e1fc6a0f60b"></a>
## with_tracked_inactivity

`function` · `tracing_opentelemetry::layer::OpenTelemetryLayer::with_tracked_inactivity` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_tracked_inactivity(self, tracked_inactivity: bool) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "T"}}], "constraints": []}}, "id": "tracing_opentelemetry::layer::OpenTelemetryLayer", "path": "OpenTelemetryLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": "otel::Tracer"}}}, {"outlives": "'static"}], "generic_params": [], "type": {"generic": "T"}}}, {"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Send", "path": "Send"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::marker::Sync", "path": "Sync"}}}], "generic_params": [], "type": {"qualified_path": {"args": null, "name": "Span", "self_type": {"generic": "T"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer::Tracer", "path": ""}}}}}]}, "is_negative": false, "span": {"begin": [570, 1], "end": [990, 2], "filename": "src/layer.rs"}, "trait": null, "trait_path": null}`

Source: `src/layer.rs:791`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Sets whether or not spans metadata should include the _busy time_
(total time for which it was entered), and _idle time_ (total time
the span existed but was not entered).

By default, inactivity tracking is enabled.
