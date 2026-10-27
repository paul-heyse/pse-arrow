# moka

Realizations considered: not in the workspace lockfile at the baseline (contrary to the brief's
note); upgrade route `moka` 0.12.16 (released 2026-08-09; source present in the local cargo
registry from another project).
Research date 2026-10-27. Sources: `moka-0.12.16/src/future/value_initializer.rs`
(`WaiterGuard` drop, `EnclosingFutureAborted`, `InitFuturePanicked` retry handling).

## Types

### moka/coalesced-cache-initialization
- **Need it serves:** compute a missing cache value once while concurrent requesters of the same
  key wait, then retain it in a bounded cache.
- **Recognizable code shapes:** "get or compute" with a pending-key map beside a cache.
- **Mechanisms and APIs:** `future::Cache::{get_with, try_get_with, optionally_get_with}`,
  `entry(k).or_try_insert_with(..)`; errors shared as `Arc<E>`.
- **Obligations discharged:** waiter coalescing, value insertion, eviction, error fan-out.
- **Residual caller obligations:** cost/weigher policy, error typing, admission limits.
- **Semantic conditions:** the `init` future runs inside the first caller's future. If that caller
  is dropped, `WaiterGuard::drop` marks the waiter `EnclosingFutureAborted`, and waiting callers
  retry from the beginning, running *their own* `init` (the first producer is dropped, not
  retained). Panicking or repeatedly aborted inits panic after a retry limit. Successful results
  are retained in the cache.
- **Limits and negative results:** no producer survival through waiter cancellation, no
  "last waiter requests cancellation and joins exit", no per-key waiter limit, no retiring state.
- **Evidence:** Interface-checked: source file above at 0.12.16.

## Current applicability

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — Library single-flight retaining its producer
- **Types:** moka/coalesced-cache-initialization.
- **Realization:** upgrade route (new dependency).
- **Conclusion:** ruled out here — producer runs in and dies with the caller; the flights need a
  native load to keep its leases until actual exit after every waiter leaves, and they coalesce
  loads whose results are not meant to be retained by the flight table.
- **Evidence:** Interface-checked; Proposed.
