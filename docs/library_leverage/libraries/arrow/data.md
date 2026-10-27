# Apache Arrow data layout and memory sizing (arrow-data, arrow-buffer)

Realizations considered: installed `arrow-data` 59.3.0 and `arrow-buffer` 59.3.0 (the `pool`
feature is off unless a dependent enables it); upgrade route 60.0.0; apache/arrow-rs `main`
(arrow-data `data.rs` fetched 2026-10-27).
Research date 2026-10-27. Sources: installed `arrow-data-59.3.0/src/data.rs` (`layout`,
`DataTypeLayout`, `BufferSpec`, `get_buffer_memory_size`, `get_slice_memory_size`,
`get_array_memory_size`, `validate_data`, `validate_full`, `ArrayData::slice`,
`ArrayDataBuilder::build`); `arrow-buffer-59.3.0/src/{pool.rs,buffer/mutable.rs}`;
`arrow-array-59.3.0` `Array::claim` implementations; release notes 60.0.0 (#10641, #10317,
#11044); `main` `get_slice_memory_size`.

## Types

### arrow-data/layout-description
- **Need it serves:** know, per Arrow `DataType`, which physical buffers exist and their element
  widths, without restating the Arrow columnar spec in local tables.
- **Recognizable code shapes:** `match DataType { Int32 => 4, Int64 | Float64 => 8, Utf8 => (n+1)*4 … }`
  in size estimators, IPC layout checks or writer-scratch bounds; `primitive_width()` plus special
  cases for offsets and bitmaps.
- **Mechanisms and APIs:** `arrow_data::layout(&DataType) -> DataTypeLayout { buffers:
  Vec<BufferSpec>, can_contain_null_mask, variadic }`, with `BufferSpec::{FixedWidth { byte_width,
  alignment }, VariableWidth, BitMap, AlwaysNull}`. Children are not described (the caller
  recurses over `DataType` children).
- **Obligations discharged:** buffer count and widths per type, including offsets width and
  boolean bitmaps, maintained with the library's own type support.
- **Residual caller obligations:** offsets carry `len + 1` entries (the spec says width, not
  count); validity bitmaps are separate from `buffers`; nested child lengths come from offsets or
  fixed list size; IPC framing adds a validity buffer slot per node and alignment padding.
- **Semantic conditions:** describes the in-memory layout, which matches the IPC body layout for
  the non-view types; IPC padding and truncation are writer rules, not layout.
- **Version / build conditions:** public at 59.3.0 and on `main`.
- **Limits and negative results:** not a size function; no child-length derivation.
- **Evidence:** Interface-checked: installed `data.rs` 59.3.0.

### arrow-data/memory-size-functions
- **Need it serves:** measure the bytes an array or batch occupies (retained) or would occupy if
  compacted (visible), for admission or accounting.
- **Recognizable code shapes:** hand-written "visible size" recursions over `ArrayData`; Arrow-
  shaped multipliers (`rows * 64`, `fields * 512 + 2048`) beside reservations.
- **Mechanisms and APIs:**
  - `ArrayData::get_array_memory_size()` / `Array::get_array_memory_size()`: retained capacity of
    all buffers plus struct sizes, recursive; shared buffers counted once per reference.
  - `get_buffer_memory_size()`: buffer capacities only.
  - `get_slice_memory_size() -> Result<usize>`: bytes a compacted copy of the visible slice would
    need, from `layout()`; checked multiplication for fixed width; offsets differences for
    `Utf8`/`Binary`; validity `ceil(len/8)`; then **adds each child's full
    `get_slice_memory_size()`**.
  - Companion counters elsewhere: DataFusion `get_record_batch_memory_size` and
    `RecordBatchMemoryCounter` (deduplicates shared buffers across batches; see `datafusion.md`).
  - `arrow-buffer` `pool` feature: `arrow_buffer::MemoryPool::reserve` (infallible) and
    `Array::claim(&dyn MemoryPool)` / `RecordBatch::claim` attach a reservation to each underlying
    `Bytes` once, released when the bytes drop.
- **Obligations discharged:** post-hoc measurement of retained or visible bytes of existing arrays.
- **Residual caller obligations:** any **prediction** before allocation (these functions measure
  arrays that already exist); nested-slice visibility for list-like types; IPC padding.
- **Semantic conditions:**
  - `get_slice_memory_size` is exact for flat fixed-width and `Utf8`/`Binary` slices. For
    `List`/`LargeList`/`Map`/`FixedSizeList` it counts the **entire child** even when the parent is
    a small slice, because `ArrayData::slice` only re-slices children for `Struct`; the result is
    a conservative over-estimate that can be arbitrarily large relative to the visible bytes.
    `Struct` slices are accurate because their children are sliced.
  - `pool` claims are infallible: a full pool is over-filled, not refused, so claims cannot
    implement prepay refusal; they are a retained-memory ledger.
- **Version / build conditions:** behaviour above at 59.3.0; on `main` (2026-10-27)
  `get_slice_memory_size` adds view-type variadic buffers but still recurses into unsliced
  children. 60.0.0 adds fallible `MutableBuffer` constructors (`try_with_capacity`,
  `try_from_len_zeroed`, #10317/#10641), which turn allocation failure into an error but do not
  predict size.
- **Limits and negative results:** no Arrow API predicts kernel output or decoder allocation sizes
  before execution (search scope: arrow-data/arrow-buffer public items at 59.3.0; arrow-select and
  arrow-ipc writer/reader); predictions remain caller arithmetic, best derived from
  `layout()` plus known output lengths (for example `FilterPredicate::count()`, see
  `arrow-compute.md`).
- **Evidence:** Interface-checked: installed sources and `main` `data.rs`.

### arrow-data/validation-and-layout
- **Need it serves:** establish that buffers, offsets, null counts and values of an `ArrayData`
  are consistent before trusting it.
- **Recognizable code shapes:** hand checks of buffer lengths against `len * width`, offset
  monotonicity, UTF-8 checks, before constructing arrays from foreign buffers.
- **Mechanisms and APIs:** `ArrayDataBuilder::build()` runs `validate_data()` (`validate`,
  `validate_nulls`, `validate_values`) unless the `unsafe` `skip_validation` is set (forced on
  with the `force_validate` feature); `ArrayData::validate_full()` recurses into children.
- **Obligations discharged:** structural and value validity of decoded arrays without allocation
  proportional to declared lengths (validation compares declared lengths with existing buffers).
- **Residual caller obligations (59.3.0):** `ArrayDataBuilder::build` constructs
  `BooleanBuffer::new(buffer, offset, len)` before validation and that constructor asserts the
  bitmap length, so an undersized validity buffer panics; callers handling untrusted buffers must
  check validity length first.
- **Version / build conditions:** the validity assertion becomes an error on `main` (#11044,
  merged 2026-09-18, after 60.0.0; unreleased).
- **Limits and negative results:** validation is O(n) CPU; it does not bound lengths of arrays
  with no buffers (`Null`).
- **Evidence:** Interface-checked: installed `data.rs` 59.3.0 `build`; upstream commit list for
  `arrow-ipc/src/reader.rs` and PR titles.

## Current applicability

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) — Prepaid memory accounting mirrors library internals (Arrow part)
- **Types:** `arrow-data/layout-description`, `arrow-data/memory-size-functions`; DataFusion
  `datafusion/memory-reservation-and-batch-sizing` (`datafusion.md`).
- **Realization:** installed 59.3.0.
- **Conclusion:** fits with conditions for replacing hard-coded Arrow widths with
  `layout()`-derived arithmetic; ruled out here as a replacement for predictions — every Arrow
  size function is post-hoc, and `pool` claims cannot refuse.
- **What it would discharge / what remains local:** `layout()` can supply widths and buffer kinds
  for estimates such as selection offsets, decoded batch extents and the IPC layout minima; the
  predicted lengths (selected count, rows decoded, kernel temporaries) remain the caller's. The
  `* 64` multiplier for DataFusion Boolean/CASE/comparison temporaries in
  `connected_results.rs` (`select_original_rows`) and the `fields * 512 + 2048` pruning extent in
  `result_snapshot.rs` estimate library-internal intermediates that neither Arrow nor DataFusion
  report; no library route replaces them. `get_array_memory_size` is already used post-hoc in
  `result_snapshot.rs`.
- **Prerequisites:** none beyond existing dependencies.
- **Semantic gaps:** `get_slice_memory_size` over-counts list children, so substituting it for
  `pse_columnar::ipc_file::visible_size` would break the pinned "small nested slice does not pay for
  its full parent" behaviour (test in `ipc_file.rs`) — visible_size's list-child slicing is a real
  residual, not a mirror.
- **Unsettled premises:** whether conservative over-admission from library functions would be
  acceptable where exactness is not pinned (a policy choice).
- **Evidence:** Interface-checked; application Proposed.

### [CBO-005](../../../library_leverage/codebase/result-data-paths.md#cbo-005) — sub-question "Does `get_slice_memory_size` now account for nested slices?"
- **Conclusion:** no at 59.3.0 and not on `main` as of 2026-10-27 (children of list-like types are
  counted whole); `Struct` is correct.
- **Evidence:** Interface-checked (installed and `main` `data.rs`).
