# Tokio family: discovery briefing

Inspected 2026-11-02 at source `7a2f73342`, including the concurrent working tree.
Current-use statements are **Interface-checked** representative reads, not a generated census.
Priorities are **Proposed**; no runtime or performance qualification was run.

## Capability territory and current usage

| Family | Current observation and research relevance |
| --- | --- |
| Runtime, tasks and blocking work | [Semantic workers](../../../../crates/pse-host/src/semantic/workers.rs) retain a `JoinSet`, reap failures, stop admission and drain retained work. Task settlement already has an explicit local owner. |
| Cancellation and lifecycle utilities | The same owner uses `CancellationToken`; cancel-on-drop requests cancellation independently of retained-worker drain. Existing use is broader than a stop flag. |
| Synchronization and channels | Mutexes, channels and readiness notifications can compose admission and delivery; their complete local population was not assessed in this pass. |
| I/O, processes, codecs and streams | Tokio and companion utilities cover asynchronous I/O and adaptation. They do not by themselves establish descendant-process settlement or native CPU cancellation. |
| Time and testing | Timers and test-time control can make selected async behavior easier to exercise. Their suitability depends on the actual operation clock and runtime requirements. |

Read the [Tokio overview](../../../libraries/runtime/tokio.md) and
[family account](README.md) for the wider surface. Runtime and utility features are distinct:
for example, `TaskTracker` is supplied by `tokio-util` with its `rt` feature.

## Research priorities

1. **Compare tracker-based drain with existing task ownership.** `TaskTracker` is promising when
   completion tracking is needed without retaining every result. Preserve the explicit admission
   boundary and failure collection now carried by `Workers`: tracker `close()` does not forbid
   spawning, and tracker completion does not deliver task results. The next question is whether
   any selected owner has bookkeeping that the tracker actually discharges after those residuals.
   [Technical contract](task-drain-contract.md).
2. **Reuse cancellation composition selectively.** Child tokens and drop guards can remove
   mechanical propagation, but `run_until_cancelled` drops the wrapped future and favors ready
   completion. Signal-and-drain operations and native callback polling may need different
   semantics. Ask whether dropping each selected future can abandon a real effect before
   recommending a combinator. [Existing cancellation type](tokio-util.md#tokio-utilcancellation-tree).
3. **Investigate native adapters around actual I/O boundaries.** Codecs, stream adapters and
   readiness utilities may simplify repeated protocol handling. Start from a concrete framing,
   buffering or compatibility need; a wrapper count alone establishes no opportunity.
   [tokio-util API](https://docs.rs/tokio-util/0.7.19/tokio_util/index.html).

Tasks, I/O and CPU work are distinct. Replacing `JoinSet` with a tracker everywhere is not this
briefing's recommendation. Channels, synchronization, process APIs, time and runtime tuning
remain eligible for deeper research when a workflow supplies the question.

## Next evidence and handoff

Generate the `tokio` family evidence in the selected production and test worlds, then interpret
resolved references through representative owners. A fresh reader should explain why
`CancellationToken::cancel`, `TaskTracker::close`, and actual drain are three different events.
Priorities enter detailed opportunity/applicability/review only when that scope is selected.
