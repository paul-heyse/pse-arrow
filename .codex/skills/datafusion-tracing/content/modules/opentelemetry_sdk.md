# `opentelemetry_sdk`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.json).

<a id="op-c31431ad2735294fd6efacbd"></a>
## opentelemetry_sdk

`module` · `opentelemetry_sdk` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod opentelemetry_sdk
```

Source: `src/lib.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Implements the [`SDK`] component of [OpenTelemetry].

*[Supported Rust Versions](#supported-rust-versions)*

[`SDK`]: https://opentelemetry.io/docs/specs/otel/overview/#sdk
[OpenTelemetry]: https://opentelemetry.io/docs/what-is-opentelemetry/
[msrv]: #supported-rust-versions

# Getting Started

```no_run
# #[cfg(feature = "trace")]
# {
use opentelemetry::{global, trace::{Tracer, TracerProvider}};
use opentelemetry_sdk::trace::SdkTracerProvider;

fn main() {
    // Choose an exporter like `opentelemetry_stdout::SpanExporter`
    # fn example<T: opentelemetry_sdk::trace::SpanExporter + 'static>(new_exporter: impl Fn() -> T) {
    let exporter = new_exporter();

    // Create a new trace pipeline that prints to stdout
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("readme_example");

    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });

    // Shutdown trace pipeline
    provider.shutdown().expect("TracerProvider should shutdown successfully")
    # }
}
# }
```

See the [examples] directory for different integration patterns.

See the API [`trace`] module docs for more information on creating and managing
spans.

[examples]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples
[`trace`]: https://docs.rs/opentelemetry/latest/opentelemetry/trace/index.html

# Metrics

### Creating instruments and recording measurements

```
# #[cfg(feature = "metrics")]
# {
use opentelemetry::{global, KeyValue};

// get a meter from a provider
let meter = global::meter("my_service");

// create an instrument
let counter = meter.u64_counter("my_counter").build();

// record a measurement
counter.add(1, &[KeyValue::new("http.client_ip", "83.164.160.102")]);
# }
```

See the [examples] directory for different integration patterns.

See the API [`metrics`] module docs for more information on creating and
managing instruments.

[examples]: https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples
[`metrics`]: https://docs.rs/opentelemetry/latest/opentelemetry/metrics/index.html

## Crate Feature Flags

The following feature flags can used to control the telemetry signals to use:

* `trace`: Includes the trace SDK (enabled by default).
* `metrics`: Includes the metrics SDK.
* `logs`: Includes the logs SDK.

For `trace` the following feature flags are available:

* `jaeger_remote_sampler`: Enables the [Jaeger remote sampler](https://www.jaegertracing.io/docs/1.53/sampling/).

For `logs` the following feature flags are available:

* `spec_unstable_logs_enabled`: control the log level

Support for recording and exporting telemetry asynchronously and perform
metrics aggregation can be added via the following flags:

* `experimental_async_runtime`: Enables the experimental `Runtime` trait and related functionality.
* `rt-tokio`: Spawn telemetry tasks using [tokio]'s multi-thread runtime.
* `rt-tokio-current-thread`: Spawn telemetry tasks on a separate runtime so that the main runtime won't be blocked.

[tokio]: https://crates.io/crates/tokio
