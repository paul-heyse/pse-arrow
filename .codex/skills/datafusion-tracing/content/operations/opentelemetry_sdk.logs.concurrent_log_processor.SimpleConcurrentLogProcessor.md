# `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.concurrent_log_processor.SimpleConcurrentLogProcessor.json).

<a id="op-7d6ea224a70d51155069ba56"></a>
## SimpleConcurrentLogProcessor

`struct` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SimpleConcurrentLogProcessor<T: LogExporter>
```

Source: `src/logs/concurrent_log_processor.rs:17`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

A concurrent log processor calls exporter's export method on each emit. This
processor does not buffer logs. Note: This invokes exporter's export method
on the current thread without synchronization. i.e multiple export() calls
can happen simultaneously from different threads. This is not a problem if
the exporter is designed to handle that. As of now, exporters in the
opentelemetry-rust project (stdout/otlp) are not thread-safe.
This is intended to be used when exporting to operating system
tracing facilities like Windows ETW, Linux TracePoints etc.

<a id="op-174f6b17710bbe3ddcdab1c9"></a>
## emit

`function` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor::emit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn emit(&self, record: &mut SdkLogRecord, instrumentation: &InstrumentationScope)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor", "path": "SimpleConcurrentLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [28, 1], "end": [65, 2], "filename": "src/logs/concurrent_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/concurrent_log_processor.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b0e4de0efef982e1eab0472b"></a>
## event_enabled

`function` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor::event_enabled` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, level: opentelemetry::logs::Severity, target: &str, name: Option<&str>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor", "path": "SimpleConcurrentLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [28, 1], "end": [65, 2], "filename": "src/logs/concurrent_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/concurrent_log_processor.rs:53`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e91287a584e08e71280ba57e"></a>
## fmt

`function` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor", "path": "SimpleConcurrentLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "core::fmt::Debug", "path": "$crate::fmt::Debug"}}}, {"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [16, 10], "end": [16, 15], "filename": "src/logs/concurrent_log_processor.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/concurrent_log_processor.rs:16`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7c7e66ea5f898d5b4fac0af2"></a>
## force_flush

`function` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor", "path": "SimpleConcurrentLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [28, 1], "end": [65, 2], "filename": "src/logs/concurrent_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/concurrent_log_processor.rs:40`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e3df56dff9f09dd7d63dbe86"></a>
## new

`function` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor::new` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new(exporter: T) -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor", "path": "SimpleConcurrentLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [21, 1], "end": [26, 2], "filename": "src/logs/concurrent_log_processor.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/concurrent_log_processor.rs:23`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creates a new `ConcurrentExportProcessor` with the given exporter.

<a id="op-e87604699b3c463781e3c224"></a>
## set_resource

`function` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor", "path": "SimpleConcurrentLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [28, 1], "end": [65, 2], "filename": "src/logs/concurrent_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/concurrent_log_processor.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3a48cd3c74d3ebee708d08be"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": {"angle_bracketed": {"args": [{"type": {"generic": "T"}}], "constraints": []}}, "id": "opentelemetry_sdk::logs::concurrent_log_processor::SimpleConcurrentLogProcessor", "path": "SimpleConcurrentLogProcessor"}}, "generics": {"params": [{"kind": {"type": {"bounds": [{"trait_bound": {"generic_params": [], "modifier": "none", "trait": {"args": null, "id": "opentelemetry_sdk::logs::export::LogExporter", "path": "LogExporter"}}}], "default": null, "is_synthetic": false}}, "name": "T"}], "where_predicates": []}, "is_negative": false, "span": {"begin": [28, 1], "end": [65, 2], "filename": "src/logs/concurrent_log_processor.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::logs::log_processor::LogProcessor", "path": "LogProcessor"}, "trait_path": "opentelemetry_sdk::logs::log_processor::LogProcessor"}`

Source: `src/logs/concurrent_log_processor.rs:47`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
