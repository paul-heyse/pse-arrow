# `opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.in_memory_exporter.LogDataWithResource.json).

<a id="op-90db5abdcdde97d771d44aaa"></a>
## LogDataWithResource

`struct` · `opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct LogDataWithResource
```

Source: `src/logs/in_memory_exporter.rs:67`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`LogDataWithResource` associates a [`SdkLogRecord`](../operations/opentelemetry_sdk.logs.record.SdkLogRecord.md#op-a6ac6aa7af3967beebb57fb0) with a [`Resource`](../operations/opentelemetry_sdk.resource.Resource.md#op-6665d2e0897d180bf11ccb3b) and
[`InstrumentationScope`](../operations/opentelemetry.common.InstrumentationScope.md#op-5a674011980d528952015efb).

<a id="op-27fcf58c03486e5c10d3a992"></a>
## clone

`function` · `opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> LogDataWithResource
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource", "path": "LogDataWithResource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 10], "end": [66, 15], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/in_memory_exporter.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f40fc50d6ede09e16fef0d60"></a>
## fmt

`function` · `opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource", "path": "LogDataWithResource"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [66, 17], "end": [66, 22], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/in_memory_exporter.rs:66`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e8f76129eea9bea084a1cc14"></a>
## instrumentation

`struct_field` · `opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource::instrumentation` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
instrumentation: opentelemetry::InstrumentationScope
```

Source: `src/logs/in_memory_exporter.rs:71`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Instrumentation details for the emitter who produced this `LogRecord`.

<a id="op-5e51bbf72c4737f64686a59a"></a>
## record

`struct_field` · `opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource::record` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
record: logs::SdkLogRecord
```

Source: `src/logs/in_memory_exporter.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Log record

<a id="op-24e1e0e0c2937b33c1898d90"></a>
## resource

`struct_field` · `opentelemetry_sdk::logs::in_memory_exporter::LogDataWithResource::resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
resource: std::borrow::Cow<'static, Resource>
```

Source: `src/logs/in_memory_exporter.rs:73`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Resource for the emitter who produced this `LogRecord`.
