# `opentelemetry_sdk::error::ExportError`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.error.ExportError.json).

<a id="op-ac94e38299dd551e2e0c98f5"></a>
## ExportError

`trait` · `opentelemetry_sdk::error::ExportError` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait ExportError: std::error::Error + Send + Sync + 'static
```

Source: `src/error.rs:8`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Trait for errors returned by exporters

<a id="op-bc0434c11de0f8d10c1300ad"></a>
## exporter_name

`function` · `opentelemetry_sdk::error::ExportError::exporter_name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn exporter_name(&self) -> &'static str
```

Source: `src/error.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The name of exporter that returned this error
