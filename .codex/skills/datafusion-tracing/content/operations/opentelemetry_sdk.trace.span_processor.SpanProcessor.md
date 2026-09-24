# `opentelemetry_sdk::trace::span_processor::SpanProcessor`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.span_processor.SpanProcessor.json).

<a id="op-d354c01a6b0b4f29c191ace7"></a>
## SpanProcessor

`trait` · `opentelemetry_sdk::trace::span_processor::SpanProcessor` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait SpanProcessor: Send + Sync + std::fmt::Debug
```

Source: `src/trace/span_processor.rs:77`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`SpanProcessor` is an interface which allows hooks for span start and end
method invocations. The span processors are invoked only when is_recording
is true.

<a id="op-20c8fd82fdef68c4bc4da36d"></a>
## force_flush

`function` · `opentelemetry_sdk::trace::span_processor::SpanProcessor::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&self) -> OTelSdkResult
```

Source: `src/trace/span_processor.rs:88`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Force the spans lying in the cache to be exported.

<a id="op-4574a6d9421820a53c4d0f16"></a>
## on_end

`function` · `opentelemetry_sdk::trace::span_processor::SpanProcessor::on_end` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_end(&self, span: SpanData)
```

Source: `src/trace/span_processor.rs:86`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`on_end` is called after a `Span` is ended (i.e., the end timestamp is
already set). This method is called synchronously within the `Span::end`
API, therefore it should not block or throw an exception.
TODO - This method should take reference to `SpanData`

<a id="op-304207c3cb4fc365f20aa9ad"></a>
## on_start

`function` · `opentelemetry_sdk::trace::span_processor::SpanProcessor::on_start` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn on_start(&self, span: &mut Span, cx: &Context)
```

Source: `src/trace/span_processor.rs:81`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`on_start` is called when a `Span` is started.  This method is called
synchronously on the thread that started the span, therefore it should
not block or throw exceptions.

<a id="op-c2347cbcf5d1678f88abc52e"></a>
## set_resource

`function` · `opentelemetry_sdk::trace::span_processor::SpanProcessor::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, _resource: &Resource)
```

Source: `src/trace/span_processor.rs:99`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the resource for the span processor.

<a id="op-9d9a704438507456507a3a28"></a>
## shutdown

`function` · `opentelemetry_sdk::trace::span_processor::SpanProcessor::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&self) -> OTelSdkResult
```

Source: `src/trace/span_processor.rs:95`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

shutdown the processor with a default timeout.

<a id="op-231b288de4942f09c27d1e65"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::trace::span_processor::SpanProcessor::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&self, timeout: Duration) -> OTelSdkResult
```

Source: `src/trace/span_processor.rs:93`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the processor. Called when SDK is shut down. This is an
opportunity for processors to do any cleanup required.

Implementation should make sure shutdown can be called multiple times.
