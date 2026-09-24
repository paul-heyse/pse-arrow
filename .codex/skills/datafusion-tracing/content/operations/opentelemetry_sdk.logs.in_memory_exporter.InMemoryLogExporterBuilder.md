# `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.in_memory_exporter.InMemoryLogExporterBuilder.json).

<a id="op-3ce186935aefef51e4e27e11"></a>
## InMemoryLogExporterBuilder

`struct` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InMemoryLogExporterBuilder
```

Source: `src/logs/in_memory_exporter.rs:103`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builder for [`InMemoryLogExporter`](../operations/opentelemetry_sdk.logs.in_memory_exporter.InMemoryLogExporter.md#op-d52de7d5d6ce22136962a3d1).
 # Example

 ```no_run
# use opentelemetry_sdk::logs::{InMemoryLogExporter, InMemoryLogExporterBuilder};
# use opentelemetry_sdk::logs::{BatchLogProcessor, SdkLoggerProvider};
# use opentelemetry_sdk::runtime;

# #[tokio::main]
# async fn main() {
    //Create an InMemoryLogExporter
    let exporter: InMemoryLogExporter = InMemoryLogExporterBuilder::default().build();
    //Create a LoggerProvider and register the exporter
    let logger_provider = SdkLoggerProvider::builder()
        .with_log_processor(BatchLogProcessor::builder(exporter.clone()).build())
        .build();
    // Setup Log Appenders and emit logs. (Not shown here)
    logger_provider.force_flush();
    let emitted_logs = exporter.get_emitted_logs().unwrap();
    for log in emitted_logs {
        println!("{:?}", log);
    }
# }

 ```


<a id="op-dee1171cdeabf7c5a745d39e"></a>
## build

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(&self) -> InMemoryLogExporter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder", "path": "InMemoryLogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [113, 1], "end": [141, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/in_memory_exporter.rs:124`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of `InMemoryLogExporter`.


<a id="op-0523fd701862ab5635c071cb"></a>
## clone

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> InMemoryLogExporterBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder", "path": "InMemoryLogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [102, 17], "end": [102, 22], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/in_memory_exporter.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eb8faa30164c87a0195d247d"></a>
## default

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder", "path": "InMemoryLogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [107, 1], "end": [111, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/logs/in_memory_exporter.rs:108`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3fecffa14d506c3d21ee3805"></a>
## fmt

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder", "path": "InMemoryLogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [102, 10], "end": [102, 15], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/in_memory_exporter.rs:102`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2d8b3b91e14017179447d5d4"></a>
## new

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporterBuilder", "path": "InMemoryLogExporterBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [113, 1], "end": [141, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/in_memory_exporter.rs:116`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of `InMemoryLogExporter`.

