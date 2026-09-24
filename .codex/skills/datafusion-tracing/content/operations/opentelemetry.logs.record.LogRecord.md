# `opentelemetry::logs::record::LogRecord`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.logs.record.LogRecord.json).

<a id="op-ed315ba3d46385b5b2839847"></a>
## LogRecord

`trait` · `opentelemetry::logs::record::LogRecord` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait LogRecord
```

Source: `src/logs/record.rs:8`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

SDK implemented trait for managing log records

<a id="op-6ea6076c97fbd0127756e706"></a>
## add_attribute

`function` · `opentelemetry::logs::record::LogRecord::add_attribute` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_attribute<K, V>(&mut self, key: K, value: V) where K: Into<Key>, V: Into<AnyValue>
```

Source: `src/logs/record.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Adds a single attribute.

<a id="op-6618af64cba71f4c9fc00611"></a>
## add_attributes

`function` · `opentelemetry::logs::record::LogRecord::add_attributes` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_attributes<I, K, V>(&mut self, attributes: I) where I: IntoIterator<Item = (K, V)>, K: Into<Key>, V: Into<AnyValue>
```

Source: `src/logs/record.rs:36`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Adds multiple attributes.

<a id="op-5eea564a520c4fb77ae1056d"></a>
## set_body

`function` · `opentelemetry::logs::record::LogRecord::set_body` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_body(&mut self, body: AnyValue)
```

Source: `src/logs/record.rs:33`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the message body of the log.

<a id="op-cb0471fa36ffa188659f8dd2"></a>
## set_event_name

`function` · `opentelemetry::logs::record::LogRecord::set_event_name` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_event_name(&mut self, name: &'static str)
```

Source: `src/logs/record.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the `event_name` of a record

<a id="op-6d8197a0b96806f00de494b9"></a>
## set_observed_timestamp

`function` · `opentelemetry::logs::record::LogRecord::set_observed_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_observed_timestamp(&mut self, timestamp: SystemTime)
```

Source: `src/logs/record.rs:24`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the observed event timestamp.

<a id="op-54ad8395061d2fd925ba3cd6"></a>
## set_severity_number

`function` · `opentelemetry::logs::record::LogRecord::set_severity_number` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_severity_number(&mut self, number: Severity)
```

Source: `src/logs/record.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets severity as a numeric value.

<a id="op-e893aa21d204bb706b9e2e1a"></a>
## set_severity_text

`function` · `opentelemetry::logs::record::LogRecord::set_severity_text` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_severity_text(&mut self, text: &'static str)
```

Source: `src/logs/record.rs:27`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets severity as text.

<a id="op-e37769dc1a7bd815601b95ad"></a>
## set_target

`function` · `opentelemetry::logs::record::LogRecord::set_target` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_target<T>(&mut self, _target: T) where T: Into<Cow<'static, str>>
```

Source: `src/logs/record.rs:16`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the `target` of a record.
Currently, both `opentelemetry-appender-tracing` and `opentelemetry-appender-log` create a single logger
with a scope that doesn't accurately reflect the component emitting the logs.
Exporters MAY use this field to override the `instrumentation_scope.name`.

<a id="op-e904f4dd731bf9262a36db92"></a>
## set_timestamp

`function` · `opentelemetry::logs::record::LogRecord::set_timestamp` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_timestamp(&mut self, timestamp: SystemTime)
```

Source: `src/logs/record.rs:21`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the time when the event occurred measured by the origin clock, i.e. the time at the source.

<a id="op-bb4da72e26cf068ae58c1389"></a>
## set_trace_context

`function` · `opentelemetry::logs::record::LogRecord::set_trace_context` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_trace_context(&mut self, trace_id: TraceId, span_id: SpanId, trace_flags: Option<TraceFlags>)
```

Source: `src/logs/record.rs:49`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Sets the trace context of the log.
