> Supporting evidence for [the efficiency and solver target review](../../reviews/design_review_efficiency-and-solver-target_2026-10-12.md). Inquiry R3 (library-research), Part A: observation, results and analytics. Baseline: `main` at `6c368c88a`. This contribution is preserved as returned; its local labels are not finding identifiers, and the principal review owns the judgment.

# R3: observation, results and analytics (Part A)

Library-research inquiry R3 for the consolidated efficiency and solver target review. Bounded
supporting contribution: no verdicts, no finding identifiers, no decision.

- **Baseline:** `main` at `6c368c88a`, clean tree, 2026-10-12. Static reading only.
- **Pinned versions (Cargo.lock):** arrow/arrow-* 59.3.0, parquet 59.3.0, datafusion 55.1.0
  (default features), object_store 0.13.2, datafusion-tracing and instrumented-object-store 55.0.0,
  tracing 0.1.44, tracing-subscriber 0.3.23 (lock shows `env-filter`, `json`, `fmt`, `registry`
  dependencies: matchers, regex-automata, serde_json, tracing-serde, sharded-slab), statrs 0.18.0,
  pyo3-arrow 0.19.0.
- **Not in the lock:** opentelemetry*, tracing-opentelemetry, tracing-appender, datafusion-proto,
  datafusion-substrait, datafusion-ffi, arrow-flight, parquet-variant, deltalake (search: `grep
  '^name = "<crate>"' Cargo.lock`).
- **Sources:** `datafusion` and `datafusion-tracing` skills (indexes, reviewed briefs, probe
  receipts; both pin these exact releases); pinned upstream source under `~/.cargo/registry/src`;
  upstream changelogs (arrow-rs, object_store, statrs) fetched 2026-10-12; crates.io max versions
  fetched 2026-10-12.
- **Prior evidence reused (not redone):** efficiency inquiry F
  (`docs/design_review/evidence/efficiency-alignment-codebase-2026-10-11/relational-and-columnar-layer.md`)
  and inquiry E (`.../publication-persistence-and-results.md`), both at `b01a45d71`; solver A2
  (`docs/design_review/evidence/solver-methods-and-diagnostics-2026-10-12/observation-and-persistence-map.md`)
  and the principal analysis (`.../diagnostics-and-measurement.md`), both at `ff5a4aa2f` + Plan 36
  tree. Deltas since are recorded in §3.
- **Labels:** library facts are *Interface-checked* (pinned source, rustdoc index or brief) unless
  marked *Tested* by a named existing skill probe. Everything under "serves", §4 and §5 is
  interpretation; benefits are *Proposed*.

---

## 1. Capability surface at the pinned versions (neutral survey)

### 1.1 Arrow 59.3

**A1. IPC stream and file formats** (`arrow_ipc::{reader,writer}`).
- *What.* Stream format: schema message, then dictionary and record-batch messages, optional
  end-of-stream marker; readable sequentially (`StreamReader`, push-style `StreamDecoder`).
  File format ("Feather v2"): same messages plus a footer holding block offsets, enabling random
  access to any batch (`FileReader`, `FileReaderBuilder`, `FileDecoder` over caller-owned
  buffers). Writers: `StreamWriter`, `FileWriter`, `StreamEncoder`, `IpcDataGenerator`,
  `IpcWriteOptions` (alignment, metadata version, optional lz4/zstd buffer compression,
  `DictionaryHandling`).
- *Contract.* A stream is self-describing and incrementally decodable; a file is unreadable
  until its footer is written (writer `finish`). No atomicity, durability or checksums: an
  interrupted writer leaves a truncated, detectable-but-unrecoverable artefact (stream: missing
  end marker; file: no footer). Readers validate decoded arrays by default; `FileDecoder`,
  `RecordBatchDecoder` and `StreamDecoder` offer `with_require_alignment` and an `unsafe`
  `with_skip_validation` documented only for trusted input "written by the same process"
  (`arrow-ipc-59.3.0/src/reader.rs:1066-1076`). Single-threaded per reader; batches are `Send`.
- *Compression.* lz4/zstd are present in the build through DataFusion feature unification
  (`arrow-ipc` deps in Cargo.lock), although the workspace declares `arrow-ipc` without them.
- *Limits.* No statistics, no index, no predicate pushdown; random access only by batch ordinal
  (file format). Schema metadata is a `HashMap<String,String>` at this pin.
- *Serves.* Exact transfer and short-lived exchange of batches (process boundary, durable blob
  payloads, spill files), and a queryable file format through `datafusion-datasource-arrow`.

**A2. Zero-copy decode.** `FileDecoder`/`StreamDecoder` over `arrow_buffer::Buffer` (which can
wrap `bytes::Bytes` or an mmap) produce arrays that slice the input buffer when alignment holds
(`with_require_alignment(false)` copies to realign instead of failing). Validation still walks
offsets/UTF-8 unless skipped. *Serves:* reopening stored blocks without a payload copy.

