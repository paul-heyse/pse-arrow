# `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.simple_log_processor.SimpleLogProcessor.json).

<a id="op-36c5104696b2e52fa1d3440a"></a>
## SimpleLogProcessor

`struct` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SimpleLogProcessor<T: LogExporter>
```

Source: `src/logs/simple_log_processor.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A [`LogProcessor`](../operations/opentelemetry_sdk.logs.log_processor.LogProcessor.md#op-006d2a495c0f4272e9c3d480) designed for testing and debugging purpose, that immediately
exports log records as they are emitted. Log records are exported synchronously
in the same thread that emits the log record.
When using this processor with the OTLP Exporter, the following exporter
features are supported:
- `grpc-tonic`: This requires LoggerProvider to be created within a tokio
  runtime. Logs can be emitted from any thread, including tokio runtime
  threads.
- `reqwest-blocking-client`: LoggerProvider may be created anywhere, but
  logs must be emitted from a non-tokio runtime thread.
- `reqwest-client`: LoggerProvider may be created anywhere, but logs must be
  emitted from a tokio runtime thread.

## Example

### Using a SimpleLogProcessor

```rust
use opentelemetry_sdk::logs::{SimpleLogProcessor, SdkLoggerProvider, LogExporter};
use opentelemetry::global;
use opentelemetry_sdk::logs::InMemoryLogExporter;

let exporter = InMemoryLogExporter::default(); // Replace with an actual exporter
let provider = SdkLoggerProvider::builder()
    .with_simple_exporter(exporter)
    .build();

```


<a id="op-ad9d8e74f83f51ca02f30ad2"></a>
## emit

`function` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor::emit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor", "path": "SimpleLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [151, 2], "filename": "src/logs/simple_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/simple_log_processor.rs:78`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4c964212bbbc08b053561f69"></a>
## event_enabled

`function` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor::event_enabled` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, level: opentelemetry::logs::Severity, target: &str, name: Option<&str>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor", "path": "SimpleLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [151, 2], "filename": "src/logs/simple_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/simple_log_processor.rs:139`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5fcc671934d7915455674efd"></a>
## fmt

`function` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor", "path": "SimpleLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [61, 10], "end": [61, 15], "filename": "src/logs/simple_log_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/simple_log_processor.rs:61`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c2de2b9f895484c65ab1bee6"></a>
## force_flush

`function` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor", "path": "SimpleLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [151, 2], "filename": "src/logs/simple_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/simple_log_processor.rs:115`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-af20e5eb37ae77de654471bb"></a>
## new

`function` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(exporter: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor", "path": "SimpleLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [67, 1], "end": [75, 2], "filename": "src/logs/simple_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/simple_log_processor.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new instance of `SimpleLogProcessor`.

<a id="op-a54e4b02291c44b3abdf139d"></a>
## set_resource

`function` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor", "path": "SimpleLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [151, 2], "filename": "src/logs/simple_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/simple_log_processor.rs:131`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-742269e5511f821ffec79d69"></a>
## shutdown

`function` · `opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::simple_log_processor::SimpleLogProcessor", "path": "SimpleLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [77, 1], "end": [151, 2], "filename": "src/logs/simple_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/simple_log_processor.rs:119`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
