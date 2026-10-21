# Data and results family: capability-gap inventory (revised against the usage census)

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Capability-gap inventory: Arrow 59.3 / Parquet 59.3 / DataFusion 55.1 / object_store.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

The census changes none of my recommendations. It confirms the absences they rest on. I verified each one by hand: no `ParquetSource`, `FileScanConfig`, `ListingTable`, `WriterProperties`, `RowSelection` or `PruningPredicate` use, Parquet writers only in tests, and the Arrow `pool` feature off.

The census has one error, and one gap you should fix before it feeds the review:
- **Error: physical optimizer rules.** It reports zero uses of `PhysicalOptimizerRule`, but there are two. `SemanticFields` and `AggregateFields` are at `crates/pse-engine/src/session/physical_fields.rs:24` and `:80`. They are written with fully qualified paths, which the census pattern missed. `AggregateFields` matters to §4.1 below.
- **Gap: the pool stack.** The census lists the pool as `TrackConsumersPool<FairSpillPool>`. The real stack is `ReleaseNotifyingPool(PeakRecordingPool(TrackConsumersPool<FairSpillPool>))`, and `PeakRecordingPool` is DataFusion 55.1's own type (`crates/pse-engine/src/resources.rs:12`, `:222-231`).
- **Minor:** the census's `owned_buffer.rs:1487` `impl MemoryPool` is a `CancellingPool` inside a test module.

