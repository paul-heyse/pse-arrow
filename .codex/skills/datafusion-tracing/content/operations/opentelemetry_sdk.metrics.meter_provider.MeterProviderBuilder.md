# `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.meter_provider.MeterProviderBuilder.json).

<a id="op-4c19a404103c7b3177855dcd"></a>
## MeterProviderBuilder

`struct` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct MeterProviderBuilder
```

Source: `src/metrics/meter_provider.rs:232`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Configuration options for a [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b).

<a id="op-f5be1bdbe02b59fe68ed58d3"></a>
## build

`function` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> SdkMeterProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder", "path": "MeterProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 1], "end": [399, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:376`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Construct a new [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) with this configuration.

<a id="op-7b72074a89c16b94d19477e5"></a>
## default

`function` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> MeterProviderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder", "path": "MeterProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [231, 10], "end": [231, 17], "filename": "src/metrics/meter_provider.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/meter_provider.rs:231`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-253d48e126946a8849d21fb0"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder", "path": "MeterProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [401, 1], "end": [409, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/meter_provider.rs:402`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3571f1a138368cdc7c39e880"></a>
## with_periodic_exporter

`function` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder::with_periodic_exporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_periodic_exporter<T>(self, exporter: T) -> Self where T: PushMetricExporter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder", "path": "MeterProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 1], "end": [399, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:282`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a [`PushMetricExporter`](../operations/opentelemetry_sdk.metrics.exporter.PushMetricExporter.md#op-6d578ed5c0fd27cf6ab55b23) to the [`MeterProvider`](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) and configures it
to export metrics at **fixed** intervals (60 seconds) using a
[`PeriodicReader`](../operations/opentelemetry_sdk.metrics.periodic_reader.PeriodicReader.md#op-7af448cc206ead85a28fc452).

To customize the export interval, set the
**"OTEL_METRIC_EXPORT_INTERVAL"** environment variable (in
milliseconds).

Most users should use this method to attach an exporter. Advanced users
who need finer control over the export process can use
[`crate::metrics::PeriodicReaderBuilder`](../operations/opentelemetry_sdk.metrics.periodic_reader.PeriodicReaderBuilder.md#op-1ccb928c4d2c2ae19ce4569c) to configure a custom reader and attach it
using [`MeterProviderBuilder::with_reader()`](../operations/opentelemetry_sdk.metrics.meter_provider.MeterProviderBuilder.md#op-9da9ee3a647ce6b7538ecbea).

<a id="op-9da9ee3a647ce6b7538ecbea"></a>
## with_reader

`function` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder::with_reader` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_reader<T: MetricReader>(self, reader: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder", "path": "MeterProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 1], "end": [399, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:265`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Associates a [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6) with a [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b).
[`MeterProviderBuilder::with_periodic_exporter()](../operations/opentelemetry_sdk.metrics.meter_provider.MeterProviderBuilder.md#op-3571f1a138368cdc7c39e880) can be used to add a PeriodicReader which is
the most common use case.

A [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) will export no metrics without [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6)
added.

<a id="op-ba40ecc38af80fe2a860ec07"></a>
## with_resource

`function` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder::with_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_resource(self, resource: Resource) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder", "path": "MeterProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 1], "end": [399, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:250`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Associates a [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) with a [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b).

This [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) represents the entity producing telemetry and is associated
with all [Meter]s the [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) will create.

By default, if this option is not used, the default [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) will be used.

*Note*: Calls to this method are additive, each call merges the provided
resource with the previous one.

[Meter]: opentelemetry::metrics::Meter

<a id="op-aa35adec4514e43d89129e3f"></a>
## with_view

`function` · `opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder::with_view` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_view<T>(self, view: T) -> Self where T: Fn(&Instrument) -> Option<Stream> + Send + Sync + 'static
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::MeterProviderBuilder", "path": "MeterProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [238, 1], "end": [399, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:367`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a view to the [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b).

Views allow you to customize how metrics are aggregated, renamed, or
otherwise transformed before export, without modifying instrument
definitions in your application or library code.

You can pass any function or closure matching the signature
`Fn(&Instrument) -> Option<Stream> + Send + Sync + 'static`. The
function receives a reference to the [`Instrument`] and can return an
[`Option`] of [`Stream`] to specify how matching instruments should be
exported. Returning `None` means the view does not apply to the given
instrument, and the default behavior will be used.


# Examples

Renaming a metric:

```
# use opentelemetry_sdk::metrics::{Stream, Instrument};
let view_rename = |i: &Instrument| {
    if i.name() == "my_counter" {
        Some(Stream::builder().with_name("my_counter_renamed").build().expect("Stream should be valid"))
    } else {
        None
    }
};
# let builder = opentelemetry_sdk::metrics::SdkMeterProvider::builder();
# let _builder =
builder.with_view(view_rename);
```

Setting a cardinality limit to control resource usage:

```
# use opentelemetry_sdk::metrics::{Stream, Instrument};
let view_change_cardinality = |i: &Instrument| {
    if i.name() == "my_counter" {
        Some(
            Stream::builder()
                .with_cardinality_limit(100).build().expect("Stream should be valid"),
        )
    } else {
        None
    }
};
# let builder = opentelemetry_sdk::metrics::SdkMeterProvider::builder();
# let _builder =
builder.with_view(view_change_cardinality);
```

Silently ignoring Stream build errors:

```
# use opentelemetry_sdk::metrics::{Stream, Instrument};
let my_view_change_cardinality = |i: &Instrument| {
    if i.name() == "my_second_histogram" {
        // Note: If Stream is invalid, build() will return `Error` variant.
        // By calling `.ok()`, any such error is ignored and treated as if the view does not match
        // the instrument.
        // If this is not the desired behavior, consider handling the error explicitly.
        Stream::builder().with_cardinality_limit(0).build().ok()
    } else {
        None
    }
};
# let builder = opentelemetry_sdk::metrics::SdkMeterProvider::builder();
# let _builder =
builder.with_view(my_view_change_cardinality);
```

If no views are added, the [MeterProvider](../operations/opentelemetry.metrics.meter.MeterProvider.md#op-8ef66c51648972f9d178f50b) uses the default view.

[`Instrument`]: crate::metrics::Instrument
[`Stream`]: crate::metrics::Stream
[`Option`]: core::option::Option

Unresolved upstream links (retained, not inferred): `core::option::Option`.
