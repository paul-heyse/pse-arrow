# `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.logger_provider.LoggerProviderBuilder.json).

<a id="op-94307fa3cc13278b7b91a49e"></a>
## LoggerProviderBuilder

`struct` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct LoggerProviderBuilder
```

Source: `src/logs/logger_provider.rs:184`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Builder for provider attributes.

<a id="op-a7b0c74de9ab620a40443c29"></a>
## build

`function` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder::build` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn build(self) -> SdkLoggerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder", "path": "LoggerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [189, 1], "end": [282, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:263`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new provider from this configuration.

<a id="op-39460fa014ded6ceb0250986"></a>
## default

`function` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> LoggerProviderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder", "path": "LoggerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [182, 17], "end": [182, 24], "filename": "src/logs/logger_provider.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/logs/logger_provider.rs:182`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5bfb4f67926289aa4ec4c0c6"></a>
## fmt

`function` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder", "path": "LoggerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [182, 10], "end": [182, 15], "filename": "src/logs/logger_provider.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/logger_provider.rs:182`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-eff394e1a87599e00fec30fc"></a>
## with_batch_exporter

`function` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder::with_batch_exporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_batch_exporter<T: LogExporter + 'static>(self, exporter: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder", "path": "LoggerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [189, 1], "end": [282, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:226`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a [BatchLogProcessor](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchLogProcessor.md#op-ab788d8cd13fbb60e7393fa3) with the configured exporter to the pipeline,
using the default [super::BatchConfig](../operations/opentelemetry_sdk.logs.batch_log_processor.BatchConfig.md#op-bb4a63bc88aa52bc3da0560f).

The following environment variables can be used to configure the batching configuration:

* `OTEL_BLRP_SCHEDULE_DELAY` - Corresponds to `with_scheduled_delay`.
* `OTEL_BLRP_MAX_QUEUE_SIZE` - Corresponds to `with_max_queue_size`.
* `OTEL_BLRP_MAX_EXPORT_BATCH_SIZE` - Corresponds to `with_max_export_batch_size`.

# Arguments

* `exporter` - The exporter to be used by the `BatchLogProcessor`.

# Returns

A new `LoggerProviderBuilder` instance with the `BatchLogProcessor` added to the pipeline.

Processors are invoked in the order they are added.

<a id="op-e1a0fb743e28d6965eee08a9"></a>
## with_log_processor

`function` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder::with_log_processor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_log_processor<T: LogProcessor + 'static>(self, processor: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder", "path": "LoggerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [189, 1], "end": [282, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:242`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a custom [LogProcessor](../operations/opentelemetry_sdk.logs.log_processor.LogProcessor.md#op-006d2a495c0f4272e9c3d480) to the pipeline.

# Arguments

* `processor` - The `LogProcessor` to be added.

# Returns

A new `Builder` instance with the custom `LogProcessor` added to the pipeline.

Processors are invoked in the order they are added.

<a id="op-5b21af66a3b26b732e6da988"></a>
## with_resource

`function` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder::with_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_resource(self, resource: Resource) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder", "path": "LoggerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [189, 1], "end": [282, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:253`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The `Resource` to be associated with this Provider.

*Note*: Calls to this method are additive, each call merges the provided
resource with the previous one.

<a id="op-49fead75c6e9d8666d529e56"></a>
## with_simple_exporter

`function` · `opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder::with_simple_exporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn with_simple_exporter<T: LogExporter + 'static>(self, exporter: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::LoggerProviderBuilder", "path": "LoggerProviderBuilder"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [189, 1], "end": [282, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:201`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Adds a [SimpleLogProcessor](../operations/opentelemetry_sdk.logs.simple_log_processor.SimpleLogProcessor.md#op-36c5104696b2e52fa1d3440a) with the configured exporter to the pipeline.

# Arguments

* `exporter` - The exporter to be used by the SimpleLogProcessor.

# Returns

A new `Builder` instance with the SimpleLogProcessor added to the pipeline.

Processors are invoked in the order they are added.
