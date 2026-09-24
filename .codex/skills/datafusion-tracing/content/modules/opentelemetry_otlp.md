# `opentelemetry_otlp`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_otlp.json).

<a id="op-04856461615192ea1c63e20e"></a>
## opentelemetry_otlp

`module` · `opentelemetry_otlp` · opentelemetry-otlp 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
mod opentelemetry_otlp
```

Source: `src/lib.rs:1`. [Exact documentation build](https://docs.rs/crate/opentelemetry-otlp/0.31.0/json).

# OpenTelemetry OTLP Exporter

The OTLP Exporter enables exporting telemetry data (logs, metrics, and traces) in the
OpenTelemetry Protocol (OTLP) format to compatible backends. These backends include:

- OpenTelemetry Collector
- Open-source observability tools (Prometheus, Jaeger, etc.)
- Vendor-specific monitoring platforms

This crate supports sending OTLP data via:
- gRPC
- HTTP (binary protobuf or JSON)

## Quickstart with OpenTelemetry Collector

### HTTP Transport (Port 4318)

Run the OpenTelemetry Collector:

```shell
$ docker run -p 4318:4318 otel/opentelemetry-collector:latest
```

Configure your application to export traces via HTTP:

```no_run
# #[cfg(all(feature = "trace", feature = "http-proto"))]
# {
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Initialize OTLP exporter using HTTP binary protocol
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()?;

    // Create a tracer provider with the exporter
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .build();

    // Set it as the global provider
    global::set_tracer_provider(tracer_provider);

    // Get a tracer and create spans
    let tracer = global::tracer("my_tracer");
    tracer.in_span("doing_work", |_cx| {
        // Your application logic here...
    });

    Ok(())
# }
}
```

### gRPC Transport (Port 4317)

Run the OpenTelemetry Collector:

```shell
$ docker run -p 4317:4317 otel/opentelemetry-collector:latest
```

Configure your application to export traces via gRPC (the tonic client requires a Tokio runtime):

- With `[tokio::main]`

```no_run
# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
# {
use opentelemetry::{global, trace::Tracer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Initialize OTLP exporter using gRPC (Tonic)
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()?;

    // Create a tracer provider with the exporter
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(otlp_exporter)
        .build();

    // Set it as the global provider
    global::set_tracer_provider(tracer_provider);

    // Get a tracer and create spans
    let tracer = global::tracer("my_tracer");
    tracer.in_span("doing_work", |_cx| {
        // Your application logic here...
    });

    Ok(())
# }
}
```

- Without `[tokio::main]`

 ```no_run
# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
# {
use opentelemetry::{global, trace::Tracer};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Initialize OTLP exporter using gRPC (Tonic)
    let rt = tokio::runtime::Runtime::new()?;
    let tracer_provider = rt.block_on(async {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()
            .expect("Failed to create span exporter");
        opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .build()
    });

    // Set it as the global provider
    global::set_tracer_provider(tracer_provider);

    // Get a tracer and create spans
    let tracer = global::tracer("my_tracer");
    tracer.in_span("doing_work", |_cx| {
        // Your application logic here...
    });

    // Ensure the runtime (`rt`) remains active until the program ends
    Ok(())
# }
}
```

## Using with Jaeger

Jaeger natively supports the OTLP protocol, making it easy to send traces directly:

```shell
$ docker run -p 16686:16686 -p 4317:4317 -e COLLECTOR_OTLP_ENABLED=true jaegertracing/all-in-one:latest
```

After running your application configured with the OTLP exporter, view traces at:
`http://localhost:16686`

## Using with Prometheus

