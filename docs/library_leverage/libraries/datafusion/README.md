# Apache DataFusion (pruning, memory, caches, nested plans)

Realizations considered: installed `datafusion` 55.1.0 family (`datafusion-pruning`,
`datafusion-common`, `datafusion-execution`, `datafusion-physical-plan`,
`datafusion-functions-nested`, `datafusion-sql` 55.1.0); 55.2.0 (installed in the registry, still on
arrow 59.2 / object_store 0.13.2); DataFusion `main` (on arrow 60.0.0 and object_store 0.14.2,
unreleased as of 2026-10-27).
Research date 2026-10-27. Sources: installed sources
`datafusion-common-55.1.0/src/{pruning.rs,utils/memory.rs,unnest.rs}`,
`datafusion-execution-55.1.0/src/{memory_pool/mod.rs,memory_pool/arrow.rs,cache/mod.rs,
cache/default_cache.rs}`, `datafusion-physical-plan-55.1.0/src/unnest.rs`,
`datafusion-functions-nested-55.1.0/src/{array_transform.rs,lambda_utils.rs,position.rs,range.rs}`,
`datafusion-sql-55.1.0/src/relation/mod.rs`; Context7 `/apache/datafusion` upgrade guides 54/55
(`PruningStatistics::row_counts` signature, `PruningPredicateBuilder`); GitHub: `main`
`datafusion/execution/src/cache/mod.rs`, commit history of the cache module, PR #22310 (UNNEST WITH
ORDINALITY, closed unmerged).

## Types

### datafusion/pruning-arbitrary-containers
- **Need it serves:** decide, from recorded per-container statistics, which containers cannot
  contain rows satisfying a predicate, for any notion of "container" (file, block, row group,
  page, segment of a block).
- **Recognizable code shapes:** hand-written interval overlap checks of a requested window against
  recorded `min`/`max` per segment; selection logic that knows a segment's key is constant.
- **Mechanisms and APIs:** `datafusion_pruning::PruningPredicateBuilder::new()
  .with_file_schema(schema).try_build(physical_expr) -> PruningPredicate`; `prune(&dyn
  PruningStatistics) -> Result<Vec<bool>>` (one entry per container); `always_true()`. Statistics
  sources: implement `PruningStatistics` (`min_values`, `max_values`, `num_containers`,
  `null_counts`, `row_counts()` without a column parameter since 54, `contained`) or build
  `datafusion_common::pruning::PrunableStatistics::new(Vec<Arc<Statistics>>, schema)` from
  per-container `Statistics`/`ColumnStatistics` with `Precision::Exact`/`Inexact`;
  `CompositePruningStatistics` combines sources. A column whose min equals max per container
  (a constant key) lets equality predicates prune exactly; `contained` supports `IN`-lists.
- **Obligations discharged:** predicate rewriting into statistics expressions, conservative
  evaluation (unknown → keep), null semantics, type coercion of literals, multi-container
  vectorized evaluation.
- **Residual caller obligations:** producing trustworthy per-container statistics; mapping
  container indices back to rows; evaluating the residual row predicate on kept containers;
  deciding what to do when pruning errors (pruning is advisory, so "keep all" is a valid fallback).
- **Semantic conditions:** pruning is conservative and inexact by design; float comparisons follow
  `ScalarValue` ordering (verify NaN and signed-zero handling for IEEE-sensitive predicates);
  statistics marked `Inexact` still prune when bounds allow.
- **Version / build conditions:** `PruningPredicate::try_new` deprecated in 55 in favour of the
  builder.
- **Limits and negative results:** does not perform refusals or integrity checks on the statistics
  themselves.
- **Evidence:** Interface-checked: installed `pruning.rs` (trait, `PrunableStatistics`), Context7
  upgrade notes.

### datafusion/memory-reservation-and-batch-sizing
- **Need it serves:** charge memory to one shared budget before allocating, refuse with a typed
  error when it is exhausted, and measure retained batch memory.
