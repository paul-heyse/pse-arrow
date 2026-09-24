# Wiring a subscriber and an exporter

Tracing dispatch and subscriber layers observe the instrumentation. Formatting, per-layer filters, OpenTelemetry sampling and export have distinct boundaries. A local formatted span does not prove collector receipt.

## Entry points

| Item | Visibility | Methods | Reached via |
|---|---|---:|---|
| [`Registry`](../api/tracing_subscriber.registry.sharded.md) | supported | 16 | — |
| [`SubscriberInitExt`](../api/tracing_subscriber.util.md) | supported | 3 | — |
| [`OtelData`](../api/tracing_opentelemetry.md) | supported | 3 | — |

## What this seam cannot tell you

- Whether every filter matches the intended spans. Execution macros default to the call-site target; planning instrumentation also emits spans under datafusion_tracing. Inspect each family or explicit target separately.
- Whether an uncharacterized OpenTelemetry profile composes. The compatibility catalog records historical requirements, not evidence of incompatibility or proof that upstream never tested an absent pair.
- A replacement for the OpenTelemetry SDK's own documentation on sampling, batching or resource attributes.

## Read next

- [`catalogs/compatibility.md`](../catalogs/compatibility.md)
- [`content/corpus/examples/otlp.rs`](../corpus/examples/otlp.rs) — upstream, verbatim
- [`content/corpus/tests/integration_tests.rs`](../corpus/tests/integration_tests.rs) — upstream, verbatim
- [`content/corpus/tests/test_utils/in_memory_writer.rs`](../corpus/tests/test_utils/in_memory_writer.rs) — upstream, verbatim
