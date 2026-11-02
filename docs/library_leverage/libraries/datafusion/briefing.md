# DataFusion and Arrow: discovery briefing

Inspected 2026-11-02 at source `7a2f73342`, including the concurrent working tree.
Current-use explanations are **Interface-checked** from representative source reads;
the later census has its own baseline and does not refresh applicability. Research priorities are
**Proposed**. Product integration, performance and scientific qualification were not run.

The later [pilot census](../../pilot-evidence.md) observed 59,490 resolved references for this
family in the frozen `ad69066e7` source, including `PruningPredicateBuilder`, `prune` and
`MemoryReservation::try_grow` witnesses. These include generated code, imports and type uses;
counts do not measure importance, runtime frequency or whole-configuration coverage.

## Capability territory and current usage

| Family | Current observation and research relevance |
| --- | --- |
| Columnar types, kernels and interchange | Arrow supplies the shared data boundary. Kernels, nested representations and metadata semantics remain eligible beyond familiar batch construction; see the [Arrow overview](../../../libraries/analytical_infrastructure/arrow.md). |
| Expressions, planning and execution | DataFusion owns relational expression execution. [NativeResultSelection](../../../../crates/pse-runtime/src/workflow/result_snapshot.rs) creates a physical expression and uses it for both pruning and residual masks. |
| Statistics and selective access | The same owner already constructs `PruningPredicateBuilder`; investigate propagation into other container/selection shapes rather than recommending first adoption. |
| Memory, cache and runtime services | [Engine cache inspection](../../../../crates/pse-engine/src/cache_service/details.rs) reserves staging through `MemoryReservation` before native enumeration. This is existing library leverage, not evidence of an absent budget. |
| Extension points and integrations | Table providers, execution plans, functions, optimizer rules, storage and companion packages offer larger compositions. This pass did not establish their complete local usage. |

The [DataFusion overview](../../../libraries/analytical_infrastructure/datafusion.md) provides
the wider map. A family can expose capabilities through several defining crates and facade
reexports; inspect defining artifacts and configuration before interpreting a missing entry.

## Research priorities

1. **Reuse conservative pruning across compatible container readers.** `PruningPredicateBuilder`
   plus `PruningStatistics` can operate on application-defined containers, not only Parquet.
   Shared physical-predicate semantics could reduce duplicated window/filter logic. First ask
   which readers possess reliable statistics and an equivalent residual predicate. Integrity
   refusals and scientific validity checks are separate obligations and must survive.
   [Technical contract](pruning-contract.md); [existing types](README.md#types).
2. **Compose memory and cache services across complete operations.** Existing reservations and
   cache controls deserve investigation for reuse before proposing another counter/cache layer.
   Ask which actual allocations or retained buffers are charged, which are shared, and when
   reservations release. A budget API does not prove all native allocations are covered.
   [Memory pool API](https://docs.rs/datafusion-execution/55.1.0/datafusion_execution/memory_pool/index.html).
3. **Keep bulk predicates and projections visible to the engine.** Investigate provider pushdown
   or composed expressions where a selected workflow currently crosses the boundary repeatedly.
   The important question is equivalent null/order/metadata behavior and residual work, not
   whether SQL can spell the operation. No detailed caller audit was performed here.
   [TableProvider API](https://docs.rs/datafusion/55.1.0/datafusion/datasource/trait.TableProvider.html).

Nested functions, row encodings, specialized kernels, partitioning and storage integrations
remain open candidate territory; the shortlist is not a closed API list. Configuration- or
upgrade-dependent functionality must be labeled separately from the pinned local surface.

## Next evidence and handoff

Refresh the `datafusion-arrow` evidence family when its inspected source changes; inspect
facade-to-definition joins and read representative witnesses. A fresh reader should locate the defining pruning crate
and explain why pruning cannot replace residual filtering. A concrete local footprint can
then enter the existing audit/applicability/review routes; this briefing creates no new CBOs.