**A3. C Data and C Stream interfaces** (`arrow::ffi`, `arrow::ffi_stream`, workspace feature
`ffi`). `FFI_ArrowArray`/`FFI_ArrowSchema` move one array; `FFI_ArrowArrayStream` and
`ArrowArrayStreamReader` move a pull stream with producer-owned release callbacks. Ownership
crosses by release callback; the consumer must call release; errors cross as C strings.
pyo3-arrow 0.19 wraps these as PyCapsule-protocol objects (`PyRecordBatchReader`,
`__arrow_c_stream__`), so pyarrow, polars, DuckDB or datafusion-python consume them zero-copy.
*Serves:* the Python results boundary and any in-process foreign consumer.

**A4. Compute kernels.** arrow-arith (checked/wrapping arithmetic, Kleene boolean, `sum`,
`min`/`max`), arrow-ord (`cmp`, `sort_to_indices`, `lexsort`, `rank`, `partition`), arrow-select
(`filter` with reusable `FilterPredicate`, `take`, `concat`, `interleave`, `zip`,
`BatchCoalescer`), arrow-cast (`CastOptions` safe/strict), arrow-string. All are pure functions
on immutable arrays; panics are avoided on `try_` paths (some `try_` panics fixed only in 60).
*Serves:* per-batch selection, ordering and reshaping of results without a query planner.

**A5. `RowConverter`** (arrow-row). Encodes multi-column keys to memcmp-comparable bytes
(`Rows`, `OwnedRow`) and decodes back; byte format is documented as unstable across releases
(skill brief `arrow.row-keys`). *Serves:* in-memory composite-key sort/dedup/merge; not a
durable key format.

**A6. Extension types** (`canonical_extension_types` feature on). `ExtensionType` trait,
`Field::try_extension_type` (the panicking `extension_type` is banned locally), canonical
`FixedShapeTensor`, `VariableShapeTensor`, `Json`, `Uuid`, `Opaque`, `Bool8`. Metadata travels
in field metadata through IPC, Parquet (`ARROW:schema` key) and C Data; engines that do not
know an extension see the storage type. *Serves:* carrying physical-quantity typing (unit,
dimension, convention) on scientific columns end to end.

**A7. Memory accounting.** arrow-buffer allocation is untracked by default; DataFusion's
`ArrowMemoryPool` (datafusion-execution 55.1) bridges arrow's claim API to a DataFusion
`MemoryPool`. *Serves:* charging decoded result buffers to the same budget as query operators.

### 1.2 Parquet 59.3 (features `arrow`, `async`, `object_store`; codecs zstd, lz4, snappy,
brotli, gzip unified into the build via DataFusion)

**P1. Writing.** `ArrowWriter` (sync), `AsyncArrowWriter` + `ParquetObjectWriter` (object_store
multipart), low-level `ArrowColumnWriter`/`ArrowRowGroupWriterFactory` for parallel column
encoding. `WriterProperties`: row-group size, page size/row limits, per-column encoding and
compression, dictionary, `set_statistics_enabled` (none/chunk/page), `set_bloom_filter_enabled`
(+ fpp/ndv), column/offset index (page index) on by default with page statistics,
`set_sorting_columns`, `set_key_value_metadata`, `set_content_defined_chunking`.
- *Contract.* Columnar, compressed, immutable once closed. The footer is written at `close()`;
  an interrupted file has no footer and is unreadable (no partial recovery, no checksum of data
  pages beyond optional page CRC). No append to a closed file; "append" means a new file.
  Float min/max statistics skip NaN (`column/writer/mod.rs::is_nan`, l.1706–1730); ±0 handling
  per spec. Not thread-safe per writer; columns can be encoded in parallel via the low-level API.
- *Serves:* long-lived, bulk, analytics-oriented storage of results, trajectories and
  observation histories with good compression for doubles.

**P2. Reading and selection.** `ParquetRecordBatchReaderBuilder` (sync),
`ParquetRecordBatchStreamBuilder` + `ParquetObjectReader` (async, ranged reads), the
sans-IO `ParquetPushDecoder` (caller supplies bytes; row-group-local `RowSelection` only from
60.0). `ProjectionMask` (leaf-column projection), `with_row_groups`, `RowSelection` (skip/select
runs; coordinates relative to retained groups — skill brief `parquet.selection`, *Tested* by the
skill's fixture), `RowFilter`/`ArrowPredicate` (late materialization: predicate columns decoded
first), `PageIndexPolicy`, `ParquetMetaDataReader`, `StatisticsConverter` (row-group and page
statistics to Arrow arrays for pruning), `ArrowReaderMetrics`.
- *Contract.* Statistics and bloom filters are inexact pruning aids (false positives only);
  page-index skipping depends on what the writer emitted. Output order is file order.
- *Serves:* selective reopen of one variable, one run, one time window or one study slice.

