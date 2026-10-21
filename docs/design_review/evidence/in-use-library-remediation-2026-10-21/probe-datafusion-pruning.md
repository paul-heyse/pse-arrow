# Probe report: DataFusion float semantics, statistics pruning and FixedSizeBinary keys

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Probe report: DataFusion float semantics, statistics pruning grid, FixedSizeBinary key pruning. Crate under probe-artifacts/pruning/.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

**Result:** the C1 canonical-predicate lowering gives the same filter result as the reference implementation on every grid case. Pruning never dropped a matching container. One premise in inv-data §1 and §4.2 is wrong: in DataFusion 55.1, `-0.0 = 0.0` is **true**. Comparisons follow totalOrder for NaN but treat +0 and -0 as equal. The `min` and `max` aggregates are fully totalOrder.

## Crate, dependencies and commands

- **Crate:** `/tmp/claude-1000/-home-paul-pse-arrow/15c6fe71-a184-4329-a417-2fc2e56d8c52/scratchpad/pruning-probe/` (source in `src/main.rs`). I deleted the old `parquet-probe/` first.
- **Size:** 136K of source, lock and logs, plus a 4.7G `target/`. The crate is left in place.
- **Repository:** I did not modify `/home/paul/pse-arrow`; `git status` is clean.

The dependencies copy the repository's pins and features:

```toml
arrow = { version = "=59.3.0", default-features = false, features = ["ipc","ffi","canonical_extension_types"] }
datafusion = "=55.1.0"            # default features, as in the repo
datafusion-common = "=55.1.0"
datafusion-expr = "=55.1.0"
datafusion-physical-expr = "=55.1.0"
datafusion-pruning = "=55.1.0"
tokio = { version = "=1.53.2", default-features = false, features = ["rt-multi-thread","sync","macros","time"] }
# [profile.dev] opt-level=1, panic="unwind"; standalone [workspace]
```

`Cargo.lock` is a copy of the repository's lock, which cargo pruned. The resolved versions are arrow 59.3.0, datafusion, datafusion-common and datafusion-pruning 55.1.0, and tokio 1.53.2.

Commands, run from the crate directory. The memory-capped scope was full at first, so both commands ran in a retry loop:
```
/home/paul/pse-arrow/scripts/pse-env -- rustup run nightly-2026-09-29 cargo build --offline --target-dir $PWD/target
/home/paul/pse-arrow/scripts/pse-env -- ./target/debug/pruning-probe > run.log
```
The toolchain is rustc 1.101.0-nightly (c1070d693 2026-09-28). The full output is in `run.log` in the crate directory.

**API drift at the pins** (affects the citations in inv-data §1):
- `PruningPredicate::try_new` is `#[deprecated(since = "55.0.0")]`. The supported path is `PruningPredicateBuilder::new().with_file_schema(s).try_build(expr)`, or `.build(expr) -> Option<Arc<_>>`, which returns `None` for a predicate that is always true.
- `create_physical_expr` takes a fourth argument, `&PhysicalPlanningContext::default()`.

## Probe 2: DataFusion float semantics (MemTable and `SessionContext::sql`)

| query | result | bits |
|---|---|---|
| `CAST('NaN' AS DOUBLE) > 5` | true | |
| `CAST('NaN') > CAST('Infinity')` | true | |
| `-0.0 = 0.0` | **true** | |
| `CAST(-0.0) = CAST(0.0)`; `CAST(-0.0) < CAST(0.0)` | **true**; false | |
| column `x > 5` for x = NaN | true | |
| column `x = x` for x = NaN | true | |
| columns `a = b` (-0, +0); `a < b` | **true**; false | |
| `count(*) WHERE x = 0.0`, `= -0.0`, `>= 0.0` over {+0, -0} | 2, 2, 2 | |
| `max(x)` over {1, NaN, 5} | NaN | 0x7ff8000000000000 |
| `max(x)` over {1, -NaN, 5}; `min(x)` | 5.0; -NaN | 0xfff8000000000000 for the min |
| `min(x)` over {+0, -0} and over {-0, +0} | -0.0 | 0x8000000000000000 |
| `max(x)` over {+0, -0} in either order | +0.0 | 0x0 |
| `max(x)` over {NaN with payload, NaN} | the payload NaN | 0x7ff8deadbeef0001 |
| `count(DISTINCT x)` over {+0, -0} | 1 | |

Direct `arrow::compute::{min,max}` over [1, +NaN, -NaN, -0, +0] gives min 0xfff8… (-NaN) and max 0x7ff8… (+NaN).

**Why:** `datafusion-physical-expr-common-55.1.0/src/datum.rs` `apply_cmp` passes both operands through `datafusion_common::utils::normalize_float_zero` before calling Arrow's totalOrder kernels. That function copies the array only if it contains a -0, and does the same to scalars; the comment states the aim as "SQL semantics (`+0.0 == -0.0`)". Comparison is therefore totalOrder with the two zeros collapsed: NaN of either sign stays ordered and bit-distinct, and +0 equals -0. inv-data §1 "Float semantics" ("`-0.0 = 0.0` is false") and the ±0 half of §4.2 should be corrected. Only the NaN half of the trap remains.

