# `opentelemetry_sdk::resource::env::SdkProvidedResourceDetector`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.env.SdkProvidedResourceDetector.json).

<a id="op-26736b4967b9ad610ce0d1d3"></a>
## SdkProvidedResourceDetector

`struct` · `opentelemetry_sdk::resource::env::SdkProvidedResourceDetector` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SdkProvidedResourceDetector
```

Source: `src/resource/env.rs:73`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

There are attributes which MUST be provided by the SDK as specified in
[the Resource SDK specification]. This detector detects those attributes and
if the attribute cannot be detected, it uses the default value.

This detector will first try `OTEL_SERVICE_NAME` env. If it's not available,
then it will check the `OTEL_RESOURCE_ATTRIBUTES` env and see if it contains
`service.name` resource. If it's also not available, it will use `unknown_service`.

If users want to set an empty service name, they can provide
a resource with empty value and `service.name` key.

[the Resource SDK specification]:https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/sdk.md#sdk-provided-resource-attributes

<a id="op-a3e183355f9bd5162fca9f31"></a>
## detect

`function` · `opentelemetry_sdk::resource::env::SdkProvidedResourceDetector::detect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn detect(&self) -> Resource
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::env::SdkProvidedResourceDetector", "path": "SdkProvidedResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [75, 1], "end": [93, 2], "filename": "src/resource/env.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::resource::ResourceDetector", "path": "ResourceDetector"}, "trait_path": "opentelemetry_sdk::resource::ResourceDetector"}`

Source: `src/resource/env.rs:76`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5028bb564e38642784828b9d"></a>
## fmt

`function` · `opentelemetry_sdk::resource::env::SdkProvidedResourceDetector::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::env::SdkProvidedResourceDetector", "path": "SdkProvidedResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [72, 10], "end": [72, 15], "filename": "src/resource/env.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/resource/env.rs:72`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
