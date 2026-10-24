# OpenTelemetry for Rust — Comprehensive Technical Reference

Research date: October 24, 2026 | Core release: 0.33.1 | Rust ecosystem

## 1. Overview and architectural model

OpenTelemetry (OTel) for Rust is the official implementation of the OpenTelemetry observability standard, providing APIs, SDKs, exporters, and integrations for generating and transporting distributed traces, metrics, and structured logs.

Its principal value is the ability to express application behavior through standardized, correlated telemetry while separating instrumentation from processing, aggregation, transport, and observability backends.

OpenTelemetry is not itself a database, profiler, trace analyzer, or visualization system. It is an instrumentation and telemetry-processing framework that interfaces with systems such as Prometheus, Jaeger, Grafana, and OpenTelemetry Collector.

Verified version scope:

| Component                           | Version / status              |
| ----------------------------------- | ----------------------------- |
| `opentelemetry`                     | 0.33.1 — October 9, 2026      |
| `opentelemetry_sdk`                 | 0.33.1                        |
| `opentelemetry-otlp`                | 0.33.1                        |
| `tracing-opentelemetry`             | 0.34.0 — separately versioned |
| Metrics API and SDK                 | Stable                        |
| Logs bridge API and SDK             | Stable                        |
| OTLP metrics and logs exporters     | Stable as of 0.33.1           |
| Tracing API, SDK, and OTLP exporter | Beta                          |

The project applies component-specific stability classifications rather than treating the entire crate family as uniformly stable. The logs and metrics OTLP exporters were promoted from release candidate to stable in 0.33.1.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+5



Primary references:

- [https://github.com/open-telemetry/opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry/0.33.1/opentelemetry/](https://docs.rs/opentelemetry/0.33.1/opentelemetry/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/0.33.1/opentelemetry_sdk/](https://docs.rs/opentelemetry_sdk/0.33.1/opentelemetry_sdk/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/](https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/?utm_source=chatgpt.com)

### 1.1 Conceptual architecture

OpenTelemetry has four principal layers:

Application and library instrumentation

Rust `tracing`, OTel Tracer/Meter APIs, log bridges, framework instrumentation

OpenTelemetry API

Spans · Instruments · Context · Baggage · Propagation · Instrumentation scopes

OpenTelemetry SDK

Providers · Resources · Sampling · Processing · Views · Aggregation · Readers · Batching

Exporters and OTLP

gRPC · HTTP/protobuf · HTTP/JSON · Prometheus export

Collector and observability backends

Routing · Enrichment · Filtering · Sampling · Storage · Querying · Visualization

These layers are independently configurable and have different responsibilities.

- API: Defines instrumentation behavior and common semantic types. Instrumented libraries can depend on it without selecting a telemetry destination.
- SDK: Implements concrete instrumentation, buffering, sampling, aggregation, resource identification, and lifecycle management.
- Exporter: Converts SDK telemetry into an external representation and manages outbound communication.
- Collector/backend: Provides additional processing, infrastructure-level telemetry management, and persistent analysis.

The Collector is optional: compatible backends can accept OTLP directly. Conversely, the Collector can centralize capabilities that would otherwise need to be configured separately in every application.

### 1.2 Capability map

| Capability family           | Main mechanisms                                                      | Architectural significance                                                   |
| --------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Distributed tracing         | Spans, events, links, context, samplers                              | Reconstruct execution and causal relationships across components             |
| Metrics                     | Counters, gauges, histograms, observable instruments                 | Maintain efficient statistical representations of operational behavior       |
| Structured logs             | Logs bridge, processors, correlation fields                          | Connect diagnostic events with trace and service identity                    |
| Context propagation         | W3C Trace Context, baggage, propagators                              | Preserve causal continuity across processes and protocols                    |
| Instrumentation integration | `tracing`, `tracing-subscriber`, bridges                             | Reuse established Rust instrumentation rather than creating parallel systems |
| SDK processing              | Span/log processors, metric readers, views                           | Control collection, representation, overhead, and export behavior            |
| Transport                   | OTLP gRPC, HTTP/protobuf, HTTP/JSON                                  | Decouple telemetry producers from backend implementations                    |
| Semantic standardization    | Resources, instrumentation scopes, semantic conventions              | Make telemetry consistently identifiable and aggregatable                    |
| Extension and testing       | Custom samplers, exporters, processors, readers, in-memory exporters | Support specialized instrumentation and deterministic inspection             |

### 1.3 Rust-specific architectural distinction

Rust has an established instrumentation ecosystem centered around the `tracing` crate. That ecosystem overlaps substantially with OpenTelemetry tracing, but the two are not identical.

`tracing` describes structured operations and events inside a Rust program; OpenTelemetry describes standardized telemetry that can be correlated and transported across system boundaries.

The integration crates make it possible to retain Rust-native `tracing` instrumentation while using OpenTelemetry for distributed tracing and log export.

This division is especially valuable because an application can combine:

- Local formatted diagnostics.
- Distributed trace spans.
- Structured OpenTelemetry logs.
- Independent OpenTelemetry metrics.

All without introducing an unrelated application-specific logging or telemetry abstraction.

The distinction between these signals and their bridges is fundamental to understanding the Rust implementation.

## 2. Crate ecosystem and ownership boundaries

OpenTelemetry Rust is a coordinated family of crates, not a single monolithic library.

| Crate                                | Responsibility                                                                                               |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------ |
| `opentelemetry`                      | Public API, instrumentation primitives, context, propagation, metrics, tracing, logging bridge interfaces    |
| `opentelemetry_sdk`                  | Providers, samplers, span processors, metrics aggregation, views, log processors, readers, resource handling |
| `opentelemetry-otlp`                 | OTLP exporters for all three signals                                                                         |
| `opentelemetry-http`                 | HTTP transport abstractions, header propagation helpers, configurable HTTP clients                           |
| `opentelemetry-proto`                | Generated OTLP protocol types and SDK-to-protobuf transformations                                            |
| `opentelemetry-stdout`               | Human-readable exporters for diagnostics and development                                                     |
| `opentelemetry-prometheus`           | Pull-based metrics export through a Prometheus registry                                                      |
| `opentelemetry-semantic-conventions` | Generated standardized names for attributes, metrics, and resources                                          |
| `opentelemetry-appender-tracing`     | Converts Rust `tracing` events into OTel log records                                                         |
| `opentelemetry-appender-log`         | Bridges the Rust `log` facade to OTel logs                                                                   |
| `tracing-opentelemetry`              | Integrates Rust `tracing` spans and selected metric events with OTel                                         |
| `opentelemetry-rust-contrib`         | Companion instrumentation, resource detectors, platform-specific exporters, and other integrations           |

The separation between `opentelemetry` and `opentelemetry_sdk` is particularly consequential.

A reusable instrumentation library can depend on the API without selecting a sampler, runtime, exporter, or backend. The application can then configure one or more providers for its own execution environment.

This avoids coupling reusable libraries to transport configuration or vendor-specific observability infrastructure.

Version compatibility: The core crates generally evolve together, but companion libraries need not share their version numbers. For example, `tracing-opentelemetry` 0.34.0 declares `opentelemetry` 0.33 compatibility. Likewise, the `opentelemetry-appender-log` publication history differs from the actively updated `opentelemetry-appender-tracing` crate. Compatible dependencies should be verified per crate rather than inferred from their numeric versions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+4



Sources:

- [https://github.com/open-telemetry/opentelemetry-rust](https://github.com/open-telemetry/opentelemetry-rust?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust-contrib](https://github.com/open-telemetry/opentelemetry-rust-contrib?utm_source=chatgpt.com)
- [https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/](https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry-appender-tracing/0.33.1/opentelemetry_appender_tracing/](https://docs.rs/opentelemetry-appender-tracing/0.33.1/opentelemetry_appender_tracing/?utm_source=chatgpt.com)

## 3. Distributed tracing

Distributed tracing represents execution as operations with identifiable causal relationships and timing information.

OpenTelemetry provides a standardized trace data model with APIs for instrumenting both local operations and requests spanning multiple systems.

### 3.1 Spans, span contexts, and execution structure

Each span can include:

| Component            | Meaning                                                        |
| -------------------- | -------------------------------------------------------------- |
| Trace ID             | Identifies the overall distributed trace                       |
| Span ID              | Identifies an individual operation                             |
| Parent span ID       | Establishes direct causal ancestry                             |
| Name                 | Stable classification of an operation                          |
| Span kind            | `Internal`, `Server`, `Client`, `Producer`, or `Consumer`      |
| Start/end timestamps | Define an operation's elapsed time                             |
| Attributes           | Structured key-value metadata                                  |
| Events               | Timestamped occurrences during execution                       |
| Links                | Associations with other span contexts outside direct parentage |
| Status               | Operation outcome status                                       |
| Trace state          | Vendor/system-specific information propagated with the trace   |

The API exposes `TracerProvider`, `Tracer`, `Span`, `SpanBuilder`, `SpanContext`, `TraceState`, and `TraceContextExt`.

Advanced operations include `add_event_with_timestamp`, `add_link`, `record_error`, `set_status`, `update_name`, and explicit span-ending methods.

### 3.2 Parent-child relationships versus span links

These mechanisms represent different kinds of relationships.

Parent-child relationships express nested or causally dependent execution. They are appropriate when a new operation has a definite parent operation.

Span links represent relationships that are not conveniently modeled as a single parent hierarchy.

For example:

- A batch-processing operation consumes messages produced by several independent traces.
- A retry occurs as a distinct operation but remains related to an earlier attempt.
- A fan-in operation combines results from parallel workflows.
- A queue consumer needs to relate its work to a producer without treating the complete queue-processing lifecycle as a synchronous call.

Links can reference span contexts belonging to different traces. Unlike parentage, they do not imply hierarchical ownership or a contiguous synchronous call structure.

This is an important capability for asynchronous processing, message queues, fan-out/fan-in architectures, and distributed execution engines.

Architectural implication: Span links can express causal graphs that are richer than a strict call tree, without introducing a separate bespoke correlation mechanism. However, backend query and visualization support for linked spans varies.

### 3.3 Span events, logs, and status

OpenTelemetry exposes several superficially similar mechanisms with distinct purposes.

| Mechanism       | Principal purpose                                           |
| --------------- | ----------------------------------------------------------- |
| Span            | Represent a timed operation                                 |
| Span event      | Record a noteworthy occurrence within that operation        |
| OTel log record | Capture a standalone structured diagnostic event            |
| Span status     | Describe an operation's completion status                   |
| Span attribute  | Describe stable or updated characteristics of the operation |

For instance, a numerical operation could have a span for its overall execution, events for solver transitions or checkpoints, attributes for algorithm and problem dimensions, and a final status indicating an error.

A log can additionally preserve structured diagnostic data and remain independently queryable as part of the logs signal.

A span event generally travels as part of the span. A log record travels through the logs pipeline. That distinction affects sampling, retention, storage, and downstream indexing.

### 3.4 Head sampling and custom sampling

Sampling is one of the tracing SDK's most important performance and data-volume controls.

The Rust SDK supports:

- `Sampler::AlwaysOn`
- `Sampler::AlwaysOff`
- `Sampler::TraceIdRatioBased`
- Parent-based sampling
- `ShouldSample` for custom sampling implementations
- `JaegerRemoteSampler` behind the `jaeger_remote_sampler` feature

The sampling decision is made when the span begins, using the parent context, trace ID, span name, kind, initial attributes, and links.

The decisions distinguish:

| Decision          | Behavior                                           |
| ----------------- | -------------------------------------------------- |
| `Drop`            | Non-recording span; no detailed span data retained |
| `RecordOnly`      | Recording span without setting the sampled flag    |
| `RecordAndSample` | Recording span with the sampled flag set           |

An unsampled span may still carry a valid `SpanContext`, so sampling does not necessarily destroy downstream trace-context continuity.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



Important limitation: A head sampler cannot make its initial decision based on the eventual duration, completion status, or future events of a span. Consequently, sampling only completed error or slow traces normally requires downstream tail sampling or another mechanism that observes completed trace data.

The OpenTelemetry Collector has tail-sampling processors for this purpose. Such processing is a Collector capability, not an automatic behavior of the Rust SDK.

Jaeger remote sampling provides a further option: dynamically retrieving probabilistic, rate-limiting, or per-operation sampling policies from a compatible remote service. This can separate sampling-policy administration from application deployments.

Sources:

- [https://docs.rs/opentelemetry/latest/opentelemetry/trace/trait.Span.html](https://docs.rs/opentelemetry/latest/opentelemetry/trace/trait.Span.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/enum.Sampler.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/enum.Sampler.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/trait.ShouldSample.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/trait.ShouldSample.html?utm_source=chatgpt.com)
- [https://opentelemetry.io/docs/collector/components/processor/](https://opentelemetry.io/docs/collector/components/processor/?utm_source=chatgpt.com)

### 3.5 Span processors and export lifecycle

The SDK provides two principal processing strategies.

`SimpleSpanProcessor` exports spans as they finish. It is useful for immediate inspection, debugging, or controlled tests, but exporting in the caller's execution path introduces I/O and serialization costs.

`BatchSpanProcessor` queues completed spans and uses an SDK-owned background thread to export batches.

The standard batch processor is configurable through queue capacity, batch size, and export scheduling. The documented defaults are:

| Parameter                        | Default     |
| -------------------------------- | ----------- |
| `OTEL_BSP_MAX_QUEUE_SIZE`        | 2,048 spans |
| `OTEL_BSP_MAX_EXPORT_BATCH_SIZE` | 512 spans   |
| `OTEL_BSP_SCHEDULE_DELAY`        | 5,000 ms    |

Batching reduces the number of export operations and decouples the application's main execution path from most transport activity. It does not make telemetry delivery lossless.

A bounded queue can overflow, and abrupt process termination can lose buffered spans. `force_flush()` and `shutdown()` provide lifecycle controls but cannot eliminate failures or guarantee delivery across process crashes.

Advanced extension points: `SpanProcessor` enables custom start/end processing, and `SpanExporter` allows alternative destinations or representations without altering instrumentation code. Custom `IdGenerator` implementations can also control the generation of trace and span identifiers.

Sources:

- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/trait.SpanProcessor.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/trait.SpanProcessor.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/trait.SpanExporter.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/trait.SpanExporter.html?utm_source=chatgpt.com)

## 4. Context propagation and asynchronous Rust

Context propagation is distinct from recording spans. It determines whether operations recognize that they belong to the same execution context.

This distinction is especially important in Rust because asynchronous tasks are scheduled independently of their originating call stacks.

### 4.1 In-process context

`opentelemetry::Context` is an immutable collection of execution-scoped values.

Relevant APIs include:

- `Context::current()` — access the currently attached context.
- `Context::attach()` — attach a context for a lexical scope.
- `ContextGuard` — restore previous context on drop.
- `Context::current_with_span()` — construct a context containing a span.
- `TraceContextExt` — access and manipulate trace information.
- `FutureExt::with_context()` — associate context with a future.

The context may carry tracing state as well as other cross-cutting values.

This is a composable representation of contextual state, rather than a requirement to manually pass trace IDs through every application function.

### 4.2 Async context is not automatically task-local

The principal Rust-specific hazard is the relationship between thread-local state and asynchronous execution.

A guard created inside an `async` block can remain attached across `.await` suspension points. The future may later resume on a different thread, while other work could have run on the original thread.

This can produce incorrect parentage or context leakage.

OpenTelemetry documents attaching the context to the future itself:

```
use opentelemetry::{
    Context,
    trace::{FutureExt, TraceContextExt, Tracer},
};

let span = tracer.start("process_request");
let context = Context::current_with_span(span);

async_work()
    .with_context(context)
    .await;
```

This is a concise illustrative pattern using the documented API.

The future receives its own context association rather than relying on an attachment that unintentionally persists across suspension.

A separate spawned task also requires intentional context transfer when the chosen instrumentation integration does not handle it automatically.

For `tracing`-instrumented futures, `tracing`'s own `Instrument` mechanism and the OpenTelemetry integration layer provide another approach, with different context semantics.

`tracing` span propagation and OpenTelemetry `Context` propagation are related but not interchangeable. Crossing between them requires the appropriate bridge or explicit context operation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



### 4.3 Cross-process propagation

The SDK provides W3C propagators:

- `TraceContextPropagator`: Uses `traceparent` and `tracestate`.
- `BaggagePropagator`: Uses the W3C `baggage` header.
- `TextMapCompositePropagator`: Combines multiple propagators.

`TextMapPropagator`, `Injector`, and `Extractor` abstract the transport carrier.

An HTTP header map is a common carrier, but the mechanism can also work with message metadata or other string key-value transports.

Typical behavior is:

1. A client obtains its current context and injects it into outgoing metadata.
2. A receiving component extracts the remote context.
3. Its new operation uses the extracted context as the parent.
4. Subsequent outgoing operations continue the propagated trace.

This is what enables trace continuity across processes and services implemented in different languages.

The SDK also enforces relevant W3C limits. In the 0.33 series, baggage extraction enforces an 8,192-byte header limit and a 64-member limit, while `TraceState` enforces its 32-member bound.

### 4.4 Baggage is not telemetry attributes

Baggage represents application-defined key-value metadata propagated across component boundaries. It is not automatically equivalent to an ordinary span, metric, or log attribute.

That distinction matters for both privacy and performance:

- Baggage can travel to downstream services.
- Its contents may be exposed across trust boundaries.
- Adding baggage does not guarantee that the values become indexed telemetry attributes.
- Repeated propagation adds header and processing overhead.

Baggage is useful for controlled contextual information that must survive process boundaries. It should not be treated as an unrestricted mechanism for storing diagnostic payloads.

Sources:

- [https://docs.rs/opentelemetry/latest/opentelemetry/context/](https://docs.rs/opentelemetry/latest/opentelemetry/context/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry/latest/opentelemetry/trace/](https://docs.rs/opentelemetry/latest/opentelemetry/trace/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/propagation/](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/propagation/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/propagation/struct.BaggagePropagator.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/propagation/struct.BaggagePropagator.html?utm_source=chatgpt.com)

## 5. Metrics: instruments, aggregation, and stream control

The metrics SDK is substantially more than a collection of counters. It is an in-process aggregation engine that transforms measurements into bounded, structured metric streams.

Unlike tracing, where individual spans are generally retained until processing or sampling, metrics aggregate repeated observations before export.

### 5.1 Instruments and measurement semantics

OpenTelemetry defines instruments according to the meaning of the recorded quantity.

| Instrument                | Measurement semantics                                      | Typical use                           |
| ------------------------- | ---------------------------------------------------------- | ------------------------------------- |
| `Counter`                 | Nonnegative increments to a monotonically increasing total | Operations completed, bytes processed |
| `UpDownCounter`           | Signed changes to a quantity                               | Active tasks, allocated resources     |
| `Histogram`               | Observations of a distribution                             | Latency, size, iteration counts       |
| `Gauge`                   | Direct observation of a current value                      | Queue depth, temperature, utilization |
| `ObservableCounter`       | Cumulative monotonic observation made during collection    | Total bytes from an external source   |
| `ObservableUpDownCounter` | Observed cumulative value that may rise or fall            | Active sessions from a registry       |
| `ObservableGauge`         | Current value retrieved during collection                  | Memory utilization, free capacity     |

The API supports numeric variants such as `u64`, `i64`, and `f64`, subject to the instrument's permitted type.

Synchronous instruments record when application code executes. They are appropriate for operations whose occurrence or measured value is known at the point of execution.

Observable instruments invoke registered callbacks during metric collection. They are appropriate when a current or cumulative value is best obtained by inspecting another component's state.

These are not interchangeable:

- A synchronous counter records increments. An observable counter reports a cumulative value.
- A histogram accumulates a distribution of observations. A gauge records an instantaneous value.
- An up-down counter describes accumulated changes, whereas an observable up-down counter reports the resulting observed quantity.

Selecting the right semantics matters because the SDK and downstream systems interpret their aggregation and temporality differently.

Callback execution is part of collection. Observable callbacks are not background samplers that run at arbitrary high frequency. They run when a reader collects metrics, and expensive or blocking callbacks can delay collection. The 0.33.1 API documentation explicitly addresses callback panic behavior; the periodic reader's worker is not isolated from such failures.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



Sources:

- [https://docs.rs/opentelemetry/latest/opentelemetry/metrics/](https://docs.rs/opentelemetry/latest/opentelemetry/metrics/?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md?utm_source=chatgpt.com)

### 5.2 Aggregations and statistical representations

The SDK exposes the `Aggregation` enum for selecting how measurements are summarized.

| Aggregation                 | Behavior and significance                                              |
| --------------------------- | ---------------------------------------------------------------------- |
| `Default`                   | Select the specification-defined aggregation for the instrument        |
| `Drop`                      | Discard measurements for the matching stream                           |
| `Sum`                       | Aggregate an arithmetic total                                          |
| `LastValue`                 | Retain the most recently observed value                                |
| `ExplicitBucketHistogram`   | Track counts in buckets with explicitly specified boundaries           |
| `Base2ExponentialHistogram` | Use exponential-width buckets with configurable scale and bounded size |

The two histogram representations are particularly important.

Explicit-bucket histograms offer predictable, application-relevant boundaries—for example, known latency thresholds. They are useful when the distribution representation should align with predefined service-level thresholds or an existing backend's expectations.

Base-2 exponential histograms represent distributions over wide ranges while controlling the number of buckets through `max_size` and `max_scale`. They can retain useful relative resolution across several orders of magnitude without manually specifying an extensive list of boundaries.

Both are aggregated distributions, not collections of raw samples. They preserve useful statistical information but cannot reconstruct the original observations.

The `record_min_max` option controls whether histogram extrema are retained. Its value depends on whether per-interval extrema or long-running cumulative extrema are analytically useful.

Source: [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/enum.Aggregation.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/enum.Aggregation.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.3 Views: changing metric streams without modifying instrumentation

Views are among the SDK's most architecturally important capabilities.

A view matches an instrument and determines the exported metric stream's representation.

Through `SdkMeterProvider::builder().with_view(...)`, the SDK can associate a selection criterion with a configured `Stream`.

A stream can control:

- Exported metric name.
- Description and unit.
- Aggregation type.
- Cardinality limit.
- Retained measurement attribute keys, under the experimental `spec_unstable_metrics_views` feature.

This makes instrumentation and exported representation distinct layers.

A library can define an instrument according to its operational semantics, while the application independently decides whether to retain that stream, change its aggregation, use a different histogram resolution, or reduce its dimensions.

For example, instrumentation could record operation durations using a histogram. One deployment might prefer explicit buckets aligned with operational thresholds; another might use exponential histograms to inspect a much broader duration range.

Neither needs to modify the instrument call sites.

Important limit: A view transforms or filters measurements available to the SDK. It cannot recover information already discarded by instrumentation, sampling, or an upstream aggregation stage.

Attribute filtering with `with_allowed_attribute_keys` is specifically feature-gated in this release; it should not be mistaken for an unconditionally stable SDK feature.

Sources:

- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.SdkMeterProviderBuilder.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.SdkMeterProviderBuilder.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.StreamBuilder.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.StreamBuilder.html?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 5.4 Metric readers and push versus pull collection

A meter provider requires an appropriate reader to make its metrics available for collection and export.

The primary reader implementations support distinct operating models.

`PeriodicReader` periodically collects aggregated values and invokes a `PushMetricExporter`.

The default interval is 60 seconds. Its worker runs on an SDK-owned background thread, and the export duration is not included in the collection interval. The interval is configurable, including via `OTEL_METRIC_EXPORT_INTERVAL`.

A metric exporter's temporality can influence the representation collected by the reader.

`ManualReader` permits explicit collection. It is suitable for controlled tests, on-demand inspection, and integrations that coordinate their own collection cycle.

Prometheus exporter provides another model: a metrics reader connected to a Prometheus registry. A scraper requests metrics when it needs them, rather than the SDK initiating periodic network exports.

These approaches affect ownership and timing:

| Model                      | Collection initiator | Typical advantage                                                  | Important constraint                                                 |
| -------------------------- | -------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------- |
| OTLP with `PeriodicReader` | SDK                  | Independent push pipeline; backend-independent OTLP representation | Periodic background work and endpoint delivery                       |
| `ManualReader`             | Caller               | Deterministic collection and direct data inspection                | Caller must coordinate collection                                    |
| Prometheus exporter        | Scraper              | Fits established pull-based metrics infrastructure                 | Requires an accessible scrape endpoint and compatible representation |

A reader's collection operation can also invoke observable instrument callbacks. Consequently, the choice of reader affects when external state is sampled, not merely where aggregated measurements are sent.

A custom reader integration is possible through the SDK's `MetricReader` machinery, with the relevant custom-reader facility feature-gated.

Sources:

- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.PeriodicReader.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.PeriodicReader.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.ManualReader.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.ManualReader.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry-prometheus/latest/opentelemetry_prometheus/](https://docs.rs/opentelemetry-prometheus/latest/opentelemetry_prometheus/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 5.5 Delta versus cumulative temporality

Temporality determines the time range represented by aggregated metric values.

Cumulative temporality reports values accumulated since a fixed start time.

Delta temporality reports changes within the most recent collection interval.

For a counter recording increments of 5 and then 8 in successive intervals:

| Interval | Cumulative result | Delta result |
| -------- | ----------------- | ------------ |
| First    | 5                 | 5            |
| Second   | 13                | 8            |

This distinction has several consequences.

1. State retention: Cumulative aggregation generally retains instrument state across collection cycles. Delta aggregation can release interval-specific aggregation state.
2. Downstream interpretation: A backend needs to interpret the start timestamp and temporality correctly. Simply treating a delta value as a cumulative counter changes its meaning.
3. Infrastructure compatibility: Some metric consumers require cumulative representations. The Collector can perform temporality conversion in appropriate pipelines.
4. Changing dimensions: Delta collection can reclaim aggregation state for attributes no longer observed, whereas cumulative streams may retain more historical combinations.

The Rust OTLP exporter exposes environment-driven temporality configuration, and its exporter implements the corresponding preference for the metric reader.

Temporality is not a cosmetic export setting. It changes the metric data model and can determine both memory consumption and the validity of downstream calculations.

Sources:

- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md?utm_source=chatgpt.com)
- [https://opentelemetry.io/docs/specs/otel/metrics/data-model/](https://opentelemetry.io/docs/specs/otel/metrics/data-model/?utm_source=chatgpt.com)

### 5.6 Cardinality management and overflow aggregation

A metrics stream may contain multiple attribute combinations, each defining a separate aggregation series.

The Rust SDK enforces a default cardinality limit of 2,000 attribute combinations per metric stream, configurable using `StreamBuilder::with_cardinality_limit()`.

When the limit is reached, additional attribute combinations are combined into an overflow aggregation identified by:

```
otel.metric.overflow = true
```

This is not equivalent to ignoring new measurements.

The overflow bucket preserves the aggregate contribution of measurements that cannot be represented as separate attribute combinations. For appropriate additive aggregates, the overall total remains represented, but the original dimensional breakdown is lost.

For example, a service might correctly report 100,000 completed operations even though some operations cannot be broken down by their original route or other attributes because the series limit has been exceeded.

There are two distinct kinds of cardinality to consider:

- In-process stream cardinality: Limited by the SDK for each configured stream.
- Backend-wide cardinality: Potentially much larger because of resource attributes, many application instances, and distinct attribute combinations across collection intervals.

A per-process limit of 2,000 does not imply that the observability backend will store only 2,000 time series.

The SDK's metric guide also distinguishes delta and cumulative cardinality behavior. Delta collection can admit different attribute combinations in later intervals, while cumulative collection retains a longer-lived set of series.

This gives a practical architectural tradeoff: cardinality limits bound an important category of memory and export growth, but they sacrifice dimensional query fidelity when overflow occurs.

Sources:

- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.StreamBuilder.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.StreamBuilder.html?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1

### 5.7 Experimental bound instruments: avoiding per-measurement attribute lookup

The 0.33 series includes an easily overlooked high-performance capability: bound metric instruments.

An ordinary synchronous measurement supplies its attributes with each call:

```
counter.add(1, &attributes);
```

The SDK needs to resolve those attributes to the appropriate aggregation state.

With the experimental `experimental_metrics_bound_instruments` feature, a caller can bind a fixed attribute combination once and retain a handle for subsequent measurements.

The feature covers:

- `BoundCounter`
- `BoundUpDownCounter`
- `BoundHistogram`
- `BoundGauge`

The implementation resolves the attribute-to-aggregator association at binding time and caches the result. This removes repeated attribute-map lookups and some associated processing from subsequent measurements.

Where it matters: Very frequent measurements with a small, stable set of attribute combinations can benefit.

Where it does not: Dynamically changing labels, one-off measurements, or high-cardinality sets may incur extra state-management costs without meaningful reuse.

Binding does not change the metrics aggregation model or cardinality limits. It is a hot-path optimization, not an alternative representation of metric dimensions.

This functionality is released in 0.33.1 but remains experimental and feature-gated, not part of the unconditional stable metrics API. The SDK and API feature configuration must be compatible.

Sources:

- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry/CHANGELOG.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry/CHANGELOG.md?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/CHANGELOG.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/CHANGELOG.md?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1

### 5.8 Metric-to-trace correlation: an important gap

The OpenTelemetry metric data model supports exemplars, in which selected metric measurements can retain information such as trace and span identifiers.

Exemplars provide an analytically useful connection from an aggregated metric—such as an unusually large latency observation—to a specific trace.

However, the Rust project's metrics guide explicitly states that exemplars are not yet implemented in the Rust SDK.

Thus, exemplar-based correlation is a capability of the broader OpenTelemetry ecosystem, but should not be presented as an available native Rust SDK feature in this release.

Embedding unique trace IDs directly into ordinary metric attributes is not an equivalent substitute. It introduces high cardinality and defeats much of the benefit of metrics pre-aggregation.

Source: [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



## 6. OpenTelemetry logs and structured diagnostic pipelines

The OpenTelemetry logs model is intentionally different from its tracing and metrics models.

Rather than introducing a new logging facade for applications, it defines a Logs Bridge API that integrates established logging libraries with OpenTelemetry's structured log data model.

For Rust, the major bridges target `tracing` and `log`.

### 6.1 Structured log records

An OpenTelemetry log record can represent:

- Event time and observed time.
- Severity number and severity text.
- Structured body and attributes.
- Trace ID, span ID, and trace flags for correlation.
- Resource and instrumentation scope information.
- Additional flags and metadata defined by the telemetry model.

The `AnyValue` type supports integers, floating-point values, strings, booleans, bytes, lists, and nested maps.

This is materially more expressive than a plain string message. For example, an implementation could represent a structured diagnostics record containing an operation identifier, error details, a set of constraint violations, and nested diagnostic fields.

However, the bridge determines how much structure is preserved.

The `tracing` bridge maps supported primitive fields into structured values, but custom values formatted through `Debug` may become strings. Arbitrary nested `tracing` values are not automatically transformed into nested `AnyValue` maps.

The `tracing` appender documentation also notes that `Valuable` integration is not supported in this version.

Therefore, the OpenTelemetry data model's ability to carry nested structures should not be confused with automatic structured serialization of every Rust type through the standard logging bridge.

Sources:

- [https://docs.rs/opentelemetry/latest/opentelemetry/logs/](https://docs.rs/opentelemetry/latest/opentelemetry/logs/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry/latest/opentelemetry/logs/enum.AnyValue.html](https://docs.rs/opentelemetry/latest/opentelemetry/logs/enum.AnyValue.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry-appender-tracing/0.33.0/opentelemetry_appender_tracing/](https://docs.rs/opentelemetry-appender-tracing/0.33.0/opentelemetry_appender_tracing/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1

### 6.2 Log processors and SDK behavior

`SdkLoggerProvider` owns the logs pipeline configuration.

The principal SDK components include `SdkLogger`, `SdkLogRecord`, `LogProcessor`, and `LogExporter`.

As with tracing, there are two common processor strategies:

| Processor            | Behavior                                 | Consequence                                                                         |
| -------------------- | ---------------------------------------- | ----------------------------------------------------------------------------------- |
| `SimpleLogProcessor` | Immediately invokes an exporter          | Low-latency export but adds work to the emitting thread                             |
| `BatchLogProcessor`  | Queues log records for background export | Lower average emission overhead; requires buffering and accepts loss under overload |

The batch log processor defaults to a queue of 2,048 records, a batch size of 512, and an export delay of approximately one second. These are configurable through `BatchConfigBuilder` and environment variables.

When the bounded queue is full, new log records may be dropped rather than blocking application execution. Thus the pipeline deliberately trades guaranteed retention for bounded memory usage and reduced application interference.

`LogProcessor` is also an extension point for enrichment, filtering, transformation, and custom export behavior.

The upstream logs design describes an efficient processing model in which processors can inspect or mutate log records without taking ownership of each record. The SDK also uses small inline storage for common log attributes. These implementation choices reduce copying and allocation for simple processing, although their benefits depend on the processor and exporter selected.

Sources:

- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/struct.BatchLogProcessor.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/struct.BatchLogProcessor.html?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1

### 6.3 `tracing` events to OTel logs

`opentelemetry-appender-tracing` provides `OpenTelemetryTracingBridge`, implemented as a `tracing-subscriber` layer.

It converts `tracing` events into OpenTelemetry log records, mapping event fields, severity, and context.

When suitable OpenTelemetry context is available, it can attach trace and span identifiers so that diagnostic events are queryable alongside related distributed spans.

It supports two notable experimental extensions:

- `experimental_span_attributes`: Copies selected or all attributes from active `tracing` spans into emitted log records. It can use an allowlist to constrain which contextual fields are duplicated.
- `experimental_metadata_attributes`: Adds source-code location metadata such as file, namespace, and line number.

These extensions are useful because they allow existing structured spans to enrich log events without repeating the same attributes at each logging call site.

The cost is potential duplication of attributes on many records, additional processing, and greater exported volume. A log event with a short message can become substantially larger if it inherits an extensive ancestry of span fields.

Version distinction: The separately published appender's verified release is 0.33.0, rather than the core SDK's 0.33.1. The correct version-specific documentation URL is the 0.33.0 address above, replacing the 0.33.1 appender URL in §2.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 6.4 Telemetry recursion and internal diagnostics

OpenTelemetry's own transport and processing libraries emit diagnostic information.

If those diagnostics are routed into the same OpenTelemetry pipeline, telemetry generation can trigger more telemetry generation—for example, an exporter failure produces a log, which initiates another export.

The SDK has context-based telemetry suppression intended to prevent such feedback loops.

However, context suppression alone does not cover every log emitted by external transport libraries, because those libraries may not propagate OpenTelemetry context.

An additional safeguard is layer-specific filtering through `tracing_subscriber::EnvFilter`, particularly for noisy exporter-related targets such as `hyper`, `h2`, `reqwest`, and `tonic`.

Layer-specific filtering matters: it allows local diagnostics to retain transport errors while preventing the OpenTelemetry export layer from feeding those events back into itself.

OpenTelemetry also emits its own internal diagnostics through `tracing` and exposes internal logging macros such as `otel_debug!`, `otel_warn!`, and `otel_error!`.

The 0.33 SDK additionally includes experimental internal telemetry counters for created log records, processed records/spans, and selected processor outcomes, gated by `experimental_metrics_bound_instruments`.

These mechanisms provide useful insight into the telemetry system's own functioning, especially when queues overflow or exporters fail.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



Sources:

- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/logs.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/logs.md?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/CHANGELOG.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/CHANGELOG.md?utm_source=chatgpt.com)

## 7. Integration with Rust `tracing`

Understanding the distinctions between Rust `tracing` and OpenTelemetry is essential for making effective use of both.

There are two separate bridges with different responsibilities:

Rust `tracing` instrumentation

Spans, events, structured fields, subscribers

`tracing-opentelemetry`

OpenTelemetryLayer

OTel traces

Span hierarchy, context, events

`opentelemetry-appender-tracing`

OpenTelemetryTracingBridge

OTel logs

Structured log records

### 7.1 `OpenTelemetryLayer`: distributed tracing from existing spans

`tracing-opentelemetry` provides `OpenTelemetryLayer`, which integrates `tracing` spans with OpenTelemetry tracing.

This preserves useful Rust-native mechanisms:

- `#[tracing::instrument]` for function-oriented instrumentation.
- `tracing::span!` for arbitrary operation scopes.
- Structured span fields and events.
- Subscriber-level filtering and layer composition.
- Instrumented asynchronous futures.

The OpenTelemetry layer adds the ability to export spans in the OTel data model and associate them with distributed trace contexts.

`OpenTelemetrySpanExt` also supports operations including:

- Setting a remote or explicit OTel parent.
- Retrieving the associated OTel context.
- Adding attributes and events.
- Setting span status.

These are valuable when a program receives an incoming distributed context but primarily uses `tracing` for its application instrumentation.

### 7.2 `OpenTelemetryTracingBridge`: structured logs, not spans

The logs bridge consumes `tracing` events and emits OTel log records.

It does not create OTel distributed trace spans from the surrounding `tracing` spans. That is the responsibility of `OpenTelemetryLayer`.

Consequently, installing only the logs bridge does not provide complete distributed tracing, even though the logging events originate from a crate named `tracing`.

Likewise, installing only `OpenTelemetryLayer` does not necessarily establish an independent OTel logs pipeline.

### 7.3 `MetricsLayer`: metrics through specially named events

`tracing-opentelemetry` also offers a feature-gated `MetricsLayer`.

It interprets specially structured `tracing` events as metric measurements. Supported field naming patterns include prefixes such as:

- `monotonic_counter.`
- `counter.`
- `histogram.`
- `gauge.`

This creates a useful alternative to manually acquiring OTel metric instruments in every call site.

It can be appropriate when instrumentation is already expressed as events and the instrumentation layer is expected to translate those events into metrics.

However, it introduces an additional conversion path and conventions for field naming, types, and attributes. The native Metrics API offers more direct control over instrument creation and reuse, observable callbacks, views, and—in the experimental implementation—bound instruments.

These mechanisms complement one another rather than being interchangeable.

Source: [https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/struct.MetricsLayer.html](https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/struct.MetricsLayer.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.4 Composing subscriber layers

A `tracing_subscriber::Registry` can host independent layers for:

- Local formatted output via `fmt::Layer`.
- Distributed trace conversion via `OpenTelemetryLayer`.
- OTel logs via `OpenTelemetryTracingBridge`.
- Metrics generated from specifically structured events.
- Application-defined subscribers and filters.

This permits one set of `tracing` call sites to serve multiple observational purposes.

The main limitation is that a single event can appear in multiple exported representations. For example, a `tracing` event might become a span event through the trace layer and an independent log record through the logs bridge.

That may be desirable for correlation, but it can also lead to duplicate storage, export costs, or confusing event counts.

Layer-specific filtering provides a mechanism for controlling these representations without suppressing the event globally.

The `tracing` ecosystem also includes `tracing-log`, which can route events originating from the conventional Rust `log` facade into `tracing`. This can consolidate instrumentation before OpenTelemetry bridging, provided the integration is configured to avoid duplicate forwarding and recursive logging.

Sources:

- [https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/](https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/?utm_source=chatgpt.com)
- [https://docs.rs/tracing-subscriber/latest/tracing_subscriber/](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/?utm_source=chatgpt.com)
- [https://docs.rs/tracing-log/latest/tracing_log/](https://docs.rs/tracing-log/latest/tracing_log/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry-appender-tracing/0.33.0/opentelemetry_appender_tracing/](https://docs.rs/opentelemetry-appender-tracing/0.33.0/opentelemetry_appender_tracing/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+1

## 8. OTLP exporters and transport architecture

`opentelemetry-otlp` implements OpenTelemetry Protocol export for traces, metrics, and logs.

OTLP is the primary cross-language interoperability boundary. It allows Rust instrumentation to communicate with collectors and backends without requiring an application-specific exporter for each destination.

### 8.1 Exporter builders and protocols

The crate provides:

- `SpanExporter`
- `MetricExporter`
- `LogExporter`

Each has a signal-specific builder. Exporters are then connected to the corresponding SDK provider, usually through a batch processor or metric reader.

Supported transport combinations include:

| Transport | Representation  | Relevant features | Characteristics                                  |
| --------- | --------------- | ----------------- | ------------------------------------------------ |
| OTLP/gRPC | Protobuf        | `grpc-tonic`      | gRPC transport, metadata, channels, interceptors |
| OTLP/HTTP | Binary protobuf | `http-proto`      | HTTP endpoints and configurable HTTP clients     |
| OTLP/HTTP | JSON            | `http-json`       | JSON-encoded OTLP over HTTP                      |

HTTP/protobuf is enabled by default with the normal crate features, together with a blocking Reqwest HTTP client.

The exporter builders support explicit transport selection through `.with_http()` and `.with_tonic()`.

Since 0.32, the builders can also select an appropriate transport when `.build()` is called directly, according to environment configuration and enabled features.

An abbreviated exporter construction pattern is:

```
use opentelemetry_otlp::{
    Protocol, SpanExporter, WithExportConfig,
};

let exporter = SpanExporter::builder()
    .with_http()
    .with_protocol(Protocol::HttpBinary)
    .build()?;
```

The exporter itself is not a tracer provider. The SDK provider still controls resource identity, sampling, and span processing.

Source: [https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/](https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 8.2 Environment configuration and precedence

The exporter supports the standard `OTEL_EXPORTER_OTLP_*` environment configuration family.

| Configuration       | Environment variable                                                            |
| ------------------- | ------------------------------------------------------------------------------- |
| General endpoint    | `OTEL_EXPORTER_OTLP_ENDPOINT`                                                   |
| Per-signal endpoint | `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT`, `...METRICS_ENDPOINT`, `...LOGS_ENDPOINT` |
| Protocol            | `OTEL_EXPORTER_OTLP_PROTOCOL` and signal-specific variants                      |
| Timeout             | `OTEL_EXPORTER_OTLP_TIMEOUT`                                                    |
| Request headers     | `OTEL_EXPORTER_OTLP_HEADERS`                                                    |
| Compression         | `OTEL_EXPORTER_OTLP_COMPRESSION`                                                |
| Metrics temporality | `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE`                             |

Precedence is:

Programmatic builder configuration → signal-specific environment variable → generic environment variable → built-in default.

There is a significant endpoint distinction:

- For OTLP/HTTP, the generic endpoint is treated as a base endpoint, with `/v1/traces`, `/v1/metrics`, or `/v1/logs` appended as appropriate.
- A signal-specific endpoint is used verbatim; it must contain the intended signal path.

The conventional defaults are:

- gRPC: `http://localhost:4317`
- HTTP: `http://localhost:4318`

The metrics temporality preference accepts `cumulative`, `delta`, and `lowmemory`, with cumulative as the default for the exporter.

This configuration model is useful for environments where an application should not embed backend-specific endpoint logic in its instrumentation.

Source: [https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/](https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



### 8.3 Custom clients, TLS, and transport integration

The OTLP exporter exposes considerably more than endpoint selection.

HTTP integration: `with_http_client()` accepts an implementation of the `opentelemetry_http::HttpClient` abstraction.

This permits transport configuration such as connection pooling, proxy handling, TLS configuration, and client reuse without rewriting OTLP serialization.

gRPC integration: `WithTonicConfig` offers:

- Custom metadata.
- Compression.
- TLS configuration.
- A preconstructed `tonic::transport::Channel`.
- A gRPC interceptor for modifying outgoing requests.
- Export retry policy.

A preconstructed channel can accommodate custom endpoint resolution, connection policy, or Unix-domain socket transport.

The exporter also has distinct rustls TLS feature options:

- `tls-ring`
- `tls-aws-lc`
- `tls-provider-agnostic`
- `tls-roots` and `tls-webpki-roots`

The provider-agnostic option allows integration with a separately configured cryptographic provider, which can be relevant when specific TLS implementations or compliance requirements apply.

The 0.32/0.33 configuration also distinguishes explicitly secure/insecure gRPC endpoints from schemeless endpoints. A schemeless endpoint should not be assumed to mean plaintext.

These extension points are transport facilities of the exporter, not capabilities of the instrumentation API.

### 8.4 Retry, partial success, and reliability limits

In the 0.33 series, retries are enabled by default for both OTLP/gRPC and OTLP/HTTP.

The default policy uses exponential backoff with jitter, with up to three retries after the initial attempt. It distinguishes retryable from non-retryable failures and can account for server throttling hints such as HTTP `Retry-After` and gRPC retry information.

`RetryPolicy` can be customized or disabled.

This is a meaningful improvement over treating temporary network failures as immediate permanent export failures.

However, retry support does not mean guaranteed delivery:

- Data can be dropped before reaching an exporter because an SDK queue is full.
- A process can terminate with telemetry still buffered.
- Network failures can exhaust the retry budget.
- A timeout can leave uncertainty about whether a backend actually accepted data.
- An OTLP server can accept part of a batch while rejecting other records.

The exporter includes handling for OTLP partial-success responses and surfaces warnings about rejected records.

Architectural distinction: OpenTelemetry export is a best-effort observability pipeline with explicit reliability controls. It should not be substituted for an application transaction log or another persistence mechanism requiring durable, exactly-once event storage.

The OTLP/HTTP implementation also supports configurable request-body size limits. An oversized serialized payload can be rejected before network transmission, which is a separate failure mode from ordinary network retry.

Source: [https://docs.rs/crate/opentelemetry-otlp/0.33.1/source/CHANGELOG.md](https://docs.rs/crate/opentelemetry-otlp/0.33.1/source/CHANGELOG.md?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 8.5 The important runtime mismatch: SDK threads versus async HTTP clients

One subtle Rust-specific constraint is especially easy to miss.

The standard `BatchSpanProcessor`, `BatchLogProcessor`, and `PeriodicReader` use dedicated background threads rather than requiring a Tokio runtime for their scheduling logic.

This avoids making the SDK universally dependent on an asynchronous runtime.

But the OTLP exporter may use asynchronous transport clients.

The default thread-based processors are designed for the supported combinations of:

- OTLP/gRPC through `grpc-tonic`, with a suitable live Tokio runtime.
- OTLP/HTTP using a blocking HTTP client, such as `reqwest-blocking-client`.

They are not generally compatible with the default use of asynchronous HTTP clients such as `reqwest-client` or `hyper-client`.

This is further complicated by Cargo feature unification: activating an async HTTP client feature elsewhere in the dependency graph can influence which HTTP client the exporter selects by default.

When a custom HTTP client is supplied, its execution model must match the processor or reader that drives it.

The project also provides experimental async-runtime alternatives for:

- Batch span processing.
- Batch log processing.
- Periodic metrics reading.

These provide a different integration model but remain behind experimental features.

Finally, `shutdown()` on the normal SDK providers can block. Calling shutdown directly from a Tokio current-thread runtime can deadlock if export completion requires that same runtime to make progress. The SDK documentation identifies running the blocking operation on another thread or through `spawn_blocking` as an appropriate mitigation.

This is an example of a consequential interaction between otherwise independent library layers: the exporter can be valid, and the SDK processor can be valid, while their execution models are incompatible.

Sources:

- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/struct.BatchSpanProcessor.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/struct.BatchLogProcessor.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/struct.BatchLogProcessor.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.PeriodicReader.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/struct.PeriodicReader.html?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/](https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2

## 9. Resource identity, instrumentation scopes, and semantic conventions

OpenTelemetry supplies multiple levels of metadata, with distinct meanings and lifecycles.

### 9.1 Three different metadata scopes

| Scope                           | Represents                                        | Examples                                       |
| ------------------------------- | ------------------------------------------------- | ---------------------------------------------- |
| `Resource`                      | The entity producing telemetry                    | Service, deployment, process, host             |
| `InstrumentationScope`          | The instrumentation library or component          | Library name, version, schema URL              |
| Measurement/span/log attributes | Details of an individual observation or operation | Method, outcome, algorithm, operation category |

This distinction prevents repeatedly attaching service-wide identity to every measurement at the application call site.

For example:

- `service.name` describes a service resource.
- An instrumentation-scope name identifies the code or library that emitted the data.
- `operation.type` can describe an individual span or metric measurement.

Metadata should be placed at the level that matches its meaning and lifecycle.

### 9.2 Resource detectors and environment attributes

The SDK uses `Resource` and `ResourceBuilder` to attach identity to telemetry.

`Resource::builder()` includes standard default detection behavior, while `Resource::builder_empty()` begins without the standard defaults.

Resource builders support:

- `with_service_name()`
- `with_attribute()` / `with_attributes()`
- `with_detector()` / `with_detectors()`
- `with_schema_url()`

Environment variables include:

- `OTEL_SERVICE_NAME`
- `OTEL_RESOURCE_ATTRIBUTES`

Custom resource detectors implement the SDK's `ResourceDetector` abstraction. The broader ecosystem supplies detectors for additional environments and platforms.

This can replace repeated service-label construction in every instrumentation library while preserving the ability to identify telemetry independently of how it is emitted.

Resource merging matters: Attributes from different detection sources may overlap. The order and configuration of resource creation determine which values are retained. Schema URLs also have explicit compatibility and merge behavior; merging resources with conflicting schema URLs does not imply that the SDK transparently converts one convention schema into another.

Source: [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/resource/struct.ResourceBuilder.html](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/resource/struct.ResourceBuilder.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 9.3 Semantic conventions

`opentelemetry-semantic-conventions` provides constants for standardized names used in telemetry.

Its generated definitions cover categories such as:

- Service and deployment resources.
- HTTP and RPC attributes.
- Database operations.
- Messaging systems.
- Exceptions and errors.
- Other domain and platform-specific telemetry.

Using shared conventions makes telemetry from independent instrumentation libraries easier to combine and query.

The crate also exposes a `SCHEMA_URL` corresponding to the semantic convention version it represents.

Important distinction: The semantic-conventions crate provides names and version metadata; it does not automatically enforce correct instrumentation semantics, fill in required attributes, or migrate historical telemetry between convention versions.

Semantic conventions also evolve independently of the three core signals. The verified companion release here is `opentelemetry-semantic-conventions` 0.33.0.

Sources:

- [https://docs.rs/crate/opentelemetry-semantic-conventions/0.33.0](https://docs.rs/crate/opentelemetry-semantic-conventions/0.33.0?utm_source=chatgpt.com)
- [https://opentelemetry.io/docs/specs/semconv/](https://opentelemetry.io/docs/specs/semconv/?utm_source=chatgpt.com)
- [https://opentelemetry.io/docs/specs/otel/common/schema/](https://opentelemetry.io/docs/specs/otel/common/schema/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

## 10. Advanced ecosystem integrations and complementary infrastructure

The official Rust implementation is deliberately narrower than the complete observability ecosystem. Several valuable capabilities exist in companion crates, instrumentation libraries, or the Collector rather than in the core SDK.

### 10.1 Prometheus integration

`opentelemetry-prometheus` allows OpenTelemetry metric instruments and SDK aggregations to participate in Prometheus-style collection.

It connects a metric reader to a `prometheus::Registry`, enabling a pull-based export architecture.

This preserves the primary benefit of native OpenTelemetry instrumentation—the ability to represent measurements with standard OTel instrument semantics—while supporting an existing Prometheus scraping model.

A different option is to export OTLP metrics directly to a Prometheus OTLP receiver or through an OpenTelemetry Collector.

These approaches differ in where translation and aggregation occur.

| Integration                | Processing path                                          |
| -------------------------- | -------------------------------------------------------- |
| Native Prometheus exporter | OTel metrics SDK → Prometheus registry → scrape          |
| Direct OTLP ingestion      | OTel metrics SDK → OTLP → compatible Prometheus receiver |
| Collector-mediated         | OTel metrics SDK → OTLP → Collector → metrics backend    |

The Prometheus exporter is separately versioned. Its verified release is 0.33.0.

Two additional distinctions are worth noting:

- The 0.32 release reversed an earlier deprecation of the crate.
- The 0.33 release changed how instrumentation-scope metadata appears as Prometheus labels. Scope attributes can contribute additional dimensions and therefore affect cardinality.

Its 0.32 release also raised the minimum supported Rust version to 1.81, higher than the core API's stated manifest minimum.

Sources:

- [https://docs.rs/crate/opentelemetry-prometheus/0.33.0](https://docs.rs/crate/opentelemetry-prometheus/0.33.0?utm_source=chatgpt.com)
- [https://docs.rs/crate/opentelemetry-prometheus/0.33.0/source/CHANGELOG.md](https://docs.rs/crate/opentelemetry-prometheus/0.33.0/source/CHANGELOG.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

### 10.2 Framework and middleware instrumentation

The `opentelemetry-rust-contrib` repository includes instrumentation integrations for Rust frameworks and infrastructure, including Actix Web and Tower.

These can reduce the need to manually represent common framework operations, such as inbound requests and service-layer processing, as spans.

The relationship is:

- A framework integration understands framework-specific execution semantics.
- Rust `tracing` or the OTel API represents the resulting instrumentation.
- The OpenTelemetry SDK processes and exports telemetry.
- Propagators carry trace context across protocol boundaries.

This separation lets framework libraries provide meaningful defaults while leaving exporter and sampler configuration to the consuming application.

Coverage varies by framework and version. Unlike some environments with extensive runtime monkey-patching, Rust should not be assumed to provide universal automatic instrumentation of every library or asynchronous task.

Source: [https://github.com/open-telemetry/opentelemetry-rust-contrib](https://github.com/open-telemetry/opentelemetry-rust-contrib?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



### 10.3 Platform-native telemetry exporters

The contrib ecosystem also contains integrations with platform-specific instrumentation systems, including:

- Windows Event Tracing for Windows (ETW).
- Linux `user_events`.
- Platform-specific metrics and logs exporters.
- Cloud-specific export integrations.

These systems can support a different execution model from network OTLP export.

For example, the upstream logs design discusses the possibility of synchronous, stack-oriented log processing when exporting to operating-system facilities, avoiding a network export queue in that particular path.

However, such exporters have platform and implementation constraints. A platform-native telemetry sink is not interchangeable with a distributed observability backend, and it may not support the same attributes, retention, transport guarantees, or query features.

The distinction is particularly relevant when instrumentation overhead and integration with host-level tracing or profiling are more important than immediate remote export.

Sources:

- [https://github.com/open-telemetry/opentelemetry-rust-contrib](https://github.com/open-telemetry/opentelemetry-rust-contrib?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md?utm_source=chatgpt.com)

### 10.4 Collector capabilities versus Rust SDK capabilities

The OpenTelemetry Collector offers processors and deployment models beyond the Rust SDK.

These can include:

- Tail-based trace sampling.
- Telemetry filtering and transformation.
- Attribute enrichment and redaction.
- Metric temporality conversion.
- Memory limiting and batching.
- Routing telemetry to multiple destinations.

This permits telemetry processing to be centralized outside application processes.

It also introduces a crucial architectural distinction between information available to the SDK and information available to the Collector.

If a Rust head sampler has already dropped a span, downstream tail sampling cannot recover that span. Similarly, if metrics pre-aggregation has removed original measurements, a Collector cannot reconstruct their individual values.

A downstream processor can transform retained telemetry, but it cannot recover information that upstream components never exported.

Sources:

- [https://opentelemetry.io/docs/collector/](https://opentelemetry.io/docs/collector/?utm_source=chatgpt.com)
- [https://opentelemetry.io/docs/collector/components/](https://opentelemetry.io/docs/collector/components/?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-collector](https://github.com/open-telemetry/opentelemetry-collector?utm_source=chatgpt.com)

## 11. Extension points, testing, and controlled observation

The SDK exposes substantial extensibility without requiring a replacement telemetry framework.

### 11.1 Principal extension interfaces

| Extension point       | Interface or mechanism                       | Example use                                     |
| --------------------- | -------------------------------------------- | ----------------------------------------------- |
| Trace sampling        | `ShouldSample`                               | Domain-aware head-sampling policy               |
| Identifier generation | `IdGenerator`                                | Compatible or specialized trace identifiers     |
| Span processing       | `SpanProcessor`                              | Enrichment, inspection, additional destinations |
| Trace export          | `SpanExporter`                               | Alternative transport or output                 |
| Log processing        | `LogProcessor`                               | Filtering and transforming records              |
| Log export            | `LogExporter`                                | Custom diagnostic sink                          |
| Metric export         | `PushMetricExporter`                         | Alternative push protocol                       |
| Metric collection     | `MetricReader`                               | Specialized collection integration              |
| Metric streams        | Views and `Stream`                           | Output aggregation and dimension control        |
| Resource detection    | `ResourceDetector`                           | Application- or environment-specific identity   |
| Context transport     | `TextMapPropagator`, `Injector`, `Extractor` | Nonstandard protocol metadata                   |

This is an important architectural feature: the SDK's native abstractions permit replacing individual processing stages without requiring every instrumentation call site to know about the replacement.

Custom implementations still need to respect concurrency, ownership, shutdown, and feature-gating contracts.

### 11.2 In-memory exporters for testing

The SDK supplies in-memory exporters behind testing-related features:

- `InMemorySpanExporter`
- `InMemoryLogExporter`
- `InMemoryMetricExporter`

These allow telemetry data to be inspected without sending it to a network backend.

For example, a test can verify that a generic operation:

1. Produces a span with the expected attributes and parentage.
2. Records an error event or status when appropriate.
3. Emits a correlated log record.
4. Contributes to an expected metric aggregation.

An in-memory metric exporter combined with `ManualReader` can provide especially controlled collection behavior.

This distinguishes testing instrumentation semantics from testing external observability infrastructure.

The former can often be deterministic and performed in process. The latter depends on exporter configuration, connectivity, transport behavior, and backend processing.

In-memory exporters do not reproduce all production behavior, including queue overflow, Collector transformations, and remote retention rules.

Sources:

- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/testing/](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/testing/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/trace/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/metrics/?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/](https://docs.rs/opentelemetry_sdk/latest/opentelemetry_sdk/logs/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 11.3 Experimental context observation and profiling interoperability

A less obvious addition in the 0.33 series is the feature-gated global context observer.

With `experimental_context_observer`, the API exposes mechanisms including:

- `ContextObserver`
- `GlobalContextObserver`
- `ObserverContextView`
- Notifications for context-entry and context-exit transitions.

The intended use includes publishing a representation of the current execution context through an alternative channel accessible to external observers, such as an eBPF profiler.

This is a meaningful bridge between distributed tracing context and lower-level execution observation.

Conceptually, it can help associate an external profile sample with the execution context active when that sample was collected.

However, the context observer is not itself a profiler. It does not automatically collect CPU samples, reconstruct stack traces, or establish a complete profiling pipeline.

It is also experimental and carries additional context-management complexity.

Sources:

- [https://docs.rs/opentelemetry/0.33.1/opentelemetry/context/](https://docs.rs/opentelemetry/0.33.1/opentelemetry/context/?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry/CHANGELOG.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry/CHANGELOG.md?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+1

## 12. Performance characteristics and important limitations

OpenTelemetry instrumentation can be made efficient, but it is not free. Its cost comes from distinct activities that are worth evaluating separately.

| Source of overhead     | Relevant controls or mechanisms                         | Consequential tradeoff                                         |
| ---------------------- | ------------------------------------------------------- | -------------------------------------------------------------- |
| Span creation          | Head sampling, instrumentation granularity              | Lower cost can reduce execution detail                         |
| Attribute construction | Reusable keys, bounded attributes, avoiding allocations | Limits diagnostic richness or requires careful instrumentation |
| Metric recording       | Pre-aggregation, views, bound instruments               | Aggregation state and attribute lookup still carry costs       |
| Metric cardinality     | Views, cardinality limits, temporality                  | Bounded memory may sacrifice dimensional detail                |
| Span/log export        | Batching and bounded queues                             | Lower hot-path overhead introduces buffering and possible loss |
| Serialization/network  | Protobuf, batching, compression, transport clients      | Compression and retries consume CPU and time                   |
| Observable callbacks   | Collection interval, lightweight callbacks              | Slow callbacks delay collection                                |
| Async propagation      | Future context association, `tracing` integration       | Incorrect propagation can produce misleading traces            |

### 12.1 Instrumentation is not a substitute for profiling

OpenTelemetry spans measure logical operations. They are well suited to answering questions such as:

- Which operations dominate end-to-end latency?
- Where does a distributed request spend time?
- Which stage fails or retries?
- How do operation durations vary across inputs?
- Which components contribute to an observed service-level problem?

A statistical CPU profiler answers a different question: where does execution time accumulate at the instruction, function, or stack level?

Similarly, an allocation profiler can reveal memory behavior not represented in ordinary telemetry.

OpenTelemetry can provide operational context for profiling, while profiling can explain the low-level costs inside an OpenTelemetry span. Combining the two can be more informative than treating them as substitutes.

### 12.2 Observability versus exact accounting

Different signals have different information-loss models.

Traces may be sampled, so an exported collection of spans is not necessarily the complete set of executed operations.

Logs may be filtered or dropped when queues overflow.

Metrics are aggregated, and cardinality limits may combine distinct attribute combinations into overflow buckets.

Therefore:

- An observed span count need not equal the actual operation count.
- A log count need not equal the number of emitted diagnostic events.
- A counter may preserve aggregate totals while losing the ability to attribute those totals to particular dimensions.

These distinctions matter when telemetry is used for correctness validation, performance analysis, auditing, or other tasks requiring a defined notion of completeness.

Telemetry can provide powerful diagnostic evidence without guaranteeing exhaustive records of every operation.

### 12.3 Lifecycle and initialization

The three SDK providers have independent state and shutdown operations.

A useful provider lifecycle includes resource configuration, exporter attachment, provider creation, global registration where appropriate, and explicit shutdown.

Several details are easy to overlook:

- The global tracer and meter APIs have no-op behavior in the absence of an appropriately configured SDK provider.
- Library instrumentation should not assume that an application has installed a particular SDK or exporter.
- Log bridging is configured through its logger provider rather than an equivalent global application-facing logger API.
- Explicit `shutdown()` is important because providers may remain referenced by global state, instrumentation handles, or subscriber layers.
- Initialization order can influence SDK self-observability. The upstream observability design describes configuring the meter provider before logs processors that obtain internal meters.

A shutdown attempt provides an opportunity to flush outstanding telemetry; it is not a substitute for durable storage or guaranteed process-exit handling.

Source: [https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/observability.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/observability.md?utm_source=chatgpt.com)

### 12.4 Build features and supported Rust environments

The core crates support selectively enabling `trace`, `metrics`, and `logs`.

The 0.33 SDK release also reduces dependencies for minimal builds. With `default-features = false`, substantial optional SDK functionality can be excluded.

Other features selectively enable exporters, TLS implementations, runtime integration, testing facilities, and experimental functionality.

The `opentelemetry` core manifest specifies Rust 1.75.0 as its minimum supported version. Companion crates can have higher requirements, as demonstrated by the Prometheus exporter.

Rust compile-time feature unification deserves particular attention: even when code explicitly configures one exporter or runtime, features activated by another dependency can alter which implementations are compiled or selected.

It is also inappropriate to assume that every SDK/exporter combination supports embedded targets, WebAssembly, `no_std`, or environments without background threads. The API's portability does not establish the portability of the full SDK and transport pipeline.

Sources:

- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry/Cargo.toml](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry/Cargo.toml?utm_source=chatgpt.com)
- [https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/CHANGELOG.md](https://github.com/open-telemetry/opentelemetry-rust/blob/main/opentelemetry-sdk/CHANGELOG.md?utm_source=chatgpt.com)
- [https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/](https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/?utm_source=chatgpt.com)

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2

## 13. Release maturity and areas requiring particular attention

The 0.33 series is a relatively mature implementation of metrics and logs, but it has important feature-specific boundaries.

| Area                                  | Position as of 0.33.1                                               |
| ------------------------------------- | ------------------------------------------------------------------- |
| Core metrics API and SDK              | Stable                                                              |
| Logs bridge API and SDK               | Stable                                                              |
| OTLP logs and metrics exporters       | Promoted to stable in 0.33.1                                        |
| Tracing API, SDK, OTLP trace exporter | Beta                                                                |
| Context, baggage, propagation         | Component-specific beta / release-candidate maturity                |
| Prometheus exporter                   | Separately maintained, beta status in upstream status table         |
| Bound metric instruments              | Released, experimental feature                                      |
| Metric view attribute allowlisting    | Experimental feature                                                |
| Async-runtime processors/readers      | Experimental variants                                               |
| Global context observers              | Released, experimental feature                                      |
| Metric exemplars                      | Not implemented in the Rust SDK                                     |
| Collector tail sampling               | Available in Collector infrastructure, not native SDK head sampling |

The project documentation's high-level status table has not been updated everywhere to reflect the 0.33.1 OTLP exporter promotion. For that detail, the 0.33.1 exporter changelog is more authoritative than older status summaries.

Similarly, development-branch changes must not be assumed present in 0.33.1. The SDK changelog currently lists subsequent work under `vNext`, including enhanced parent-based sampler configuration and fixes for non-finite metric measurements. Those entries describe unreleased work rather than guaranteed behavior of the target release.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+2



## 14. Primary-source reference map

The following sources are especially useful for further investigation. All URLs are given in full.

| Area                                  | Source                                                                                  |
| ------------------------------------- | --------------------------------------------------------------------------------------- |
| Core project and crate inventory      | `https://github.com/open-telemetry/opentelemetry-rust`                                  |
| Core API documentation                | `https://docs.rs/opentelemetry/0.33.1/opentelemetry/`                                   |
| SDK documentation                     | `https://docs.rs/opentelemetry_sdk/0.33.1/opentelemetry_sdk/`                           |
| OTLP exporter documentation           | `https://docs.rs/opentelemetry-otlp/0.33.1/opentelemetry_otlp/`                         |
| OTLP exporter release history         | `https://docs.rs/crate/opentelemetry-otlp/0.33.1/source/CHANGELOG.md`                   |
| Metrics advanced guide                | `https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/metrics.md`        |
| Logs design                           | `https://github.com/open-telemetry/opentelemetry-rust/blob/main/docs/design/logs.md`    |
| `tracing` interoperability            | `https://docs.rs/tracing-opentelemetry/0.34.0/tracing_opentelemetry/`                   |
| OTel logs bridge for `tracing`        | `https://docs.rs/opentelemetry-appender-tracing/0.33.0/opentelemetry_appender_tracing/` |
| Prometheus exporter                   | `https://docs.rs/crate/opentelemetry-prometheus/0.33.0`                                 |
| Semantic conventions                  | `https://docs.rs/crate/opentelemetry-semantic-conventions/0.33.0`                       |
| Contrib crates                        | `https://github.com/open-telemetry/opentelemetry-rust-contrib`                          |
| OpenTelemetry specification           | `https://opentelemetry.io/docs/specs/otel/`                                             |
| OpenTelemetry Rust documentation      | `https://opentelemetry.io/docs/languages/rust/`                                         |
| Collector architecture and components | `https://opentelemetry.io/docs/collector/`                                              |
| Official source and runnable examples | `https://github.com/open-telemetry/opentelemetry-rust/tree/main/examples`               |

## 15. Architectural synthesis

The most consequential capabilities of OpenTelemetry for Rust are not limited to recording execution spans or shipping logs to an external service.

They arise from the interaction of several distinct mechanisms:

First, separation of instrumentation from implementation. Libraries can use standard APIs or `tracing` instrumentation without owning exporter configuration. SDK providers, processors, and exporters can then be selected independently.

Second, native structured telemetry across three signals. Traces represent timed and causal execution, metrics provide efficient statistical summaries, and logs preserve diagnostic events. Resources, scopes, and trace context make those different representations meaningfully related.

Third, advanced control over information volume and representation. Sampling controls which spans are recorded, views determine metric streams and aggregations, cardinality caps bound metric state, and batch processors decouple most network export work from application execution.

Fourth, a substantial extension architecture. Custom samplers, processors, readers, exporters, propagators, and resource detectors make it possible to integrate specialized systems while retaining the common OpenTelemetry data model.

Fifth, a rich Rust-specific integration surface. The `tracing` ecosystem, async context propagation, experimental bound instruments, configurable transport clients, and emerging context-observer facilities provide opportunities to balance structured diagnostics, distributed correlation, and execution overhead.

The main boundaries to preserve are equally important: OpenTelemetry is not a lossless event store; metrics are not raw measurements; span sampling is not tail analysis; and distributed tracing is not low-level profiling.

Within those boundaries, the Rust ecosystem supplies a cohesive foundation for structured, correlated observability while avoiding substantial amounts of bespoke instrumentation, aggregation, context propagation, and transport machinery.