**P3. Metadata.** Footer `ParquetMetaData` (row groups, column chunks, statistics, sorting
columns, key-value metadata, `HeapSize`). DataFusion caches it (C9). *Serves:* schema and
provenance stamps, per-file zone maps, cheap counts without data reads.

### 1.3 DataFusion 55.1

**C1. Sources.** `MemTable` (partitioned in-memory batches), `ViewTable` (logical plan),
`StreamingTable` + `PartitionStream` (live producers), `ListingTable` + `ListingTableConfig`
over Parquet/Arrow IPC/CSV/JSON with hive-style partition columns, file-statistics collection
and pruning; custom `TableProvider` with `supports_filters_pushdown` (Exact/Inexact/Unsupported)
and the newer `scan_with_args`/`ScanArgs`/`ScanResult` entry (skill brief `df.pushdown`,
*Tested* by its differential probe; the brief records an upstream doc defect on `Exact`).

**C2. Writes.** `DataFrame::write_parquet`/`write_csv`/`write_json`, SQL `COPY ... TO`,
`INSERT INTO` a `ListingTable` (`InsertOp::Append`; new files), `DataSink`/`DataSinkExec`,
`ParquetSink`. *Contract:* no transactions, no atomic multi-file commit, no overwrite
isolation; partial writes possible on failure (skill brief `df.storage-reuse`, claim
`df.storage-reuse.failure`). A table format (Delta/Iceberg) is what normally adds commit
atomicity; none is in the lock.

**C3. Relational and statistical operators.** Hash/sort-merge/nested-loop joins, `UNION`,
`UNNEST`, sort with TopK and dynamic filters (`TopKDynamicFilters`,
`DynamicFilterPhysicalExpr`), aggregation with partial/final split, window functions with
frames. Registered in the default session (skill `runtime-registry.json`, profile
`df55-sql-parquet-nested-string`): `median`, `percentile_cont`, `approx_percentile_cont`,
`approx_distinct`, `stddev`, `var_samp`, `corr`, `covar_samp`, `regr_slope`/`regr_r2`,
`array_agg`, `string_agg`, `first_value`, `lag`, `ntile`, `cume_dist`, `generate_series`,
among others. *Serves:* study summaries, distributions, success rates, per-method rankings.

**C4. Execution.** Pull-based async streams per partition (`execute_stream`,
`execute_stream_partitioned`), `target_partitions` parallelism on the tokio runtime it is
polled on, cooperative yielding (`coop::CooperativeExec`, optimizer rule `EnsureCooperative`)
so long operators return to the scheduler. Cancellation: dropping the output stream aborts
execution and frees its resources (brief `df.consume`, claim `lifecycle`); not a guarantee for
unrelated background tasks.

**C5. Memory pool and spill.** `MemoryPool` implementations `UnboundedMemoryPool` (default),
`GreedyMemoryPool`, `FairSpillPool`, wrappers `TrackConsumersPool`, `PeakRecordingPool`;
`MemoryConsumer`/`MemoryReservation`; `DiskManager` (OS temp by default,
`DEFAULT_MAX_TEMP_DIRECTORY_SIZE`), `SpillManager` writing Arrow IPC spill files. *Contract:*
a pool budget is not an RSS bound; only participating operators account (brief `df.consume`,
*Tested* only for "1-byte pool sort fails").

**C6. Front ends.** SQL (sqlparser 0.62), `DataFrame`, `LogicalPlanBuilder`, `Expr` helpers;
placeholders with `ParamValues`/`with_param_values` (prepared plans); analyzer → optimizer →
physical planner → physical optimizer pipeline; `EXPLAIN [ANALYZE]` with operator metrics.

**C7. Statistics and pruning.** `Statistics` with `Precision::{Exact,Inexact,Absent}` per
column; datafusion-pruning evaluates predicates against min/max/null-count for files, row
groups and pages; used for join ordering and limit pushdown.

**C8. Extension points.** Scalar/aggregate/window/table UDFs (`ScalarUDFImpl`,
`AggregateUDFImpl` with `GroupsAccumulator`, `WindowUDFImpl`, `TableFunctionImpl`), analyzer
and optimizer rules, physical optimizer rules, `UserDefinedLogicalNode` + `ExtensionPlanner`,
custom `ExecutionPlan`, `FileFormat`/`FileSource`, `ExprPlanner`, schema adapters
(`datafusion-physical-expr-adapter`).

**C9. Caches.** `CacheManager`: file-metadata cache (Parquet footers, `DEFAULT_METADATA_CACHE_LIMIT`),
list-files cache (TTL), file-statistics cache; `DataFrame::cache` (materialize into memory).
Invalidation is by path/size/mtime or TTL; no notion of an external transactional version.

**C10. Interchange.** Proto/Substrait/FFI/Flight exist upstream (skill brief `df.interchange`)
but are not in the lock.

### 1.4 datafusion-tracing and instrumented-object-store 55.0.0