## Probe 4: lowering the canonical predicate, and the pruning grid

**Setup.** The probe re-implements `scalar_predicate` as the reference ("oracle"), copied from `result_projection.rs`, without the per-relation field list. The test data:
- 3391 containers ("containers" are small record batches) holding 20107 rows;
- 26 value classes: +NaN, -NaN, ±signalling NaN, a NaN with a payload, ±maximum-payload NaN, ±0, ±inf, ±subnormal, MIN_POSITIVE, ±MAX, -5, -1, -0.5, 0.5, 1, 5, 7, ±1e300 and null;
- every value alone in one container, and every unordered pair;
- 14 named containers: all NaN, all -NaN variants, all -0, mixed ±0, all null, mixtures;
- 3000 random containers of 1–12 rows.

Statistics per container are `Precision::Exact` arrow `min`/`max`, `null_count` and `num_rows`, wrapped in `PrunableStatistics`. The lowering follows the brief: `x > -inf AND x < +inf`, `x >= (m==0 ? -0.0 : m)`, `x <= (M==0 ? +0.0 : M)`, and `x IS NULL` for missing=true.

**Grid results.** The grid has 6 minimum values {none, 0, -0, -1, 1, 5} × 6 maximum values {none, 0, -0, 1, 5, 1e300} × 3 missing values, plus 6 non-finite bounds.

| outcome | count |
|---|---|
| combinations refused by the oracle (non-finite bound, min > max, missing=true with a bound) | 51 |
| evaluated combinations | 63 |
| filter-equivalent row by row (null counted as false) | 63 / 63 |
| exact bits through `DataFrame::filter` (full optimizer) match the oracle's selected rows | 63 / 63 |
| strict mask equality, validity included | only `missing=true` |
| soundness violations | **0** |

Strict equality fails elsewhere because DataFusion returns null at the 833 null rows, where the oracle returns false. A filter treats the two the same.

Rows show minimum / maximum / missing; "–" means not set:

| predicate | oracle rows | containers pruned | containers with matches | violations |
|---|---|---|---|---|
| – / – / true | 833 | 2677 | 714 | 0 |
| – / – / false | 12405 | 181 | 3162 | 0 |
| – / 0 (or -0) / any | 6283 | 473 | 2665 | 0 |
| – / 1 | 9315 | 291 | 2976 | 0 |
| – / 5 | 10140 | 261 | 3031 | 0 |
| – / 1e300 | 11677 | 204 | 3123 | 0 |
| 0 (or -0) / – | 7716 | 392 | 2789 | 0 |
| 0 / 0 (any zero signs) | 1594 | 684 | 1227 | 0 |
| 0 / 1 | 4626 | 502 | 2320 | 0 |
| 0 / 5 | 5451 | 472 | 2489 | 0 |
| 0 / 1e300 | 6988 | 415 | 2706 | 0 |
| -1 / – | 10045 | 272 | 2998 | 0 |
| -1 / 0 | 3923 | 564 | 2172 | 0 |
| -1 / 1 | 6955 | 382 | 2712 | 0 |
| -1 / 5 | 7780 | 352 | 2809 | 0 |
| -1 / 1e300 | 9317 | 295 | 2938 | 0 |
| 1 / – | 3834 | 680 | 2136 | 0 |
| 1 / 1 | 744 | 790 | 658 | 0 |
| 1 / 5 | 1569 | 760 | 1213 | 0 |
| 1 / 1e300 | 3106 | 703 | 1912 | 0 |
| 5 / – | 3086 | 779 | 1905 | 0 |
| 5 / 5 | 821 | 859 | 719 | 0 |
| 5 / 1e300 | 2358 | 802 | 1623 | 0 |

With a bound set, missing=none and missing=false give identical rows. Swapping a zero bound for -0 gives identical rows.

**Raw predicates.** These were pruned and then evaluated row by row, with Rust `f64::total_cmp` as a second reference:

| predicate | rows true | equals `total_cmp` | containers pruned | violations |
|---|---|---|---|---|
| `x > 5` | 6052 | yes | 786 | 0 |
| `x = 0.0` / `x = -0.0` | 1594 / 1594 | **no** (±0 normalized) | 684 / 684 | 0 |
| `x < 0` | 7771 | **no** | 553 | 0 |
| `x >= 0` | 11503 | **no** | 299 | 0 |
| `x > 1e308` / `x < -1e308` | 4514 / 3868 | yes | 1069 / 1242 | 0 |
| `x != 5` | 18453 | yes | 21 | 0 |
| `x = NaN` (canonical bits only) | 774 | yes | 1851 | 0 |
| `x > inf` (+NaN rows) / `x < -inf` (-NaN rows) | 3037 / 2330 | yes | 1509 / 1789 | 0 |
| `x IS NOT NULL` | 19274 | yes | 13 | 0 |
| `NOT (x <= 5)` | 6052 | yes | **0** (not rewritten) | 0 |
| `x BETWEEN -1 AND 1` | 6955 | yes | 382 | 0 |

