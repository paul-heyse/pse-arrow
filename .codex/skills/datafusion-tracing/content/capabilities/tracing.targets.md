# Match macro targets and levels

The default macro target comes from module_path!() at the call site; explicit target and macro level change filter selection.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| default target | Call-site module identity is useful | A datafusion_tracing-only directive need not select it. |
| explicit target | A stable chosen target is useful | Filter that exact target and an enabled level. |

## Contract

**Target.** Macro arms forward an explicit target or module_path!() to tracing span construction.
Claim `tracing.targets.1`; source_observation; evidence: source.

**Level.** INFO/DEBUG and other macro families choose their emission level. A layer can exclude spans at that level.
Claim `tracing.targets.2`; source_observation; evidence: source.

**Executed scope.** A matching explicit-target filter captures execution spans; a nonmatching filter captures none. The call-site target contracts is separately observed under its matching filter.
Claim `tracing.targets.observed`; runtime_observation; evidence: consumer.

## Implementation

- Inspect the actual invocation and installed filter scope.
- Object-store spans use their own source target; test that branch separately.

## Limits and unknowns

- A missing capture alone cannot distinguish filtering from work that did not run.

## Exact contracts

- [`datafusion_tracing::instrument_with_info_spans`](../operations/datafusion_tracing.instrument_with_info_spans.md#op-8596983b2e498bf8b81a1b42) — `macro_rules! instrument_with_info_spans`
- [`datafusion_tracing::instrument_with_info_spans`](../operations/datafusion_tracing.instrument_with_info_spans.md#op-c802d29699964499eb5192a3) — `macro_rules! instrument_with_info_spans`
- [`tracing_subscriber::filter::env::EnvFilter`](../operations/tracing_subscriber.filter.env.EnvFilter.md#op-f18b1a3b867d9bc485e30055) — `struct EnvFilter`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/datafusion-tracing/exec_instrument_macros.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): A matching explicit-target filter captures execution spans; a nonmatching filter captures none. The call-site target contracts is separately observed under its matching filter.
  Tests: explicit_target_and_filter_control
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
