# `opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.in_memory_exporter.OwnedLogData.json).

<a id="op-26b749087ee33265ce79eb1e"></a>
## OwnedLogData

`struct` · `opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct OwnedLogData
```

Source: `src/logs/in_memory_exporter.rs:57`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`OwnedLogData` represents a single log event without resource context.

<a id="op-f6b14a74a4c8bbd1965e91f9"></a>
## clone

`function` · `opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> OwnedLogData
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData", "path": "OwnedLogData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [56, 17], "end": [56, 22], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/in_memory_exporter.rs:56`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cebc07263c34aa07333403cd"></a>
## fmt

`function` · `opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData", "path": "OwnedLogData"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [56, 10], "end": [56, 15], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/in_memory_exporter.rs:56`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-84f86166181a6fa1ad8709df"></a>
## instrumentation

`struct_field` · `opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData::instrumentation` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
instrumentation: opentelemetry::InstrumentationScope
```

Source: `src/logs/in_memory_exporter.rs:61`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Instrumentation details for the emitter who produced this `LogEvent`.

<a id="op-c78be603fdcd01f4964ea58f"></a>
## record

`struct_field` · `opentelemetry_sdk::logs::in_memory_exporter::OwnedLogData::record` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
record: logs::SdkLogRecord
```

Source: `src/logs/in_memory_exporter.rs:59`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Log record, which can be borrowed or owned.