- **Recognizable code shapes:** `MemoryConsumer::new(name).register(&pool)` followed by
  `try_grow(estimate)`; ad-hoc counters beside a pool.
- **Mechanisms and APIs:** `MemoryConsumer::{new, with_can_spill, register}`,
  `MemoryReservation::{try_grow, grow, try_resize, resize, shrink, try_shrink, split, take, free,
  new_empty, size}`; pools `GreedyMemoryPool`, `FairSpillPool`, `TrackConsumersPool`,
  `UnboundedMemoryPool`; `datafusion_common::utils::memory::{get_record_batch_memory_size,
  RecordBatchMemoryCounter, estimate_memory_size}`; feature `arrow_buffer_pool` exposes
  `memory_pool::arrow::ArrowMemoryPool`, which implements `arrow_buffer::MemoryPool` so
  `Array::claim`/`RecordBatch::claim` charge shared Arrow buffers once to a DataFusion pool.
- **Obligations discharged:** pool accounting, typed `ResourcesExhausted` refusal on `try_grow`,
  release on drop, splitting reservations between owners, deduplicated measurement of shared
  buffers.
- **Residual caller obligations:** the size to reserve before an allocation (DataFusion supplies
  no prediction for expression temporaries, decoder output or kernel scratch); reconciling an
  estimate with a later measurement (`try_resize` to the measured size).
- **Semantic conditions:** `ArrowMemoryPool::reserve` calls infallible `grow`, so `claim` can
  over-fill the pool; it is a ledger for retained buffers, not an admission gate.
  `estimate_memory_size` covers hash-table growth for a known element count only.
- **Version / build conditions:** `arrow_buffer_pool` is an opt-in feature of
  `datafusion-execution` that enables `arrow-buffer/pool` for the whole family build.
- **Limits and negative results:** no API predicts `PhysicalExpr::evaluate` temporaries (search
  scope: `datafusion-execution` memory_pool, `datafusion-common` utils/memory, physical-plan
  spill helpers at 55.1.0).
- **Evidence:** Interface-checked: installed sources listed above.

### datafusion/default-cache
- **Need it serves:** a byte-budgeted LRU (optional TTL) for values with self-reported sizes,
  usable outside DataFusion's own metadata caches.
- **Recognizable code shapes:** bespoke LRU maps with byte limits; caches wrapped to add metrics.
- **Mechanisms and APIs:** `datafusion_execution::cache::{Cache, CacheKey, CacheValue,
  default_cache::DefaultCache}`; `DefaultCache::{new(limit), new_with_ttl, with_name,
  with_time_provider, memory_used}`; trait methods `get`, `put`, `remove`, `contains_key`, `len`,
  `clear`, `cache_limit`, `update_cache_limit`, `cache_ttl`, `update_cache_ttl`,
  `drop_table_entries`, `list_entries`.
- **Obligations discharged:** LRU ordering, byte accounting from `CacheKey::size` +
  `CacheValue::size`, rejection of oversize or zero-size entries, lazy TTL expiry.
- **Residual caller obligations:** load deduplication (there is no get-or-load or placeholder
  API), eviction observation (no callback or counter), integration with a `MemoryPool` (the byte
  budget is private to the cache, not a reservation), protecting values still in use after
  eviction.
- **Semantic conditions:** one `Mutex` per cache; `list_entries` clones every entry into a
  `HashMap`; `put` returns the previous value.
