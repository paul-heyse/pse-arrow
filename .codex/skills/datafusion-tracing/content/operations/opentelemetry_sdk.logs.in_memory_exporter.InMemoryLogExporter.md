# `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.in_memory_exporter.InMemoryLogExporter.json).

<a id="op-d52de7d5d6ce22136962a3d1"></a>
## InMemoryLogExporter

`struct` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InMemoryLogExporter
```

Source: `src/logs/in_memory_exporter.rs:42`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

 An in-memory logs exporter that stores logs data in memory.

 This exporter is useful for testing and debugging purposes.
 It stores logs in a `Vec<OwnedLogData>`. Logs can be retrieved using
 `get_emitted_logs` method.

 # Example
 ```no_run
# use opentelemetry_sdk::logs::{BatchLogProcessor, SdkLoggerProvider};
# use opentelemetry_sdk::runtime;
# use opentelemetry_sdk::logs::InMemoryLogExporter;

# #[tokio::main]
# async fn main() {
    // Create an InMemoryLogExporter
    let exporter: InMemoryLogExporter = InMemoryLogExporter::default();
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


<a id="op-175c90cd4df32ba818fd3637"></a>
## clone

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> InMemoryLogExporter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 10], "end": [41, 15], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/in_memory_exporter.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b3b0ffb740d3564287ed2cff"></a>
## default

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [49, 1], "end": [53, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/logs/in_memory_exporter.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4312239ba21e746fe81dae96"></a>
## export

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::export` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
async fn export(&self, batch: LogBatch<'_>) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [195, 1], "end": [223, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}, "trait_path": "opentelemetry_sdk::logs::export::LogExporter"}`

Source: `src/logs/in_memory_exporter.rs:196`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eb75f179a10f7fea35bc5ec3"></a>
## fmt

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [41, 17], "end": [41, 22], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/in_memory_exporter.rs:41`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4c4d21376a2b6fa4a8f990ae"></a>
## get_emitted_logs

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::get_emitted_logs` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get_emitted_logs(&self) -> Result<Vec<LogDataWithResource>, InMemoryExporterError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [143, 1], "end": [193, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/in_memory_exporter.rs:161`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the logs emitted via Logger as a vector of `LogDataWithResource`.

# Example

```
use opentelemetry_sdk::logs::{InMemoryLogExporter, InMemoryLogExporterBuilder};

let exporter = InMemoryLogExporterBuilder::default().build();
let emitted_logs = exporter.get_emitted_logs().unwrap();
```


<a id="op-49ddd7d9459daee8279c0a6e"></a>
## is_shutdown_called

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::is_shutdown_called` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_shutdown_called(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [143, 1], "end": [193, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/in_memory_exporter.rs:145`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns true if shutdown was called.

<a id="op-1b1efa93cef27fc679b000f9"></a>
## reset

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::reset` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn reset(&self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [143, 1], "end": [193, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/in_memory_exporter.rs:186`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Clears the internal (in-memory) storage of logs.

# Example

```
use opentelemetry_sdk::logs::{InMemoryLogExporter, InMemoryLogExporterBuilder};

let exporter = InMemoryLogExporterBuilder::default().build();
exporter.reset();
```


<a id="op-39e596ddab143209655fbf18"></a>
## set_resource

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [195, 1], "end": [223, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}, "trait_path": "opentelemetry_sdk::logs::export::LogExporter"}`

Source: `src/logs/in_memory_exporter.rs:219`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-82908a5dfd4fd59f0b7ff2e3"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: time::Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::in_memory_exporter::InMemoryLogExporter", "path": "InMemoryLogExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [195, 1], "end": [223, 2], "filename": "src/logs/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}, "trait_path": "opentelemetry_sdk::logs::export::LogExporter"}`

Source: `src/logs/in_memory_exporter.rs:210`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
