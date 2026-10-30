# Result data paths

This page covers how retained scientific results are projected, sealed, selected and read in
`pse-runtime/src/workflow/`, `pse-relations` and `pse-columnar`. Contracts are owned by
[§20](../../authoritative_design/sections/identity-and-publication.md#section-20) and
[§21](../../authoritative_design/sections/workflows-and-results.md#section-21). The baseline is
on the [entry page](README.md).

**System picture.** Typed relation rows are projected into Arrow batches, sorted, and sealed as
uncompressed Arrow IPC files with recorded per-batch statistics. Their metadata is appended to
the canonical store. Reads build a `ResultReader` over a `ResultSnapshot`. Library use is
already strong:
- A scientific predicate is lowered once to a DataFusion physical expression
  (`ValuePredicate::scalar_expression` is the sole scalar lowering, `ValuePredicate::new` the one validator).
- That expression prunes blocks through `datafusion-pruning::PruningPredicate` against recorded
  statistics and masks residual rows (`result_snapshot.rs`).
- Trajectory range indexing uses `arrow::compute::partition`, and scalar selection builds its mask with `collect_bool`.
- Key validation and sorting use `RowConverter` and `lexsort_to_indices`
  (`pse-relations/src/result_batch.rs`).
- The `SnapshotProvider` reports `Inexact` filter pushdown honestly and drops the limit under a residual.
- Export keeps lineage in Arrow schema metadata through one codec, `ScientificMetadata`.
- IPC admission lives in one owner, `pse-columnar` (`ipc_file`, `ipc_stream`, `ipc_admission`).

Three behaviours are deliberate domain and identity rules, not leverage targets:
- IEEE-bit predicate identity, including the zero-normalization `CASE` in `scalar_expression`.
- Original row coordinates.
- "Retention never merges runs".

The opportunities cluster in the dense trajectory producer ([CBO-075](#cbo-075)), in what the
decoder and DataFusion are told about a read ([CBO-076](#cbo-076), [CBO-077](#cbo-077),
[CBO-078](#cbo-078)), in the local IPC admission layer ([CBO-005](#cbo-005)), and in sidecar JSON
payloads ([CBO-008](#cbo-008)).

**Coverage on this page.** Closely read: `connected_results`, `result_snapshot`, `result_selection`,
`result_files` (to `open_file_reader`), `durable.rs` (lease, streamer, seeds, chunks, table export,
termination), `project_portability` and `portable_payload` (export, archive, import), `modeling/trajectory.rs`
(`project_impl`, `live_projection`), the relation codec (`columnar/row.rs`, `collection.rs`, `codec.rs`),
and `pse-columnar` (`ipc_stream` through its reader, `ipc_file` reader and writer). Read at representative
depth: `cohort_provider` (the provider and `CohortPartition`), `study_workbench` (`aggregate`),
`result_projection`, `result_export` (the cursor poll body was not read), `result_blocks` and the
modeling result emitters. Skimmed or unread: `result_inspection` (head and function list), `access_block`
and the interior of `scan`, `ipc_admission::layout` (skimmed), `physical/decode.rs` (head), and the
tail of durable settlement. Nothing was run and no cost is measured.

<a id="cbo-075"></a>

## CBO-075 — Dense native results produced row by row through per-cell dispatch

- **Operation and consumers.** Projecting native outcomes into `runtime.*` relations for retained publication,
  cursors and tables: trajectories (`simulation_samples`), `response_sensitivities`, `simulation_events`,
  `live_simulation_samples`, `solve_variables` and `solve_constraints`. Consumers are the result store, selected-output
  readers, study detail demand and result inspection. Trajectories are the largest ordinary product: row count is
  samples times outputs, and times parameters for sensitivities.
- **Current mechanism.** Three linked parts.
  - *Producing.* The integrator owns a sample-by-output `f64` matrix. `modeling/trajectory.rs` (`encode_simulation`,
    `project_impl`) pushes one generated `Row` per cell through `SelectedCollection::push`, repeating run, symbol,
    quantity and unit identity. Each push runs `request.wants`, `skip_next` and `ensure`, then `Collection::push` with a
    per-row `allocation_size`, `reservation.try_grow`, cancel checkpoint, `BTreeMap<TypeId, _>` lookup and `downcast_mut`.
    The generated `append_columns` then calls `ArrowValue::append(&mut dyn ArrayBuilder)` per cell, about seven dynamic
    downcasts per sample row. Relations other than `simulation_samples` resume a late range by `skip_next`, walking the
    prefix and its extent checks, and nine `ensure` calls run per projection. `live_projection` repeats the output,
    symbol, quantity and unit join with its own algorithm.
  - *Consuming.* Views hold typed column references, but non-generated source has 17 `_column()` uses against 18 runtime
    `View` references, and most consumption is `.rows()` or `.row(i)`. `physical/decode.rs` hydrates complete rows to read
    a few fields, and `Collection` and `encode_rows` clone each row before push.
  - *Physical form.* The retained form is long rows with repeated identity columns, uncompressed. `trajectory_endpoints`
    already uses list columns. A search for IPC compression options in `pse-relations`, `pse-columnar`, `workflow/result_*.rs`
    and `canonical_files.rs` found none.
- **Why it matters.** Most per-row work (admission, cancel checks, dispatch) is unrelated to the data. Coordinate columns
  are computable and values already sit in native memory. A columnar build could batch charging, checks and dispatch and
  hold each symbol's slice for the range partition in `store_trajectory`. Changing the retained physical form is a separate
  decision from bulk building under the same form. Both are *Proposed* gains, not *Measured*.
- **Patterns.** LOP-04, LOP-06, LOP-05, LOP-X007; LOP-09 for the two lineage joins.
- **Evidence.** Read `row.rs::RowBuilder::push`, `collection.rs::Collection::push`, `codec.rs` (`scalar!`, `i64`), generated
  `runtime/simulation_samples.rs::append_columns`, `result_export.rs::SelectedCollection::push`, `trajectory.rs`,
  `result_projection.rs::store_trajectory` and `store_result_chunk`, and `pse-schema` catalog entries. A search of `workflow/`
  found direct `Float64Array` construction in test modules only.
- **Footprint.** Trajectory, sensitivity and live emitters inspected. Fit, study and other large `runtime.*` emitters are
  suspected, not read. Downstream users of the long form (`study_demand.rs`, `result_selection.rs`) were located only.
- **Preservation constraints.** Exact IEEE bits, and the recorded `coordinate_min` and `coordinate_max` bit patterns.
  Arrow's min and max kernels differ from the recorded `f64::min` and `max` folding when a run contains NaN or both
  signed zeros, so any substitution is a separate semantic choice. Canonical result order and half-open row coordinates;
  arbitrary range restart; chunking by requested rows with backpressure; per-relation demand; prepaid admission; generated
  field-contract checks at `finish`; cancellation latency; per-output selection and missing-tail rules; retained units and
  quantity per output; digest over encoded bytes.
- **Counterevidence and uncertainty.** The row codec is one authority shared by all relations, and a typed path for a few
  dense relations would be a second form unless generated. Arrow builders are fast, and the overhead may be small beside
  IPC and store work. Long form is what lets relational selection and DataFusion pruning work directly, and compression may
  not survive the prepay-safe admission layer. Trajectory size is bounded by `max_cells`, and small `Rows` emitters do not
  warrant change. The per-cell cost relative to solving is unknown.
- **Status.** Current mechanism.
- **Research questions.** Can the generator emit a per-relation struct-of-columns bulk form from the same declaration?
  Which of `FixedSizeList`, dictionary or run-end columns and IPC body compression survive bounded admission and still prune?
  Does a `serde_arrow`-style route add a second projection (LOP-X007)?

<a id="cbo-005"></a>

## CBO-005 — Local IPC admission around Arrow's decoders needs check-by-check upstream tracking

- **Operation and consumers.** Admit a stored or transported Arrow IPC file or stream and refuse hostile or corrupt framing
  before allocating. Consumers are durable result files (`result_files.rs`, `result_physical.rs`, `result_snapshot.rs`),
  portable payloads, reproducers and the review-pack preview.
- **Current mechanism.**
  - Admission is owned by `pse-columnar`: `ipc_file.rs` (`preflight`, `read_batch`, `AdmittedFile::open`, `writer_extent`),
    `ipc_stream.rs` (a local admitted `StreamReader` over Arrow's `StreamDecoder`) and `ipc_admission.rs` (`trusted_shape`,
    `schema_field`, `layout`), together 2,236 lines including tests. `result_files.rs` delegates to them.
  - The four production stream routes (`result_physical.rs`, `reproducer.rs`, `portable_payload::finalized_ipc_schema`,
    `tooling/review_pack/presentation.rs`) use that reader or `ipc_stream::validate`. Stock `arrow_ipc::reader::StreamReader`
    appears in non-test source only in `pse-schema/src/builder/products.rs`, whose input is presumably embedded and trusted
    (not traced).
  - The checks fall into four kinds:
    - *Policy,* which stays whatever the library does: V5 only, little-endian, no dictionaries, compression or variadic
      buffers, a type allowlist, depth, field, batch and value limits.
    - *Corrupt-input guards,* candidates for upstream status review: `schema_field` before Arrow's infallible schema
      conversion, footer and block-span overlap, node and buffer counts, offsets and overlap, `layout` and the FlatBuffer
      verifier options.
    - *Prepayment mirrors:* `metadata_extent`, `ipc_array_working_extent` (4096 per node), and `alignment_copies`, which
      predicts Arrow's realignment by walking `arrow_data::layout`; writer-side `writer_metadata_tables` in `result_blocks.rs`.
    - *Decoder-shape workarounds:* `StreamReader::frame` appends a 4-byte continuation and `record` tracks
      `decoder_continuation` and zero-body lookahead to drive `StreamDecoder` a frame at a time, and both readers rebuild
      the decoder per block.
- **Why it matters.** Each kind has a different fate. Corrupt-input guards may retire one by one as Arrow hardens; mirrors
  drift on a family upgrade; the stream workarounds look like fighting a push decoder. The local reader is now a maintained
  protocol component whose upstream status is unrecorded per check (*Proposed*).
- **Patterns.** LOP-X009 (primary), LOP-X001, LOP-10, LOP-01.
- **Evidence.** Read `ipc_stream.rs` (`frame`, `record`, `validate_record`, `alignment_copies`, `metadata_extent`),
  `ipc_file.rs` (`AdmittedFile::open`, `read_batch`) and `ipc_admission.rs`, with a caller list.
- **Footprint.** Inspected. `ipc_admission::layout` body and tests were skimmed. No upstream issue was checked, so "still
  needed" is unestablished per check.
- **Preservation constraints.** Refusal before large allocation, with typed errors and no panic on corrupt input; bit-exact
  floats; leases on escaped slices; one-footer reading of a selected block; whole-stream scan without decode
  (`validate_all`); deterministic bytes. Scientific export streams carry lineage metadata, so bare registry-schema equality
  is not a universal stream policy.
- **Counterevidence and uncertainty.** The guards are justified at the pinned arrow-ipc 59.3.0. The lower-level decoders do not
  provide allocation admission: the exact raw block span excludes metadata, schema and wrapper inventories, rounded capacity
  and permitted copies, and `StreamDecoder` copies fragmented input and can decompress from embedded decoded lengths
  (reassessment, 2026-10-27). A probe at that version showed `FileReaderBuilder` allocating the advertised footer length
  before seeking and `StreamReader` allocating advertised body lengths and asserting on malformed input (*Tested* for the
  library behavior; [evidence](../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)).
  A fix that turns an abort into an error could still allocate advertised sizes, and portable payloads can arrive from outside,
  so a checksum does not cover every route.
- **Status.** Current promise. The upstream-tracking work is *Proposed*.
- **Research questions.** For each corrupt-input check, the arrow-rs issue and first fixed version. Is `read_record_batch` or
  an equivalent public for framed stream messages, so the lookahead state could go? Arrow's prepaid-extent mirrors in general
  are [CBO-001](cross-cutting-mechanisms.md#cbo-001).

<a id="cbo-076"></a>

## CBO-076 — Result scans hide column, parallel and statistics intent

- **Operation and consumers.** Reading retained results through `ResultSnapshot` and `SelectedResultSnapshot`
  (`SnapshotProvider`), study cohorts (`CohortProvider`), KPI aggregation (`study_workbench.rs::aggregate`), `inspect_result`
  values and schema views, `output_results` and review and export tooling. Study analysis can span up to 10,000 members.
- **Current mechanism.**
  - `SnapshotProvider::scan` receives `projection`, passes it only to `StreamingTable::scan` and uses one `SnapshotPartition`.
    Every column of every selected block is decoded and admitted first. `ProtectedResultDecode` serializes decoding through
    `decoder_turn`.
  - `pse_columnar::ipc_file::read_batch` reads the whole block with no projection set on its `FileDecoder`.
    `FileDecoder::with_projection` exists in the pinned arrow-ipc 59.3 source (by search only).
  - `inspect_result` knows the requested Arrow field paths and row window but calls `results(run, attempt, relation, start, end,
    cancel)`, which has no projection parameter, so `selected_cell` reads one field after all columns were decoded.
    `output_results` masks after decode; pruning avoids blocks, not columns.
  - Neither provider implements `statistics()` or declares output ordering, although exact per-block statistics are stored and
    used only for pruning (`prune_members`). Cohort aggregation runs one sequential scan.
- **Why it matters.** A narrow query over a wide or nested relation (`trajectory_endpoints`, study outcomes) pays for full
  hydration. Count, min and max queries cannot use known statistics, ordered consumers may sort again, and aggregation over many
  members does not use the 16-core host (*Proposed*; relation width and cost unmeasured).
- **Patterns.** LOP-05, LOP-03, LOP-04.
- **Evidence.** Read `result_snapshot.rs` (`SnapshotProvider`, `SnapshotPartition`, `NativeResultSelection`),
  `cohort_provider.rs` (head and provider sites), `study_workbench.rs::aggregate`, `ipc_file.rs::read_batch`,
  `result_inspection.rs` (`inspect_result`, `selected_cell`) and `connected_results.rs` (`results`, `result_reader_body`). A search of
  both provider files for `statistics`, `output_ordering`, `with_sort`, `EquivalenceProperties` and `LexOrdering` had no match.
- **Footprint.** The `CohortPartition` body, the interior of `access_block` and `scan`, and `pse-relations::result_batch` are
  unread.
- **Preservation constraints.** `FieldCheckedBatch::admit_owned` and `validate_scalar_keys` currently see all fields, so a
  projected read needs a defined narrower admission, and `inspect_result` documents full-batch validation. Original row
  coordinates, exact bits, prepaid decode grants and the same deployment pool; pruned blocks are still validated
  (`validate_pruned_block`); residual before limit; exact cohort classification when partitions finish out of order. Statistics
  must be proven exact under `Inexact` filters and the IEEE zero rules (LOP-X013).
- **Counterevidence and uncertainty.** Block files are bounded (up to 32 MiB, trajectories 1,024 rows), and the body is read as one
  span, so projection saves array construction and checks rather than I/O. The payload digest covers the whole file anyway. The
  decoder turn may be deliberate memory control.
- **Status.** Current mechanism.
- **Research questions.** Can `FileDecoder::with_projection` and `TableProvider::statistics` plus ordering be used with field-level
  admission and several partitions under one reservation model?

<a id="cbo-078"></a>

## CBO-078 — Selected result tables re-walk the whole report once per relation

- **Operation and consumers.** `store_tables` at attempt finish, plus `RunResult` cursors for in-process readers.
- **Current mechanism.** `store_tables` loops over advertised relations. Each creates `cursor_by_id_range_controlled`, whose producer
  calls `RunResult::encode`, dispatching to `encode_fit`, `encode_simulation`, `encode_modeling` or `encode_shooting`. Those functions
  construct `Rows<T>` for every relation they own and walk the report, with `request.wants(id)` and `skip_next()` filtering at row
  level. Resuming a range after a memory refusal re-walks with skips, except `simulation_samples`, which seeks. A tokio channel, a
  yield primitive, `ProjectionRetention`, the append queue and a cold-retry path sit between walk and append.
- **Why it matters.** Cost scales with relations times report size, and adding a relation means editing a multi-relation function that
  mixes row construction, demand checks and working-memory charges. Each relation's physical layout and sort are decided after row
  construction, in `store_result_chunk` (*Proposed*; walk cost unmeasured).
- **Patterns.** LOP-05, LOP-07, LOP-02, LOP-08.
- **Evidence.** Read `durable.rs::store_tables`, `results.rs` (`cursor_by_id_range_controlled`, `encode`),
  `result_export.rs` (`Projection::wants`, `Rows::skip_next`, `yield_delivered_chunk`, `ProjectionRetention`),
  `fitting/results.rs` (`encode_fit`, `uncertainty`) and `result_projection.rs::append_run_result_table_queued`.
- **Footprint.** Fit and simulation producers and append are inspected. `encode_modeling`, `encode_shooting` and `project_common` are
  suspected to share the shape and were read only as dispatch targets. Per-cell production inside each walk is
  [CBO-075](#cbo-075).
- **Preservation constraints.** Lazy per-relation availability; per-relation failure recording (`encodings.record`); pull
  backpressure; prepaid working memory; deterministic canonical order; retained-prefix replay; typed row validation.
- **Counterevidence and uncertainty.** Walks may be cheap against solver time and many reports are small. Laziness exists so unread
  relations cost nothing. A per-relation producer table may be a purely local refactor with no library involved.
- **Status.** Current mechanism.
- **Research questions.** Would a DataFusion `StreamingTable` or `PartitionStream` be the per-relation producer boundary, replacing
  the channel, or would it fight prepay and completion ownership (LOP-X005)?

<a id="cbo-077"></a>

## CBO-077 — Selected output reads hand-compose what a physical plan expresses

- **Operation and consumers.** `SelectedResultReader` and `output_results` read scalar and dense selections, `CanonicalResultReader`
  reads whole relations, and pack export and selected snapshots consume them.
- **Current mechanism.** `ResultReader::next_checked_body` in `connected_results.rs` is a long state machine: metadata page RPCs,
  block and descriptor maps, pruning with `NativeResultSelection`, decode, a single-batch cache, `select_original_rows` (slice,
  evaluate the physical expression, build a mask) and `SelectedRows::apply`. `SnapshotProvider` independently runs pruning and
  decode and leaves residual filtering to DataFusion. Both share `NativeResultSelection`, but page and exactness logic differ by
  path. `cohort_provider` is a suspected third driver and `result_inspection` a fourth consumer (not traced).
- **Why it matters.** The read pipeline has several drivers with the same stages, so a new selection shape or read optimization
  ([CBO-076](#cbo-076)) must be built more than once. A shared stream of checked blocks, or a plan, would make limits, filters and
  cancellation visible to one engine (*Proposed*).
- **Patterns.** LOP-09, LOP-05, LOP-10.
- **Evidence.** Read `connected_results.rs` (`next_checked_body`, `select_original_rows`, `export_ipc_with_prefix`) and
  `result_snapshot.rs` (`NativeResultSelection`, with `FrozenSnapshot::next` by signature).
- **Footprint.** The two read paths are inspected; the other drivers are suspected only.
- **Preservation constraints.**
  - Dense window overlap across outputs is decided by the store's output index (`result_output_page`, on `result_output_range`),
    so a shared driver must sit below metadata paging and not replace it with DataFusion pruning.
  - Dense selection verifies neighbor rows and recorded coordinate min and max against actual data, and export is sealed with the
    exact range and predicate lineage.
  - Output rows stay in stored order, pruned blocks keep their integrity check, and selection metadata feeds
    `SelectedSnapshotBuilder`, which a generic plan would not record by itself.
  - Predicate validation stays in `ValuePredicate::new` and metadata in `ScientificMetadata::entries`; `PackResultPredicate` is the
    exact-bits wire form. `portable_payload` and `review_export` validate with `dense = false`, so whether a dense predicate with
    `missing` can reach them was not traced.
- **Counterevidence and uncertainty.** `select_original_rows` carries real semantics that generic filter execution does not, and
  `Exact` pushdown semantics would need care. The right unit may be a block stream below the two drivers rather than a plan.
- **Status.** Current mechanism.
- **Research questions.** Can a custom `ExecutionPlan` or `DataSourceExec`/`FileSource` for the block stream, with `FilterExec` above
  it, carry the selected-row record and coordinate checks without bespoke escape hatches?

<a id="cbo-008"></a>

## CBO-008 — Seeds, progress and receipts as JSON payloads beside typed relations

- **Operation and consumers.** Record warm-start seeds, run progress and completion as retained, replayable data. Consumers are
  continuation, progress readers, observers, permission reads, replay and the reproducer.
- **Current mechanism.**
  - Seeds: `SeedPayload` holds IEEE bits as decimal `u64` values in `serde_json`, split into `RESULT_BATCH_BYTES` chunks by
    `write_chunks` and rejoined by `read_chunks_selection` with digest checks. `SeedCapsule` is also JSON. A 64-bit float takes up
    to about 21 bytes rather than 8, and decoding allocates twice.
  - Progress: `Streamer::new` in `durable.rs` writes JSON arrays into a `__progress` result set with range halving under
    `pse_ids::BoundedSink`; `decode_progress` reads them. Progress is a fence-bound set in the closed manifest (`PublicationAcks`),
    so it has lifecycle parity with relations and lacks only a registered schema. It is invisible to SQL and DataFusion.
  - Completions: `terminate_inner` serializes `StoredCompletion` and `CompletionReceipt` as JSON.
- **Why it matters.** Each codec carries its own bounds and splitting logic ([CBO-002](cross-cutting-mechanisms.md#cbo-002)). Progress
  cannot be queried with existing result tools, and the chunked byte format is a second persistence path to maintain. The
  value is separate for three decisions: a binary form for seeds (largest for big NLPs), a registered schema for progress (query),
  and document contracts such as completion and replay, which are genuine document formats and probably stay JSON (*Proposed*).
- **Patterns.** LOP-06, LOP-12, LOP-09.
- **Evidence.** Read `durable.rs` (`SeedPayload`, `bits`, `floats`, `Streamer::new`, `decode_progress`, `terminate_inner`,
  `write_chunks`, head of `read_chunks_selection`).
- **Footprint.** The progress readers (`progress.rs`, `progress_documents.rs`) were skimmed. Replay equality by serialize-and-compare
  (`serde_json::to_vec(a) == to_vec(b)`) was not found by search of `workflow/` and `tooling/`, so it is unconfirmed.
- **Preservation constraints.** Exact IEEE bits; bounded per-event drop accounting; attempt fence and acknowledgments; completion
  immutability and digest coverage; version-first decoding; regeneration rather than historical reinterpretation; the document
  schemas tools consume; replay identity.
- **Counterevidence and uncertainty.** Seeds are small for most problems and an existing test covers long bit patterns. JSON is
  debuggable. Nested, heterogeneous progress events map poorly onto a stable schema, and a JSON column may suit them better.
- **Status.** Current mechanism. Weak to moderate.
- **Research questions.** Would Arrow `FixedSizeBinary` or `Float64` for seeds, and a registered `runtime.*` relation for progress,
  be adopted by existing permissions and manifests?

## Examined and not retained

- **Project pack framing.** The archive concatenates an 8-byte length, manifest and members, and is also the retained
  `project-import-evidence` content. A single standard container might ease inspection, but the framing is about ten lines with
  random access by computed offset, `tar` would need its own bounds hardening and §20.6 does not require one file. Recorded as a
  sub-question under [CBO-074](persistence-and-transport.md#cbo-074).
- **Dense covariance propagation loops.** `propagate_checked` in `uncertainty.rs` uses hand-written triple loops beside faer. A faer
  matrix kernel could use FMA at run time, conflicting with the no-contraction invariant and the signed-zero summation start (LOP-X006),
  so the loops may be right. Low confidence either way.
- **QP prediction verification location.** `ModelingResult::qp_prediction` re-checks a submitted problem against the coefficient
  problem in the result type. It is an orchestrator trust boundary over caller input, so the move may only relocate code.
- **Scientific permission composed at several boundaries.** Fit completion assessment is re-derived at several layers and stored both
  as completion JSON and as result blocks. This is mostly sound layering (strategy decides retry, completion decides retention) and the
  doubled storage has a recovery reason. No library applies.
- **Result projection cache.** The cache in `result_export.rs`, keyed on `Arc::ptr_eq` owners over `quick_cache`, serves in-memory
  runs. The snapshot path serves sealed files with different lifetimes, so no consolidation was established.

## Retired opportunities

<a id="cbo-006"></a>

- **CBO-006 — Dense trajectory indexing and selection done row by row.** Retired 2026-10-30: resolved by `c3e2881a6`.
  `result_projection.rs::trajectory_ranges` uses `arrow::compute::partition` on the symbol column, `trajectory_outputs` is now
  `#[cfg(test)]`, and `connected_results.rs::select_original_rows` slices first and builds its mask with `collect_bool`. The retained
  extrema fold deliberately keeps Rust `f64` folding. The larger remaining dense cost is upstream, in producing the rows. Successor:
  [CBO-075](result-data-paths.md#cbo-075). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/result-data-paths.md#cbo-006).

<a id="cbo-007"></a>

- **CBO-007 — Output-predicate semantics in several forms; dense selection outside the planner.** Retired 2026-10-30: resolved by
  `c3e2881a6`. `ValuePredicate::new` is the one validator (callers in `connected_results.rs`, `result_projection.rs`,
  `review_export.rs`, `portable_payload.rs`), `ScientificMetadata::entries` the one metadata formatter and `scalar_expression` the sole
  scalar lowering. Dense overlap is decided by the store's output index, not Rust. Successor:
  [CBO-077](result-data-paths.md#cbo-077). Last described at
  [revision 93946f572](https://github.com/paul-heyse/pse-arrow/blob/93946f5721a27786fd3c4f281eb6f8036dde405b/docs/library_leverage/codebase/result-data-paths.md#cbo-007).