- **Version / build conditions:** unchanged on `main` as of 2026-10-27 (last substantive change:
  unification of LRU caches, #22613, 2026-06-18).
- **Limits and negative results:** no single-flight, no eviction listener, no pool integration at
  55.1.0 or on `main` (search scope: `cache/mod.rs` and `default_cache.rs`).
- **Evidence:** Interface-checked: installed and `main` sources.

### datafusion/nested-occurrence-plans
- **Need it serves:** locate violating elements inside nested values (lists, maps, structs) with
  their row and element coordinates, using plan operators rather than imperative traversal.
- **Recognizable code shapes:** recursive walkers over `ArrayData` children with per-node `take`;
  path strings built per element.
- **Mechanisms and APIs:**
  - `LogicalPlanBuilder::unnest_columns_with_options(cols, UnnestOptions { null_handling,
    recursions })`; physical `UnnestExec` supports `List`, `LargeList`, `FixedSizeList`, `ListView`,
    `LargeListView` and struct columns, emits batches of about `batch_size` rows, and holds no
    `MemoryReservation`.
  - Ordinals: no native ordinality. SQL `UNNEST … WITH ORDINALITY` returns `not_impl_err` at
    55.1.0; PR #22310 adding it closed unmerged. The working composition is
    `range(0, array_length(col))` unnested in lockstep with the list column.
  - Higher-order functions: `array_transform`, `array_filter`, `array_any_match`, `array_first`
    take single-parameter lambdas (no element-index parameter). `array_positions(list, needle)`
    returns all 1-based positions equal to a scalar; composed as
    `array_positions(array_transform(col, x -> <violation>), true)` it can yield per-row violating
    positions for one nesting level without unnest.
  - `map_keys`, `map_values`, `named_struct`, `get_field`, `concat` for coordinates.
- **Obligations discharged:** vectorized element expansion and predicate evaluation, batch
  sizing, null-list handling (`NullHandling::Drop`/`Preserve`).
- **Residual caller obligations:** ordinal synthesis, path rendering, visibility rules for unions,
  dictionaries, run-end encoding and tagged alternatives (not unnest targets), memory admission
  (unnest repeats every carried column per element and does not reserve).
- **Semantic conditions:** null elements inside lambdas evaluate to null; `array_positions`
  matches with `not_distinct` semantics against the needle, so a null violation flag is not a
  match unless the lambda normalizes it (`IS NOT TRUE`).
- **Version / build conditions:** lambda (higher-order UDF) support is present at 55.1.0.
- **Limits and negative results:** no element-index lambda parameter; no ordinality; no union
  unnest (search scope: installed functions-nested sources, `UnnestOptions`, sql relation
  planner, DataFusion PR search for "unnest ordinality").
- **Evidence:** Interface-checked: installed sources, GitHub PR search. The `array_positions` ∘
  `array_transform` composition is not exercised.

## Current applicability

### [CBO-007](../../../library_leverage/codebase/result-data-paths.md#cbo-007) — Output-predicate semantics; dense selection outside the planner
- **Types:** `datafusion/pruning-arbitrary-containers`; residual mask from
  `arrow-compute/predicate-selection`.
- **Realization:** installed 55.1.0.
- **Conclusion:** fits with conditions. Recorded dense outputs can be pruning containers: per
  `Output`, `symbol_id` min = max = the output's symbol and `time` min/max = `coordinate_min`/
  `coordinate_max`, `num_rows` exact; the same `PrunableStatistics` + `PruningPredicate` route the
  scalar path already uses (`result_snapshot.rs::NativeResultSelection::prune_members`) can then
  select outputs for `symbol = s AND time BETWEEN a AND b`.
- **What it would discharge / what remains local:** discharges the hand-written window overlap
  decision across many outputs (cohort and pack reads); the dense integrity checks — adjacent-row
  refusal, equality of recorded extrema with actual data, contiguity, no `missing` on dense
  outputs — are refusals about the data, not selection, and stay local. For a single output
  already chosen by identity, pruning reduces to one interval check and adds little.
- **Prerequisites:** a `Statistics` per dense output from the recorded `Output` rows (the schema
  is already bound); a predicate `Expr` built from the dense filter.
- **Semantic gaps:** pruning never refuses, so refusal diagnostics pinned by tests must remain on
  the local path; IEEE-bit predicate identity requires confirming float comparison of statistics
  (`ScalarValue` ordering for NaN/−0.0) matches the domain rule.
- **Not a library route:** the triplicated finiteness / ordering / missing-value validation and the
  `PackResultPredicate` duplicate are local model consolidation (LOP-02/09). DataFusion's
  `interval_arithmetic::Interval` was considered by name only (not inspected); a closed interval
  type would at most carry `min <= max`, while finiteness and "missing excludes bounds" stay
  domain rules.
- **Unsettled premises:** how many outputs a typical dense read must choose among (if one, the
  value is consistency, not work avoided).
- **Evidence:** Interface-checked; application Proposed.

**Reviewer correction (2026-10-27).** Ruled out here, not "fits with conditions". The window-overlap
decision across dense outputs already runs in generated SurrealQL (the
`canonical_result_block_outputs` query over coordinate projections), which filters outputs on the
`result_output_range` index (`result_set, output, partition, start`) before any transfer, with bit
and finite-projection assertions on `coordinate_min`/`coordinate_max`. A DataFusion pruning route
would move the selection away from its index. If that query or index changes, this conclusion
must be revisited. See the [comparative review](../../reviews/result-data-and-prepaid-accounting.md#cbo-007).

### [CBO-012](../../../library_leverage/codebase/persistence-and-transport.md#cbo-012) — Imperative nested-occurrence traversal beside the plan-based route
- **Types:** `datafusion/nested-occurrence-plans`.
- **Realization:** installed 55.1.0.
- **Conclusion:** ruled out here as a replacement for the imperative traversal; unsettled as a
  partial convergence.
- **Reasoning:** the routes do different jobs. `validate/occurrences.rs` is the field-local
  domain and non-null check run by `prepared.rs` in 4096-row chunks, over every storage form
  including `Union`, `Dictionary`, `RunEndEncoded` and tagged alternatives, with diagnostic paths
  materialized only for sampled findings. `obligations/nested_values.rs` builds plans for
  cross-row obligations (ordinals, references, quantities, source spans, physical) and already
  answers the research question: it obtains occurrence coordinates with
  `range(0, array_length)` unnested in lockstep and `concat` path strings. DataFusion 55.1.0 has no
  ordinality or index-carrying lambda that would simplify that, cannot unnest unions or
  run-end-encoded storage, and `UnnestExec` does not reserve memory, which conflicts with "memory
  bounded by the pool". `predicates.rs` already uses `array_any_match` lambdas for row-level
  detection.
- **What could converge:** for list-of-primitive fields at depth one,
  `array_positions(array_transform(col, x -> violation IS TRUE), true)` would give violating
  positions in the plan route without unnest; deeper nesting and non-list storage remain
  imperative.
- **Semantic gaps:** visible-value semantics for union arms and tagged alternatives; diagnostic
  sampling budget; pool-bounded memory.
- **Unsettled premises:** whether the CHECK-text parser adapter in `predicates.rs` (LOP-11 part of
  the CBO) has a DataFusion-native replacement was outside this cluster's scope.
- **Evidence:** Interface-checked; application Proposed.

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — Single-flight, I/O gates and load admission (DataFusion cache part)
- **Types:** `datafusion/default-cache`.
- **Realization:** installed 55.1.0 (already used: `cache_service/envelope.rs`,
  `session/cache/model.rs`).
- **Conclusion:** ruled out here for load deduplication and admission. `DefaultCache` has no
  get-or-load, no eviction notification (confirming the codebase comment) and a private byte
  budget rather than a `MemoryReservation`, so it cannot be "the DataFusion pool as the single
  budget" or retain a producer past waiter cancellation. Its existing use as the LRU store is the
  fit it offers.
- **Evidence:** Interface-checked.

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) — Prepaid memory accounting (DataFusion part)
- **Types:** `datafusion/memory-reservation-and-batch-sizing`.
- **Conclusion:** already adopted for the pool and reservations; ruled out as a source of
  predictions. `try_resize` to `get_record_batch_memory_size` after decode is the available way to
  reconcile an over-estimate with measured retention.
- **Evidence:** Interface-checked.
