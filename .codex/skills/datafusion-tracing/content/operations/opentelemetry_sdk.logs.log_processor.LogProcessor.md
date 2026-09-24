# `opentelemetry_sdk::logs::log_processor::LogProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.log_processor.LogProcessor.json).

<a id="op-006d2a495c0f4272e9c3d480"></a>
## LogProcessor

`trait` · `opentelemetry_sdk::logs::log_processor::LogProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait LogProcessor: Send + Sync + Debug
```

Source: `src/logs/log_processor.rs:42`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The interface for plugging into a [`SdkLogger`].

[`SdkLogger`]: crate::logs::SdkLogger

<a id="op-1574ea14d76366f4d31fcb77"></a>
## emit

`function` · `opentelemetry_sdk::logs::log_processor::LogProcessor::emit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn emit(&self, data: &mut SdkLogRecord, instrumentation: &InstrumentationScope)
```

Source: `src/logs/log_processor.rs:54`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Called when a log record is ready to processed and exported.

This method receives a mutable reference to `LogRecord`. If the processor
needs to handle the export asynchronously, it should clone the data to
ensure it can be safely processed without lifetime issues. Any changes
made to the log data in this method will be reflected in the next log
processor in the chain.

# Parameters
- `record`: A mutable reference to `LogRecord` representing the log record.
- `instrumentation`: The instrumentation scope associated with the log record.

<a id="op-2ae4632e52dab136dc50ebf0"></a>
## event_enabled

`function` · `opentelemetry_sdk::logs::log_processor::LogProcessor::event_enabled` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, _level: Severity, _target: &str, _name: Option<&str>) -> bool
```

Source: `src/logs/log_processor.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Check if logging is enabled

<a id="op-0366827982c1b3791c7c729b"></a>
## force_flush

`function` · `opentelemetry_sdk::logs::log_processor::LogProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Source: `src/logs/log_processor.rs:56`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Force the logs lying in the cache to be exported.

<a id="op-3d18e7708ed22e5ac1c32dda"></a>
## set_resource

`function` · `opentelemetry_sdk::logs::log_processor::LogProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, _resource: &Resource)
```

Source: `src/logs/log_processor.rs:75`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the resource for the log processor.

<a id="op-2544b000c57bcfd9ed5f8d2f"></a>
## shutdown

`function` · `opentelemetry_sdk::logs::log_processor::LogProcessor::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Source: `src/logs/log_processor.rs:64`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the processor with default timeout.

<a id="op-e1e40c55f907613c7cbb1d64"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::logs::log_processor::LogProcessor::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: Duration) -> OTelSdkResult
```

Source: `src/logs/log_processor.rs:60`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the processor.
After shutdown returns the log processor should stop processing any logs.
It's up to the implementation on when to drop the LogProcessor.
