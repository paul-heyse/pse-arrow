# Result data paths

This page covers how retained scientific results are projected, sealed, selected and read in
`pse-runtime/src/workflow/` and `pse-columnar`. Contracts are owned by
[§20](../../authoritative_design/sections/identity-and-publication.md#section-20) and
[§21](../../authoritative_design/sections/workflows-and-results.md#section-21). The baseline is
on the [entry page](README.md).

**System picture.** Typed relation rows are projected into Arrow batches, sorted, and sealed as
uncompressed Arrow IPC files with recorded per-batch statistics. Their metadata is appended to
the canonical store. Reads build a `ResultReader` over a `ResultSnapshot`. Library use is
already strong:
- A scientific predicate is lowered once to a DataFusion physical expression.
- That expression prunes blocks through `datafusion-pruning::PruningPredicate` against recorded
  statistics and masks residual rows (`result_snapshot.rs`).
- Key validation and sorting use `RowConverter` and `lexsort_to_indices`
  (`pse-relations/src/result_batch.rs`).
- The `SnapshotProvider` reports `Inexact` filter pushdown honestly.

Three behaviours are deliberate domain and identity rules, not leverage targets:
- IEEE-bit predicate identity.
- Original row coordinates.
- "Retention never merges runs".

<a id="cbo-005"></a>

## CBO-005 — Arrow IPC untrusted-input verification written by hand

- **Operation and consumers.** Admit a stored or transported Arrow IPC file or stream, and
  refuse hostile or corrupt framing before allocating. Consumers are protected result reads,
  source-object ingestion, the reproducer and replay.
- **Current mechanism.**
  - `result_files.rs` (`preflight`, `layout`) hand-parses:
    - the footer;
    - block spans;
    - message metadata;
    - node and buffer extents;
    - nested layout.
  - `result_blocks.rs` (`trusted_shape`, `schema_field`) re-validates every IPC `Type`, and
    `writer_metadata_tables` estimates FlatBuffer table counts.
  - `result_blocks.rs` and `pse-columnar/src/ipc_file.rs` (`visible_size`) re-implement
    nested-slice size accounting.
  - `reproducer.rs` checks stream magic bytes.
  - Only after all of this does the stock `FileReaderBuilder` run, with FlatBuffer limits.
  - About 700–900 lines track Arrow IPC wire details at the arrow 59 pin.
- **Why it matters.**
  - Wire-format knowledge held outside the library can silently diverge from it on an upgrade.
  - If arrow-ipc's bounded decoding and validation already discharge part of this, that part
    disappears. The bounded decoding options are `FileDecoder` over a pre-sliced buffer,
    `StreamDecoder`, `validate_full`, alignment and verifier options.
  - The rest stays as explicit domain policy: single-schema equality, no dictionaries and no
    compression.
- **Patterns.** LOP-03 (primary), LOP-01, LOP-11.
- **Footprint.**
  - Inspected: three files.
  - Suspected: other bounded IPC consumers in `pse-operations` and `pse-relations`.
- **Preservation constraints.**
  - Refusal happens before allocation, with typed errors.
  - Exact registry-schema equality.
  - No dictionaries and no compression.
  - Deterministic bytes.
  - Tests pin specific attacks, such as a corrupt footer length that would allocate gigabytes.
- **Counterevidence.**
  - The Arrow reader may not cap allocations driven by untrusted metadata. That would justify
    the preflight, at least in part.
- **Status.** Current mechanism. The opportunity is proposed.
- **Research questions.**
  - What does arrow-ipc 59 verify before it allocates?
  - Is there an allocation cap or a message-size hook?
  - Does `get_slice_memory_size` now account for nested slices?
- **Library assessment correction (2026-10-27).** The preflight is justified at the pinned arrow-ipc 59.3.0. `FileReaderBuilder` allocates the advertised footer length before seeking, and `StreamReader` allocates advertised body lengths and asserts on several malformed inputs; see [LOP-X009](../assessment-patterns.md#lop-x009). The footprint is wider than the preflight sites. Three production `StreamReader` decodes run without the preflight: `result_physical.rs`, `tooling/review_pack/presentation.rs`, and `portable_payload.rs` (schema only, bounded). Whether their inputs are integrity-checked first was not traced. See [arrow/ipc](../libraries/arrow/ipc.md).
- **Comparative review correction (2026-10-27).** The decode sites differ in trust. `result_physical.rs` decodes digest-verified, system-written store objects (`canonical_staging` assembly checks each block digest and the payload digest). `reproducer.rs` decodes IPC embedded in an exported replay document after checking only its continuation and end-of-stream bytes (`recorded_source_complete`), and charges `bytes×16 + 64 KiB`. A later probe and static trace showed that this route is unauthenticated before decode: a user-supplied replay document, or a project-bundle reproducer whose member digests are only self-consistent, reaches the stock `StreamReader`, where a lying schema or batch `bodyLength` aborts the process and a negative length or an out-of-body buffer span panics. `portable_payload::finalized_ipc_schema` and the review-pack preview are exposed the same way through the schema message, because `Read::take` does not bound the advertised-length pre-allocation (*Tested* for the library behaviour, *Interface-checked* for the routes; [evidence](../../design_review/evidence/library-leverage-arrow-ipc-stream-hostile-2026-10-27/README.md)). `pse-schema/src/builder/products.rs` also uses `StreamReader` (not traced; the input is presumably embedded and trusted), and the `connected_results.rs` and `lm_inquiry.rs` sites are tests. See the [comparative review](../reviews/result-data-and-prepaid-accounting.md#cbo-005).

<a id="cbo-006"></a>

## CBO-006 — Dense trajectory indexing and selection done row by row

- **Operation and consumers.**
  - Store path: derive per-symbol contiguous output ranges with coordinate min/max when storing
    dense `simulation_samples`.
  - Read path: select a dense output window.
  - Dense trajectories are the largest result tables. Consumers are storage, `output_results`,
    pack and review export, and cohort reads.
- **Current mechanism.**
  - `result_projection.rs` (`trajectory_outputs`) hydrates `view.row(position)` up to three
    times per row and computes min and max in a scalar loop. `store_trajectory` runs it twice
    over the same range.
  - The dense branch of `connected_results.rs` (`output_results`) hydrates every row, formats a
    `"{relation}:{symbol}:value"` string per row to compare with the filter, scans time in a
    scalar loop, and collects `u32` indices.
  - The scalar branch converts a boolean mask to indices by looping over all rows.
  - `cohort_provider.rs` has the same shape.
  - Arrow compute is used elsewhere in the same stack (`compute::min/max`, `take`, `filter`).
- **Why it matters.**
  - Run boundaries could come from adjacent-slice comparison or `arrow-ord` partitioning.
  - Extrema could come from compute kernels.
  - Selection could use `arrow_select::filter` or slicing.
  - Together these would remove per-row allocation, the duplicate pass and the mask-to-index
    step on the largest tables.
- **Patterns.** LOP-04 (primary), LOP-06, LOP-03.
- **Footprint.**
  - Inspected: three sites.
  - Suspected: similar `View::row` loops in `result_inspection.rs`.
- **Preservation constraints.**
  - Exact original coordinates and IEEE bits.
  - The equality check on `coordinate_min`/`coordinate_max`.
  - The contiguity refusal.
  - `u32` offset admission.
  - Prepaid memory.
  - The checked-field access that `View::row` may provide.
- **Counterevidence.**
  - The row chunking doubles as a memory-grant unit.
  - These paths have not been measured, so their cost is unknown.
- **Status.** Proposed addition.
- **Research questions.**
  - Which `arrow-ord`/`arrow-select` kernels cover run partitioning and masked take without
    extra scratch?
  - Can `FieldCheckedBatch` be queried column-wise without losing field checking?
- **Library assessment correction (2026-10-27).** `cohort_provider.rs` calls `select_original_rows` and does not repeat the loop. Arrow min/max kernels can produce different IEEE bits from the recorded `f64::min`/`max` values when a run contains NaN or both signed zeros, so the declared domain of `time` decides that substitution. See [arrow/compute](../libraries/arrow/compute.md).
- **Comparative review correction (2026-10-27).** `trajectory_outputs` runs per window of at most 1,024 rows (`store_trajectory`), so contiguity is enforced per window, and it runs twice per window. The read-path extrema check uses IEEE `==` on `Option<f64>`. Stored coordinates keep their bits and SurrealQL asserts a finite projection, but NaN times can be stored, because `f64::min`/`max` ignore NaN and `time` is declared `real()` (unconstrained Float64). Arrow's comparison kernels, not only `min`/`max`, use totalOrder for floats, so a `gt_eq`/`lt_eq` time window would also change selection at `-0.0` and NaN. See the [comparative review](../reviews/result-data-and-prepaid-accounting.md#cbo-006).

<a id="cbo-007"></a>

## CBO-007 — Output-predicate semantics in several forms; dense selection outside the planner

- **Operation and consumers.** Define, validate, lower and record output selection: a scalar
  range, missing values, or a dense time window. Validation means finite bounds,
  `min <= max`, and missing values excluding bounds. Consumers are `output_results`, review
  export and pack selection.
- **Current mechanism.**
  - The finiteness, ordering and missing-value check is written three times:
    `connected_results.rs`, `result_projection.rs` and `review_export.rs`.
  - `PackResultPredicate` (`result_selection.rs`) carries the same bounds again.
  - Dense windows are compared by hand rather than lowered to the DataFusion expression that
    drives scalar pruning.
  - IPC export restates predicate fields as `pse.canonical.*` string metadata.
- **Why it matters.**
  - One predicate type with a single validation, lowering and metadata encoding would remove
    three independent decisions.
  - The dense window could reuse the pruning and residual machinery if recorded per-output
    `coordinate_min`/`coordinate_max` were exposed as statistics.
- **Patterns.** LOP-02, LOP-09, LOP-05.
- **Footprint.** Three duplicated validations and two selection paths.
- **Preservation constraints.**
  - Dense semantics: outputs contiguous by symbol, refusal of adjacent rows, and no `missing`
    on dense outputs.
  - IEEE-bit identity.
  - Diagnostics that tests pin.
- **Counterevidence.**
  - Dense and scalar selection differ semantically, so a shared type may need to stay a sum
    type.
- **Status.** Proposed addition.
- **Research questions.** Can `PruningPredicate` prune dense blocks from per-output statistics
  without changing refusal behaviour?
- **Comparative review correction (2026-10-27).** The premise "dense selection outside the planner" is rejected. The window-overlap decision across dense outputs already runs in generated SurrealQL (the `canonical_result_block_outputs` query over coordinate projections, on the `result_output_range` index), not in Rust, so a DataFusion pruning route would move the selection away from its index. `pse.canonical.*` predicate metadata is encoded in `connected_results.rs` and decoded separately in `portable_payload.rs`. See the [comparative review](../reviews/result-data-and-prepaid-accounting.md#cbo-007).

<a id="cbo-008"></a>

## CBO-008 — Progress events and completion receipts stored as JSON payloads beside typed relations

- **Operation and consumers.** Record run progress and completion as retained, replayable data.
  Consumers are progress readers, replay and the reproducer.
- **Current mechanism.**
  - `durable.rs` serializes progress events as JSON arrays into an `__progress` result batch
    and halves the batch on overflow.
  - `CompletionReceipt` is stored as JSON.
  - `results.rs` uses `BoundaryDocument::WIRE` for one document and plain `serde_json` for
    another.
  - Replay and the reproducer compare documents by `serde_json::to_vec(a) == to_vec(b)`.
  - Typed relations travel through `RelationRow` to Arrow.
- **Why it matters.**
  - Each codec carries its own bounds and splitting logic ([CBO-002](cross-cutting-mechanisms.md#cbo-002)).
  - Equality by serialization is an implicit contract.
  - A relation-shaped progress table, or one document codec, would reduce independent decisions.
- **Patterns.** LOP-06, LOP-02.
- **Footprint.** About six inspected sites. The wider footprint is unknown.
- **Preservation constraints.**
  - Replay identity.
  - The document schemas that tools consume.
- **Counterevidence.**
  - Progress is sparse and append-heavy, and human-readable JSON helps diagnosis.
  - Canonical JSON equality may be intentional.
- **Status.** Proposed addition. Weak to moderate.
- **Research questions.**
  - Why is progress a distinct payload rather than a registered relation?
  - Does `BoundaryDocument` already define canonical equality?

## Examined and not retained

- **Result projection cache.** The cache in `result_export.rs`, keyed on `Arc::ptr_eq` owners
  over `quick_cache`, serves in-memory runs. The snapshot path serves sealed files. Their
  lifetimes differ, so no consolidation opportunity was established. A later refresh may revisit
  this if both caches serve the same reads.
