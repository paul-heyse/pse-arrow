# `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.metrics.meter_provider.SdkMeterProvider.json).

<a id="op-d674cddddafa66c4223d1cb6"></a>
## SdkMeterProvider

`struct` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SdkMeterProvider
```

Source: `src/metrics/meter_provider.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Handles the creation and coordination of [Meter](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a)s.

All `Meter`s created by a `MeterProvider` will be associated with the same
[Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b), have the same views applied to them, and have their produced
metric telemetry passed to the configured [MetricReader](../operations/opentelemetry_sdk.metrics.reader.MetricReader.md#op-50fae5ec1b547de01d0084a6)s. This is a
clonable handle to the MeterProvider implementation itself, and cloning it
will create a new reference, not a new instance of a MeterProvider. Dropping
the last reference to it will trigger shutdown of the provider. Shutdown can
also be triggered manually by calling the `shutdown` method.
[Meter](../operations/opentelemetry.metrics.meter.Meter.md#op-5e48a510d8c00731cf60cb7a): opentelemetry::metrics::Meter

<a id="op-8a5f661ed631e12723c669c8"></a>
## builder

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> MeterProviderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [52, 1], "end": [125, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:54`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Return default [MeterProviderBuilder](../operations/opentelemetry_sdk.metrics.meter_provider.MeterProviderBuilder.md#op-4c19a404103c7b3177855dcd)

<a id="op-0539f1e66fd484e845edfb3d"></a>
## clone

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SdkMeterProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [34, 10], "end": [34, 15], "filename": "src/metrics/meter_provider.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/metrics/meter_provider.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f766d57822498b6178b9e184"></a>
## default

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [46, 1], "end": [50, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/metrics/meter_provider.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cc5baf009276cda726ecf9a8"></a>
## fmt

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [34, 17], "end": [34, 22], "filename": "src/metrics/meter_provider.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/metrics/meter_provider.rs:34`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-88d258535036e9af2f71dbac"></a>
## force_flush

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [52, 1], "end": [125, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:97`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Flushes all pending telemetry.

There is no guaranteed that all telemetry be flushed or all resources have
been released on error.

# Examples

```
use opentelemetry::{global, Context};
use opentelemetry_sdk::metrics::SdkMeterProvider;

fn init_metrics() -> SdkMeterProvider {
    // Setup metric pipelines with readers + views, default has no
    // readers so nothing is exported.
    let provider = SdkMeterProvider::default();

    // Set provider to be used as global meter provider
    let _ = global::set_meter_provider(provider.clone());

    provider
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = init_metrics();

    // create instruments + record measurements

    // force all instruments to flush
    provider.force_flush()?;

    // record more measurements..

    // shutdown ensures any cleanup required by the provider is done,
    // and also invokes shutdown on the readers.
    provider.shutdown()?;

    Ok(())
}
```

<a id="op-be47e361a58fb5a761499930"></a>
## meter

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::meter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn meter(&self, name: &'static str) -> Meter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [185, 1], "end": [228, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": {"args": null, "id": "opentelemetry::metrics::meter::MeterProvider", "path": "MeterProvider"}, "trait_path": "opentelemetry::metrics::meter::MeterProvider"}`

Source: `src/metrics/meter_provider.rs:186`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d30e2311a3c0740bd37b1ea0"></a>
## meter_with_scope

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::meter_with_scope` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn meter_with_scope(&self, scope: InstrumentationScope) -> Meter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [185, 1], "end": [228, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": {"args": null, "id": "opentelemetry::metrics::meter::MeterProvider", "path": "MeterProvider"}, "trait_path": "opentelemetry::metrics::meter::MeterProvider"}`

Source: `src/metrics/meter_provider.rs:191`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-54bcfe57b4191a69c08e73f3"></a>
## shutdown

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [52, 1], "end": [125, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:122`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

shutdown with default timeout

<a id="op-e98e5800da11dc70df13ac1c"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::metrics::meter_provider::SdkMeterProvider", "path": "SdkMeterProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [52, 1], "end": [125, 2], "filename": "src/metrics/meter_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/metrics/meter_provider.rs:113`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the meter provider flushing all pending telemetry and releasing
any held computational resources.

This call is idempotent. The first call will perform all flush and releasing
operations. Subsequent calls will perform no action and will return an error
stating this.

Measurements made by instruments from meters this MeterProvider created will
not be exported after Shutdown is called.

There is no guaranteed that all telemetry be flushed or all resources have
been released on error.
