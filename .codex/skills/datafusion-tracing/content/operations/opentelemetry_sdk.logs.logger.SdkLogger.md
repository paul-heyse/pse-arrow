# `opentelemetry_sdk::logs::logger::SdkLogger`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.logger.SdkLogger.json).

<a id="op-5cdec15e2917ca4714194deb"></a>
## SdkLogger

`struct` · `opentelemetry_sdk::logs::logger::SdkLogger` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SdkLogger
```

Source: `src/logs/logger.rs:16`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The object for emitting [`LogRecord`]s.

[`LogRecord`]: opentelemetry::logs::LogRecord

<a id="op-338a6768ab7bd83cf2260ff1"></a>
## LogRecord

`assoc_type` · `opentelemetry_sdk::logs::logger::SdkLogger::LogRecord` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
LogRecord
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger::SdkLogger", "path": "SdkLogger"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [27, 1], "end": [71, 2], "filename": "src/logs/logger.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::Logger", "path": "Logger"}, "trait_path": "opentelemetry::logs::logger::Logger"}`

Source: `src/logs/logger.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-3803374c567198ede6f5979d"></a>
## clone

`function` · `opentelemetry_sdk::logs::logger::SdkLogger::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SdkLogger
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger::SdkLogger", "path": "SdkLogger"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [12, 17], "end": [12, 22], "filename": "src/logs/logger.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/logger.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-cd95fd6c0f8d2f6dc2c30668"></a>
## create_log_record

`function` · `opentelemetry_sdk::logs::logger::SdkLogger::create_log_record` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn create_log_record(&self) -> Self::LogRecord
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger::SdkLogger", "path": "SdkLogger"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [27, 1], "end": [71, 2], "filename": "src/logs/logger.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::Logger", "path": "Logger"}, "trait_path": "opentelemetry::logs::logger::Logger"}`

Source: `src/logs/logger.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-a62faad9757c1cf2fab09f63"></a>
## emit

`function` · `opentelemetry_sdk::logs::logger::SdkLogger::emit` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn emit(&self, record: Self::LogRecord)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger::SdkLogger", "path": "SdkLogger"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [27, 1], "end": [71, 2], "filename": "src/logs/logger.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::Logger", "path": "Logger"}, "trait_path": "opentelemetry::logs::logger::Logger"}`

Source: `src/logs/logger.rs:35`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Emit a `LogRecord`.

<a id="op-495c170d0ae197ba6348662e"></a>
## event_enabled

`function` · `opentelemetry_sdk::logs::logger::SdkLogger::event_enabled` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn event_enabled(&self, level: Severity, target: &str, name: Option<&str>) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger::SdkLogger", "path": "SdkLogger"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [27, 1], "end": [71, 2], "filename": "src/logs/logger.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::Logger", "path": "Logger"}, "trait_path": "opentelemetry::logs::logger::Logger"}`

Source: `src/logs/logger.rs:62`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e3310901d174e258ac0531db"></a>
## fmt

`function` · `opentelemetry_sdk::logs::logger::SdkLogger::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger::SdkLogger", "path": "SdkLogger"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [12, 10], "end": [12, 15], "filename": "src/logs/logger.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/logger.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
