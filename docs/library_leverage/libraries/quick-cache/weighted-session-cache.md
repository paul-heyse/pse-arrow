# quick_cache (weighted unsync cache) — native-session retention

Realizations considered: installed `quick_cache` 0.6.24 (already a workspace dependency, used by
another crate in the workspace for weighted retention). Newer releases were not assessed.
Research date 2026-10-27. Sources: installed `quick_cache-0.6.24/src/{unsync.rs,lib.rs}`. CBO-030
was also screened against native-library session facilities: the `highs` 2.4.0 `HighsPtr` `Drop`;
the SUNDIALS `SUNContext` (logger, profiler and error-handler scope only); SUNDIALS, PETSc and
IBEX global initialization; and generic Rust pool/guard crates (`r2d2`, `deadpool`, `object-pool`,
`scopeguard`), from general knowledge only, without version inspection.

## Types

### quick-cache/weighted-native-handle-retention
- **Need it serves:** keeping reusable native sessions (solver memory blocks) per worker within a
  byte budget, dropping (and thereby freeing) the overflow.
- **Recognizable code shapes:** a thread-local `Vec<(key, session, bytes)>` with a hand-written
  LRU `evict()` that sums bytes; take-out/give-back around each use.
- **Mechanisms and APIs:** `unsync::Cache::with_options(Options, Weighter, hasher, Lifecycle)`,
  `OptionsBuilder` (estimated items, weight capacity), `Weighter::weight(&k, &v) -> u64`,
  `remove(&k)` (take-out for exclusive use), `insert(k, v)` / `insert_with_lifecycle`,
  `Lifecycle::{is_pinned, before_evict, on_evict}`, `weight()`, `capacity()`, `retain`, `drain`, `clear`.
- **Obligations discharged:** eviction ordering, weight bookkeeping and evicted-value hand-off.
  A value that exceeds the capacity is not retained.
- **Residual caller obligations:** computing each session's bytes; the take-out discipline that
  prevents aliasing; where evicted values are dropped (outside borrows); compatibility checks
  on reuse; budget changes.
- **Semantic conditions:** eviction policy is CLOCK-PRO-like (hot/cold/ghost), not strict LRU.
  Weights are read at insertion. Changing the capacity of an `unsync::Cache` means rebuilding
  it (no setter was found in `unsync.rs` at 0.6.24). Eviction happens during `insert`, so
  native frees run there unless `Lifecycle` defers them.
- **Limits and negative results:** a cache does not provide admission before allocation,
  cancellation or quiescence; it owns retention only.
- **Evidence:** Interface-checked: `unsync.rs` public functions and `Lifecycle` trait.

## Current applicability

### [CBO-030](../../../library_leverage/codebase/native-solver-integration.md#cbo-030) — Native session lifecycles re-expressed per adapter
- **Types:** quick-cache/weighted-native-handle-retention
- **Realization:** installed 0.6.24 (no new dependency).
- **Conclusion:** the "no library route" premise is confirmed for the lifecycle guard itself
  (acquire, quiesce, release capacity before a native wait, exactly-once free on unwind,
  cancellation on the original clock). Only one sub-part fits with conditions: the per-thread
  KINSOL session cache's byte-budgeted LRU could become a `quick_cache` weighted cache, as the
  workspace already does elsewhere.
- **What it would discharge / what remains local:** the eviction loop and byte summation. The
  RAII owner, budget acquisition from the job lease, quiescence and solver-global state (HiGHS
  scheduler, PETSc initialization, IBEX random state) remain local. Native libraries offer only
  per-object `Drop` (highs `HighsPtr`) or global init/finalize. `SUNContext` scopes logging,
  profiling and errors, not ownership or budgets.
- **Prerequisites, configuration or extension:** budget changes rebuild the cache. Evicted
  sessions must be dropped outside the thread-local borrow (as now).
- **Semantic gaps against the CBO's preservation constraints:** eviction order changes from
  strict LRU, which is behaviourally visible only through reuse counts. Exactly-once free is
  preserved by ownership.
- **Unsettled premises and what would settle them:** whether strict LRU order is part of any
  registered control's expectation — check `implicit.rs` tests (for example
  `inner_session_cache_bounded_by_bytes`).
- **Evidence:** Interface-checked; application Proposed. Generic pool crates were screened from
  general knowledge, not source-inspected: none counts bytes or integrates cancellation.
