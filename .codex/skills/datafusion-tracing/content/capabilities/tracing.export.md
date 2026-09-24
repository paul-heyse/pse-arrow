# Own the bridge, provider and export lifecycle

The bridge connects tracing spans to an SDK tracer. Provider/processor lifetime and span completion govern export; transport success and collector retention are further observations.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| local formatting | Only local tracing output is required | No SDK or OTLP transport needed. |
| simple processor | Immediate local export behavior is useful | Invokes exporter on completed spans. |
| batch processor | Buffering is useful | Queue, timer, flush and shutdown affect delivery. |

## Contract

**Ownership.** Keep the provider and any required runtime alive through completed work, then inspect force_flush/shutdown results.
Claim `tracing.export.1`; source_observation; evidence: source.

**Boundary.** An in-memory exporter establishes local SDK output only. OTLP transport and backend persistence need separately recorded evidence.
Claim `tracing.export.2`; source_observation; evidence: source.

**Executed scope.** An in-memory batch exporter has no completed record while the span lives. Dropping the span and force-flushing makes one record available; successful default shutdown clears the in-memory records. No OTLP transport or collector is involved.
Claim `tracing.export.observed`; runtime_observation; evidence: consumer.

## Implementation

- Select exporter transport features compatible with the runtime and endpoint.
- Use a tracer from the provider in tracing_opentelemetry::layer().with_tracer(tracer).

## Limits and unknowns

- No collector deployment, delivery SLA, queue-loss bound or backend persistence qualification.

## Exact contracts

- [`tracing_opentelemetry::layer::OpenTelemetryLayer`](../operations/tracing_opentelemetry.layer.OpenTelemetryLayer.md#op-62abbc216c2acb69ef65794a) — `struct OpenTelemetryLayer<S, T>`
- [`opentelemetry_sdk::trace::provider::SdkTracerProvider`](../operations/opentelemetry_sdk.trace.provider.SdkTracerProvider.md#op-68f8d3c6d535fa6ce330056b) — `struct SdkTracerProvider`
- [`opentelemetry_sdk::trace::span_processor::SimpleSpanProcessor`](../operations/opentelemetry_sdk.trace.span_processor.SimpleSpanProcessor.md#op-251eecc5a13ec3fb5a757796) — `struct SimpleSpanProcessor<T: SpanExporter>`
- [`opentelemetry_sdk::trace::span_processor::BatchSpanProcessor`](../operations/opentelemetry_sdk.trace.span_processor.BatchSpanProcessor.md#op-7b6dded2e02e9c652c240628) — `struct BatchSpanProcessor`
- [`opentelemetry_otlp::span::SpanExporterBuilder`](../operations/opentelemetry_otlp.span.SpanExporterBuilder.md#op-065d94718617452bd0b1b303) — `struct SpanExporterBuilder<C>`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/examples/otlp.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): An in-memory batch exporter has no completed record while the span lives. Dropping the span and force-flushing makes one record available; successful default shutdown clears the in-memory records. No OTLP transport or collector is involved.
  Tests: batch_flush_exports_completed_spans_and_shutdown_is_observed, local_layer_and_otel_sampling_are_independent
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
