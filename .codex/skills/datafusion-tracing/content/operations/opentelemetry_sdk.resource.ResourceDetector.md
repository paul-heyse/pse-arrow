# `opentelemetry_sdk::resource::ResourceDetector`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.ResourceDetector.json).

<a id="op-b0082125b9054a261f267f3c"></a>
## ResourceDetector

`trait` · `opentelemetry_sdk::resource::ResourceDetector` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait ResourceDetector
```

Source: `src/resource/mod.rs:258`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

ResourceDetector detects OpenTelemetry resource information

Implementations of this trait can be passed to
the [`ResourceBuilder::with_detectors`](../operations/opentelemetry_sdk.resource.ResourceBuilder.md#op-61042f593a630f40ef2305fc) function to generate a Resource from the merged information.

<a id="op-040436f9a1da1672f2062614"></a>
## detect

`function` · `opentelemetry_sdk::resource::ResourceDetector::detect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn detect(&self) -> Resource
```

Source: `src/resource/mod.rs:265`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

detect returns an initialized Resource based on gathered information.

If source information to construct a Resource is inaccessible, an empty Resource should be returned

If source information to construct a Resource is invalid, for example,
missing required values. an empty Resource should be returned.
