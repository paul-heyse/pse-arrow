# quick_cache

Realizations considered: installed `quick_cache` 0.6.24; upgrade route 0.7.0 (released
2026-06-27; not examined for placeholder changes).
Research date 2026-10-27. Sources: `quick_cache-0.6.24/src/sync.rs` doc comments for
`get_value_or_guard`, `get_value_or_guard_async`, `get_or_insert_async`.

## Types

### quick-cache/placeholder-guard
- **Need it serves:** coalesce concurrent misses for one key with an explicit guard object that the
  computing caller later fills.
- **Mechanisms and APIs:** `get_value_or_guard(_async)` returns the value or a `PlaceholderGuard`;
  `guard.insert(v)`; `get_or_insert_async(key, fut)`; optional wait timeout for the sync variant.
- **Obligations discharged:** miss coalescing, waiter wake, bounded cache storage.
- **Semantic conditions:** "while the returned guard is alive, other calls with the same key …
  wait until the guard is dropped or the value is inserted"; a dropped guard lets the next waiter
  compute. The guard lives in the caller, so producer survival depends on the caller moving the
  guard into a detached task.
- **Limits and negative results:** no last-waiter cancellation signal, no waiter capacity, no
  retiring state; values are retained by the cache.
- **Evidence:** Interface-checked: `sync.rs` doc comments at 0.6.24.

## Current applicability

### [CBO-011](../../../library_leverage/codebase/persistence-and-transport.md#cbo-011) — Single-flight beside caching libraries
- **Types:** quick-cache/placeholder-guard.
- **Realization:** installed (used elsewhere for math retention).
- **Conclusion:** ruled out for the flight table for the same reasons as moka; a detached task
  holding the guard would reproduce `flight.rs`'s spawn without its waiter accounting.
- **Evidence:** Interface-checked; Proposed.
