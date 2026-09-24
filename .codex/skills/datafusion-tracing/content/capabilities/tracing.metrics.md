# Record native plan metrics

The recorder reads native plan metrics, aggregates by name, and records dynamic datafusion.metrics.* fields when its recorder is dropped.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| metrics disabled | Only structure or other fields matter | No metric recording requested. |
| metrics enabled | Native node measurements are useful | Available fields depend on the node and span field schema. |

## Contract

**Recording.** MetricsRecorder::drop reads execution_plan.metrics(), returns when absent, and visits aggregate_by_name().
Claim `tracing.metrics.1`; source_observation; evidence: source.

**Interpretation.** Native metric meanings, units and aggregation come from DataFusion. Snapshot catalogs list observations, not all available names.
Claim `tracing.metrics.2`; source_observation; evidence: source.

**Executed scope.** With metrics enabled the completed plan exposes output_rows; disabling metrics removes metric fields in this workload. This is not a complete metric vocabulary or an early-drop value guarantee.
Claim `tracing.metrics.observed`; runtime_observation; evidence: consumer.

## Implementation

- Enable record_metrics and observe the relevant completion boundary.
- Compare values only with compatible units, execution scope and native metric semantics.

## Limits and unknowns

- No exhaustive field list or proof of per-execution metric isolation for every reusable plan.

## Exact contracts

- [`datafusion_tracing::options::InstrumentationOptionsBuilder::record_metrics`](../operations/datafusion_tracing.options.InstrumentationOptionsBuilder.md#op-a779b20995ab3ffb96b8b687) — `record_metrics: bool`
- [`datafusion_tracing::options::InstrumentationOptionsBuilder::record_metrics`](../operations/datafusion_tracing.options.InstrumentationOptionsBuilder.md#op-d9e4c09853923f411cd2ac0b) — `fn record_metrics(self, record: bool) -> Self`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/datafusion-tracing/metrics.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): With metrics enabled the completed plan exposes output_rows; disabling metrics removes metric fields in this workload. This is not a complete metric vocabulary or an early-drop value guarantee.
  Tests: metrics_and_custom_fields_have_separate_controls
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
