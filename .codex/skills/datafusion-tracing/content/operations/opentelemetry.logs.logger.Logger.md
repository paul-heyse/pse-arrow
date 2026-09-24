# `opentelemetry::logs::logger::Logger`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.logs.logger.Logger.json).

<a id="op-f433cc26744e85b74bee1072"></a>
## Logger

`trait` · `opentelemetry::logs::logger::Logger` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait Logger
```

Source: `src/logs/logger.rs:9`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The interface for emitting [`LogRecord`](../operations/opentelemetry.logs.record.LogRecord.md#op-ed315ba3d46385b5b2839847)s.

<a id="op-b07d3d1dcfd2552a12896a2e"></a>
## LogRecord

`assoc_type` · `opentelemetry::logs::logger::Logger::LogRecord` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
LogRecord
```

Source: `src/logs/logger.rs:11`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Specifies the `LogRecord` type associated with this logger.

<a id="op-5e394ad90231ac88b07cb3b6"></a>
## create_log_record

`function` · `opentelemetry::logs::logger::Logger::create_log_record` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn create_log_record(&self) -> Self::LogRecord
```

Source: `src/logs/logger.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Creates a new log record builder.

<a id="op-9097ebee40c4f48e7306dc29"></a>
## emit

`function` · `opentelemetry::logs::logger::Logger::emit` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn emit(&self, record: Self::LogRecord)
```

Source: `src/logs/logger.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Emit a [`LogRecord`](../operations/opentelemetry.logs.record.LogRecord.md#op-ed315ba3d46385b5b2839847). If there is active current thread's [`Context`],
 the logger will set the record's `TraceContext` to the active trace context,

[`Context`]: crate::Context

<a id="op-da3472c3a2bb9f8347883625"></a>
## event_enabled

`function` · `opentelemetry::logs::logger::Logger::event_enabled` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool
```

Source: `src/logs/logger.rs:24`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Check if the given log level is enabled.
