# `opentelemetry::logs::noop::NoopLoggerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.logs.noop.NoopLoggerProvider.json).

<a id="op-5f207c179c422bd4d8121c03"></a>
## NoopLoggerProvider

`struct` · `opentelemetry::logs::noop::NoopLoggerProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct NoopLoggerProvider
```

Source: `src/logs/noop.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

A no-op implementation of a [`LoggerProvider`](../operations/opentelemetry.logs.logger.LoggerProvider.md#op-730d84df0e8b727068bb7ebd).

<a id="op-dd7ef508092ff5f1e5fe86d4"></a>
## Logger

`assoc_type` · `opentelemetry::logs::noop::NoopLoggerProvider::Logger` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Logger
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::noop::NoopLoggerProvider", "path": "NoopLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [19, 1], "end": [25, 2], "filename": "src/logs/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::LoggerProvider", "path": "LoggerProvider"}, "trait_path": "opentelemetry::logs::logger::LoggerProvider"}`

Source: `src/logs/noop.rs:20`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-00e759fc963749ca08ae99f6"></a>
## clone

`function` · `opentelemetry::logs::noop::NoopLoggerProvider::clone` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> NoopLoggerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::noop::NoopLoggerProvider", "path": "NoopLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 10], "end": [9, 15], "filename": "src/logs/noop.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/logs/noop.rs:9`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-90b3303e320587c35a5bb01b"></a>
## default

`function` · `opentelemetry::logs::noop::NoopLoggerProvider::default` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> NoopLoggerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::noop::NoopLoggerProvider", "path": "NoopLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 24], "end": [9, 31], "filename": "src/logs/noop.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/logs/noop.rs:9`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0be47c9e07a6e80b22975a26"></a>
## fmt

`function` · `opentelemetry::logs::noop::NoopLoggerProvider::fmt` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::noop::NoopLoggerProvider", "path": "NoopLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [9, 17], "end": [9, 22], "filename": "src/logs/noop.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/logs/noop.rs:9`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bec0b400b1a7183a1c192b7a"></a>
## logger_with_scope

`function` · `opentelemetry::logs::noop::NoopLoggerProvider::logger_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn logger_with_scope(&self, _scope: InstrumentationScope) -> Self::Logger
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::noop::NoopLoggerProvider", "path": "NoopLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [19, 1], "end": [25, 2], "filename": "src/logs/noop.rs"}, "trait": {"args": null, "id": "opentelemetry::logs::logger::LoggerProvider", "path": "LoggerProvider"}, "trait_path": "opentelemetry::logs::logger::LoggerProvider"}`

Source: `src/logs/noop.rs:22`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-bc1502301c094cb9242265d4"></a>
## new

`function` · `opentelemetry::logs::noop::NoopLoggerProvider::new` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn new() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry::logs::noop::NoopLoggerProvider", "path": "NoopLoggerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [12, 1], "end": [17, 2], "filename": "src/logs/noop.rs"}, "trait": null, "trait_path": null}`

Source: `src/logs/noop.rs:14`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Create a new no-op logger provider.
