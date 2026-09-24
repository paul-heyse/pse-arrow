# Follow execution and recorder lifetimes

The implementation groups active execution streams and holds recording state through stream ownership. Plan handles, stream handles and exported spans have distinct lifetimes.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| consume to end | Complete stream work is needed | In the resolved DataFusion 55.1.0 profile, EOF drops the inner recording stream even if its public adapter handle remains alive. |
| early drop | Only partial results are consumed | Finalized telemetry may describe partial work. |
| retained plan | The plan is reused | Retaining a plan is distinct from retaining its active streams. |

## Contract

**Evidence boundary.** InstrumentedExec is internal; its source is evidence, not a constructible consumer API.
Claim `tracing.lifecycle.1`; source_observation; evidence: source.

**Ownership.** ExecutionRecordingStream releases its reservation in PinnedDrop. The returned DataFusion RecordBatchStreamAdapter can trigger that destruction on EOF; internal wrapper lifetime and the public handle lifetime differ.
Claim `tracing.lifecycle.2`; source_observation; evidence: source, stream-adapter.

**Executed scope.** Two partitions containing three batches keep spans open before consumption. In the resolved DataFusion 55.1.0 profile, reaching EOF closes spans while the public stream adapters and plan clone remain alive. Dropping exhausted adapters adds no close events; the preview is capped across the group.
Claim `tracing.lifecycle.observed`; runtime_observation; evidence: consumer.

## Implementation

- Use public instrumentation macros in consumers.
- Record identities and lifecycle callbacks to distinguish independent executions and parents.

## Limits and unknowns

- Arbitrary custom ExecutionPlan implementations and cancellation timing remain workload-specific.
- Other resolved DataFusion versions or custom adapters may release their inner streams at a different boundary. Internal PinnedDrop alone cannot establish public EOF behavior.

## Exact contracts

- [`datafusion_tracing::instrumented_exec::InstrumentedExec`](../operations/datafusion_tracing.instrumented_exec.InstrumentedExec.md#op-6b12316c8f26fe0ad5f029de) — `struct InstrumentedExec`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/datafusion-tracing/instrumented_exec.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): Two partitions containing three batches keep spans open before consumption. In the resolved DataFusion 55.1.0 profile, reaching EOF closes spans while the public stream adapters and plan clone remain alive. Dropping exhausted adapters adds no close events; the preview is capped across the group.
  Tests: partition_preview_cap_and_live_stream_lifetime, early_drop_partial_partition_and_repeated_execution, independent_execution_groups_do_not_mix_previews
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
- [stream-adapter](../../skill_improvement/evidence/implementation/upstream-sources/datafusion-physical-plan-55.1.0/src/stream.rs): RecordBatchStreamAdapter stores Option<S> and drops the inner stream on None. This is a boundary source from the resolved consumer profile, not an additional indexed crate.
  Tests: 