- `instrument_with_{info,debug,...}_spans!` builds a `PhysicalOptimizerRule` that wraps each
  physical node in a span; options: `record_metrics` (native `MetricsSet` aggregated by name,
  recorded as `datafusion.metrics.*` fields when the recorder drops), `preview_limit` +
  formatter, declared `custom_fields`. `instrument_rules_with_*_spans!` wraps `SessionState`
  for planning phases (`phase_only`, `full`, selected rules). `instrument_object_store` wraps an
  `Arc<dyn ObjectStore>` with request spans (skill briefs `tracing.execution`, `tracing.metrics`,
  `tracing.storage`, `tracing.rules.*`; *Tested* in the skill's locked profile).
- *Contract and limits.* Observes only plans executed after the rule is registered; later rules
  can add uninstrumented nodes; spans close at stream EOF in the 55.1 profile
  (`tracing.lifecycle`); preview cap is not a memory bound; metric vocabulary is DataFusion's.
  Declared compatibility: DataFusion 55.x; OpenTelemetry bridge 0.32/0.31 recorded but absent here.
- *Serves:* query-level telemetry only; it says nothing about native solver work.

### 1.5 object_store 0.13.2

`ObjectStore` trait: `put_opts` (`PutMode::{Overwrite, Create, Update}`), multipart, ranged
`get_opts`, `list`, `copy`/`rename` (+ if-not-exists), `head`. Implementations: `InMemory`,
`LocalFileSystem`, cloud stores behind features. *LocalFileSystem contract*
(`object_store-0.13.2/src/local.rs:336-385, 559-592`): writes stage a temp file then
`rename` (Overwrite) or `hard_link` (Create, no-replace); `PutMode::Update` returns
`NotImplemented`; no `fsync` call exists in `local.rs` (grep `sync_all|fsync`: none), so
visibility is atomic but crash durability is the OS's. *Serves:* a uniform byte-store seam for
Parquet/IPC files, local or remote.

### 1.6 tracing 0.1.44 and tracing-subscriber 0.3.23

- **tracing:** spans (enter/exit, parent, follows-from), events, callsite metadata (target,
  level, module), structured fields declared at the callsite (at most 32 per callsite;
  `Span::record` only for declared names), `#[instrument]`, `Instrument` for futures.
  Dispatch is synchronous into the current subscriber; with no subscriber, callsite interest
  caching makes disabled spans near-free. *Contract:* best-effort observation, no delivery,
  ordering across threads, durability or back-pressure guarantee; a subscriber may drop,
  sample or reorder. Field values are `Debug`/primitive; `valuable` is unstable-cfg.
- **tracing-subscriber:** `Registry` (span storage, per-span `Extensions`), `Layer` callbacks
  (`on_new_span`, `on_record`, `on_enter`, `on_event`, `on_close`), per-layer filters, `EnvFilter`,
  `Targets`, `reload`, `fmt` (incl. `json`). A custom `Layer` can accumulate typed data in span
  extensions and emit a typed record at `on_close`.
- **Absent:** a non-blocking writer (`tracing-appender`, not in lock; its `NonBlocking` is lossy
  by default when its bounded channel fills) and any exporter.
- *Serves:* low-overhead, opt-in instrumentation of phases and nested work; a capture carrier
  that a typed layer can turn into rows. It is not a record of truth.

### 1.7 statrs 0.18.0

Univariate/multivariate distributions with `pdf`/`cdf`/`inverse_cdf` (Normal, StudentsT,
ChiSquared, FisherSnedecor, Gamma, Beta, …), descriptive statistics traits (`Statistics`,
`OrderStatistics`, `Data` quantiles). Pure, deterministic, thread-safe. *Serves:* confidence
intervals and tests in fitting; small summary statistics.

### 1.8 Newer releases (checked 2026-10-12)

| Crate | Pinned | Latest | Material change |
|---|---|---|---|
| datafusion | 55.1.0 | 55.2.0 (2026-10-07) | patch line; no DataFusion release on Arrow 60 yet, so Arrow 60 cannot join the one type universe |
| arrow / parquet | 59.3.0 | 60.0.0 (2026-09-10) | Parquet ALP encoding (lossless float compression); IEEE 754 total-order float statistics (PARQUET-2249); new `PageIndex` struct; row-group distinct counts in `StatisticsConverter`; >32k row groups; push-decoder row-group RowSelection; IPC: "bound IPC message allocations by the bytes actually read", errors instead of panics on malformed IPC/footer; schema metadata becomes an ordered `Metadata` struct (breaking); `force_validate` feature removed from arrow-cast; FFI struct fields made private |
| object_store | 0.13.2 | 0.14.2 | multipart retry/abort fixes, DNS cache; cloud-only material |
| statrs | 0.18.0 | 0.19.1 | `OnlineMoments` (Welford), KS test, KDE (feature), non-panicking `try_inverse_cdf`, safeguarded Newton inverse CDFs |
| datafusion-tracing | 55.0.0 | 55.0.0 | none |
| tracing / tracing-subscriber | 0.1.44 / 0.3.23 | same | none |

The family moves together (AGENTS invariant; family major needs an ADR).

---

## 2. Current use (pointers only)

| Family | Where today (non-test unless noted) |
|---|---|
| IPC stream write (canonical bytes) | `crates/pse-columnar/src/canon/stages.rs` (`StreamWriter`, fixed alignment) |
| IPC result blocks write/decode | `crates/pse-runtime/src/workflow/result_blocks.rs` (`StreamWriter`, `StreamDecoder`); `connected_results.rs:177,1752` |
| Result storage | SurrealDB `canonical_result_blocks` / `canonical_result_batches` (payload bytes ≤ 512 KiB, `RESULT_BLOCK_BYTES`), `canonical_result_block_outputs` and `canonical_result_rows` (row/output index), per-block `coordinate_min/max` (bits + float projection) — `crates/pse-operations/src/generated/surreal.surql:315-402` |
| C stream to Python | `crates/pse-engine/src/arrow_stream.rs`; `crates/pse-py/src/inspection/stream.rs` (pyo3-arrow `PyRecordBatchReader`, `__arrow_c_stream__`) |
| RowConverter | `crates/pse-columnar/src/canon/{stages,preflight}.rs` |
| Kernels (interleave/sort/coalesce) | `pse-schema/src/builder/native.rs`, `pse-runtime/src/workflow/{staged,physical_cache}.rs` |
| Extension types | `pse-relations/src/ext/`, `pse-schema/src/model/extension.rs`, `pse-engine/src/session/registry.rs` |
| Parquet | production read of authored data documents only: `pse-admission/src/authoring_driver/document/data.rs:84`; writers only in tests |
| DataFusion query execution | physical-inventory admission: `pse-admission/src/physical.rs:87-88` (`prepare_many`, `execute_group`) |
| DataFusion expressions for validation | `pse-relations/src/validate/*`, `pse-engine/src/validation.rs` (`NativeValidation(SessionState)`); bare `SessionContext::new().state()` in `pse-runtime/src/workflow/{result_projection.rs:1095,result_export.rs:1113,study_demand.rs:561}`, `pse-admission/.../{owned.rs:463,load.rs:1021}` |
| DataFusion memory pool | `pse-engine/src/resources.rs` (`ReleaseNotifyingPool`→`PeakRecordingPool`→`TrackConsumersPool`→`FairSpillPool`, per prior F §2c); consumers in `pse-runtime` (`EngineResources`, `ReleaseNotifyingPool`) |
| Custom providers / plans | 9 `TableProvider` and 7 `ExecutionPlan` impls in `pse-engine` (e.g. `session/round.rs`, `session/cache.rs`, `session/capture/tables.rs`, `operation/ownership.rs`, `cache_service/inspection.rs`) |
| datafusion-tracing | `pse-engine/src/session/assurance.rs:92-108` (execution INFO spans, `phase_only` rules) |
| instrumented-object-store | `pse-engine/src/stores.rs:35,71` |
| object_store | `pse-engine/src/{stores.rs,stores/admission.rs,cache_service/*}`; `pse-testkit` fault/counting stores |
| tracing | sparse spans/events in `pse-engine`, `pse-compiler`, `pse-math/src/execution.rs`, `pse-runtime` (`math.rs`, `workflow/diagnostics.rs`), `pse-operations/src/canonical.rs`; none in `pse-backend-native` (A2 §9) |
| tracing-subscriber | `pse-testkit/src/capture.rs` only; no production subscriber (search: `set_global_default|with_default|Dispatch::new` over `crates`, `python`) |
| statrs | `pse-runtime/src/workflow/fitting/{covariance,profile}.rs` |
| Observation records | `runtime.solve_metrics`, `solve_strategy_events`, `run_lineage`, `execution_statistics`, `cache_statistics`, durable `__progress` result set (A2 §2) |

---

## 3. Deltas against prior evidence

- **Result index shape changed** (E-01 at `b01a45d71` described per-field `canonical_result_cells`
  with five indexes). The current generated schema has no `canonical_result_cells` table
  (`grep` count 0 in `surreal.surql`); it has `canonical_result_rows` (one row per indexed
  scientific row: run, owner, step, experiment, output, partition, row, offset) and
  `canonical_result_block_outputs`/`canonical_result_blocks` with per-block `coordinate_min`/
  `coordinate_max`. *Interpretation:* these per-block min/max coordinates are a hand-built zone
  map — the same artefact Parquet row-group/page statistics and DataFusion pruning provide.
- **Physical admission moved** from `pse-runtime` to `pse-admission` (`ff5a4aa2f`); the single
  production DataFusion query path (prior F §2a) is unchanged in kind.
- **pse-engine size:** 35.2k lines `.rs` under `crates/pse-engine/src` (tests included);
  Plan 36 wave 1 touched `factory.rs`, `cache_service/*`, `resources.rs`, `assembly.rs`
  (`git diff --stat b01a45d71 6c368c88a`). I did not re-establish whether F-01 (transient
  validation owner) still holds.
- **Stale text found:** `pse-relations/src/validate/row_checks.rs` module doc still says the CHECK
  is "also persisted as Delta CHECK"; Delta is retired (blueprint §20.1) and absent from the lock.
- A2/principal findings on observation content (thin typed evidence, no attempt wall time, no
  cross-point aggregation, no structure-only history key, no production tracing sink) are
  taken as the current baseline; nothing in `b01a45d71..6c368c88a` I inspected changes them.

---

## 4. Responsibility questions

### 4.1 Best home for results, attempt/observation records and analytics

Four homes compared on library-given properties (facts) and fit (interpretation).

| Property | (a) Arrow IPC bytes in the store (today) | (b) Parquet files + DataFusion | (c) tracing capture | (d) Store-native records |
|---|---|---|---|---|
| Atomic visibility with lifecycle | Yes: same transaction family as run/attempt records (§20.2) | No library commit; file rename/link is atomic per file only; multi-file sets need a manifest owned elsewhere | None | Yes |
| Crash durability | Store's (RocksDB WAL) | OS's (object_store local has no fsync) unless the project fsyncs | None (best-effort, lossy writers) | Store's |
| Exactness of doubles | Exact (IPC) | Exact (plain/dictionary/byte-stream-split; ALP in 60 is lossless) | Lossy unless the layer formats bits | Store numeric type; exact only if stored as bytes/bits (today's `coordinate_*.bits`) |
| Selective read | By block key and hand index (`canonical_result_rows`, coordinate min/max) | Projection, row-group/page pruning, bloom, `RowFilter`, `RowSelection` — library-owned | n/a | Indexed queries on scalar fields; poor for bulk columns |
| Set-oriented analytics across runs/points | Not available without decoding blocks and writing kernels | Native: aggregates, windows, percentiles, joins, spill | n/a | SurrealQL aggregates over records; volume grows with rows |
| Compression of float columns | None (compression refused in canonical blocks, §20.1) | zstd/lz4/snappy + encodings | n/a | none |
| Retention / reclaim | Store-owned deletes; reclaimed bytes are engine compaction (I4) | File deletion = space return; no MVCC | n/a | Store-owned |
| Concurrency of writers | Store transactions, guards | One writer per file; many files concurrently; readers never see partial files if written-then-renamed | lock-free per subscriber | Store transactions |
| Python access | C stream via pyo3-arrow (exists) | Any Arrow/Parquet reader, or C stream from DataFusion | none | via the Rust API |
| Upgrade surface | arrow-ipc format is stable across versions | Parquet format stable; DataFusion API churns each major | stable | SurrealDB schema/codegen |

*Interpretation.* The table splits the responsibility rather than picking one home:
- Visibility/authority facts (that a result set exists, is complete, is sealed, belongs to an
  attempt) need (a)/(d)'s transactional store; Parquet and tracing offer none of that.
- Bulk scientific columns and accumulating observation histories are the workload Parquet +
  DataFusion are built for (compression, pruning, aggregation, spill), and which the current
  bespoke row index and coordinate zone map partially re-implement.
- tracing is a capture mechanism, not a home: it has no durability or completeness contract.

### 4.2 Should a query engine own admission and validation?

**What DataFusion actually contributes in `pse-engine` today** (facts; reachability from prior F,
re-checked for imports at `6c368c88a`):
1. *Expression machinery for validation.* Registry-declared field predicates and row CHECKs are
   SQL/`Expr`, coerced, simplified and compiled to `PhysicalExpr` against a captured
   `SessionState` (`NativeValidation`), then evaluated per batch. This uses the optimizer's
   expression simplifier and type coercion, not the logical optimizer or physical planner.
2. *A planning pipeline with project-owned checks:* `session/admission.rs` runs "active checks at
   every platform logical-plan boundary, including subqueries"; `observation.rs` records actual
   analyzer/optimizer output; `contract.rs`, `obligation.rs`, `freshness.rs`, `cache.rs` add
   custom `ExecutionPlan`s for contracts, obligations, invocation-local caching and spill. In
   production these run on one path: physical-inventory admission (about 13 scan+sort roots,
   prior F §2a).
3. *The memory pool* as the process-wide accounting authority for platform reservations.
4. *Unreferenced or test-only in production* (prior F §2d/F-04): SQL entry points, DML/mutation,
   `schema_transform`, inspection UDTFs, file-cache envelopes, invariant programs.

**Neutral comparison.**
- Typed Rust validation (+ an incremental owner such as Salsa): validation of authored and
  scientific relations is per-field predicates, keys and references over small-to-medium
  batches. Arrow kernels (`cmp`, `filter`, `is_null`, arithmetic) evaluate predicates
  vectorized without a planner. What is lost: SQL as the declaration language for CHECKs
  (the registry would need a typed predicate form or keep DataFusion's `Expr` without a session),
  and engine-identical semantics with any SQL consumer.
- DataFusion `Expr` without the session/planner pipeline: `create_physical_expr` over a
  `DFSchema` plus `ExecutionProps` compiles an `Expr` once; reuse is by holding the
  `Arc<dyn PhysicalExpr>` (skill brief `df.expressions`). This keeps SQL-shaped declarations
  and drops planning cost and session identity questions (prior F-01).
- DataFusion as planner/executor: earns its keep where work is set-oriented (joins across
  relations, aggregation over many runs, spill). The admission workload (one inventory,
  ~13 scans) is small; the analytics workload (studies, history) has no relational route today.

*Interpretation.* The library's planner/optimizer contributes most on the workload that is not
using it (results/observation analytics) and least on the workload that is (admission and
validation), where expression compilation alone suffices.

### 4.3 One observation record for telemetry, measurement, history and diagnosis?

Library facts that bear on it:
- Arrow/Parquet can hold a wide, typed, nullable record with nested lists (per-iteration
  trajectory as `List<Struct>`), dense unions or per-adapter struct columns, and extension-typed
  quantities; schema evolution across files is handled by DataFusion schema adapters
  (`datafusion-physical-expr-adapter`) when columns are added.
- DataFusion aggregates/windows (C3) compute success rates, PAR-k-style penalized means,
  percentiles, per-method distributions and censoring flags as plain SQL/DataFrame over such
  records; `statrs` (0.19 adds `OnlineMoments`) supplies distribution functions and online
  moments in-process.
- tracing span trees naturally carry the *phase* hierarchy (admission → preparation → routing →
  rung → native → callback → assessment → publication) with timing; a custom `Layer` can fold a
  span tree into one typed row at `on_close`. tracing cannot carry the solver's typed statistics
  or guarantee completeness.

*Interpretation.* One record schema can serve all four consumers if it is a typed Arrow schema
with tiers (T0 common fields + per-adapter struct, T1 bounded trajectory list, phase timings);
the distinction between "decision-readable" and "diagnostic-only" then becomes a column
property, not a separate map (consistent with the principal's §1 rule-impact lead). Efficiency
telemetry (phase wall time, cache hits, admission wait) and solver measurement share keys
(attempt, rung, identities) and differ only in which columns are populated. Layer-2 switching
must read in-memory typed evidence during the solve; the persisted record is its copy, not its
source.

---

## 5. Library-shaped allocations

### S1 — "Parquet + DataFusion as the results and observation plane"

*Shape.* The store (SurrealDB) keeps authority: run/attempt/result-set existence, seal,
manifest (file paths + digests + row counts), protection and retention decisions. Scientific
result columns, trajectories and observation records are written per attempt as immutable
Parquet files (statistics + page index on; sorted by output/partition/row; KV metadata stamps
identities) through `object_store::LocalFileSystem` (write → fsync by project → link/rename),
then sealed by reference. A `ListingTable` (or a thin `TableProvider` over the manifest) exposes
results, observations and study tables; Python receives DataFusion streams over the C stream
interface; study and history analytics are SQL/DataFrame queries.

- *Library discharges:* columnar encoding and compression; per-file/row-group/page zone maps and
  pruning; projection and late materialization; selective reopen by variable, run, step range;
  aggregation, windows, percentiles, joins across runs; bounded-memory execution with spill;
  footer metadata caching; schema evolution across files; Python zero-copy hand-off.
- *Remains the project's:* atomic multi-file publication (manifest in the store; files are
  orphans until sealed — a reclaim sweep is still needed); fsync discipline; exact-bits
  verification (digest per file); the registry → Arrow schema contract and extension types;
  retention policy and deletion; protection of files under readers (no MVCC: a reader must hold
  a manifest protection while files are open); identity keys for history.
- *Lost or weakened:* single-transaction visibility of payload and metadata (now two systems:
  store + filesystem, with an orphan window); store-side backup/export as one unit; the
  uncompressed/no-dictionary canonical byte form (if canonical IPC bytes are also an identity
  input, Parquet bytes cannot serve as identity — identity must stay on canonical IPC or on
  decoded values); remote-store deployment would need object_store cloud features and their
  failure semantics.
- *Costs:* integration of a manifest-to-provider seam; file lifecycle tests (crash between write
  and seal, reader-held protection, reclaim); DataFusion API churn on each major (already carried);
  Parquet format itself is stable.
- *Makes unnecessary (responsibility level):* hand-built result row index and block coordinate
  zone maps; 512 KiB block chunking and per-block RPC paging for bulk reads; bespoke
  per-relation/per-range readers; bespoke cross-point summaries; store growth from bulk payloads
  (I4's reclaimed-bytes concern moves to file deletion, which returns space directly).

### S2 — "tracing as the observation carrier" (feeding S1 or the store)

*Shape.* Every phase and rung is a span with declared typed fields (identities, mechanism,
attempt); native adapters fill a typed statistics struct in the span's extensions; a project
`Layer` (on tracing-subscriber's `Registry`) folds closed attempt spans into typed Arrow rows
(phase timings from span open/close, nested children) and hands batches to a writer (S1 file or
store append). The same spans feed `fmt`/`json` for live diagnostics and, optionally, an
OpenTelemetry exporter for external profiling. datafusion-tracing and instrumented-object-store
already plug into this tree.

- *Library discharges:* zero-cost-when-disabled instrumentation; hierarchical timing and
  parent/child attribution across async and threads (`Instrument`); filtering by target/level
  per layer; fan-out to several sinks; DataFusion and object_store telemetry for free.
- *Remains the project's:* the typed record schema; the folding `Layer`; a non-lossy,
  bounded hand-off to storage (tracing-appender would be lossy by default; a project channel
  must define drop accounting — consistent with the "never back-pressure a solve" tier rule);
  propagation into native threads/FFI callbacks (spans must be entered explicitly on foreign
  threads); persistence and its durability; tying records to attempt identity.
- *Lost or weakened:* completeness — dispatch is best-effort and filter-dependent, so a record
  that decisions or history rely on cannot come only from tracing; field limit of 32 per callsite
  and `Debug`-shaped values push rich statistics into extensions rather than fields; a global
  or scoped dispatcher must be owned by the runtime (none exists today).
- *Costs:* instrumenting `pse-backend-native` (no tracing today); one Layer; tests for
  partiality/drop evidence; negligible upgrade cost (stable 0.1/0.3 lines).
- *Makes unnecessary:* ad hoc timing (`Event.elapsed`, `callback.*.seconds` string metrics,
  `Instant` bookkeeping per adapter); separate process-level sidecars for phase timing;
  string-keyed metric maps as the transport for phase measurements.

### S3 — "store-native authority + Arrow-native analytics over the existing blocks"
(minimal-change library maximization of today's shape)

*Shape.* Keep IPC blocks in the store as the scientific payload and authority. Add a DataFusion
`TableProvider` over the canonical result tables whose `supports_filters_pushdown` classifies
run/relation/partition/row-range predicates as Inexact against the existing block coordinates
and row index, with `scan_with_args` limit hints; decode blocks with `StreamDecoder` into
`ArrowMemoryPool`-charged buffers; let DataFusion own aggregation, windows and spill for study
and history analytics; small typed observation records (attempt summary, ladder record) are
store records, bulk trajectories stay IPC blocks.

- *Library discharges:* relational analytics, spill, statistics-based pruning (with the
  project's statistics), Python streaming via DataFusion streams.
- *Remains the project's:* the provider (pushdown soundness: Inexact must not lose rows),
  block statistics production, paging and protection through the store RPC, decode admission.
- *Lost or weakened:* nothing relative to today; compression and native page-level pruning are
  not gained; RPC-per-block read cost remains.
- *Costs:* one provider; differential tests (Unsupported vs Inexact) per the skill's
  `df.pushdown` brief; low upgrade cost beyond today's.
- *Makes unnecessary:* bespoke cross-run/cross-point readers and summary code; gives the
  documented inspection capability (§14.2) a real route.

### Cross-cutting notes (interpretation)

- S2 composes with S1 or S3; it is a carrier, not a home.
- Under S1 or S3, much of `pse-engine`'s project-owned planning machinery (admission checks at
  every plan boundary, obligation/contract/freshness plans, invocation caches) would serve the
  analytics plane instead of admission; whether it is needed there is an open question for the
  coordinator, not answered here.
- Arrow 60's ALP encoding and total-order float statistics strengthen S1 specifically
  (compression and sound NaN/±0 pruning of doubles) once DataFusion moves to Arrow 60.

---

## 6. Uncertainties and unfinished

- No probe, build or measurement was run. Compression ratios, pruning effectiveness, decode
  cost and DataFusion overhead on these workloads are unmeasured; skill probes cover only their
  named fixtures.
- Whether canonical IPC bytes are an identity input for results (vs. decoded values) decides
  how much of S1's "lost identity" applies; I did not trace §5.3 hashing for result payloads.
- F-01 (transient validation owner) status after Plan 36 wave 1 not re-established.
- Internal reachability of `pse-engine` `round`, `traversal`, `cache/*`, `operation/*` from the
  physical-inventory path not traced (same limit as prior F).
- tracing behaviour on native solver threads entering spans from FFI callbacks is
  source-reasoned only.
- Absence claims: production tracing subscriber (search `set_global_default|with_default|
  Dispatch::new` over `crates`, `python`; only `pse-testkit`); production Parquet writers
  (search `ArrowWriter|AsyncArrowWriter` over `pse-admission`, `pse-runtime`: tests only).
