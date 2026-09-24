# `opentelemetry_sdk::logs::export::LogExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.export.LogExporter.json).

<a id="op-9c9fdf7e313b8939d1e3635d"></a>
## LogExporter

`trait` · `opentelemetry_sdk::logs::export::LogExporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait LogExporter: Send + Sync + Debug
```

Source: `src/logs/export.rs:116`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`LogExporter` defines the interface that log exporters should implement.

<a id="op-cc20038b275a0266d63d193b"></a>
## event_enabled

`function` · `opentelemetry_sdk::logs::export::LogExporter::event_enabled` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, _level: Severity, _target: &str, _name: Option<&str>) -> bool
```

Source: `src/logs/export.rs:148`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Check if logs are enabled.

<a id="op-4a1b9eded851a0d1c0c2df0e"></a>
## export

`function` · `opentelemetry_sdk::logs::export::LogExporter::export` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export(&self, batch: LogBatch<'_>) -> impl std::future::Future<Output = OTelSdkResult> + Send
```

Source: `src/logs/export.rs:134`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Exports a batch of log records and their associated instrumentation scopes.

The `export` method is responsible for sending a batch of log records to an external
destination. It takes a `LogBatch` as an argument, which contains references to the
log records and their corresponding instrumentation scopes. The method returns
a `LogResult` indicating the success or failure of the export operation.

# Arguments

* `batch` - A `LogBatch` containing the log records and instrumentation scopes
  to be exported.

# Returns

A `LogResult<()>`, which is a result type indicating either a successful export (with
`Ok(())`) or an error (`Err(LogError)`) if the export operation failed.


<a id="op-0709f4368fab17a264f4881b"></a>
## set_resource

`function` · `opentelemetry_sdk::logs::export::LogExporter::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, _resource: &Resource)
```

Source: `src/logs/export.rs:153`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the resource for the exporter.

<a id="op-92e02fa32373c51138d36a96"></a>
## shutdown

`function` · `opentelemetry_sdk::logs::export::LogExporter::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Source: `src/logs/export.rs:143`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the exporter with a default timeout.

<a id="op-7abb41e0ea9a66857fa7590f"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::logs::export::LogExporter::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, _timeout: time::Duration) -> OTelSdkResult
```

Source: `src/logs/export.rs:139`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the exporter.
