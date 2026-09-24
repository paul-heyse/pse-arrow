# `opentelemetry_sdk::resource::telemetry::TelemetryResourceDetector`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.telemetry.TelemetryResourceDetector.json).

<a id="op-a1229b78527252090864a606"></a>
## TelemetryResourceDetector

`struct` · `opentelemetry_sdk::resource::telemetry::TelemetryResourceDetector` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct TelemetryResourceDetector
```

Source: `src/resource/telemetry.rs:15`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Detect the telemetry SDK information used to capture data recorded by the instrumentation libraries.

It provides:
- The name of the telemetry SDK(`telemetry.sdk.name`). It will be `opentelemetry` for SDK provided by opentelemetry project.
- The language of the telemetry SDK(`telemetry.sdk.language`). It will be `rust` for this SDK.
- The version of the telemetry SDK(`telemetry.sdk.version`). It will be current `opentelemetry_sdk` crate version.


See [semantic conventions](https://github.com/open-telemetry/semantic-conventions/blob/main/docs/resource/README.md#telemetry-sdk) for details.

<a id="op-1e3253a6907818690acbe7df"></a>
## detect

`function` · `opentelemetry_sdk::resource::telemetry::TelemetryResourceDetector::detect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn detect(&self) -> Resource
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::telemetry::TelemetryResourceDetector", "path": "TelemetryResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [17, 1], "end": [27, 2], "filename": "src/resource/telemetry.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::resource::ResourceDetector", "path": "ResourceDetector"}, "trait_path": "opentelemetry_sdk::resource::ResourceDetector"}`

Source: `src/resource/telemetry.rs:18`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-19ceb13da6b001fc58fa159d"></a>
## fmt

`function` · `opentelemetry_sdk::resource::telemetry::TelemetryResourceDetector::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::telemetry::TelemetryResourceDetector", "path": "TelemetryResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [14, 10], "end": [14, 15], "filename": "src/resource/telemetry.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/resource/telemetry.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
