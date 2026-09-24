# `opentelemetry_sdk::trace::provider::SdkTracerProvider`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.provider.SdkTracerProvider.json).

<a id="op-68f8d3c6d535fa6ce330056b"></a>
## SdkTracerProvider

`struct` · `opentelemetry_sdk::trace::provider::SdkTracerProvider` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct SdkTracerProvider
```

Source: `src/trace/provider.rs:158`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Creator and registry of named [`SdkTracer`](../operations/opentelemetry_sdk.trace.tracer.SdkTracer.md#op-994f3cb8ad5bc1ae17502b0e) instances.

`TracerProvider` is a container holding pointers to `SpanProcessor` and other components.
Cloning a `TracerProvider` instance and dropping it will not stop span processing. To stop span processing, users
must either call the `shutdown` method explicitly or allow the last reference to the `TracerProvider`
to be dropped. When the last reference is dropped, the shutdown process will be automatically triggered
to ensure proper cleanup.

<a id="op-f36a9b93068ded2eb9207a07"></a>
## Tracer

`assoc_type` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::Tracer` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
Tracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [278, 1], "end": [296, 2], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer_provider::TracerProvider", "path": "TracerProvider"}, "trait_path": "opentelemetry::trace::tracer_provider::TracerProvider"}`

Source: `src/trace/provider.rs:280`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

This implementation of `TracerProvider` produces `Tracer` instances.

<a id="op-f46b48181a29b37afad48766"></a>
## builder

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::builder` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn builder() -> TracerProviderBuilder
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [168, 1], "end": [276, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:177`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Create a new [`SdkTracerProvider`](../operations/opentelemetry_sdk.trace.provider.SdkTracerProvider.md#op-68f8d3c6d535fa6ce330056b) builder.

<a id="op-01bca4548c1d2d89390576b3"></a>
## clone

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> SdkTracerProvider
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 10], "end": [157, 15], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/provider.rs:157`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-d79b4692a939afb8253d112f"></a>
## default

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [162, 1], "end": [166, 2], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/provider.rs:163`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-4e580fb10a17ff56a5193f95"></a>
## fmt

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [157, 17], "end": [157, 22], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/provider.rs:157`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-0975520fe88eba7c73531181"></a>
## force_flush

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [168, 1], "end": [276, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:230`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Force flush all remaining spans in span processors and return results.

# Examples

```
use opentelemetry::global;
use opentelemetry_sdk::trace::SdkTracerProvider;

fn init_tracing() -> SdkTracerProvider {
    let provider = SdkTracerProvider::default();

    // Set provider to be used as global tracer provider
    let _ = global::set_tracer_provider(provider.clone());

    provider
}

fn main() {
    let provider = init_tracing();

    // create spans..

    // force all spans to flush
    if let Err(err) = provider.force_flush() {
        // .. handle flush error
    }

    // create more spans..

    // dropping provider ensures all remaining spans are exported
    drop(provider);
}
```

<a id="op-4ace81416f640af98ed8f81b"></a>
## shutdown

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [168, 1], "end": [276, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:273`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

shutdown with default timeout

<a id="op-bf9af7d324593b1bb74d52ee"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [168, 1], "end": [276, 2], "filename": "src/trace/provider.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/provider.rs:246`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the current `TracerProvider`.

Note that shut down doesn't means the TracerProvider has dropped

<a id="op-6535d432f050474d176541dd"></a>
## tracer

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::tracer` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer(&self, name: impl Into<Cow<'static, str>>) -> Self::Tracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [278, 1], "end": [296, 2], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer_provider::TracerProvider", "path": "TracerProvider"}, "trait_path": "opentelemetry::trace::tracer_provider::TracerProvider"}`

Source: `src/trace/provider.rs:282`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8d31ebfe22d5127ae9da9ef0"></a>
## tracer_with_scope

`function` · `opentelemetry_sdk::trace::provider::SdkTracerProvider::tracer_with_scope` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn tracer_with_scope(&self, scope: InstrumentationScope) -> Self::Tracer
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::provider::SdkTracerProvider", "path": "SdkTracerProvider"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [278, 1], "end": [296, 2], "filename": "src/trace/provider.rs"}, "trait": {"args": null, "id": "opentelemetry::trace::tracer_provider::TracerProvider", "path": "TracerProvider"}, "trait_path": "opentelemetry::trace::tracer_provider::TracerProvider"}`

Source: `src/trace/provider.rs:287`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