Prometheus natively supports accepting metrics via the OTLP protocol
(HTTP/protobuf). You can [run
Prometheus](https://prometheus.io/docs/prometheus/latest/installation/) with
the following command:

```shell
docker run -p 9090:9090 -v ./prometheus.yml:/etc/prometheus/prometheus.yml prom/prometheus --config.file=/etc/prometheus/prometheus.yml --web.enable-otlp-receiver
```

(An empty prometheus.yml file is sufficient for this example.)

Modify your application to export metrics via OTLP:

```no_run
# #[cfg(all(feature = "metrics", feature = "http-proto"))]
# {
use opentelemetry::global;
use opentelemetry::metrics::Meter;
use opentelemetry::KeyValue;
use opentelemetry_otlp::Protocol;
use opentelemetry_otlp::WithExportConfig;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    // Initialize OTLP exporter using HTTP binary protocol
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint("http://localhost:9090/api/v1/otlp/v1/metrics")
        .build()?;

    // Create a meter provider with the OTLP Metric exporter
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .build();
    global::set_meter_provider(meter_provider.clone());

    // Get a meter
    let meter = global::meter("my_meter");

    // Create a metric
    let counter = meter.u64_counter("my_counter").build();
    counter.add(1, &[KeyValue::new("key", "value")]);

    // Shutdown the meter provider. This will trigger an export of all metrics.
    meter_provider.shutdown()?;

    Ok(())
# }
}
```

After running your application configured with the OTLP exporter, view metrics at:
`http://localhost:9090`
## Show Logs, Metrics too (TODO)

[`tokio`]: https://tokio.rs

# Feature Flags
The following feature flags can enable exporters for different telemetry signals:

* `trace`: Includes the trace exporters.
* `metrics`: Includes the metrics exporters.
* `logs`: Includes the logs exporters.

The following feature flags generate additional code and types:
* `serialize`: Enables serialization support for type defined in this crate via `serde`.

The following feature flags offer additional configurations on gRPC:

For users using `tonic` as grpc layer:
* `grpc-tonic`: Use `tonic` as grpc layer.
* `gzip-tonic`: Use gzip compression for `tonic` grpc layer.
* `zstd-tonic`: Use zstd compression for `tonic` grpc layer.
* `tls-roots`: Adds system trust roots to rustls-based gRPC clients using the rustls-native-certs crate
* `tls-webpki-roots`: Embeds Mozilla's trust roots to rustls-based gRPC clients using the webpki-roots crate

The following feature flags offer additional configurations on http:

* `http-proto`: Use http as transport layer, protobuf as body format. This feature is enabled by default.
* `gzip-http`: Use gzip compression for HTTP transport.
* `zstd-http`: Use zstd compression for HTTP transport.
* `reqwest-blocking-client`: Use reqwest blocking http client. This feature is enabled by default.
* `reqwest-client`: Use reqwest http client.
* `reqwest-rustls`: Use reqwest with TLS with system trust roots via `rustls-native-certs` crate.
* `reqwest-rustls-webpki-roots`: Use reqwest with TLS with Mozilla's trust roots via `webpki-roots` crate.

# Kitchen Sink Full Configuration

Example showing how to override all configuration options.

Generally there are two parts of configuration. One is the exporter, the other is the provider.
Users can configure the exporter using [SpanExporter::builder()](../operations/opentelemetry_otlp.span.SpanExporter.md#op-72dfdf8f60dc2fdebaf4c76a) for traces,
and [MetricExporter::builder()](../operations/opentelemetry_otlp.metric.MetricExporter.md#op-64286b6029ff8c3301a61ea4) + [opentelemetry_sdk::metrics::PeriodicReader::builder()] for metrics.
Once you have an exporter, you can add it to either a [opentelemetry_sdk::trace::SdkTracerProvider::builder()] for traces,
or [opentelemetry_sdk::metrics::SdkMeterProvider::builder()] for metrics.

```no_run
use opentelemetry::{global, KeyValue, trace::Tracer};
use opentelemetry_sdk::{trace::{self, RandomIdGenerator, Sampler}, Resource};
# #[cfg(feature = "metrics")]
use opentelemetry_sdk::metrics::Temporality;
use opentelemetry_otlp::{Protocol, WithExportConfig, Compression};
# #[cfg(feature = "grpc-tonic")]
use opentelemetry_otlp::WithTonicConfig;
# #[cfg(any(feature = "http-proto", feature = "http-json"))]
use opentelemetry_otlp::WithHttpConfig;
use std::time::Duration;
# #[cfg(feature = "grpc-tonic")]
use tonic::metadata::*;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    # #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
    # let tracer = {
    let mut map = MetadataMap::with_capacity(3);

    map.insert("x-host", "example.com".parse().unwrap());
    map.insert("x-number", "123".parse().unwrap());
    map.insert_bin("trace-proto-bin", MetadataValue::from_bytes(b"[binary data]"));
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .with_timeout(Duration::from_secs(3))
        .with_metadata(map)
        .build()?;

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_max_events_per_span(64)
        .with_max_attributes_per_span(16)
        .with_resource(Resource::builder_empty().with_attributes([KeyValue::new("service.name", "example")]).build())
        .build();
    global::set_tracer_provider(tracer_provider.clone());
    let tracer = global::tracer("tracer-name");
        # tracer
    # };

    // HTTP exporter example with compression
    # #[cfg(all(feature = "trace", feature = "http-proto"))]
    # let _http_tracer = {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint("http://localhost:4318/v1/traces")
        .with_timeout(Duration::from_secs(3))
        .with_protocol(Protocol::HttpBinary)
        .with_compression(Compression::Gzip)  // Requires gzip-http feature
        .build()?;
        # exporter
    # };

    # #[cfg(all(feature = "metrics", feature = "grpc-tonic"))]
    # {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
       .with_tonic()
       .with_endpoint("http://localhost:4318/v1/metrics")
       .with_protocol(Protocol::Grpc)
       .with_timeout(Duration::from_secs(3))
       .build()
       .unwrap();

   let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(Resource::builder_empty().with_attributes([KeyValue::new("service.name", "example")]).build())
        .build();
    # }

    // HTTP metrics exporter example with compression
    # #[cfg(all(feature = "metrics", feature = "http-proto"))]
    # {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
       .with_http()
       .with_endpoint("http://localhost:4318/v1/metrics")
       .with_protocol(Protocol::HttpBinary)
       .with_timeout(Duration::from_secs(3))
       .with_compression(Compression::Zstd)  // Requires zstd-http feature
       .build()
       .unwrap();
    # }

# #[cfg(all(feature = "trace", feature = "grpc-tonic"))]
# {
    tracer.in_span("doing_work", |cx| {
        // Traced app logic here...
    });
# }

    Ok(())
}
```

Unresolved upstream links (retained, not inferred): `opentelemetry_sdk::metrics::SdkMeterProvider::builder()`, `opentelemetry_sdk::trace::SdkTracerProvider::builder()`, `opentelemetry_sdk::metrics::PeriodicReader::builder()`.
