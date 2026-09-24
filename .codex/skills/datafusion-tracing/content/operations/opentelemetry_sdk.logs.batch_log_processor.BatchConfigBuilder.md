# `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.batch_log_processor.BatchConfigBuilder.json).

<a id="op-8a44e53cd1337d327a79ec1c"></a>
## BatchConfigBuilder

`struct` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct BatchConfigBuilder
```

Source: `src/logs/batch_log_processor.rs:602`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A builder for creating [`BatchConfig`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchConfig.md#op-bb4a63bc88aa52bc3da0560f) instances.

<a id="op-9eba7ef9379603d46e92a8af"></a>
## build

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> BatchConfig
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [632, 1], "end": [733, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:687`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builds a `BatchConfig` enforcing the following invariants:
* `max_export_batch_size` must be less than or equal to `max_queue_size`.

<a id="op-0cf47a535d361f6287ae719c"></a>
## default

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [610, 1], "end": [630, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/logs/batch_log_processor.rs:620`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new [`BatchConfigBuilder`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchConfigBuilder.md#op-8a44e53cd1337d327a79ec1c) initialized with default batch config values as per the specs.
The values are overridden by environment variables if set.
The supported environment variables are:
* `OTEL_BLRP_MAX_QUEUE_SIZE`
* `OTEL_BLRP_SCHEDULE_DELAY`
* `OTEL_BLRP_MAX_EXPORT_BATCH_SIZE`
* `OTEL_BLRP_EXPORT_TIMEOUT`

Note: Programmatic configuration overrides any value set via the environment variable.

<a id="op-88cb3c2d16ec62251946d9b8"></a>
## fmt

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [601, 10], "end": [601, 15], "filename": "src/logs/batch_log_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/batch_log_processor.rs:601`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4d8e0e100381e515c6f9cfe6"></a>
## with_max_export_batch_size

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder::with_max_export_batch_size` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_export_batch_size(self, max_export_batch_size: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [632, 1], "end": [733, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:680`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set max_export_batch_size for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchConfigBuilder.md#op-8a44e53cd1337d327a79ec1c).
It's the maximum number of logs to process in a single batch. If there are
more than one batch worth of logs then it processes multiple batches
of logs one batch after the other without any delay.
The default value is 512.

Corresponding environment variable: `OTEL_BLRP_MAX_EXPORT_BATCH_SIZE`.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-eb7fcfe981d21c41b03d0bb8"></a>
## with_max_export_timeout

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder::with_max_export_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_export_timeout(self, max_export_timeout: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [632, 1], "end": [733, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:666`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set max_export_timeout for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchConfigBuilder.md#op-8a44e53cd1337d327a79ec1c).
It's the maximum duration to export a batch of data.
The default value is 30000 milliseconds.

Corresponding environment variable: `OTEL_BLRP_EXPORT_TIMEOUT`.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-3ca124ae7b228ded676eebb6"></a>
## with_max_queue_size

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder::with_max_queue_size` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_max_queue_size(self, max_queue_size: usize) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [632, 1], "end": [733, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:641`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set max_queue_size for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchConfigBuilder.md#op-8a44e53cd1337d327a79ec1c).
It's the maximum queue size to buffer logs for delayed processing.
If the queue gets full it will drop the logs.
The default value is 2048.

Corresponding environment variable: `OTEL_BLRP_MAX_QUEUE_SIZE`.

Note: Programmatically setting this will override any value set via the environment variable.

<a id="op-8b98f896b0a52e8b45f5a6b1"></a>
## with_scheduled_delay

`function` · `opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder::with_scheduled_delay` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_scheduled_delay(self, scheduled_delay: Duration) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::batch_log_processor::BatchConfigBuilder", "path": "BatchConfigBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [632, 1], "end": [733, 2], "filename": "src/logs/batch_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/batch_log_processor.rs:653`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set scheduled_delay for [`BatchConfigBuilder`](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchConfigBuilder.md#op-8a44e53cd1337d327a79ec1c).
It's the delay interval in milliseconds between two consecutive processing of batches.
The default value is 1000 milliseconds.

Corresponding environment variable: `OTEL_BLRP_SCHEDULE_DELAY`.

Note: Programmatically setting this will override any value set via the environment variable.
