# Instrument storage requests and returned streams

The returned wrapper observes calls made through it. Registration and use of the wrapped instance determine whether query I/O reaches that boundary.

Reviewed 2026-09-18. reviewed decision brief; runtime scope is limited to named tests.

## Alternatives

| Mechanism | Applicable condition | Distinction |
|---|---|---|
| wrapped instance | Object-store call observations are required | The consumer must actually use this instance. |
| underlying instance | No wrapper observations are requested | Query results can be the same with a different trace. |

## Contract

**API.** instrument_object_store takes an Arc<dyn ObjectStore> and &str name and returns Arc<dyn ObjectStore>.
Claim `tracing.storage.1`; source_observation; evidence: source.

**Lifecycle.** Method completion and consumption of returned streams are distinct. Inspect each method: a span around returning GetResult is not proof of payload-transfer coverage.
Claim `tracing.storage.2`; source_observation; evidence: source.

**Errors.** instrument_result records result metadata or error text and returns the original Result.
Claim `tracing.storage.3`; source_observation; evidence: source.

**Executed scope.** Direct wrapped and underlying in-memory store calls return identical bytes/ranges; only the wrapper emits store spans. Missing-path errors are recorded, get closes before bytes consumption, and list stays live until consumed. Registration into a DataFusion runtime is not exercised.
Claim `tracing.storage.observed`; runtime_observation; evidence: consumer.

## Implementation

- Wrap the store before passing it to the consuming storage registration point.
- Check target and concrete method; lazy streams/multipart operations need their own boundary evidence.

## Limits and unknowns

- Local controlled storage does not qualify cloud retries, authentication or network traffic.

## Exact contracts

- [`instrumented_object_store::instrumented_object_store::instrument_object_store`](../operations/instrumented_object_store.instrumented_object_store.instrument_object_store.md#op-10b0281459aa2e972c143f48) — `fn instrument_object_store(store: std::sync::Arc<dyn ObjectStore>, name: &str) -> std::sync::Arc<dyn ObjectStore>`
- [`instrumented_object_store::instrumented_object_store::instrument_object_store`](../operations/instrumented_object_store.instrumented_object_store.instrument_object_store.md#op-4eafbb6f5c51434957c89e55) — `fn instrument_object_store(store: std::sync::Arc<dyn ObjectStore>, name: &str) -> std::sync::Arc<dyn ObjectStore>`

## Evidence

`upstream` refers to the exact contracts above, preserving source spans and raw types.
- [source](../../content/corpus/source/instrumented-object-store/instrumented_object_store.rs): Retained exact-release source or documentation; local runtime assertions are linked separately.
  Tests: 
- [consumer](../../skill_improvement/evidence/implementation/probe-results.json): Direct wrapped and underlying in-memory store calls return identical bytes/ranges; only the wrapper emits store spans. Missing-path errors are recorded, get closes before bytes consumption, and list stays live until consumed. Registration into a DataFusion runtime is not exercised.
  Tests: storage_wrapper_result_error_and_lazy_lifetime
- [fixture](../../build/fixtures/probe-crate/tests/contracts.rs): Public consumer implementation and discriminating assertions; captures and resolved profile accompany the receipt.
  Tests: 
