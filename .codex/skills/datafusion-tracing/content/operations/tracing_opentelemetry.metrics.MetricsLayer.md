# `tracing_opentelemetry::metrics::MetricsLayer`

Full upstream contracts; raw type trees and source locators in [structured records](tracing_opentelemetry.metrics.MetricsLayer.json).

<a id="op-89f067ec31171359b3c6376e"></a>
## MetricsLayer

`struct` · `tracing_opentelemetry::metrics::MetricsLayer` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
struct MetricsLayer<S, M>
```

Source: `src/metrics.rs:372`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

A layer that publishes metrics via the OpenTelemetry SDK.

# Usage

No configuration is needed for this Layer, as it's only responsible for
pushing data out to the `opentelemetry` family of crates. For example, when
using `opentelemetry-otlp`, that crate will provide its own set of
configuration options for setting up the duration metrics will be collected
before exporting to the OpenTelemetry Collector, aggregation of data points,
etc.

```no_run
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::Registry;
# use opentelemetry_sdk::metrics::SdkMeterProvider;

// Constructing a MeterProvider is out-of-scope for the docs here, but there
// are examples in the opentelemetry repository. See:
// https://github.com/open-telemetry/opentelemetry-rust/blob/dfeac078ff7853e7dc814778524b93470dfa5c9c/examples/metrics-basic/src/main.rs#L7
# let meter_provider: SdkMeterProvider = unimplemented!();

let opentelemetry_metrics =  MetricsLayer::new(meter_provider);
let subscriber = Registry::default().with(opentelemetry_metrics);
tracing::subscriber::set_global_default(subscriber).unwrap();
```

To publish a new metric, add a key-value pair to your `tracing::Event` that
contains following prefixes:
- `monotonic_counter.` (non-negative numbers): Used when the counter should
  only ever increase
- `counter.`: Used when the counter can go up or down
- `histogram.`: Used to report arbitrary values that are likely to be statistically meaningful
- `gauge.`: Used to report instantaneous values that can go up or down

Examples:
```
# use tracing::info;
info!(monotonic_counter.foo = 1);
info!(monotonic_counter.bar = 1.1);

info!(counter.baz = 1);
info!(counter.baz = -1);
info!(counter.xyz = 1.1);

info!(histogram.qux = 1);
info!(histogram.abc = -1);
info!(histogram.def = 1.1);

info!(gauge.foo = 1);
info!(gauge.bar = 1.1);
```

# Mixing data types

## Floating-point numbers

Do not mix floating point and non-floating point numbers for the same
metric. If a floating point number will be used for a given metric, be sure
to cast any other usages of that metric to a floating point number.

Do this:
```
# use tracing::info;
info!(monotonic_counter.foo = 1_f64);
info!(monotonic_counter.foo = 1.1);
```

This is because all data published for a given metric name must be the same
numeric type.

## Integers

Positive and negative integers can be mixed freely. The instrumentation
provided by `tracing` assumes that all integers are `i64` unless explicitly
cast to something else. In the case that an integer *is* cast to `u64`, this
subscriber will handle the conversion internally.

For example:
```
# use tracing::info;
// The subscriber receives an i64
info!(counter.baz = 1);

// The subscriber receives an i64
info!(counter.baz = -1);

// The subscriber receives a u64, but casts it to i64 internally
info!(counter.baz = 1_u64);

// The subscriber receives a u64, but cannot cast it to i64 because of
// overflow. An error is printed to stderr, and the metric is dropped.
info!(counter.baz = (i64::MAX as u64) + 1)
```

# Attributes

When `MetricsLayer` outputs metrics, it converts key-value pairs into [Attributes] and associates them with metrics.

[Attributes]: https://opentelemetry.io/docs/specs/otel/common/#attribute

For example:
```
# use tracing::info;
// adds attributes bar="baz" and qux=2 to the `foo` counter.
info!(monotonic_counter.foo = 1, bar = "baz", qux = 2);
```

# Implementation Details

`MetricsLayer` holds a set of maps, with each map corresponding to a
type of metric supported by OpenTelemetry. These maps are populated lazily.
The first time that a metric is emitted by the instrumentation, a `Metric`
instance will be created and added to the corresponding map. This means that
any time a metric is emitted by the instrumentation, one map lookup has to
be performed.

In the future, this can be improved by associating each `Metric` instance to
its callsite, eliminating the need for any maps.

<a id="op-2d232cde6a05a2b5aafeaace"></a>
## enabled

`function` · `tracing_opentelemetry::metrics::MetricsLayer::enabled` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:484`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f554c9b4499957aa19106c54"></a>
## new

