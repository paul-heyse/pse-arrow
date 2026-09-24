# `opentelemetry_sdk::logs::record::SdkLogRecord`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.record.SdkLogRecord.json).

<a id="op-a6ac6aa7af3967beebb57fb0"></a>
## SdkLogRecord

`struct` · `opentelemetry_sdk::logs::record::SdkLogRecord` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SdkLogRecord
```

Source: `src/logs/record.rs:26`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

LogRecord represents all data carried by a log record, and
is provided to `LogExporter`s as input.

<a id="op-2b3a865fc57dc04ae1e35d5b"></a>
## add_attribute

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::add_attribute` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_attribute<K, V>(&mut self, key: K, value: V) where K: Into<Key>, V: Into<AnyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:99`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6a3075ed98356c59d6002175"></a>
## add_attributes

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::add_attributes` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn add_attributes<I, K, V>(&mut self, attributes: I) where I: IntoIterator<Item = (K, V)>, K: Into<Key>, V: Into<AnyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-b220d79f89642481677cb1c2"></a>
## attributes_iter

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::attributes_iter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn attributes_iter(&self) -> impl Iterator<Item = &(Key, AnyValue)>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:187`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Provides an iterator over the attributes.

<a id="op-124583e22019895eb727a1db"></a>
## body

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::body` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn body(&self) -> Option<&AnyValue>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:181`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the body

<a id="op-46aef46580bbf6474e1eb731"></a>
## clone

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SdkLogRecord
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [22, 17], "end": [22, 22], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/record.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-7a92fd69d44318ea4236d677"></a>
## eq

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::eq` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn eq(&self, other: &SdkLogRecord) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [22, 24], "end": [22, 33], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::cmp::PartialEq", "path": "PartialEq"}, "trait_path": "core::cmp::PartialEq"}`

Source: `src/logs/record.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-13622c20f024479f41abdc88"></a>
## event_name

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::event_name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn event_name(&self) -> Option<&'static str>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:139`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the event name

<a id="op-46182d397617cd3adbaf5770"></a>
## fmt

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [22, 10], "end": [22, 15], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/record.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3a752fbd3cea0821cec9762a"></a>
## observed_timestamp

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::observed_timestamp` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn observed_timestamp(&self) -> Option<SystemTime>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:157`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the observed timestamp

<a id="op-abaa5937af40e762df2fad9b"></a>
## set_body

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_body` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_body(&mut self, body: AnyValue)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:84`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6a58d7221df48e2a489b56f5"></a>
## set_event_name

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_event_name` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_event_name(&mut self, name: &'static str)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:56`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3c7d1767e1c1e384b91ccea0"></a>
## set_observed_timestamp

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_observed_timestamp` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_observed_timestamp(&mut self, timestamp: SystemTime)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:72`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-61acb6b4edfffbb251d04932"></a>
## set_severity_number

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_severity_number` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_severity_number(&mut self, severity_number: Severity)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:80`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2033ec7917dc718233a6cb16"></a>
## set_severity_text

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_severity_text` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_severity_text(&mut self, severity_text: &'static str)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:76`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-6e6bd319a2e6ceb9137ee96d"></a>
## set_target

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_target` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_target<T>(&mut self, _target: T) where T: Into<Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:61`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f5c06bd95e6a805ffcb41e76"></a>
## set_timestamp

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_timestamp` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_timestamp(&mut self, timestamp: SystemTime)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:68`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c6318c294c9dd13952cf7986"></a>
## set_trace_context

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::set_trace_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_trace_context(&mut self, trace_id: TraceId, span_id: SpanId, trace_flags: Option<TraceFlags>)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [55, 1], "end": [119, 2], "filename": "src/logs/record.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::record::LogRecord", "path": "LogRecord"}, "trait_path": "opentelemetry::logs::record::LogRecord"}`

Source: `src/logs/record.rs:107`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-1e93327f6a11655b40b85ab5"></a>
## severity_number

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::severity_number` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn severity_number(&self) -> Option<Severity>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:175`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the severity number

<a id="op-9d6d397975d7a6086cc312f9"></a>
## severity_text

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::severity_text` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn severity_text(&self) -> Option<&'static str>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:169`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the severity text

<a id="op-ef18d526be66f8658e91732b"></a>
## target

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::target` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn target(&self) -> Option<&Cow<'static, str>>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:145`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the target

<a id="op-e66ed4dad91a7439ebeff9c5"></a>
## timestamp

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::timestamp` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn timestamp(&self) -> Option<SystemTime>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:151`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the timestamp

<a id="op-24b6a790fa418401045249f3"></a>
## trace_context

`function` · `opentelemetry_sdk::logs::record::SdkLogRecord::trace_context` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn trace_context(&self) -> Option<&TraceContext>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::record::SdkLogRecord", "path": "SdkLogRecord"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [121, 1], "end": [205, 2], "filename": "src/logs/record.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/record.rs:163`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the trace context
