# Match dependency and feature profiles

The historical release table records declared combinations. Resolved dependencies, features, successful compilation and delivery are different evidence levels.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| recorded profile | The consumer matches the qualified composition | Transfer only the assertions actually executed. |
| different profile | Versions, targets or features differ | Inspect resolution and compile the relevant composition; absence from the table is not incompatibility. |

## Contract

**Version evidence.** The retained wiring profile uses tracing-opentelemetry 0.32.0 and OpenTelemetry 0.31.0. Version arithmetic is not a compatibility contract.
Claim `tracing.compatibility.1`; source_observation; evidence: source.

**Executed scope.** The retained locked consumer compiles and exports locally with bridge 0.32.0, API/SDK 0.31.0, tracing 0.1.44 and the recorded features. This does not generalize to uncharacterized combinations.
Claim `tracing.compatibility.observed`; runtime_observation; evidence: consumer.

## Implementation

- Inspect actual resolved packages and duplicate crate versions at the trait boundary.
- Qualify compile and runtime separately; do not silently upgrade the consumer.

## Limits and unknowns

- No universal matrix of arbitrary versions/features/targets.

## Exact contracts

- [`opentelemetry_sdk::trace::provider::SdkTracerProvider`](../operations/opentelemetry_sdk.trace.provider.SdkTracerProvider.md#op-68f8d3c6d535fa6ce330056b) — `struct SdkTracerProvider`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/index/compatibility.tsv): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): The retained locked consumer compiles and exports locally with bridge 0.32.0, API/SDK 0.31.0, tracing 0.1.44 and the recorded features. This does not generalize to uncharacterized combinations.
  Tests: local_layer_and_otel_sampling_are_independent
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
