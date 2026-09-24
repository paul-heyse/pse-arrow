# `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.batch_log_processor.BatchLogProcessor.json).

<a id="op-ab788d8cd13fbb60e7393fa3"></a>
## BatchLogProcessor

`struct` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchLogProcessor
```

Source: `src/logs/batch_log_processor.rs:128`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The `BatchLogProcessor` collects finished logs in a buffer and exports them
in batches to the configured `LogExporter`. This processor is ideal for
high-throughput environments, as it minimizes the overhead of exporting logs
individually. It uses a **dedicated background thread** to manage and export logs
asynchronously, ensuring that the application's main execution flow is not blocked.

This processor supports the following configurations:
- **Queue size**: Maximum number of log records that can be buffered.
- **Batch size**: Maximum number of log records to include in a single export.
- **Scheduled delay**: Frequency at which the batch is exported.

When using this processor with the OTLP Exporter, the following exporter
features are supported:
- `grpc-tonic`: Requires `LoggerProvider` to be created within a tokio runtime.
- `reqwest-blocking-client`: Works with a regular `main` or `tokio::main`.

In other words, other clients like `reqwest` and `hyper` are not supported.

`BatchLogProcessor` buffers logs in memory and exports them in batches. An
export is triggered when `max_export_batch_size` is reached or every
`scheduled_delay` milliseconds. Users can explicitly trigger an export using
the `force_flush` method. Shutdown also triggers an export of all buffered
logs and is recommended to be called before the application exits to ensure
all buffered logs are exported.

**Warning**: When using tokio's current-thread runtime, `shutdown()`, which
is a blocking call ,should not be called from your main thread. This can
cause deadlock. Instead, call `shutdown()` from a separate thread or use
tokio's `spawn_blocking`.


### Using a BatchLogProcessor:

```rust
use opentelemetry_sdk::logs::{BatchLogProcessor, BatchConfigBuilder, SdkLoggerProvider};
use opentelemetry::global;
use std::time::Duration;
use opentelemetry_sdk::logs::InMemoryLogExporter;

let exporter = InMemoryLogExporter::default(); // Replace with an actual exporter
let processor = BatchLogProcessor::builder(exporter)
    .with_batch_config(
        BatchConfigBuilder::default()
            .with_max_queue_size(2048)
            .with_max_export_batch_size(512)
            .with_scheduled_delay(Duration::from_secs(5))
            .build(),
    )
    .build();

let provider = SdkLoggerProvider::builder()
    .with_log_processor(processor)
    .build();


<a id="op-87f99252f7c01d4c22042f70"></a>
## builder

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder<E>(exporter: E) -> BatchLogProcessorBuilder<E> where E: LogExporter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [331, 1], "end": [511, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:502`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new batch processor builder

<a id="op-7404f3ea403d71bee14f77ff"></a>
## emit

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor::emit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [152, 1], "end": [329, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/batch_log_processor.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-baf3264d83834f06094f437a"></a>
## fmt

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [144, 1], "end": [150, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/batch_log_processor.rs:145`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-41b5a8ea83f8ed34cc85a31d"></a>
## force_flush

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [152, 1], "end": [329, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/batch_log_processor.rs:224`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-aafdf3718464e5f60708f3fb"></a>
## set_resource

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [152, 1], "end": [329, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/batch_log_processor.rs:323`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-07022473ce464f1994bf2a7d"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchLogProcessor", "path": "BatchLogProcessor"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [152, 1], "end": [329, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/batch_log_processor.rs:260`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
