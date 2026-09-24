# `opentelemetry_sdk::resource::env::EnvResourceDetector`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.resource.env.EnvResourceDetector.json).

<a id="op-ba004408ea4c1fba764aa989"></a>
## EnvResourceDetector

`struct` · `opentelemetry_sdk::resource::env::EnvResourceDetector` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct EnvResourceDetector
```

Source: `src/resource/env.rs:17`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

EnvResourceDetector extract resource from environment variable
`OTEL_RESOURCE_ATTRIBUTES`. See [OpenTelemetry Resource
Spec](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/sdk.md#specifying-resource-information-via-an-environment-variable)
for details.

<a id="op-ce292741dd9d69fd7128d628"></a>
## default

`function` · `opentelemetry_sdk::resource::env::EnvResourceDetector::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::env::EnvResourceDetector", "path": "EnvResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [37, 1], "end": [41, 2], "filename": "src/resource/env.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/resource/env.rs:38`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0ead790df98b4979ef213218"></a>
## detect

`function` · `opentelemetry_sdk::resource::env::EnvResourceDetector::detect` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn detect(&self) -> Resource
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::env::EnvResourceDetector", "path": "EnvResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [21, 1], "end": [28, 2], "filename": "src/resource/env.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::resource::ResourceDetector", "path": "ResourceDetector"}, "trait_path": "opentelemetry_sdk::resource::ResourceDetector"}`

Source: `src/resource/env.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-63b9617c6ed9c011c915487b"></a>
## fmt

`function` · `opentelemetry_sdk::resource::env::EnvResourceDetector::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::env::EnvResourceDetector", "path": "EnvResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [16, 10], "end": [16, 15], "filename": "src/resource/env.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/resource/env.rs:16`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-32b106d65feee5c86d7df625"></a>
## new

`function` · `opentelemetry_sdk::resource::env::EnvResourceDetector::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::resource::env::EnvResourceDetector", "path": "EnvResourceDetector"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [30, 1], "end": [35, 2], "filename": "src/resource/env.rs"}, "trait": null, "trait_path": null}`

Source: `src/resource/env.rs:32`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create `EnvResourceDetector` instance.
