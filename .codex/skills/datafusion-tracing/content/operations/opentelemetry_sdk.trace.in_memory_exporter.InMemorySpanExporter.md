# `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.in_memory_exporter.InMemorySpanExporter.json).

<a id="op-d2e217a7d8c3b8dfc5bb623b"></a>
## InMemorySpanExporter

`struct` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
struct InMemorySpanExporter
```

Source: `src/trace/in_memory_exporter.rs:51`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

 An in-memory span exporter that stores span data in memory.

 This exporter is useful for testing and debugging purposes. It stores
 span data in a `Vec<SpanData>`. Spans can be retrieved
 using the `get_finished_spans` method.
 # Example
 ```
# use opentelemetry::trace::{SpanKind, TraceContextExt};
# use opentelemetry::{global, trace::Tracer, Context};
# use opentelemetry_sdk::propagation::TraceContextPropagator;
# use opentelemetry_sdk::runtime;
# use opentelemetry_sdk::trace::InMemorySpanExporterBuilder;
# use opentelemetry_sdk::trace::{BatchSpanProcessor, SdkTracerProvider};

# #[tokio::main]
# async fn main() {
     let exporter = InMemorySpanExporterBuilder::new().build();
     let provider = SdkTracerProvider::builder()
         .with_span_processor(BatchSpanProcessor::builder(exporter.clone()).build())
         .build();

     global::set_tracer_provider(provider.clone());

     let tracer = global::tracer("example/in_memory_exporter");
     let span = tracer
         .span_builder("say hello")
         .with_kind(SpanKind::Server)
         .start(&tracer);

     let cx = Context::current_with_span(span);
     cx.span().add_event("handling this...", Vec::new());
     cx.span().end();

     if let Err(e) = provider.force_flush() {
         println!("{:?}", e)
     }
     let spans = exporter.get_finished_spans().unwrap();
     for span in spans {
         println!("{:?}", span)
     }
# }
 ```

<a id="op-fad48ac7bdc3a305a3948797"></a>
## clone

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::clone` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn clone(&self) -> InMemorySpanExporter
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [50, 10], "end": [50, 15], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::clone::Clone", "path": "Clone"}, "trait_path": "core::clone::Clone"}`

Source: `src/trace/in_memory_exporter.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-8335dab33245fb803821c1bd"></a>
## default

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::default` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn default() -> Self
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [58, 1], "end": [62, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::default::Default", "path": "Default"}, "trait_path": "core::default::Default"}`

Source: `src/trace/in_memory_exporter.rs:59`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-e41e87eafb8d7d252cbdb208"></a>
## export

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::export` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [155, 1], "end": [180, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}, "trait_path": "opentelemetry_sdk::trace::export::SpanExporter"}`

Source: `src/trace/in_memory_exporter.rs:156`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-2eb542d99f03f36545c2cc21"></a>
## fmt

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::fmt` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [50, 17], "end": [50, 22], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "core::fmt::Debug", "path": "Debug"}, "trait_path": "core::fmt::Debug"}`

Source: `src/trace/in_memory_exporter.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-c953f978f36d1816e9229284"></a>
## get_finished_spans

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::get_finished_spans` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn get_finished_spans(&self) -> Result<Vec<SpanData>, InMemoryExporterError>
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 1], "end": [153, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/in_memory_exporter.rs:131`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the finished span as a vector of `SpanData`.

# Errors

Returns a `TraceError` if the internal lock cannot be acquired.

# Example

```
# use opentelemetry_sdk::trace::InMemorySpanExporter;

let exporter = InMemorySpanExporter::default();
let finished_spans = exporter.get_finished_spans().unwrap();
```

<a id="op-1a85cd4d65ae959e482990f6"></a>
## is_shutdown_called

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::is_shutdown_called` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn is_shutdown_called(&self) -> bool
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 1], "end": [153, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/in_memory_exporter.rs:112`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns true if shutdown was called.

<a id="op-8fda2125db4cf73bd57765fb"></a>
## reset

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::reset` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn reset(&self)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [110, 1], "end": [153, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": null, "trait_path": null}`

Source: `src/trace/in_memory_exporter.rs:150`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Clears the internal storage of finished spans.

# Example

```
# use opentelemetry_sdk::trace::InMemorySpanExporter;

let exporter = InMemorySpanExporter::default();
exporter.reset();
```

<a id="op-64649cd4c557369b69d1cd23"></a>
## set_resource

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, resource: &Resource)
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [155, 1], "end": [180, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}, "trait_path": "opentelemetry_sdk::trace::export::SpanExporter"}`

Source: `src/trace/in_memory_exporter.rs:174`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.

<a id="op-5a543addc93e7463d0f2fb62"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&mut self, _timeout: Duration) -> OTelSdkResult
```

Implementation context: `{"for": {"resolved_path": {"args": null, "id": "opentelemetry_sdk::trace::in_memory_exporter::InMemorySpanExporter", "path": "InMemorySpanExporter"}}, "generics": {"params": [], "where_predicates": []}, "is_negative": false, "span": {"begin": [155, 1], "end": [180, 2], "filename": "src/trace/in_memory_exporter.rs"}, "trait": {"args": null, "id": "opentelemetry_sdk::trace::export::SpanExporter", "path": "SpanExporter"}, "trait_path": "opentelemetry_sdk::trace::export::SpanExporter"}`

Source: `src/trace/in_memory_exporter.rs:165`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

No upstream documentation on this item; consult its owner/trait contract.
