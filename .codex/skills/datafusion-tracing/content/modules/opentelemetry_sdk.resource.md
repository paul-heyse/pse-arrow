# `opentelemetry_sdk::resource`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.json).

<a id="op-25bc270cb7b54207239e8276"></a>
## resource

`module` · `opentelemetry_sdk::resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod resource
```

Source: `src/resource/mod.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Representations of entities producing telemetry.

A [Resource](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) is an immutable representation of the entity producing
telemetry as attributes. For example, a process producing telemetry that is
running in a container on Kubernetes has a Pod name, it is in a namespace
and possibly is part of a Deployment which also has a name. All three of
these attributes can be included in the `Resource`. Note that there are
certain ["standard attributes"] that have prescribed meanings.

["standard attributes"]: https://github.com/open-telemetry/opentelemetry-specification/blob/v1.9.0/specification/resource/semantic_conventions/README.md

# Resource detectors

[`ResourceDetector`](../operations/opentelemetry_sdk.resource.ResourceDetector.md#op-b0082125b9054a261f267f3c)s are used to detect resource from runtime or
environmental variables. The following are provided by default with this
SDK.

- [`EnvResourceDetector`](../operations/opentelemetry_sdk.resource.env.EnvResourceDetector.md#op-ba004408ea4c1fba764aa989) - detect resource from environmental variables.
- [`TelemetryResourceDetector`](../operations/opentelemetry_sdk.resource.telemetry.TelemetryResourceDetector.md#op-a1229b78527252090864a606) - detect telemetry SDK's information.

The OS and Process resource detectors are packaged separately in the
[`opentelemetry-resource-detector` crate](https://github.com/open-telemetry/opentelemetry-rust-contrib/tree/main/opentelemetry-resource-detectors).
