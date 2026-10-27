# Apache Arrow compute kernels (arrow-ord, arrow-select, arrow-arith, arrow-buffer bit iterators)

Realizations considered: installed `arrow-ord`, `arrow-select`, `arrow-arith`, `arrow-buffer`
59.3.0; 59.3.1 includes "Fix filter_nulls for predicates selecting all or no rows" (#11444);
upgrade route 60.0.0 (family major).
Research date 2026-10-27. Sources: installed `arrow-ord-59.3.0/src/{partition.rs,cmp.rs}`,
`arrow-select-59.3.0/src/filter.rs`, `arrow-arith-59.3.0/src/aggregate.rs`,
`arrow-buffer-59.3.0/src/buffer/boolean.rs`; release notes 59.3.1 and 60.0.0.

## Types

### arrow-compute/run-partitioning
- **Need it serves:** split an ordered column (or tuple of columns) into maximal runs of equal
  values — run boundaries, run ranges, contiguous groups — without decoding rows.
- **Recognizable code shapes:** `while pos < end && row(pos).key == key { pos += 1 }` loops; per-row
  struct hydration to read one key column; "start of new group" flags computed by hand.
- **Mechanisms and APIs:** `arrow_ord::partition::partition(&[ArrayRef]) -> Partitions`, then
  `Partitions::ranges() -> Vec<Range<usize>>`, `len()`. Internally `find_boundaries` compares the
  column with itself shifted by one using `cmp::distinct` (bulk, vectorized) for non-nested
  types (including `FixedSizeBinary`), or `make_comparator` for nested types, and ORs boundaries
  across columns.
- **Obligations discharged:** boundary detection and range construction, null-aware (a change
  in nullability is a boundary).
- **Residual caller obligations:** any rule about runs (for example, "a key may appear in only one
  run" needs a set over run keys, now O(runs) instead of O(rows)); per-run aggregates.
- **Semantic conditions:** float equality follows IEEE totalOrder (`-0.0` and `0.0` are distinct,
  NaNs compare by bits); scratch is one boolean bitmap of `n-1` bits plus the `Vec<Range>`.
- **Version / build conditions:** stable API at 59.3.0.
- **Limits and negative results:** returns ranges only; no per-range values or aggregates.
- **Evidence:** Interface-checked: installed `partition.rs`, `cmp.rs` (`distinct`,
  `supports_distinct`).

### arrow-compute/predicate-selection
- **Need it serves:** turn a Boolean selection (possibly with nulls, possibly restricted to a
  window) into selected rows, selected indices or a filtered batch, knowing the output count
  before materializing.
- **Recognizable code shapes:** `for i in 0..n { if mask.is_valid(i) && mask.value(i) &&
  in_window(i) { indices.push(i as u32) } }`; hand-built `UInt32Array` then `take`.
- **Mechanisms and APIs:**
  - `arrow_select::filter::{filter, filter_record_batch, FilterBuilder::new(&mask)
    [.optimize()] .build() -> FilterPredicate}`, `FilterPredicate::{count, filter,
    filter_record_batch}`; `prep_null_mask_filter` (nulls become false; `FilterBuilder::new` does
    this itself). A window becomes `mask.slice(lo, len)` against `batch.slice(lo, len)` (both
    zero-copy).
  - `BooleanBuffer::{count_set_bits, set_indices, set_indices_u32, set_slices}`: bit-iterator
    conversion of a mask into `u32` indices or contiguous slices without a per-row branch; combine
    validity first (`mask.values() & nulls.inner()`).
  - `arrow_select::take::take` / `take_record_batch` with `UInt32Array` indices.
- **Obligations discharged:** selection, null handling, strategy choice between slice copying and
  index gathering; exact output row count (`count()`) before any output allocation, from which
  fixed-width output sizes follow (`count * byte_width + ceil(count/8)` per column).
- **Residual caller obligations:** retaining row coordinates when the caller needs them (filter
  discards original positions; indices must be produced separately with `set_indices_u32`);
  overflow admission for `u32`; prepay arithmetic for variable-width outputs.
- **Semantic conditions:** `FilterPredicate::selection` is `pub(crate)` at 59.3.0, so the
  predicate's internal index/slice form is not reusable as coordinates. `optimize()` materializes
  indices or slices (allocation proportional to selected rows).
- **Version / build conditions:** 59.3.1 fixes `filter_nulls` for all/none selections (#11444);
  60.0.0 documents `FilterSelection` (#10056) and speeds up `FixedSizeBinary` filtering.
- **Limits and negative results:** no fused "mask AND coordinate window → indices" kernel; compose
  slice + bit iteration.
- **Evidence:** Interface-checked: installed `filter.rs`, `boolean.rs`.

### arrow-compute/extrema
- **Need it serves:** minimum and maximum of a numeric column or slice.
- **Recognizable code shapes:** scalar `min = min.min(x); max = max.max(x)` loops.
- **Mechanisms and APIs:** `arrow_arith::aggregate::{min, max}(&PrimitiveArray<T>) ->
  Option<T::Native>` (also `min_max` helpers, SIMD-friendly lanes); apply to `array.slice(r.start,
  r.len())`.
- **Obligations discharged:** vectorized aggregation over non-null values.
- **Semantic conditions:** for floats the kernels order by `is_lt`/`is_gt` from
  `ArrowNativeTypeOp` (`total_cmp`), i.e. IEEE totalOrder: positive-sign NaN is greater than
  every number (a negative-sign NaN bit pattern sorts below −∞), `-0.0 < 0.0`.
  Rust `f64::min`/`f64::max` instead ignore NaN and may return either zero for `(-0.0, 0.0)`.
  Results can therefore differ in IEEE bits for inputs containing NaN or signed zeros; all-null or
  empty input returns `None`.
- **Version / build conditions:** stable at 59.3.0.
- **Limits and negative results:** no per-range (segmented) aggregate in arrow-arith; one call per
  run.
- **Evidence:** Interface-checked: installed `aggregate.rs` (doc comments and accumulator code).

## Current applicability

### [CBO-006](../../../library_leverage/codebase/result-data-paths.md#cbo-006) — Dense trajectory indexing and selection done row by row
- **Types:** `arrow-compute/run-partitioning`, `arrow-compute/extrema`,
  `arrow-compute/predicate-selection`.
- **Realization:** installed 59.3.0.
- **Conclusion:** fits with conditions. Run ranges (`partition` over the `symbol_id`
  `FixedSizeBinary` column of a slice), per-run time extrema and window selection map onto
  existing kernels; the extrema substitution is conditional on the IEEE-bit question below.
- **What it would discharge / what remains local:**
  - Store path (`result_projection.rs::trajectory_outputs`): `partition` replaces the nested
    `view.row(position)` loops (up to three full-row decodes per row); the contiguity refusal stays
    local as a set over run keys; per-run `min`/`max` on `time_column().slice(..)`; the
    `u64`/`u32` coordinate arithmetic and `Output` construction stay local. The second
    `trajectory_outputs` call in `store_trajectory` (over the accepted prefix) can reuse the first
    call's ranges truncated at `rows` — local, not a library matter.
  - Read path (`connected_results.rs::select_original_rows`, dense branch): the per-row
    `format!("{relation}:{symbol}:value")` comparison becomes a `distinct`/`eq` of the symbol column
    slice against one `FixedSizeBinary` scalar (or a check that `partition` of the slice yields one
    run plus neighbour checks); the time window becomes `gt_eq`/`lt_eq` kernels with `Scalar`
    bounds, combined with the coordinate window via slicing, then `set_indices_u32`.
  - Scalar branch: mask → indices via slice + `set_indices_u32`, nulls folded first.
  - Remains local: adjacent-row refusal, recorded-extrema equality check, `u32` admission,
    prepaid offsets (now exactly `count_set_bits() * 4` before collecting).
- **Prerequisites:** the generated view already exposes typed column accessors from a checked
  owner (`View::from_checked(..).symbol_id_column()`, `time_column()`; its docs say "Columnar
  consumers should borrow the concrete column accessors"), so field checking is preserved without
  a new API.
- **Semantic gaps against preservation constraints:** recorded `coordinate_min`/`coordinate_max`
  were produced with `f64::min`/`f64::max`; Arrow's totalOrder extrema differ when a run contains
  NaN or both signed zeros, which would make the read-path equality check refuse previously sealed
  results unless both sides change together or the time field excludes such values.
- **Unsettled premises:** whether the registry contract for `simulation_samples.time` admits NaN
  or `-0.0` (inspect the field's declared domain); whether time is monotone within a symbol run
  (if so, a window is a slice found by `partition_point` on `values()`, with no mask). Cost is
  unmeasured.
- **Corrections:** `cohort_provider.rs` does not hold a third copy of the loop; it calls
  `connected_results::select_original_rows`, so the footprint is two sites plus that shared caller.
- **Evidence:** Interface-checked; application Proposed.

### [CBO-007](../../../library_leverage/codebase/result-data-paths.md#cbo-007) — Dense selection outside the planner (kernel part)
- **Types:** `arrow-compute/predicate-selection`.
- **Conclusion:** fits with conditions for the residual row mask inside a selected dense output
  (comparison kernels with scalar bounds); output pruning itself is in `datafusion.md`.
- **Evidence:** Interface-checked; application Proposed.
