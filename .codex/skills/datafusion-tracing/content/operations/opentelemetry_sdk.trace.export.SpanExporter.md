# `opentelemetry_sdk::trace::export::SpanExporter`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.export.SpanExporter.json).

<a id="op-a27301be3bc2ee4d49a48c4a"></a>
## SpanExporter

`trait` · `opentelemetry_sdk::trace::export::SpanExporter` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait SpanExporter: Send + Sync + Debug
```

Source: `src/trace/export.rs:17`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

`SpanExporter` defines the interface that protocol-specific exporters must
implement so that they can be plugged into OpenTelemetry SDK and support
sending of telemetry data.

The goal of the interface is to minimize burden of implementation for
protocol-dependent telemetry exporters. The protocol exporter is expected to
be primarily a simple telemetry data encoder and transmitter.

<a id="op-ded55e69f0dae38e503e897d"></a>
## export

`function` · `opentelemetry_sdk::trace::export::SpanExporter::export` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn export(&self, batch: Vec<SpanData>) -> impl std::future::Future<Output = OTelSdkResult> + Send
```

Source: `src/trace/export.rs:30`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Exports a batch of readable spans. Protocol exporters that will
implement this function are typically expected to serialize and transmit
the data to the destination.

This function will never be called concurrently for the same exporter
instance. It can be called again only after the current call returns.

This function must not block indefinitely, there must be a reasonable
upper limit after which the call must time out with an error result.

Any retry logic that is required by the exporter is the responsibility
of the exporter.

<a id="op-8fbc0c41b7970bce95cfcb17"></a>
## force_flush

`function` · `opentelemetry_sdk::trace::export::SpanExporter::force_flush` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn force_flush(&mut self) -> OTelSdkResult
```

Source: `src/trace/export.rs:69`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

This is a hint to ensure that the export of any Spans the exporter
has received prior to the call to this function SHOULD be completed
as soon as possible, preferably before returning from this method.

This function SHOULD provide a way to let the caller know
whether it succeeded, failed or timed out.

This function SHOULD only be called in cases where it is absolutely necessary,
such as when using some FaaS providers that may suspend the process after
an invocation, but before the exporter exports the completed spans.

This function SHOULD complete or abort within some timeout. This function can be
implemented as a blocking API or an asynchronous API which notifies the caller via
a callback or an event. OpenTelemetry client authors can decide if they want to
make the flush timeout configurable.

<a id="op-5653f305530d42435a65a645"></a>
## set_resource

`function` · `opentelemetry_sdk::trace::export::SpanExporter::set_resource` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn set_resource(&mut self, _resource: &Resource)
```

Source: `src/trace/export.rs:74`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Set the resource for the exporter.

<a id="op-ca669c5ef953c3eff06a87e5"></a>
## shutdown

`function` · `opentelemetry_sdk::trace::export::SpanExporter::shutdown` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown(&mut self) -> OTelSdkResult
```

Source: `src/trace/export.rs:50`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the exporter with default timeout.

<a id="op-1bb553100e2b3dc3ef8eea89"></a>
## shutdown_with_timeout

`function` · `opentelemetry_sdk::trace::export::SpanExporter::shutdown_with_timeout` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn shutdown_with_timeout(&mut self, _timeout: Duration) -> OTelSdkResult
```

Source: `src/trace/export.rs:46`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Shuts down the exporter. Called when SDK is shut down. This is an
opportunity for exporter to do any cleanup required.

This function should be called only once for each `SpanExporter`
instance. After the call to `shutdown`, subsequent calls to `export` are
not allowed and should return an error.

This function should not block indefinitely (e.g. if it attempts to
flush the data and the destination is unavailable). SDK authors
can decide if they want to make the shutdown timeout
configurable.
