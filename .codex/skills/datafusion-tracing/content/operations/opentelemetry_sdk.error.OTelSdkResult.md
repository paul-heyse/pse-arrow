# `opentelemetry_sdk::error::OTelSdkResult`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.error.OTelSdkResult.json).

<a id="op-f679ec97d54627042378207b"></a>
## OTelSdkResult

`type_alias` · `opentelemetry_sdk::error::OTelSdkResult` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
type OTelSdkResult = std::result::Result<(), OTelSdkError>
```

Source: `src/error.rs:45`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A specialized `Result` type for Shutdown operations.
