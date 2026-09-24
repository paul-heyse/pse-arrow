# Planning instrumentation: phase

Phase-level observation omits individual Rule spans.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| phase_only() | Phase duration is the question | No individual Rule detail. |
| full() | Individual rule activity is useful | More spans; no fixed volume multiplier. |
| builder() | A subset of planning activity matters | Builder is returned through a public call, but has no public import path. |

## Contract

**Composition.** instrument_rules_with_info_spans!(options: options, state: state) returns the wrapped SessionState.
Claim `tracing.rules.phase.1`; source_observation; evidence: source.

**Selection.** The builder has analyzer, optimizer and physical_optimizer selectors plus phase-only variants. Exact selected phases should be checked against the emitted structure.
Claim `tracing.rules.phase.2`; source_observation; evidence: source.

**Executed scope.** Phase-only and full instrumentation have equal phase membership for the VALUES workload. Only full instrumentation emits Rule spans; their recorded parent IDs are Phase spans.
Claim `tracing.rules.phase.observed`; runtime_observation; evidence: consumer.

## Implementation

- Apply planning instrumentation to the configured state before creating SessionContext.
- Execution instrumentation is a separate optimizer rule; combine when both observations matter.

## Limits and unknowns

- Individual optimizer semantics and cost belong to DataFusion.
- Plan diffs and volume are workload-dependent.

## Exact contracts

- [`datafusion_tracing::instrument_rules_with_info_spans`](../operations/datafusion_tracing.instrument_rules_with_info_spans.md#op-01e5687a47e5f7757a71a3ce) — `macro_rules! instrument_rules_with_info_spans`
- [`datafusion_tracing::instrument_rules_with_info_spans`](../operations/datafusion_tracing.instrument_rules_with_info_spans.md#op-dead3b541632d01ee6f1caec) — `macro_rules! instrument_rules_with_info_spans`
- [`datafusion_tracing::rule_options::RuleInstrumentationOptions`](../operations/datafusion_tracing.rule_options.RuleInstrumentationOptions.md#op-a12428b574074fc1e6db1e0d) — `struct RuleInstrumentationOptions`
- [`datafusion_tracing::rule_options::RuleInstrumentationOptions`](../operations/datafusion_tracing.rule_options.RuleInstrumentationOptions.md#op-a1e41f5dc1584eed32cb9de8) — `struct RuleInstrumentationOptions`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/datafusion-tracing/rule_options.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): Phase-only and full instrumentation have equal phase membership for the VALUES workload. Only full instrumentation emits Rule spans; their recorded parent IDs are Phase spans.
  Tests: phase_selection_and_rule_parentage
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
