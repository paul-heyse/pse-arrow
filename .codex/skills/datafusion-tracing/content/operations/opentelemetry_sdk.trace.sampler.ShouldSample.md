# `opentelemetry_sdk::trace::sampler::ShouldSample`

Full upstream contracts; raw type trees and source locators in [structured records](opentelemetry_sdk.trace.sampler.ShouldSample.json).

<a id="op-e261fe03f8ec2adecc6ef339"></a>
## ShouldSample

`trait` · `opentelemetry_sdk::trace::sampler::ShouldSample` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
trait ShouldSample: CloneShouldSample + Send + Sync + std::fmt::Debug
```

Source: `src/trace/sampler.rs:64`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

The [`ShouldSample`](../operations/opentelemetry_sdk.trace.sampler.ShouldSample.md#op-e261fe03f8ec2adecc6ef339) interface allows implementations to provide samplers
which will return a sampling [`SamplingResult`](../operations/opentelemetry.trace.tracer.SamplingResult.md#op-e423c1725dcb165dcc01f8f4) based on information that
is typically available just before the [`Span`] was created.

# Sampling

Sampling is a mechanism to control the noise and overhead introduced by
OpenTelemetry by reducing the number of samples of traces collected and
sent to the backend.

Sampling may be implemented on different stages of a trace collection.
[OpenTelemetry SDK] defines a [`ShouldSample`](../operations/opentelemetry_sdk.trace.sampler.ShouldSample.md#op-e261fe03f8ec2adecc6ef339) interface that can be used at
instrumentation points by libraries to check the sampling [`SamplingDecision`](../operations/opentelemetry.trace.tracer.SamplingDecision.md#op-29fe2075c223853678046a1f)
early and optimize the amount of telemetry that needs to be collected.

All other sampling algorithms may be implemented on SDK layer in exporters,
or even out of process in Agent or Collector.

The OpenTelemetry API has two properties responsible for the data collection:

* [`Span::is_recording()`]. If `true` the current [`Span`] records
  tracing events (attributes, events, status, etc.), otherwise all tracing
  events are dropped. Users can use this property to determine if expensive
  trace events can be avoided. [`SpanProcessor`]s will receive
  all spans with this flag set. However, [`SpanExporter`]s will
  not receive them unless the `Sampled` flag was set.
* `Sampled` flag in [`SpanContext::trace_flags()`]. This flag is propagated
  via the [`SpanContext`] to child Spans. For more details see the [W3C
  specification](https://w3c.github.io/trace-context/). This flag indicates
  that the [`Span`] has been `sampled` and will be exported. [`SpanProcessor`]s
  and [`SpanExporter`]s will receive spans with the `Sampled` flag set for
  processing.

The flag combination `Sampled == false` and `is_recording == true` means
that the current `Span` does record information, but most likely the child
`Span` will not.

The flag combination `Sampled == true` and `is_recording == false` could
cause gaps in the distributed trace, and because of this OpenTelemetry API
MUST NOT allow this combination.

[OpenTelemetry SDK]: https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/trace/sdk.md#sampling
[`SpanContext`]: opentelemetry::trace::SpanContext
[`SpanContext::trace_flags()`]: opentelemetry::trace::SpanContext#method.trace_flags
[`SpanExporter`]: crate::trace::SpanExporter
[`SpanProcessor`]: crate::trace::SpanProcessor
[`Span`]: opentelemetry::trace::Span
[`Span::is_recording()`]: opentelemetry::trace::Span#tymethod.is_recording

<a id="op-0483327ff4b358d3b3605811"></a>
## should_sample

`function` · `opentelemetry_sdk::trace::sampler::ShouldSample::should_sample` · opentelemetry_sdk 0.31.0
Reachability: `supported`.  Capture: hosted.

```rust
fn should_sample(&self, parent_context: Option<&Context>, trace_id: TraceId, name: &str, span_kind: &SpanKind, attributes: &[KeyValue], links: &[Link]) -> SamplingResult
```

Source: `src/trace/sampler.rs:74`. [Exact documentation build](https://docs.rs/crate/opentelemetry_sdk/0.31.0/json).

Returns the [`SamplingDecision`](../operations/opentelemetry.trace.tracer.SamplingDecision.md#op-29fe2075c223853678046a1f) for a [`Span`] to be created.

The [`should_sample`] function can use any of the information provided to it in order to
make a decision about whether or not a [`Span`] should or should not be sampled. However,
there are performance implications on the creation of a span

[`Span`]: opentelemetry::trace::Span
[`should_sample`]: ShouldSample::should_sample