**Batch-size sweep.** 16384 rows: 98% uniform values in [-1000, 1000] and 2% drawn from the grid. The figures are the share of containers pruned. There were 0 violations in every cell.

| layout | rows per batch | min=5 | max=0 | [-1,1] | [0,0] | missing=true | missing=false |
|---|---|---|---|---|---|---|---|
| unordered values | 1 | 50.9% | 50.4% | 99.3% | 99.9% | 99.9% | 0.8% |
| unordered values | 16 | 0% | 0% | 0% | 0% | 98.7% | 0% |
| unordered values | 64 / 256 / 1024 / 4096 | 0% | 0% | 0% | 0% | 95.7 / 84.4 / 56.2 / 0% | 0% |
| sorted by value (best case) | 16 | 50.8% | 50.3% | 99.2% | 99.8% | 99.9% | 0.7% |
| sorted by value (best case) | 256 | 50.0% | 50.0% | 96.9% | 98.4% | 98.4% | 0% |
| sorted by value (best case) | 4096 | 50.0% | 50.0% | 50.0% | 75.0% | 75.0% | 0% |

## Probe 8: pruning on the FixedSizeBinary(16) `symbol_id` key

The data is 512 random 16-byte ids × 32 steps = 16384 rows, sorted by (step, symbol_id), which is C1's within-file key order. Container statistics are `min/max_fixed_size_binary` → `ScalarValue::FixedSizeBinary(16, ..)` plus Int64 `step` min/max. There were 64 lookups per row of the table.

| rows per batch | containers | query | average kept | max kept | violations |
|---|---|---|---|---|---|
| 64 | 256 | `symbol_id = lit` | 32 | 32 | 0 |
| 64 | 256 | `step = k AND symbol_id = lit` | 1 | 1 | 0 |
| 64 | 256 | `symbol_id = absent lit` | 32 | 32 | 0 |
| 256 | 64 | same three queries | 32 / 1 / 32 | | 0 |
| 1024 | 16 | same three queries | 16 / 1 / 16 | | 0 |
| 4096 | 4 | same three queries | 4 / 1 / 4 | | 0 |

`symbol_id = lit` keeps one container per step. Every step holds a matching row, so that is the minimum possible. With the step given, one container remains. An id that does not exist cannot be told apart by min/max, so it keeps as many containers as one that does. A bloom filter or `contained()` would be needed to reject it.

## Evidence and caveats

*Tested* by `pruning-probe` (`src/main.rs`; probe functions `probe2`, `probe4` and `probe8`) under these conditions:
- arrow 59.3.0, datafusion and datafusion-pruning 55.1.0, nightly-2026-09-28 rustc, a debug build at opt-level 1, one deterministic xorshift seed per data set;
- in-memory batches only, and only one Float64 column `x`;
- no IPC, no `FileDecoder`, no `ExecutionPlan` or `TableProvider`.

The only baseline is the reference implementation of `scalar_predicate`. The failure target is 0 and the probe found 0 soundness violations and 0 filter-equivalence mismatches.

1. **C1's ±0 literal trick is redundant but harmless.** Because DataFusion normalizes zeros, plain `x >= 0.0` or `x <= 0.0` would also admit both zeros. The signed literals keep the lowering correct even if a future release stops normalizing; under pure totalOrder they are the right form.
2. **Why pruning stays sound.** The statistics come from totalOrder `min`/`max`, but comparisons are zero-normalized. This works because normalization is monotone. A `min(x)` answered from statistics would return -0 bits for {+0, -0}, the same as DataFusion's own aggregate. inv-data §4.1 holds for min and max; the probe did not exercise `AggregateStatistics` itself.
3. **Strict masks differ from the reference at null rows** (null versus false). This is equivalent only where the expression is used as a filter. Wrapping the lowered expression in `NOT` would differ from the complement of the reference result.
4. **Value pruning is only as good as the clustering of values.** With unordered values, batches of 16 or more rows pruned 0% on every value bound. Only null-related predicates pruned, and only because nulls are rare. Under C1's key sort, which does not cluster values, expect real pruning mainly from the key columns and nulls. Scientific Float64 bounds will rarely prune unless values cluster naturally. C1's "every Float64 column becomes prunable" is sound but probably ineffective in practice.
5. **`NOT (x <= 5)` is not rewritten** by the pruning code (0 pruned), so it is sound but cannot prune.
6. **Scope:** I did not test `PruningPredicateBuilder::build` returning `None` for an always-true predicate, null-count-only statistics (`Absent` min/max), or statistics for nested or struct columns.