`function` · `tracing_opentelemetry::metrics::MetricsLayer::new` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(meter_provider: M) -> MetricsLayer<S, M> where M: MeterProvider
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [378, 1], "end": [403, 2], "filename": "src/metrics.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics.rs:383`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

Create a new instance of MetricsLayer.

<a id="op-9136ff2a9803c14edd446e23"></a>
## on_close

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_close` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_close(&self, id: tracing_core::span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:531`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8a3c0ce517d198c24b7afcb7"></a>
## on_enter

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_enter` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_enter(&self, id: &tracing_core::span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:523`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-db4b6e308fa0bff88e525f40"></a>
## on_event

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_event` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_event(&self, event: &tracing_core::Event<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:519`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-38dfeaf22146f7305fead540"></a>
## on_exit

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_exit` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_exit(&self, id: &tracing_core::span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:527`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-90a3a48e73a1fa65d7ba105b"></a>
## on_follows_from

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_follows_from` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_follows_from(&self, span: &tracing_core::span::Id, follows: &tracing_core::span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:510`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7d3376218dd1ddbcfa5b3c36"></a>
## on_id_change

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_id_change` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_id_change(&self, old: &tracing_core::span::Id, new: &tracing_core::span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:535`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4dee6b2a7a214e394e93fbf0"></a>
## on_layer

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_layer` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_layer(&mut self, subscriber: &mut S)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:476`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-74ecc1d55ef38a7de3f2cc90"></a>
## on_new_span

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_new_span` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_new_span(&self, attrs: &tracing_core::span::Attributes<'_>, id: &tracing_core::span::Id, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:488`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e56cfeb973028c6e44d43477"></a>
## on_record

`function` · `tracing_opentelemetry::metrics::MetricsLayer::on_record` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_record(&self, span: &tracing_core::span::Id, values: &tracing_core::span::Record<'_>, ctx: Context<'_, S>)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:501`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-ecbdf2f2a03eae97984b919c"></a>
## register_callsite

`function` · `tracing_opentelemetry::metrics::MetricsLayer::register_callsite` · tracing-opentelemetry 0.32.0
Reachability: `supported`.  Capture: hosted.

```rust
fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}, {"type": {"generic": "M"}}], "constraints": []}}, "id": "tracing_opentelemetry::metrics::MetricsLayer", "path": "MetricsLayer"}}, "generics": {"params": [{"kind": {"type": {"bounds": [], "default": null, "is_synthetic": false}}, "name": "S"}, {"kind": {"type": {"bounds": [{"outlives": "'static"}], "default": null, "is_synthetic": false}}, "name": "M"}], "where_predicates": [{"bound_predicate": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "tracing_core::subscriber::Subscriber", "path": "Subscriber"}}}, {"trait_bound": {"generic_params": [{"kind": {"lifetime": {"outlives": []}}, "name": "'span"}], "modifier": "none", "trait": {"args": {"angle_bracketed": {"args": [{"lifetime": "'span"}], "constraints": []}}, "id": "tracing_subscriber::registry::LookupSpan", "path": "LookupSpan"}}}], "generic_params": [], "type": {"generic": "S"}}}]}, "is_negative": false, "span": {"begin": [472, 1], "end": [543, 2], "filename": "src/metrics.rs"}, "trait": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "S"}}], "constraints": []}}, "id": "tracing_subscriber::layer::Layer", "path": "Layer"}, "trait_path": "tracing_subscriber::layer::Layer"}`

Source: `src/metrics.rs:480`. [Exact documentation build](https://docs.rs/crate/tracing-opentelemetry/0.32.0/json).

No upstream documentation on this item; consult its owner/trait contract.
