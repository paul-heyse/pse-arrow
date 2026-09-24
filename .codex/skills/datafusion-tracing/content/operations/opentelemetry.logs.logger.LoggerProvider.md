# `opentelemetry::logs::logger::LoggerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.logs.logger.LoggerProvider.json).

<a id="op-730d84df0e8b727068bb7ebd"></a>
## LoggerProvider

`trait` · `opentelemetry::logs::logger::LoggerProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait LoggerProvider
```

Source: `src/logs/logger.rs:28`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Interfaces that can create [`Logger`](../operations/opentelemetry.logs.logger.Logger.md#op-f433cc26744e85b74bee1072) instances.

<a id="op-5b4e4352a1efdf812df9049d"></a>
## Logger

`assoc_type` · `opentelemetry::logs::logger::LoggerProvider::Logger` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Logger
```

Source: `src/logs/logger.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The [`Logger`](../operations/opentelemetry.logs.logger.Logger.md#op-f433cc26744e85b74bee1072) type that this provider will return.

<a id="op-3ae2287d589c86b8e7627db1"></a>
## logger

`function` · `opentelemetry::logs::logger::LoggerProvider::logger` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn logger(&self, name: impl Into<Cow<'static, str>>) -> Self::Logger
```

Source: `src/logs/logger.rs:60`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new logger with the given name.

The `name` should be the application name or the name of the library
providing instrumentation.

<a id="op-bb84088389b251205d4c519b"></a>
## logger_with_scope

`function` · `opentelemetry::logs::logger::LoggerProvider::logger_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn logger_with_scope(&self, scope: InstrumentationScope) -> Self::Logger
```

Source: `src/logs/logger.rs:54`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new logger with the given instrumentation scope.

# Examples

```
use opentelemetry::InstrumentationScope;
use opentelemetry::logs::LoggerProvider;
use opentelemetry_sdk::logs::SdkLoggerProvider;

let provider = SdkLoggerProvider::builder().build();

// logger used in applications/binaries
let logger = provider.logger("my_app");

// logger used in libraries/crates that optionally includes version and schema url
let scope = InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
    .with_version(env!("CARGO_PKG_VERSION"))
    .with_schema_url("https://opentelemetry.io/schema/1.0.0")
    .build();

let logger = provider.logger_with_scope(scope);
```
