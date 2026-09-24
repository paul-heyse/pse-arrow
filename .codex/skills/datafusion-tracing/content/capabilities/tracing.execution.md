# Instrument physical query execution

Execution macros construct an optimizer rule that wraps the physical plan it receives. Later plan rewrites can change which work is represented.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| execution rule | Physical operators need spans | Register at the point where the plan to observe is established. |
| planning macros | Planning phases or optimizer rules need observation | These wrap session state and describe different work. |

## Contract

**Inputs and output.** The INFO macro accepts a default or explicit target, required options, and optional declared fields and yields an Arc<dyn PhysicalOptimizerRule>.
Claim `tracing.execution.1`; source_observation; evidence: source.

**Registration.** The rule recursively wraps nodes. A later rule may preserve, replace or add nodes; blanket claims that every later rule loses spans are too strong.
Claim `tracing.execution.2`; source_observation; evidence: source.

**Executed scope.** A controlled optimizer adds GlobalLimitExec before or after instrumentation. Both return three rows; the added node has a completed span only when instrumentation sees it.
Claim `tracing.execution.observed`; runtime_observation; evidence: consumer.

## Implementation

- Build options and declare custom keys at the macro invocation.
- Add the returned rule to SessionStateBuilder. Execute and consume the resulting query; plan construction alone does not prove execution spans.

## Limits and unknowns

- No universal query-overhead bound; actual later rewrites need their own observation.

## Exact contracts

- [`datafusion_tracing::instrument_with_info_spans`](../operations/datafusion_tracing.instrument_with_info_spans.md#op-8596983b2e498bf8b81a1b42) — `macro_rules! instrument_with_info_spans`
- [`datafusion_tracing::instrument_with_info_spans`](../operations/datafusion_tracing.instrument_with_info_spans.md#op-c802d29699964499eb5192a3) — `macro_rules! instrument_with_info_spans`
- [`datafusion_tracing::options::InstrumentationOptions`](../operations/datafusion_tracing.options.InstrumentationOptions.md#op-07a1777b923d8d00bd1e5318) — `struct InstrumentationOptions`
- [`datafusion_tracing::options::InstrumentationOptions`](../operations/datafusion_tracing.options.InstrumentationOptions.md#op-e8a1e87352fc8239e55d0d93) — `struct InstrumentationOptions`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/datafusion-tracing/exec_instrument_rule.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): A controlled optimizer adds GlobalLimitExec before or after instrumentation. Both return three rows; the added node has a completed span only when instrumentation sees it.
  Tests: later_rewrite_changes_observed_node_coverage
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