**Scope and evidence.** Baseline `a791c533e`, with the audit pages at `0530dcaa1`. Nothing was edited, built or run. Library facts come from the pinned crate sources under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/` (written `REG/` below) and are *Interface-checked*. Repository facts are *Implemented*. Every cost claim is *Proposed*. I applied the maintainer's clarification: nothing below weakens exact-bit or ±0 semantics, SurrealDB's selection and commit authority, lane parallelism or live-sample streaming; anything that would is marked **remedy open**.

Crates that are already resolved and in scope: `datafusion-pruning`, `datafusion-datasource-arrow`, `datafusion-datasource-parquet`, `datafusion-physical-optimizer`, `datafusion-session`, and the arrow-buffer, arrow-data and arrow-array `pool` feature, which is currently off.

---

## 1. What the libraries offer at the pins

| Area | Capability (pinned API) | Conditions and limits |
|---|---|---|
| Arrow IPC file reading | `arrow_ipc::reader::FileDecoder::{new, with_projection, with_require_alignment, read_dictionary, read_record_batch(&Block,&Buffer)}` (`REG/arrow-ipc-59.3.0/src/reader.rs:1032-1116`) | Zero-copy over an aligned `Buffer`, one block per call. `FileReaderBuilder::with_max_footer_fb_tables/depth` verifies the footer (`:1200-1217`). |
| DataFusion IPC source | `ArrowSource::new_file_source` (`REG/datafusion-datasource-arrow-55.1.0/src/source.rs:260-280`) | Selects record batches by byte range through `ObjectStore::get_ranges` (`:112-235`). It does no predicate or statistics pruning. |
| Parquet statistics | min/max skip NaN (`REG/parquet-59.3.0/src/column/writer/mod.rs:1706-1737`). A zero min is written as -0.0 and a zero max as +0.0 (`column/writer/encoder.rs:451-470`). | There is no `nan_count` and no IEEE 754 total-order column order (`basic.rs:1004`; rg for `total_order\|ieee754` found nothing). DataFusion still reports these values as `Precision::Exact` unless truncated (`REG/datafusion-datasource-parquet-55.1.0/src/metadata.rs:620-643`). |
| Parquet writer | `WriterProperties::set_{statistics_enabled, column_statistics_enabled, bloom_filter_enabled, column_bloom_filter_*, sorting_columns, data_page_row_count_limit, encoding, column_index_truncate_length, max_row_group_row_count, content_defined_chunking}` | The dictionary interner keys values by their bytes (`util/interner.rs:62-71`), so it should be bit-exact. Probe 1 confirms. |
| Parquet reader | `with_row_selection`, `with_row_filter`, `with_row_groups`, `with_offset`, `with_limit`, `with_page_index_policy`, `with_virtual_columns` (`arrow/arrow_reader/mod.rs:213-787`); the `RowNumber` virtual column (`parquet.virtual.row_number`, Int64; `arrow/schema/virtual_type.rs:84`) | — |
| DataFusion Parquet | `ParquetSource::{with_predicate, with_pushdown_filters, with_enable_page_index, with_bloom_filter_on_read}` (`source.rs:347-469`); `ParquetAccessPlan` or a file-level `RowSelection` attached through `PartitionedFile::with_extension` (`access_plan.rs:18-130`); `ParquetVirtualColumn::RowNumber`; bloom filters accept `FixedSizeBinary` and `Int64` (`bloom_filter.rs:93-107`) | `ParquetAccessPlan` is the hook through which an external index chooses row groups and rows. |
| Pruning custom containers | `PruningPredicate::try_new(expr, schema).prune(&dyn PruningStatistics)` (`REG/datafusion-pruning-55.1.0/src/pruning_predicate.rs:598-618`); `PruningStatistics` and `PrunableStatistics::new(Vec<Arc<Statistics>>, schema)` (`REG/datafusion-common-55.1.0/src/pruning.rs:62-128`, `:314-330`) | `PrunableStatistics` uses only `Exact` values (`:342-414`). I found no NaN handling in the pruning code (search scope at the end of §5). |
| Table providers and plans | `TableProvider::{scan_with_args(ScanArgs{filters, projection, limit, statistics_requests}), supports_filters_pushdown, statistics}` (`REG/datafusion-session-55.1.0/src/table.rs:185-317`, `:428-520`); `StatisticsRequest` (`datafusion-expr-common/src/statistics.rs:1722-1737`); `ExecutionPlan::{partition_statistics → Arc<Statistics>, child_stats_requests, supports_limit_pushdown, with_fetch, gather_filters_for_pushdown, reset_state}` (`REG/datafusion-physical-plan-55.1.0/src/execution_plan.rs:473-991`); `PartitionedFile::{with_statistics, with_ordering, with_extension, with_range}` (`REG/datafusion-datasource-55.1.0/src/mod.rs:242-392`) | — |
| Float semantics | Arrow's comparison kernels and its `min`/`max` use IEEE totalOrder: -0 < +0, +NaN above +∞, -NaN below -∞ (`arrow-array/src/arithmetic.rs:39`, `:392-398`; `arrow-arith/src/aggregate.rs:127-167`, `:1002`). DataFusion's `BinaryExpr` calls those kernels (`datafusion-physical-expr-common/src/datum.rs:21`; `binary.rs:46`). | In DataFusion SQL, `-0.0 = 0.0` is false and `NaN > 5` is true. |
| Arrow-side memory tracking | `{Buffer, ArrayData, RecordBatch}::claim(&dyn arrow_buffer::MemoryPool)`. Each `Bytes` holds one reservation, which a later claim replaces and which is released on drop (`REG/arrow-buffer-59.3.0/src/bytes.rs:110-127`). `ArrowMemoryPool` adapts a DataFusion pool (`datafusion-execution/src/memory_pool/arrow.rs`). The cargo feature is `datafusion-execution/arrow_buffer_pool`. | `reserve` uses an infallible `grow` and registers one consumer per claimed `Bytes`. It records usage; it cannot refuse an allocation. |
| Memory pools | `TrackConsumersPool::metrics`, `PeakRecordingPool::{peak_reserved, reset_peak}` (`memory_pool/pool.rs:405-520`; `peak_recording.rs:86-140`) | Both are in use already. |
| Heap sizing | `datafusion_common::heap_size::{DFHeapSize, DFHeapSizeCtx}`, which counts shared `Arc` allocations once (`heap_size.rs:72-480`) | No derive macro. |
| Caches | `CacheManagerConfig::with_{file_metadata_cache, file_statistics_cache, list_files_cache(+ttl)}`; the `FileMetadata{memory_size}` trait (`cache/cache_manager.rs:247-506`) | Keyed by object path plus `ObjectMeta`. |
| Tracing | `InstrumentationOptions{record_metrics, preview_limit, custom_fields}`, `RuleInstrumentationOptions`, `instrument_object_store` | Traces only I/O that goes through an `ObjectStore`. |
| Extension types | `ExtensionType`; DataFusion's `DFExtensionType` and `ExtensionTypeRegistry` | Pruning works on top-level columns only. A value nested inside a struct cannot be pruned. |

## 2. What the repository uses (from the census, verified)

- **Census §2 table, verified.**
  - IPC streams for canon (`crates/pse-columnar/src/canon/stages.rs:169-170`) and arrow-row there.
  - Nine `TableProvider`s and six `ExecutionPlan`s; `MemTable` appears 35 times.
  - datafusion-tracing at `session/assurance.rs:103`.
  - `instrument_object_store` around a `SharedIo` I/O gate (`pse-engine/src/stores.rs:35`, `:71`; `stores/admission.rs:93`).
  - Parquet is read only for authored documents (`pse-admission/src/authoring_driver/document/data.rs:84`), and every `ArrowWriter` is in test code.
- **The one shared runtime.** Its pool stack is as corrected above. The session state is built once per factory and its assembly once per registry, then cloned per execution (`session/factory.rs:40-235`; `session/execution.rs:115-133`). Session construction is already amortized.
- **DataFusion's file caches are wired but never activated.** `NativeCacheService` wraps the footer, statistics and listing caches (`cache_service/mod.rs:108-361`). The runtime uses `unbound_config()`, which sets every limit to 0. `bind_state*` (`:427-583`) has no caller outside `cache_service/` and its tests.
- **Results** (subagent read; the decisive lines I checked myself):
  - Payloads are uncompressed IPC files written with default `FileWriter` options (`pse-runtime/src/workflow/result_files.rs:46-75`). They sit on the local filesystem behind `ProtectedFile`, not behind object_store (`pse-operations/src/canonical_files.rs:186-200`, `:814-879`).
  - **Every scalar result row gets its own `canonical_result_rows` locator** (`surreal.surql:458-473`), written inside `append` (`:1012-1060`). For `solve_variables` the output key is `"{relation}:{symbol_id}"` and the partition is `step:N` (`result_projection.rs:86-160`). In effect it is a point index on (variable, step).
  - Scalar blocks are written with `coordinate_min/max: None` (`result_projection.rs:1156`), so scalars have no value-range summaries. The block table already has typed `coordinate_min/max {bits, projection}` fields (`surreal.surql:414-434`).
  - **Read path.**
    - `SnapshotProvider` reports every filter as `Unsupported`, has no statistics and exposes one `StreamingTable` partition. Its only callers are tests (`result_snapshot.rs:684-760`).
    - Decoding goes through `FileReader::set_index` behind a mutex and is charged at 4× the file size + 4× the schema + 64 KiB (`:312-420`).
    - SurrealDB refuses scalar predicates (`canonical_results.rs:633-637`). The runtime then decodes, checks each locator against its row, applies `scalar_predicate` and gathers with `take` (`connected_results.rs:1086-1140`).
  - **`scalar_predicate`** (`result_projection.rs:209-280`) has these semantics:
    - Bounded selections and `missing=false` exclude nulls, NaN and ±∞.
    - Comparisons run on a copy in which -0.0 becomes 0.0.
    - Non-finite bounds are refused.
    - Original bits are gathered untouched.

    The fields it may filter are hand-listed per relation (`:37-80`).
- **Memory charging.** There are 145 `MemoryConsumer::new` sites in `pse-runtime/src`. They charge futures (`connected_results.rs:42-57`), SDK metadata responses, locator scratch space and durable records (`durable.rs:615-641`).
  - `pse-columnar/src/owned_buffer.rs` (1762 lines) hand-implements "the last reader keeps the reservation" (ADR-0055).
  - `allocation_extent.rs:27` works around IPC slices reporting the whole file's capacity.
  - `rg 'fn heap_bytes|fn retained_bytes|fn heap_size'` over `crates` gives 934 lines.
- **The math product cache** uses DataFusion's `DefaultCache` as a general weighted LRU (`pse-runtime/src/math.rs:27`, `:373`). This is the "compiled-program cache" DA-030 names; it holds Symbolica products, not query plans. `pse-rules` rebuilds its invariant `LogicalPlan` on every run (`invariants/program.rs:34-235`).

## 3. Unused or under-used capabilities, mapped to findings

### C1. Prune record batches by statistics inside the snapshot SurrealDB selects. Recommended; it fully resolves DA-026.

**How it fits together:**
1. **Write.**
   - Inside each IPC record batch, sort rows by key (`run_id, step, symbol_id`) with `lexsort_to_indices` + `take`, both already in use.
   - Per batch, compute `min`/`max` for each key column (FixedSizeBinary(16) and Int64), and for each declared Float64 field compute `arrow::compute::{min, max}` (totalOrder) plus `null_count`.
   - Commit those statistics in the existing guarded `append`, on the batch/block rows (`canonical_result_batches`/`canonical_result_blocks`), **instead of one locator per row**.
   - A row's coordinate becomes (batch, offset), from `file_batch_index` and the block `start`.
2. **Snapshot.** SurrealDB still returns the committed, fenced set of batches along with their statistics, so it remains the authority on which batches exist (P1, P3).
3. **Provider.** `SnapshotProvider` becomes a custom multi-partition `ExecutionPlan`:
   - Batch groups become partitions, each decoded with `FileDecoder::with_projection` from that block's byte range of the `ProtectedFile`.
   - `supports_filters_pushdown` returns `Inexact`.
   - `scan_with_args` runs `PruningPredicate::prune` over `PrunableStatistics` built from SurrealDB's statistics.
   - `partition_statistics` returns the same statistics as `Exact`.
   - It also implements `with_fetch` and declares its output ordering.
4. **Canonical predicate.** `output_results(field, minimum, maximum, missing)` lowers to a plain `Expr` that is bit-for-bit equivalent to `scalar_predicate` and needs no copy:
   - finite: `col > -inf AND col < +inf` — under totalOrder this excludes ±∞, NaNs of either sign, and nulls;
   - `col >= (m==0 ? -0.0 : m)`;
   - `col <= (M==0 ? +0.0 : M)`;
   - `missing=true` ⇒ `col IS NULL`.

   Why it is exact: for finite non-zero bounds, totalOrder and IEEE agree on finite values, and the signed zero literal admits both zeros. `PruningPredicate` understands all of these forms.
5. **Soundness for any SQL.** The statistics come from the same totalOrder `min`/`max` that DataFusion evaluates with, NaN included. So pruning is sound for arbitrary DataFusion queries, not just the canonical one.

**What it resolves:**
- All of DA-026: per-row transactional locators, the bespoke predicate engine, and the unpruned serial scan.
- DA-032's two result-side sites: the `indexed_scalar`/`check_output_field` lists and `canonical_execution.rs:2345-2356`. Every declared scientific Float64 column becomes prunable.
- DA-025's write volume: `append` metadata scales with batches, not rows.

**What it deletes:**
- `canonical_result_rows` and its index;
- `RowLocator` construction, verification and windowing;
- the 18724-row extent caps;
- replay comparison over locators;
- `scalar_predicate`;
- the SurrealDB refusal branch;
- the mutex-held decoder.

**Fit and limits:**
- Exact bits are unchanged, because the IPC payload is untouched.
- Small results remain a single file.
- Live samples are unchanged: still one sealed file per batch, with its statistics in the same acknowledged append.
- Pruning granularity is the record batch, so batch row count becomes a tunable (probe 4).

**Reverse check:**
- SurrealDB still decides membership, commit and fencing.
- Lane parallelism improves, because guarded transactions get smaller.
- Daemonless operation and DP-19 truthfulness are unaffected, and the exact-bit and signed-zero strength is kept.
- General SQL keeps DataFusion's totalOrder semantics (see §4.2).

### C2. Library-measured accounting for Arrow-held memory. Recommended; covers DA-022 for Arrow buffers only.

**Mechanism:**
1. Enable `datafusion-execution/arrow_buffer_pool`.
2. Admit with a fallible `try_grow` lease, sized exactly from the block's byte length.
3. Decode zero-copy, then call `RecordBatch::claim(&ArrowMemoryPool::new(pool, consumer))`.
4. Release the lease.

Each `Bytes` is then counted once, and its reservation dies with the last reference — ADR-0055's rule, owned by the library.

**What it replaces:**
- the 4× decode formula;
- the `allocation_extent.rs` workaround;
- most of `owned_buffer.rs`'s extent walking and its 256-bytes-per-wrapper model;
- the `get_array_memory_size()*8 + …` compaction charge;
- the charges for selected batches and `take`.

It also gives DA-022 the measured comparison it lacks, for Arrow data.

**Limits:**
- Admission still has to come before allocation, because `claim` cannot refuse.
- Each claimed `Bytes` registers a consumer in `TrackConsumersPool` (probe 7).
- The feature adds a mutex to every `Bytes` across the process (ADR-0122 unification); the cost is unmeasured.
- It does not cover Salsa or compiler structures, futures, SurrealDB metadata or native libraries: **remedy open** in this family. `DFHeapSize` helps only partially, since it still needs hand-written impls. A derive crate is a new library, so it is recorded as a lead only.

**Reverse check:** prepaid admission and the cancellation drain are kept.

### C3. Route result I/O through library caches and tracing. A clear win only together with C1.

- **Footer cache.** Store IPC footers as `FileMetadata` entries in DataFusion's `FileMetadataCache`, keyed by the immutable, content-addressed file instance. This replaces the per-decoder `CachedFile` and its 2× formula, and gives the dead cache wiring a production use. Without C1, that wiring is DA-024-type dead code and should be deleted.
- **Read-only object store.** Serve result reads through a read-only `ObjectStore` over `ProtectedFile`, registered in `ObservedStores`. Reads then fall under `instrument_object_store` and the shared I/O gate; today result I/O bypasses both. The per-read identity checks would move inside the store.
- **Rejected.**
  - `ListingTable`/`ListFilesCache` for results: a directory listing would act as a second catalog of payloads (against P1 and P3).
  - Using `ArrowSource` as is: it does not prune and it bypasses the preflight checks and file protections.

### C4. Parquet result payloads. Remedy open; not recommended as the default.

**What it would add:** page-level min/max on sorted keys, a bloom filter on `symbol_id`, `sorting_columns`, `RowNumber`, selections from SurrealDB through `ParquetAccessPlan`, and footer caching.

**Hazards:**
1. **NaN.** Float min/max skip NaN yet are reported as `Exact`. Under totalOrder evaluation, pruning on `x > 5` could drop NaN rows, and `AggregateStatistics` could answer `max(x)` without the NaN.
2. **±0.** The ±0 widening is conservative for pruning but not exact. A `min()` answered from statistics could return -0.0 bits that are not in the data.

**Mitigation:** turn off float column statistics (`set_column_statistics_enabled(col, None)`), keep page index and bloom filters on the key columns, and supply totalOrder-exact float statistics per file from SurrealDB through `PartitionedFile::with_statistics`.

**Remaining costs:** decoding copies, overhead on small results, and a change of format for live samples. C1 meets the DA-026 obligations without these costs, so Parquet waits for probes 1–3 and a measured need.

### C5. Result datasets across study points. Follows from C1; the coordinator should judge whether it is in scope.

Nothing today unions results across points. The C1 provider can take several snapshots pinned by SurrealDB, and `run_id` statistics then prune by point. It adds no new storage, and SurrealDB still chooses the point set. I count it as a minor, clear win only if a study-level reader is already part of the existing surface.

### C6. Extension types for DA-005 and DA-011. Limited fit.

- **DA-005.**
  - The identity kind already rides in `pse.semantic.identity` metadata on `pse.semantic_id` columns (`pse-schema/src/model/field_facets.rs:60-82`), so typed Python identities are a codec task.
  - Folding values into `pse.quantity_value` (`extension.rs:101-107`) would lose C1's value pruning, which works on top-level columns only. That conflicts with C1, so the unit and quantity columns should become required in the registry instead.
  - A unit resolver could be a DataFusion join, if units exist as relations (unresolved).
- **DA-011** is about JSON documents; nothing in this family resolves it.

## 4. Improvements to existing behaviour that no DA finding names

1. **Aggregates from statistics.** With C1's `Exact` statistics, `count`, `min` and `max` can be answered without decoding, and `AggregateFields` (`physical_fields.rs:80`) already preserves metadata around that rule. The answers are exact under totalOrder. *Proposed.*
2. **The totalOrder trap in SQL.** `-0.0 = 0.0` is false and `NaN > x` is true over results. Document this, or expose the canonical predicate as a table function that lowers through C1's rewrite. Clear win.
3. **Invariant plans.** Build the invariant `LogicalPlan` once per registry assembly. `ExecutionPlan::reset_state` would allow reusing the physical plan when providers are unchanged. *Proposed.*
4. **Limit and ordering pushdown** on result scans, so that `latest_results` with a `LIMIT` stops early.
5. **Preflight overlap.** The hand footer and FlatBuffer walk (`result_files.rs:198-382`) partly duplicates `FileReaderBuilder`'s verification and `with_require_alignment(true)`. Refusing compression and variadic buffers still needs project code. Unresolved; low priority.

## 5. Open questions and probes

| # | Probe | Expected | What it settles |
|---|---|---|---|
| 1 | Round-trip Float64 values {±NaN with payloads, signalling NaN, ±0, subnormals, ±∞, null} through `ArrowWriter` → reader under PLAIN, dictionary and BYTE_STREAM_SPLIT; compare `to_bits` | identical | whether Parquet can keep exact bits (C4) |
| 2 | `SELECT CAST('NaN' AS DOUBLE) > 5, -0.0 = 0.0, max(x)` | true, false, NaN | the totalOrder premise behind C1 and §4.2 |
| 3 | Row group {NaN, 1, 2}: `count(*) WHERE x > 5` and `max(x)` with `parquet.pruning` on and off | discrepancy if hazard 1 is real | C4 mitigation |
| 4 | Pruning grid: `PrunableStatistics` from Arrow `min`/`max`/`null_count` per batch, across value classes × predicates × signed-zero bounds. Assert the rewritten mask equals `scalar_predicate` bit for bit, and that pruned batches contain no matches. Sweep batch sizes. | equal; sound | C1 correctness and batch size |
| 5 | Existing probe `canonical_joined_publication_progress_and_ordinary_cost_probe` (`workflow/durable.rs:4309`): count locator inserts and appends before and after C1 | locators ≈ variables × steps | DA-026 magnitude |
| 6 | `FileDecoder` + `claim` into `TrackingMemoryPool`, compared with the 4× formula | claimed ≈ file bytes | C2 benefit and hidden copies |
| 7 | Enable `arrow-buffer/pool`; time canon, admission, decode and many-consumer tracking | small cost (unknown) | whether C2 is a clear win |
| 8 | `PruningPredicate` with `symbol_id = lit` over `FixedSizeBinary(16)` statistics | prunes | key point lookups in C1 |

**Still unresolved without probes:** whether payloads have a production collector (DA-025), whether units are relations (C6), and whether per-`Bytes` consumers interact badly with `ReleaseNotifyingPool`.

**Search scope behind the absence claims:**
- NaN handling in pruning: case-insensitive rg `\bnan\b` over `datafusion-pruning`, `datafusion-datasource-parquet` and parquet's `arrow_reader/statistics.rs` at the pins.
- `claim`/`ArrowMemoryPool` unused: `pse-runtime/src`, `pse-columnar/src`, `pse-engine/src`.
- Caches inactive: `crates`, `tests`, `benches`, `xtask`.

I wrote no files. The two read-only mapping subagents returned text only. The census is at `/tmp/claude-1000/-home-paul-pse-arrow/15c6fe71-a184-4329-a417-2fc2e56d8c52/scratchpad/review2/census.md`.