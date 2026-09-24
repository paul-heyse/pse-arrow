# `opentelemetry::trace::tracer_provider::TracerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry.trace.tracer_provider.TracerProvider.json).

<a id="op-3d14c743f48c20c76241bcf3"></a>
## TracerProvider

`trait` · `opentelemetry::trace::tracer_provider::TracerProvider` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait TracerProvider
```

Source: `src/trace/tracer_provider.rs:10`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Types that can create instances of [`Tracer`](../operations/opentelemetry.trace.tracer.Tracer.md#op-a87fa61076cf2b7384423932).

See the [`global`] module for examples of storing and retrieving tracer
provider instances.

[`global`]: crate::global

<a id="op-9d9e21edf116179b04b5df6a"></a>
## Tracer

`assoc_type` · `opentelemetry::trace::tracer_provider::TracerProvider::Tracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Tracer
```

Source: `src/trace/tracer_provider.rs:12`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

The [`Tracer`](../operations/opentelemetry.trace.tracer.Tracer.md#op-a87fa61076cf2b7384423932) type that this provider will return.

<a id="op-e1c5a8597d9d7dc1431ee722"></a>
## tracer

`function` · `opentelemetry::trace::tracer_provider::TracerProvider::tracer` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer(&self, name: impl Into<Cow<'static, str>>) -> Self::Tracer
```

Source: `src/trace/tracer_provider.rs:31`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new tracer with the given name.

The `name` should be the application name or the name of the library
providing instrumentation. If the name is empty, then an
implementation-defined default name may be used instead.

# Examples

```
use opentelemetry::{global, trace::TracerProvider};
use opentelemetry::KeyValue;

let provider = global::tracer_provider();

// tracer used in applications/binaries
let tracer = provider.tracer("my_app");
```

<a id="op-08a1438e5f6544db1c3265a3"></a>
## tracer_with_scope

`function` · `opentelemetry::trace::tracer_provider::TracerProvider::tracer_with_scope` · opentelemetry 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer_with_scope(&self, scope: InstrumentationScope) -> Self::Tracer
```

Source: `src/trace/tracer_provider.rs:57`. [Exact documentation build](https://docs.rs/crate/opentelemetry/0.31.0/json).

Returns a new versioned tracer with the given instrumentation scope.

# Examples

```
use opentelemetry::{global, InstrumentationScope, trace::TracerProvider};

let provider = global::tracer_provider();

// tracer used in applications/binaries
let tracer = provider.tracer("my_app");

// tracer used in libraries/crates that optionally includes version and schema url
let scope =
    InstrumentationScope::builder(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schema/1.0.0")
        .build();

let tracer = provider.tracer_with_scope(scope);
```
