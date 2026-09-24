# Distinguish local filtering and SDK sampling

Different layers can accept different telemetry. A local formatted span is evidence for that output branch, not proof that an SDK exported it.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| per-layer filter | Different destinations need different selection | Preserve a positive observation on another branch. |
| SDK sampler | Trace recording/export policy is the distinction | Sampling is downstream of some tracing work; not a universal zero-cost switch. |

## Contract

**Diagnosis.** Separate dispatch, layer admission, SDK sampling, span completion and export when tracing output diverges.
Claim `tracing.filtering.1`; source_observation; evidence: source.

**Executed scope.** AlwaysOn and AlwaysOff SDK sampling produce one and zero exported spans respectively while the independent local capture records both. This does not qualify every per-layer filter combination.
Claim `tracing.filtering.observed`; runtime_observation; evidence: consumer.

## Implementation

- Observe both local span capture and SDK exporter records under the same controlled span construction.

## Limits and unknowns

- SDK export is not collector receipt. Filtering/sampling cost needs its own measurement.

## Exact contracts

- [`tracing_subscriber::layer::Layer`](../operations/tracing_subscriber.layer.Layer.md#op-4c1ba1a6be909c9a1b91bff8) — `trait Layer<S> where S: Subscriber, Self: 'static`
- [`opentelemetry_sdk::trace::sampler::Sampler`](../operations/opentelemetry_sdk.trace.sampler.Sampler.md#op-18207afc94b6ea2d542774ef) — `enum Sampler`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/examples/otlp.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): AlwaysOn and AlwaysOff SDK sampling produce one and zero exported spans respectively while the independent local capture records both. This does not qualify every per-layer filter combination.
  Tests: local_layer_and_otel_sampling_are_independent
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
