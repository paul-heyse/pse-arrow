# `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.logs.logger_provider.SdkLoggerProvider.json).

<a id="op-410821400e0083f96db20f29"></a>
## SdkLoggerProvider

`struct` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SdkLoggerProvider
```

Source: `src/logs/logger_provider.rs:43`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Handles the creation and coordination of [`Logger`]s.

All `Logger`s created by a `SdkLoggerProvider` will share the same
[`Resource`] and have their created log records processed by the
configured log processors. This is a clonable handle to the `SdkLoggerProvider`
itself, and cloning it will create a new reference, not a new instance of a
`SdkLoggerProvider`. Dropping the last reference will trigger the shutdown of
the provider, ensuring that all remaining logs are flushed and no further
logs are processed. Shutdown can also be triggered manually by calling
the [`shutdown`](SdkLoggerProvider::shutdown) method.

[`Logger`]: opentelemetry::logs::Logger
[`Resource`]: crate::Resource

<a id="op-0c6df763bab3d37a05d71684"></a>
## Logger

`assoc_type` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::Logger` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Logger
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [47, 1], "end": [73, 2], "filename": "src/logs/logger_provider.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::LoggerProvider", "path": "LoggerProvider"}, "trait_path": "opentelemetry::logs::logger::LoggerProvider"}`

Source: `src/logs/logger_provider.rs:48`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-f81ecba487e55cb470f7e25a"></a>
## builder

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> LoggerProviderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [75, 1], "end": [132, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:77`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new `LoggerProvider` builder.

<a id="op-90557b88d1eac0dfba72af71"></a>
## clone

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SdkLoggerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 17], "end": [29, 22], "filename": "src/logs/logger_provider.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/logger_provider.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8213e2300e3c3b53ed0141ae"></a>
## fmt

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [29, 10], "end": [29, 15], "filename": "src/logs/logger_provider.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/logger_provider.rs:29`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-753d6dec843742df970842e7"></a>
## force_flush

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [75, 1], "end": [132, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:86`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Force flush all remaining logs in log processors and return results.

<a id="op-c69db7ae22a19b02c664e961"></a>
## logger

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::logger` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::Logger
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [47, 1], "end": [73, 2], "filename": "src/logs/logger_provider.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::LoggerProvider", "path": "LoggerProvider"}, "trait_path": "opentelemetry::logs::logger::LoggerProvider"}`

Source: `src/logs/logger_provider.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-05c17835114bcfd55b6d54a4"></a>
## logger_with_scope

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::logger_with_scope` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn logger_with_scope(&self, scope: InstrumentationScope) -> Self::Logger
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [47, 1], "end": [73, 2], "filename": "src/logs/logger_provider.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::LoggerProvider", "path": "LoggerProvider"}, "trait_path": "opentelemetry::logs::logger::LoggerProvider"}`

Source: `src/logs/logger_provider.rs:55`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d9521c09029ec8a0087d8aa8"></a>
## shutdown

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [75, 1], "end": [132, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:129`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down this `LoggerProvider` with default timeout

<a id="op-6dc972c972ff0829e5c385cf"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::logs::logger_provider::SdkLoggerProvider", "path": "SdkLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [75, 1], "end": [132, 2], "filename": "src/logs/logger_provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/logger_provider.rs:100`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down this `LoggerProvider`
